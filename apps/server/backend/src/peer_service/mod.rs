pub mod catalog;
pub mod handshake;
pub mod partykit_client;
pub mod types;
pub mod webrtc_peer;

use crate::worker_bridge::WorkerEvent;
use crate::AppState;
use catalog::CatalogCache;
use partykit_client::PartyKitClient;
use types::{DataChannelEnvelope, ServerCatalogMessage, SignalingServerMessage};
use webrtc_peer::{PeerEvent, PeerManager};

use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Manages the server's autonomous peer service: signaling, WebRTC, handshake,
/// and catalog serving — without requiring a browser window.
pub struct PeerServiceManager {
    state: AppState,
    signaling_url: String,
    server_private_key: String,
    server_address: String,
    passport_raw: String,
    passport_signature: String,
    server_passport: mhaol_identity::Passport,
    catalog_cache: CatalogCache,
}

impl PeerServiceManager {
    /// Create a new PeerServiceManager. Call `start()` to begin the event loop.
    pub fn new(state: AppState) -> Result<Self, String> {
        let identity_name = "SIGNALING_WALLET";
        state.identity_manager.ensure_identity(identity_name);

        let private_key = state
            .identity_manager
            .get_private_key(identity_name)
            .ok_or("Failed to load SIGNALING_WALLET private key")?;

        let address = state
            .identity_manager
            .get_address(identity_name)
            .ok_or("Failed to get SIGNALING_WALLET address")?;

        let passport = state
            .identity_manager
            .get_passport(identity_name)
            .ok_or("Failed to get SIGNALING_WALLET passport")?;

        let signaling_url = std::env::var("SIGNALING_URL").unwrap_or_else(|_| {
            "https://mhaol-signaling.project-arktosmos.partykit.dev".to_string()
        });

        Ok(Self {
            state,
            signaling_url,
            server_private_key: private_key,
            server_address: address.clone(),
            passport_raw: passport.raw.clone(),
            passport_signature: passport.signature.clone(),
            server_passport: passport,
            catalog_cache: catalog::new_cache(),
        })
    }

