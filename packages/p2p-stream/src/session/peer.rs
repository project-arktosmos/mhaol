use crate::error::{Error, Result};
use crate::pipeline::{BusEvent, StreamPipeline};
use crate::session::state::SessionState;
use crate::signaling::*;
use gstreamer as gst;
use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer_sdp as gst_sdp;
use gstreamer_webrtc as gst_webrtc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// A single WebRTC peer session.
///
/// Manages the GStreamer pipeline and webrtcbin for one peer connection.
/// SDP/ICE signaling flows through `signaling_tx`. Media control messages
/// (Seek, MediaInfo, PositionUpdate) flow through a WebRTC data channel
/// named "media-control".
pub struct PeerSession {
    pub id: String,
    state: Arc<Mutex<SessionState>>,
    stream_pipeline: StreamPipeline,
    signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    data_channel: glib::Object,
    data_channel_open: Arc<AtomicBool>,
    /// Set to `true` once an SDP offer has been sent. Both the
    /// `on-negotiation-needed` handler and the post-pad-added watchdog read
    /// this to ensure exactly one offer is produced per session.
    offer_sent: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
}

// Safety: glib::Object is a reference-counted GObject which is inherently
// thread-safe (g_object_ref/unref are atomic). GStreamer elements like
// webrtcbin data channels are routinely accessed from multiple threads.
// All other fields are Send+Sync. The previous PeerSession (without
// data_channel) was already Send+Sync implicitly via StreamPipeline.
unsafe impl Send for PeerSession {}
unsafe impl Sync for PeerSession {}

