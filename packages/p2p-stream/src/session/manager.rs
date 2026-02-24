use crate::error::{Error, Result};
use crate::media::MediaSource;
use crate::pipeline::PipelineBuilder;
use crate::session::peer::PeerSession;
use crate::session::state::SessionState;
use crate::signaling::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Manages multiple WebRTC peer sessions.
///
/// The SessionManager is the primary entry point for consumers. It creates
/// and tracks peer sessions, routes incoming signaling messages to the
/// correct session, and handles session lifecycle.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<PeerSession>>>>,
    pipeline_builder_factory: Arc<dyn Fn() -> PipelineBuilder + Send + Sync>,
    media_source: Arc<dyn MediaSource>,
}

impl SessionManager {
    /// Create a new SessionManager.
    ///
    /// - `pipeline_builder_factory`: creates a fresh `PipelineBuilder` per session,
    ///   allowing per-session configuration.
    /// - `media_source`: the media source shared across sessions (each session
    ///   creates its own pipeline elements from it).
    pub fn new(
        pipeline_builder_factory: impl Fn() -> PipelineBuilder + Send + Sync + 'static,
        media_source: impl MediaSource + 'static,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pipeline_builder_factory: Arc::new(pipeline_builder_factory),
            media_source: Arc::new(media_source),
        }
    }

    /// Create a new peer session and return its ID along with a receiver
    /// for outgoing signaling messages.
    pub fn create_session(
        &self,
        peer_id: impl Into<String>,
    ) -> Result<(String, mpsc::UnboundedReceiver<SignalingMessage>)> {
        let peer_id = peer_id.into();
        let (signaling_tx, signaling_rx) = mpsc::unbounded_channel();

        let builder = (self.pipeline_builder_factory)();
        let stream_pipeline = builder.build(self.media_source.as_ref())?;

        let session = PeerSession::new(peer_id.clone(), stream_pipeline, signaling_tx)?;

        let session = Arc::new(session);
        self.sessions.write().insert(peer_id.clone(), session);

        info!("Created session for peer {peer_id}");
        Ok((peer_id, signaling_rx))
    }

    /// Start a peer session (begins pipeline playback and negotiation).
    pub fn start_session(&self, peer_id: &str) -> Result<()> {
        let session = self.get_session(peer_id)?;
        session.start()
    }

    /// Route an incoming signaling message to the correct session.
    pub fn handle_signaling_message(
        &self,
        peer_id: &str,
        message: SignalingMessage,
    ) -> Result<()> {
        let session = self.get_session(peer_id)?;
        match message {
            SignalingMessage::SessionDescription(desc) => match desc.sdp_type {
                SdpType::Offer => session.handle_sdp_offer(&desc.sdp),
                SdpType::Answer => session.handle_sdp_answer(&desc.sdp),
            },
            SignalingMessage::IceCandidate(candidate) => session.handle_ice_candidate(&candidate),
            SignalingMessage::IceGatheringComplete => Ok(()),
            SignalingMessage::PeerDisconnected { .. } => self.remove_session(peer_id),
        }
    }

    /// Remove and stop a peer session.
    pub fn remove_session(&self, peer_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.write().remove(peer_id) {
            session.stop()?;
            info!("Removed session for peer {peer_id}");
        } else {
            warn!("Attempted to remove non-existent session: {peer_id}");
        }
        Ok(())
    }

    /// Get a list of all active session IDs.
    pub fn active_sessions(&self) -> Vec<String> {
        self.sessions.read().keys().cloned().collect()
    }

    /// Get the state of a specific session.
    pub fn session_state(&self, peer_id: &str) -> Result<SessionState> {
        let session = self.get_session(peer_id)?;
        Ok(session.state())
    }

    fn get_session(&self, peer_id: &str) -> Result<Arc<PeerSession>> {
        self.sessions
            .read()
            .get(peer_id)
            .cloned()
            .ok_or_else(|| Error::SessionNotFound(peer_id.to_string()))
    }
}