    /// Start the peer service event loop. This runs until dropped.
    pub async fn start(&mut self) -> Result<(), String> {
        let personal_room = mhaol_identity::to_eip55_checksum(&self.server_address);

        info!(
            address = %self.server_address,
            personal_room = %personal_room,
            "Starting peer service"
        );

        // Connect to handshakes room
        let (handshakes_client, mut handshakes_rx) = PartyKitClient::connect(
            &self.signaling_url,
            "handshakes",
            &self.server_private_key,
            &self.passport_raw,
            &self.passport_signature,
            None,
        )
        .await?;

        // Connect to personal room (server is the owner, no endorsement needed)
        let (personal_client, mut personal_rx) = PartyKitClient::connect(
            &self.signaling_url,
            &personal_room,
            &self.server_private_key,
            &self.passport_raw,
            &self.passport_signature,
            None,
        )
        .await?;

        info!("Connected to signaling rooms");

        // WebRTC peer manager
        let mut peer_manager = PeerManager::new();
        let mut peer_event_rx = peer_manager
            .take_event_rx()
            .ok_or("Failed to take peer event receiver")?;

        // Track which room each peer is in so we relay signaling correctly
        let mut peer_rooms: HashMap<String, String> = HashMap::new();

        loop {
            tokio::select! {
                // Messages from handshakes room
                Some(msg) = handshakes_rx.recv() => {
                    self.handle_signaling_message(
                        "handshakes",
                        msg,
                        &mut peer_manager,
                        &mut peer_rooms,
                    ).await;
                }

                // Messages from personal room
                Some(msg) = personal_rx.recv() => {
                    self.handle_signaling_message(
                        &personal_room,
                        msg,
                        &mut peer_manager,
                        &mut peer_rooms,
                    ).await;
                }

                // WebRTC peer events (ICE candidates, data channel messages, etc.)
                Some(event) = peer_event_rx.recv() => {
                    self.handle_peer_event(
                        event,
                        &mut peer_manager,
                        &peer_rooms,
                        &handshakes_client,
                        &personal_client,
                        &personal_room,
                    ).await;
                }

                // Both room connections closed
                else => {
                    warn!("All signaling connections lost");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a message from a PartyKit signaling room.
    async fn handle_signaling_message(
        &self,
        room_id: &str,
        msg: SignalingServerMessage,
        peer_manager: &mut PeerManager,
        peer_rooms: &mut HashMap<String, String>,
    ) {
        match msg {
            SignalingServerMessage::Connected {
                peer_id,
                ice_servers,
            } => {
                info!(room_id = %room_id, peer_id = %peer_id, "Connected to room");
                peer_manager.set_ice_servers(ice_servers);
            }

            SignalingServerMessage::RoomPeers { peers } => {
                info!(room_id = %room_id, count = peers.len(), "Room peers");
                for peer in peers {
                    peer_rooms.insert(peer.peer_id.clone(), room_id.to_string());
                }
            }

            SignalingServerMessage::PeerJoined {
                peer_id,
                name,
                instance_type,
            } => {
                info!(
                    room_id = %room_id,
                    peer_id = %peer_id,
                    name = %name,
                    instance_type = %instance_type,
                    "Peer joined"
                );
                peer_rooms.insert(peer_id, room_id.to_string());
            }

            SignalingServerMessage::PeerLeft { peer_id } => {
                info!(room_id = %room_id, peer_id = %peer_id, "Peer left");
                peer_manager.remove_peer(&peer_id).await;
                peer_rooms.remove(&peer_id);
            }

            SignalingServerMessage::Offer {
                from_peer_id,
                sdp,
            } => {
                debug!(room_id = %room_id, from = %from_peer_id, "Received SDP offer");
                peer_rooms.insert(from_peer_id.clone(), room_id.to_string());
                if let Err(e) = peer_manager.handle_offer(&from_peer_id, &sdp).await {
                    error!(from = %from_peer_id, "Failed to handle offer: {e}");
                }
            }

            SignalingServerMessage::Answer {
                from_peer_id,
                sdp: _,
            } => {
                // Server only answers, never offers, so we shouldn't receive answers
                debug!(room_id = %room_id, from = %from_peer_id, "Unexpected answer (server only answers)");
            }

            SignalingServerMessage::IceCandidate {
                from_peer_id,
                candidate,
                sdp_m_line_index,
                sdp_mid,
            } => {
                if let Err(e) = peer_manager
                    .handle_ice_candidate(
                        &from_peer_id,
                        &candidate,
                        sdp_m_line_index,
                        sdp_mid.as_deref(),
                    )
                    .await
                {
                    debug!(from = %from_peer_id, "Failed to handle ICE candidate: {e}");
                }
            }

            SignalingServerMessage::Error { message } => {
                error!(room_id = %room_id, "Signaling error: {message}");
            }
        }
    }

    /// Handle events from WebRTC peer connections.
    async fn handle_peer_event(
        &self,
        event: PeerEvent,
        peer_manager: &mut PeerManager,
        peer_rooms: &HashMap<String, String>,
        handshakes_client: &PartyKitClient,
        personal_client: &PartyKitClient,
        personal_room: &str,
    ) {
        match event {
            PeerEvent::Answer { peer_id, sdp } => {
                // Send the SDP answer back via the appropriate room's signaling
                let client = self.get_room_client(
                    &peer_id,
                    peer_rooms,
                    handshakes_client,
                    personal_client,
                    personal_room,
                );
                if let Err(e) = client.send_answer(&peer_id, &sdp) {
                    error!(peer_id = %peer_id, "Failed to send answer: {e}");
                }
            }

            PeerEvent::IceCandidate {
                peer_id,
                candidate,
                sdp_mline_index,
                sdp_mid,
            } => {
                let client = self.get_room_client(
                    &peer_id,
                    peer_rooms,
                    handshakes_client,
                    personal_client,
                    personal_room,
                );
                if let Err(e) =
                    client.send_ice_candidate(&peer_id, &candidate, sdp_mline_index, sdp_mid.as_deref())
                {
                    debug!(peer_id = %peer_id, "Failed to send ICE candidate: {e}");
                }
            }

            PeerEvent::ChannelOpen { peer_id } => {
                info!(peer_id = %peer_id, "Data channel open, ready for messages");
                // The client will send a contact-request; we wait for it.
            }

            PeerEvent::Message { peer_id, envelope } => {
                self.handle_data_message(&peer_id, &envelope, peer_manager)
                    .await;
            }

            PeerEvent::Disconnected { peer_id } => {
                info!(peer_id = %peer_id, "Peer disconnected");
                peer_manager.remove_peer(&peer_id).await;
            }
        }
    }

    /// Route a data channel message to the appropriate handler.
    async fn handle_data_message(
        &self,
        peer_id: &str,
        envelope: &DataChannelEnvelope,
        peer_manager: &mut PeerManager,
    ) {
        match envelope.channel.as_str() {
            "contact" => {
                if let Some(contact_msg) = handshake::parse_contact_message(envelope) {
                    match contact_msg {
                        types::ContactMessage::ContactRequest { passport } => {
                            match handshake::handle_contact_request(
                                &passport,
                                &self.server_passport,
                                &self.server_private_key,
                                &self.server_address,
                            ) {
                                Ok((accept_envelope, payload)) => {
                                    // Send accept back
                                    if let Err(e) =
                                        peer_manager.send_to_peer(peer_id, &accept_envelope).await
                                    {
                                        error!(peer_id = %peer_id, "Failed to send contact-accept: {e}");
                                        return;
                                    }

                                    // Save to roster
                                    self.state.roster_contacts.insert(
                                        &payload.address,
                                        &payload.name,
                                        Some(&passport.raw),
                                        Some(&payload.instance_type),
                                        None,
                                    );

                                    // Send catalog
                                    self.send_catalog_to_peer(peer_id, peer_manager).await;
                                }
                                Err(e) => {
                                    warn!(peer_id = %peer_id, "Contact request verification failed: {e}");
                                }
                            }
                        }
                        types::ContactMessage::ContactAccept { .. } => {
                            // Server doesn't initiate requests, so we shouldn't receive accepts
                            debug!(peer_id = %peer_id, "Unexpected contact-accept");
                        }
                    }
                }
            }

            "server-catalog" => {
                if let Some(catalog_msg) = catalog::parse_catalog_message(envelope) {
                    match catalog_msg {
                        ServerCatalogMessage::StreamRequest { tmdb_id } => {
                            self.handle_stream_request(peer_id, tmdb_id, peer_manager)
                                .await;
                        }
                        ServerCatalogMessage::CatalogRequest => {
                            self.send_catalog_to_peer(peer_id, peer_manager).await;
                        }
                        _ => {
                            debug!(peer_id = %peer_id, "Unexpected server-catalog message type");
                        }
                    }
                }
            }

            other => {
                debug!(peer_id = %peer_id, channel = %other, "Unhandled data channel");
            }
        }
    }

    /// Send the movie catalog to a peer.
    async fn send_catalog_to_peer(&self, peer_id: &str, peer_manager: &mut PeerManager) {
        let movies = catalog::get_or_build_catalog(&self.state, &self.catalog_cache).await;
        let envelope = catalog::build_catalog_envelope(&movies);

        if let Err(e) = peer_manager.send_to_peer(peer_id, &envelope).await {
            error!(peer_id = %peer_id, "Failed to send catalog: {e}");
        } else {
            info!(peer_id = %peer_id, count = movies.len(), "Sent movie catalog");
        }
    }

    /// Handle a stream request from a client peer.
    async fn handle_stream_request(
        &self,
        peer_id: &str,
        tmdb_id: i64,
        peer_manager: &mut PeerManager,
    ) {
        info!(peer_id = %peer_id, tmdb_id = %tmdb_id, "Stream request received");

        // Resolve TMDB ID to a local file path via the database
        let item_path = match catalog::resolve_file_path_for_tmdb(&self.state, tmdb_id) {
            Some(path) => path,
            None => {
                let error_envelope = DataChannelEnvelope {
                    channel: "server-catalog".to_string(),
                    payload: serde_json::to_value(ServerCatalogMessage::StreamError {
                        error: format!("No downloaded file for TMDB ID {}", tmdb_id),
                    })
                    .unwrap(),
                };
                let _ = peer_manager.send_to_peer(peer_id, &error_envelope).await;
                return;
            }
        };

        if !self.state.worker_bridge.is_ready() {
            let error_envelope = DataChannelEnvelope {
                channel: "server-catalog".to_string(),
                payload: serde_json::to_value(ServerCatalogMessage::StreamError {
                    error: "Streaming worker is not running".to_string(),
                })
                .unwrap(),
            };
            let _ = peer_manager.send_to_peer(peer_id, &error_envelope).await;
            return;
        }

        // Resolve to an actual video file (handles directories, etc.)
        let resolved_path = crate::api::player::resolve_media_path(&item_path);

        if !std::path::Path::new(&resolved_path).exists() {
            let error_envelope = DataChannelEnvelope {
                channel: "server-catalog".to_string(),
                payload: serde_json::to_value(ServerCatalogMessage::StreamError {
                    error: format!("File not found: {}", resolved_path),
                })
                .unwrap(),
            };
            let _ = peer_manager.send_to_peer(peer_id, &error_envelope).await;
            return;
        }

        info!(peer_id = %peer_id, resolved = %resolved_path, "Resolved stream path");

        let session_id = uuid::Uuid::new_v4().to_string();

        match self
            .state
            .worker_bridge
            .create_session(
                &session_id,
                Some(&resolved_path),
                None,
                &self.signaling_url,
                Some("video".to_string()),
                None,
                None,
                None,
            )
            .await
        {
            Ok(WorkerEvent::SessionCreated {
                session_id,
                room_id,
            }) => {
                let envelope = DataChannelEnvelope {
                    channel: "server-catalog".to_string(),
                    payload: serde_json::to_value(ServerCatalogMessage::StreamSession {
                        session_id,
                        room_id,
                        signaling_url: self.signaling_url.clone(),
                    })
                    .unwrap(),
                };
                if let Err(e) = peer_manager.send_to_peer(peer_id, &envelope).await {
                    error!(peer_id = %peer_id, "Failed to send stream session: {e}");
                }
            }
            Ok(WorkerEvent::Error { error, .. }) => {
                let envelope = DataChannelEnvelope {
                    channel: "server-catalog".to_string(),
                    payload: serde_json::to_value(ServerCatalogMessage::StreamError { error })
                        .unwrap(),
                };
                let _ = peer_manager.send_to_peer(peer_id, &envelope).await;
            }
            Err(e) => {
                let envelope = DataChannelEnvelope {
                    channel: "server-catalog".to_string(),
                    payload: serde_json::to_value(ServerCatalogMessage::StreamError { error: e })
                        .unwrap(),
                };
                let _ = peer_manager.send_to_peer(peer_id, &envelope).await;
            }
            _ => {}
        }
    }

    /// Determine which PartyKit client to use for sending signaling messages to a peer.
    fn get_room_client<'a>(
        &self,
        peer_id: &str,
        peer_rooms: &HashMap<String, String>,
        handshakes_client: &'a PartyKitClient,
        personal_client: &'a PartyKitClient,
        personal_room: &str,
    ) -> &'a PartyKitClient {
        match peer_rooms.get(peer_id) {
            Some(room) if room == personal_room => personal_client,
            _ => handshakes_client,
        }
    }
}