impl PeerSession {
    pub(crate) fn new(
        id: String,
        mut stream_pipeline: StreamPipeline,
        signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    ) -> Result<Self> {
        let state = Arc::new(Mutex::new(SessionState::New));
        let offer_sent = Arc::new(AtomicBool::new(false));
        let bus_events = stream_pipeline.take_bus_events();

        // Connect webrtcbin signals BEFORE creating the data channel.
        // create-data-channel can synchronously emit `on-negotiation-needed`;
        // any handler connected after that point misses the emission entirely
        // (GStreamer signals are not queued).
        connect_webrtcbin_signals_impl(
            stream_pipeline.webrtcbin(),
            id.clone(),
            state.clone(),
            signaling_tx.clone(),
            offer_sent.clone(),
        );

        // GStreamer 1.28 requires the pipeline to be in at least READY state
        // for create-data-channel to succeed (returns null in NULL state).
        // We transition to READY here; PeerSession::start() will move to PLAYING.
        stream_pipeline
            .pipeline()
            .set_state(gst::State::Ready)
            .map_err(|e| Error::StateChange(format!("Failed to set READY: {e}")))?;

        // GStreamer 1.28 changed the return type of "create-data-channel"
        // from GObject to GstWebRTCDataChannel (a GstObject subclass).
        // We use emit_by_name_with_values and extract via Value::get which
        // handles GType hierarchy correctly (GstWebRTCDataChannel → GstObject → GObject).
        let data_channel: glib::Object = {
            let webrtcbin = stream_pipeline.webrtcbin();
            let ret: Option<glib::Value> = webrtcbin.emit_by_name_with_values(
                "create-data-channel",
                &["media-control".to_value(), None::<gst::Structure>.to_value()],
            );
            let ret = ret.expect("create-data-channel must return a value");
            ret.get::<glib::Object>().unwrap_or_else(|e| {
                // Value::get failed (strict GType mismatch) — fall back to
                // raw pointer extraction which is less strict about subtypes.
                warn!("Value::get::<Object> failed ({e}), using raw pointer extraction");
                unsafe {
                    use glib::translate::FromGlibPtrNone;
                    let ptr = glib::gobject_ffi::g_value_get_object(
                        ret.as_ptr() as *const _,
                    );
                    assert!(!ptr.is_null(), "create-data-channel returned null");
                    glib::Object::from_glib_none(ptr)
                }
            })
        };
        debug!("Data channel created, GType: {}", data_channel.type_().name());

        let session = Self {
            id,
            state,
            stream_pipeline,
            signaling_tx,
            data_channel,
            data_channel_open: Arc::new(AtomicBool::new(false)),
            offer_sent,
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        session.connect_data_channel_signals();

        // Watch the GStreamer bus for EOS so we can tell the browser to
        // advance to the next track. Browsers don't fire `ended` on a
        // MediaStream when the upstream goes silent, so we have to push
        // an explicit signal over the data channel.
        if let Some(rx) = bus_events {
            session.start_bus_watcher(rx);
        }

        Ok(session)
    }

    pub fn state(&self) -> SessionState {
        *self.state.lock()
    }

    /// Start the pipeline and begin the WebRTC negotiation process.
    pub fn start(&self) -> Result<()> {
        *self.state.lock() = SessionState::Connecting;
        self.stream_pipeline.play()?;
        watch_for_sink_pads(
            self.stream_pipeline.webrtcbin().clone(),
            self.id.clone(),
            self.signaling_tx.clone(),
            self.state.clone(),
            self.offer_sent.clone(),
            self.shutdown.clone(),
        );
        self.start_position_ticker();
        info!("Session {} started", self.id);
        Ok(())
    }

    /// Seek the pipeline to a position in seconds.
    pub fn seek(&self, position_secs: f64) -> Result<()> {
        info!("Session {} seeking to {:.2}s", self.id, position_secs);
        self.stream_pipeline.seek(position_secs)
    }

    /// Handle an incoming SDP answer from the remote peer.
    pub fn handle_sdp_answer(&self, sdp_str: &str) -> Result<()> {
        let sdp = gst_sdp::SDPMessage::parse_buffer(sdp_str.as_bytes())
            .map_err(|_| Error::SdpParse("Failed to parse SDP answer".into()))?;

        let answer =
            gst_webrtc::WebRTCSessionDescription::new(gst_webrtc::WebRTCSDPType::Answer, sdp);

        self.stream_pipeline.webrtcbin().emit_by_name::<()>(
            "set-remote-description",
            &[&answer, &None::<gst::Promise>],
        );

        *self.state.lock() = SessionState::Connected;
        info!("Session {} set remote description (answer)", self.id);
        Ok(())
    }

    /// Handle an incoming SDP offer from the remote peer (for the answering side).
    pub fn handle_sdp_offer(&self, sdp_str: &str) -> Result<()> {
        let sdp = gst_sdp::SDPMessage::parse_buffer(sdp_str.as_bytes())
            .map_err(|_| Error::SdpParse("Failed to parse SDP offer".into()))?;

        let offer =
            gst_webrtc::WebRTCSessionDescription::new(gst_webrtc::WebRTCSDPType::Offer, sdp);

        self.stream_pipeline.webrtcbin().emit_by_name::<()>(
            "set-remote-description",
            &[&offer, &None::<gst::Promise>],
        );

        self.create_answer()?;
        Ok(())
    }

    /// Handle an incoming ICE candidate from the remote peer.
    pub fn handle_ice_candidate(&self, candidate: &IceCandidate) -> Result<()> {
        self.stream_pipeline.webrtcbin().emit_by_name::<()>(
            "add-ice-candidate",
            &[&candidate.sdp_m_line_index, &candidate.candidate],
        );
        debug!("Session {} added ICE candidate", self.id);
        Ok(())
    }

    /// Stop the session and clean up resources.
    pub fn stop(&self) -> Result<()> {
        *self.state.lock() = SessionState::Disconnecting;
        self.shutdown.store(true, Ordering::Relaxed);
        self.stream_pipeline.stop()?;
        *self.state.lock() = SessionState::Closed;
        info!("Session {} stopped", self.id);
        Ok(())
    }

    // ===== Data channel =====

    fn connect_data_channel_signals(&self) {
        let dc = &self.data_channel;

        // on-open: mark data channel as ready
        {
            let open_flag = self.data_channel_open.clone();
            let session_id = self.id.clone();

            dc.connect("on-open", false, move |_| {
                info!("Session {session_id}: data channel opened");
                open_flag.store(true, Ordering::Relaxed);
                None
            });
        }

        // on-close: mark data channel as closed
        {
            let open_flag = self.data_channel_open.clone();
            let session_id = self.id.clone();

            dc.connect("on-close", false, move |_| {
                info!("Session {session_id}: data channel closed");
                open_flag.store(false, Ordering::Relaxed);
                None
            });
        }

        // on-message-string: handle Seek commands from the browser
        {
            let pipeline = self.stream_pipeline.pipeline().clone();
            let session_id = self.id.clone();

            dc.connect("on-message-string", false, move |values| {
                let msg_str = values[1].get::<&str>().unwrap_or_default();

                let value: serde_json::Value = match serde_json::from_str(msg_str) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Session {session_id}: invalid data channel message: {e}");
                        return None;
                    }
                };

                if value.get("type").and_then(|t| t.as_str()) == Some("Seek") {
                    if let Some(pos_secs) = value
                        .get("payload")
                        .and_then(|p| p.get("position_secs"))
                        .and_then(|p| p.as_f64())
                    {
                        info!("Session {session_id}: seeking to {pos_secs:.2}s via data channel");
                        let position = gst::ClockTime::from_nseconds(
                            (pos_secs * 1_000_000_000.0) as u64,
                        );
                        if let Err(e) = pipeline.seek_simple(
                            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                            position,
                        ) {
                            error!("Session {session_id}: seek failed: {e}");
                        }
                    }
                }

                None
            });
        }
    }

    // ===== Position ticker (sends via data channel) =====

    fn start_position_ticker(&self) {
        let pipeline = self.stream_pipeline.pipeline().clone();
        let dc_open = self.data_channel_open.clone();
        let shutdown = self.shutdown.clone();
        let session_id = self.id.clone();

        // glib::Object is !Send, so we pass the raw pointer as a usize to
        // cross the thread boundary.
        // Safety: GstWebRTCDataChannel is a ref-counted GObject. The
        // send-string action signal is thread-safe (GStreamer emits data
        // channel signals from streaming threads already).  We add a ref
        // here and release it when the thread exits.
        let dc_addr = self.data_channel.as_ptr() as usize;
        unsafe {
            glib::gobject_ffi::g_object_ref(dc_addr as *mut _);
        }

        std::thread::Builder::new()
            .name("position-ticker".into())
            .spawn(move || {
                let mut sent_media_info = false;

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    if shutdown.load(Ordering::Relaxed) {
                        debug!("Session {session_id}: position ticker stopped");
                        break;
                    }

                    if !dc_open.load(Ordering::Relaxed) {
                        continue;
                    }

                    let duration_secs = query_pipeline_duration(&pipeline);

                    if !sent_media_info
                        && duration_secs.is_some() {
                            let msg = serde_json::json!({
                                "type": "MediaInfo",
                                "payload": { "duration_secs": duration_secs }
                            });
                            send_string_on_dc(dc_addr, &msg.to_string());
                            sent_media_info = true;
                        }

                    let position_secs = query_pipeline_position(&pipeline).unwrap_or(0.0);

                    let msg = serde_json::json!({
                        "type": "PositionUpdate",
                        "payload": {
                            "position_secs": position_secs,
                            "duration_secs": duration_secs
                        }
                    });
                    send_string_on_dc(dc_addr, &msg.to_string());
                }

                // Release the ref we added above
                unsafe {
                    glib::gobject_ffi::g_object_unref(dc_addr as *mut _);
                }
            })
            .expect("Failed to spawn position ticker thread");
    }

    /// Forward GStreamer EOS to the browser as a `TrackEnded` data-channel
    /// message so it can advance to the next track. Browsers don't fire
    /// `ended` on a MediaStream when upstream just stops sending packets,
    /// so an explicit signal is required.
    ///
    /// Polls with `try_recv` + a short sleep so the `shutdown` flag wakes
    /// the thread up quickly when the session is torn down.
    fn start_bus_watcher(&self, mut bus_events: mpsc::UnboundedReceiver<BusEvent>) {
        let dc_open = self.data_channel_open.clone();
        let shutdown = self.shutdown.clone();
        let session_id = self.id.clone();

        // Same lifetime trick as the position ticker: ref the data-channel
        // GObject, ship the pointer across the thread boundary, unref on
        // exit. The `send-string` action signal is thread-safe.
        let dc_addr = self.data_channel.as_ptr() as usize;
        unsafe {
            glib::gobject_ffi::g_object_ref(dc_addr as *mut _);
        }

        std::thread::Builder::new()
            .name("bus-watcher".into())
            .spawn(move || {
                let poll_interval = std::time::Duration::from_millis(100);
                loop {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    match bus_events.try_recv() {
                        Ok(BusEvent::Eos) => {
                            info!("Session {session_id}: pipeline EOS, signaling TrackEnded");
                            if dc_open.load(Ordering::Relaxed) {
                                let msg =
                                    serde_json::json!({ "type": "TrackEnded" }).to_string();
                                send_string_on_dc(dc_addr, &msg);
                            } else {
                                debug!(
                                    "Session {session_id}: EOS observed but data channel not open"
                                );
                            }
                        }
                        Ok(BusEvent::Error { message, debug: dbg }) => {
                            error!(
                                "Session {session_id}: pipeline error: {message} (debug: {:?})",
                                dbg
                            );
                        }
                        Ok(_) => {}
                        Err(mpsc::error::TryRecvError::Empty) => {
                            std::thread::sleep(poll_interval);
                        }
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            debug!("Session {session_id}: bus channel closed, watcher exiting");
                            break;
                        }
                    }
                }

                unsafe {
                    glib::gobject_ffi::g_object_unref(dc_addr as *mut _);
                }
            })
            .expect("Failed to spawn bus watcher thread");
    }

    // ===== WebRTC signaling (SDP/ICE only) =====
    // Implemented as a free function (`connect_webrtcbin_signals_impl`) so it
    // can run before `Self` is constructed — see `PeerSession::new` for why
    // the order matters for audio-only sources.

    fn create_answer(&self) -> Result<()> {
        let webrtcbin = self.stream_pipeline.webrtcbin().clone();
        let signaling_tx = self.signaling_tx.clone();
        let state = self.state.clone();

        let promise = gst::Promise::with_change_func(move |reply| {
            let reply = match reply {
                Ok(Some(reply)) => reply,
                _ => {
                    error!("Failed to create SDP answer");
                    return;
                }
            };

            let answer = reply
                .value("answer")
                .expect("Reply must contain 'answer'")
                .get::<gst_webrtc::WebRTCSessionDescription>()
                .expect("answer must be WebRTCSessionDescription");

            let sdp_text = answer.sdp().to_string();

            webrtcbin.emit_by_name::<()>(
                "set-local-description",
                &[&answer, &None::<gst::Promise>],
            );

            let _ = signaling_tx.send(SignalingMessage::SessionDescription(
                SessionDescription {
                    sdp_type: SdpType::Answer,
                    sdp: sdp_text,
                },
            ));

            *state.lock() = SessionState::Connected;
        });

        self.stream_pipeline
            .webrtcbin()
            .emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);

        Ok(())
    }
}

