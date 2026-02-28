use crate::error::{Error, Result};
use crate::pipeline::StreamPipeline;
use crate::session::state::SessionState;
use crate::signaling::*;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_sdp as gst_sdp;
use gstreamer_webrtc as gst_webrtc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// A single WebRTC peer session.
///
/// Manages the GStreamer pipeline and webrtcbin for one peer connection.
/// Communicates with the external signaling layer via channels.
pub struct PeerSession {
    pub id: String,
    state: Arc<Mutex<SessionState>>,
    stream_pipeline: StreamPipeline,
    signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    shutdown: Arc<AtomicBool>,
}

impl PeerSession {
    pub(crate) fn new(
        id: String,
        stream_pipeline: StreamPipeline,
        signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    ) -> Result<Self> {
        let state = Arc::new(Mutex::new(SessionState::New));

        let session = Self {
            id,
            state,
            stream_pipeline,
            signaling_tx,
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        session.connect_webrtcbin_signals()?;

        Ok(session)
    }

    pub fn state(&self) -> SessionState {
        *self.state.lock()
    }

    /// Start the pipeline and begin the WebRTC negotiation process.
    pub fn start(&self) -> Result<()> {
        *self.state.lock() = SessionState::Connecting;
        self.stream_pipeline.play()?;
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

    fn start_position_ticker(&self) {
        let pipeline = self.stream_pipeline.pipeline().clone();
        let signaling_tx = self.signaling_tx.clone();
        let shutdown = self.shutdown.clone();
        let session_id = self.id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

            // Send initial media info once duration is available
            let mut sent_media_info = false;

            loop {
                interval.tick().await;

                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                let duration_secs = query_pipeline_duration(&pipeline);

                if !sent_media_info {
                    if duration_secs.is_some() {
                        let _ = signaling_tx.send(SignalingMessage::MediaInfo(
                            MediaInfoPayload { duration_secs },
                        ));
                        sent_media_info = true;
                    }
                }

                let position_secs = query_pipeline_position(&pipeline).unwrap_or(0.0);

                let msg = SignalingMessage::PositionUpdate(PositionPayload {
                    position_secs,
                    duration_secs,
                });

                if signaling_tx.send(msg).is_err() {
                    debug!("Session {session_id} position ticker: channel closed");
                    break;
                }
            }
        });
    }

    fn connect_webrtcbin_signals(&self) -> Result<()> {
        let webrtcbin = self.stream_pipeline.webrtcbin();

        // on-negotiation-needed: create and send an SDP offer
        {
            let signaling_tx = self.signaling_tx.clone();
            let state = self.state.clone();
            let session_id = self.id.clone();
            let webrtcbin_clone = webrtcbin.clone();

            webrtcbin.connect("on-negotiation-needed", false, move |_values| {
                info!("Session {session_id}: negotiation needed");

                // Guard: do not call create-offer until at least one media branch
                // has been linked to webrtcbin.  Calling create-offer with no
                // transceivers produces an empty SDP and may consume the internal
                // negotiation-needed state, preventing the signal from re-firing
                // when an audio-only pad is linked later.
                let n_sink: u32 = webrtcbin_clone.property("num-sink-pads");
                if n_sink == 0 {
                    info!("Session {session_id}: no sink pads yet, deferring negotiation");
                    return None;
                }

                let signaling_tx = signaling_tx.clone();
                let state = state.clone();
                let webrtcbin_inner = webrtcbin_clone.clone();

                let promise = gst::Promise::with_change_func(move |reply| {
                    let reply = match reply {
                        Ok(Some(reply)) => reply,
                        _ => {
                            error!("Failed to create SDP offer");
                            return;
                        }
                    };

                    let offer = reply
                        .value("offer")
                        .expect("Reply must contain 'offer'")
                        .get::<gst_webrtc::WebRTCSessionDescription>()
                        .expect("offer must be WebRTCSessionDescription");

                    let sdp_text = offer.sdp().to_string();

                    // Safety net: skip offers with no media descriptions
                    if !sdp_text.contains("m=") {
                        info!("Skipping SDP offer with no media descriptions");
                        return;
                    }

                    webrtcbin_inner.emit_by_name::<()>(
                        "set-local-description",
                        &[&offer, &None::<gst::Promise>],
                    );

                    let _ = signaling_tx.send(SignalingMessage::SessionDescription(
                        SessionDescription {
                            sdp_type: SdpType::Offer,
                            sdp: sdp_text,
                        },
                    ));

                    *state.lock() = SessionState::OfferSent;
                });

                webrtcbin_clone.emit_by_name::<()>(
                    "create-offer",
                    &[&None::<gst::Structure>, &promise],
                );
                None
            });
        }

        // on-ice-candidate: forward ICE candidates to signaling
        {
            let signaling_tx = self.signaling_tx.clone();

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

        Ok(())
    }

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
