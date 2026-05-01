use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Clone, Serialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub name: String,
    pub instance_type: String,
}

struct Peer {
    peer_id: String,
    name: String,
    instance_type: String,
    tx: mpsc::UnboundedSender<String>,
}

struct Room {
    peers: HashMap<String, Peer>,
}

pub struct RoomManager {
    rooms: RwLock<HashMap<String, Room>>,
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    /// Add a peer to a room. Returns existing peer infos and broadcast tx channels.
    /// Evicts prior connections with the same peer_id.
    pub fn add_peer(
        &self,
        room_id: &str,
        connection_id: &str,
        peer_id: &str,
        name: &str,
        instance_type: &str,
        tx: mpsc::UnboundedSender<String>,
    ) -> (Vec<PeerInfo>, Vec<mpsc::UnboundedSender<String>>) {
        let mut rooms = self.rooms.write();
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(|| Room {
                peers: HashMap::new(),
            });

        // Evict existing connections for this peer_id
        let evict_keys: Vec<String> = room
            .peers
            .iter()
            .filter(|(cid, p)| p.peer_id == peer_id && *cid != connection_id)
            .map(|(cid, _)| cid.clone())
            .collect();
        for key in evict_keys {
            if let Some(old) = room.peers.remove(&key) {
                let _ = old.tx.send("__close:4002".to_string());
            }
        }

        // Collect existing peer infos (deduplicated by peer_id)
        let mut seen = std::collections::HashSet::new();
        let existing: Vec<PeerInfo> = room
            .peers
            .values()
            .filter(|p| seen.insert(p.peer_id.clone()))
            .map(|p| PeerInfo {
                peer_id: p.peer_id.clone(),
                name: p.name.clone(),
                instance_type: p.instance_type.clone(),
            })
            .collect();

        // Collect broadcast channels
        let broadcast_txs: Vec<mpsc::UnboundedSender<String>> =
            room.peers.values().map(|p| p.tx.clone()).collect();

        room.peers.insert(
            connection_id.to_string(),
            Peer {
                peer_id: peer_id.to_string(),
                name: name.to_string(),
                instance_type: instance_type.to_string(),
                tx,
            },
        );

        (existing, broadcast_txs)
    }

    /// Remove a peer from a room. Returns the peer info and remaining tx channels.
    pub fn remove_peer(
        &self,
        room_id: &str,
        connection_id: &str,
    ) -> Option<(PeerInfo, Vec<mpsc::UnboundedSender<String>>)> {
        let mut rooms = self.rooms.write();
        let room = rooms.get_mut(room_id)?;
        let removed = room.peers.remove(connection_id)?;

        let remaining_txs: Vec<mpsc::UnboundedSender<String>> =
            room.peers.values().map(|p| p.tx.clone()).collect();

        if room.peers.is_empty() {
            rooms.remove(room_id);
        }

        Some((
            PeerInfo {
                peer_id: removed.peer_id,
                name: removed.name,
                instance_type: removed.instance_type,
            },
            remaining_txs,
        ))
    }

    /// Send a message to a specific peer in a room.
    pub fn relay_to_peer(&self, room_id: &str, target_peer_id: &str, message: &str) -> bool {
        let rooms = self.rooms.read();
        if let Some(room) = rooms.get(room_id) {
            for peer in room.peers.values() {
                if peer.peer_id == target_peer_id {
                    return peer.tx.send(message.to_string()).is_ok();
                }
            }
        }
        false
    }

    /// Get the list of peers in a room.
    pub fn get_room_peers(&self, room_id: &str) -> Vec<PeerInfo> {
        let rooms = self.rooms.read();
        match rooms.get(room_id) {
            Some(room) => {
                let mut seen = std::collections::HashSet::new();
                room.peers
                    .values()
                    .filter(|p| seen.insert(p.peer_id.clone()))
                    .map(|p| PeerInfo {
                        peer_id: p.peer_id.clone(),
                        name: p.name.clone(),
                        instance_type: p.instance_type.clone(),
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }
}