/// Connect `on-negotiation-needed` and `on-ice-candidate` handlers to a
/// webrtcbin. Free function (rather than method) so the callers can run it
/// before constructing `PeerSession`, which matters for audio-only sources:
/// the very first negotiation-needed emission can be triggered synchronously
/// by `create-data-channel`, and GStreamer signals are not queued — handlers
/// connected after the emission would silently miss it.
///
/// `offer_sent` is a shared flag used to avoid sending two offers when both
/// the negotiation-needed handler and the post-pad-added watchdog (see
/// [`watch_for_sink_pads`]) try to negotiate.
fn connect_webrtcbin_signals_impl(
    webrtcbin: &gst::Element,
    session_id: String,
    state: Arc<Mutex<SessionState>>,
    signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    offer_sent: Arc<AtomicBool>,
) {
    // on-negotiation-needed: create and send an SDP offer.
    // For audio-only sources webrtcbin only fires this signal once — when
    // the data channel is created in READY state, before any media transceiver
    // exists. We must not call create-offer with zero transceivers, otherwise
    // we'd produce an empty SDP and burn the only chance to negotiate. Instead
    // we mark "deferred" and rely on `watch_for_sink_pads` (started in
    // `PeerSession::start`) to drive create-offer once the audio/video branch
    // has linked into webrtcbin.
    {
        let webrtcbin_clone = webrtcbin.clone();
        let signaling_tx = signaling_tx.clone();
        let state = state.clone();
        let session_id = session_id.clone();
        let offer_sent = offer_sent.clone();

        webrtcbin.connect("on-negotiation-needed", false, move |_values| {
            info!("Session {session_id}: negotiation needed");

            let has_sink_pads = webrtcbin_clone
                .pads()
                .iter()
                .any(|p| p.direction() == gst::PadDirection::Sink);
            if !has_sink_pads {
                info!("Session {session_id}: no sink pads yet, deferring negotiation");
                return None;
            }

            create_offer_and_send(
                &webrtcbin_clone,
                session_id.clone(),
                signaling_tx.clone(),
                state.clone(),
                offer_sent.clone(),
            );
            None
        });
    }

    // on-ice-candidate: forward ICE candidates to signaling
    {
        let signaling_tx = signaling_tx.clone();

        webrtcbin.connect("on-ice-candidate", false, move |values| {
            let mline_index = values[1].get::<u32>().expect("mlineindex must be u32");
            let candidate = values[2]
                .get::<String>()
                .expect("candidate must be string");

            let _ = signaling_tx.send(SignalingMessage::IceCandidate(IceCandidate {
                sdp_m_line_index: mline_index,
                candidate,
            }));

            None
        });
    }
}

