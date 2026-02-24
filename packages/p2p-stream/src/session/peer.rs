use crate::error::{Error, Result};
use crate::pipeline::StreamPipeline;
use crate::session::state::SessionState;
use crate::signaling::*;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_sdp as gst_sdp;
use gstreamer_webrtc as gst_webrtc;
use parking_lot::Mutex;
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
        info!("Session {} started", self.id);
        Ok(())
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
        self.stream_pipeline.stop()?;
        *self.state.lock() = SessionState::Closed;
        info!("Session {} stopped", self.id);
        Ok(())
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

                    // Skip offers with no media descriptions (fires before pads are linked)
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