/// Trigger create-offer + send the resulting SDP over signaling.
/// Idempotent — only the first call actually emits "create-offer".
fn create_offer_and_send(
    webrtcbin: &gst::Element,
    session_id: String,
    signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    state: Arc<Mutex<SessionState>>,
    offer_sent: Arc<AtomicBool>,
) {
    if offer_sent
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        debug!("Session {session_id}: offer already sent, skipping");
        return;
    }

    let webrtcbin_inner = webrtcbin.clone();
    let session_id_inner = session_id.clone();
    let promise = gst::Promise::with_change_func(move |reply| {
        let reply = match reply {
            Ok(Some(reply)) => reply,
            _ => {
                error!("Session {session_id_inner}: failed to create SDP offer");
                return;
            }
        };

        let offer = reply
            .value("offer")
            .expect("Reply must contain 'offer'")
            .get::<gst_webrtc::WebRTCSessionDescription>()
            .expect("offer must be WebRTCSessionDescription");

        let sdp_text = offer.sdp().to_string();

        if !sdp_text.contains("m=") {
            warn!("Session {session_id_inner}: SDP offer has no media descriptions, skipping");
            return;
        }

        if sdp_text.contains("m=application") {
            debug!("Session {session_id_inner}: SDP offer includes data channel");
        } else {
            warn!("Session {session_id_inner}: SDP offer MISSING data channel!");
        }

        webrtcbin_inner.emit_by_name::<()>(
            "set-local-description",
            &[&offer, &None::<gst::Promise>],
        );

        let _ = signaling_tx.send(SignalingMessage::SessionDescription(SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: sdp_text,
        }));

        *state.lock() = SessionState::OfferSent;
        info!("Session {session_id_inner}: SDP offer sent");
    });

    webrtcbin.emit_by_name::<()>("create-offer", &[&None::<gst::Structure>, &promise]);
}

/// Watchdog: poll webrtcbin for sink pads and drive create-offer when one
/// appears. This is the workaround for webrtcbin not re-firing
/// `on-negotiation-needed` after the audio (or video) branch links —
/// without this, audio-only sessions hang in "negotiating WebRTC connection"
/// because the deferred-negotiation guard (above) never gets a second chance
/// to run.
///
/// Stops as soon as either an offer is sent (`offer_sent` flag flipped) or
/// `shutdown` is signaled or the session is torn down (~10s safety cap).
fn watch_for_sink_pads(
    webrtcbin: gst::Element,
    session_id: String,
    signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    state: Arc<Mutex<SessionState>>,
    offer_sent: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
) {
    std::thread::Builder::new()
        .name(format!("p2p-stream-negotiation-watch-{session_id}"))
        .spawn(move || {
            let poll_interval = std::time::Duration::from_millis(50);
            let max_iters = 200; // ~10 seconds
            for _ in 0..max_iters {
                if offer_sent.load(Ordering::SeqCst) || shutdown.load(Ordering::Relaxed) {
                    return;
                }
                let has_sink_pads = webrtcbin
                    .pads()
                    .iter()
                    .any(|p| p.direction() == gst::PadDirection::Sink);
                if has_sink_pads {
                    info!(
                        "Session {session_id}: sink pad detected, triggering create-offer"
                    );
                    create_offer_and_send(
                        &webrtcbin,
                        session_id.clone(),
                        signaling_tx.clone(),
                        state.clone(),
                        offer_sent.clone(),
                    );
                    return;
                }
                std::thread::sleep(poll_interval);
            }
            warn!(
                "Session {session_id}: no sink pads after watchdog timeout, negotiation aborted"
            );
        })
        .expect("Failed to spawn negotiation watchdog thread");
}

/// Emit the "send-string" action signal on a GstWebRTCDataChannel.
///
/// `ptr_addr` is the GObject pointer cast to usize (to cross thread
/// boundaries — the caller must ensure the pointee is ref-counted and
/// alive).
fn send_string_on_dc(ptr_addr: usize, json: &str) {
    unsafe {
        let ptr = ptr_addr as *mut glib::gobject_ffi::GObject;
        let obj: glib::Object = glib::translate::from_glib_none(ptr);
        obj.emit_by_name::<()>("send-string", &[&json]);
        std::mem::forget(obj); // Don't unref — we don't own this reference
    }
}

/// Query position from named queue elements first (they propagate upstream
/// to filesrc), falling back to the pipeline.  webrtcbin itself cannot
/// answer position queries, so the direct pipeline query may miss for
/// audio-only pipelines that lack a video_queue.
fn query_pipeline_position(pipeline: &gst::Pipeline) -> Option<f64> {
    for name in &["video_queue", "audio_queue"] {
        if let Some(el) = pipeline.by_name(name) {
            if let Some(pos) = el.query_position::<gst::ClockTime>() {
                return Some(pos.nseconds() as f64 / 1_000_000_000.0);
            }
        }
    }
    pipeline
        .query_position::<gst::ClockTime>()
        .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
}

/// Query duration from named queue elements first, falling back to the pipeline.
fn query_pipeline_duration(pipeline: &gst::Pipeline) -> Option<f64> {
    for name in &["video_queue", "audio_queue"] {
        if let Some(el) = pipeline.by_name(name) {
            if let Some(dur) = el.query_duration::<gst::ClockTime>() {
                return Some(dur.nseconds() as f64 / 1_000_000_000.0);
            }
        }
    }
    pipeline
        .query_duration::<gst::ClockTime>()
        .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
}
