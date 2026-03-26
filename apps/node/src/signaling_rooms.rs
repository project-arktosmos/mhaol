use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;

struct Peer {
    peer_id: String,
    tx: mpsc::UnboundedSender<String>,
}

struct Room {
    peers: HashMap<String, Peer>,
}

pub struct SignalingRoomManager {
    rooms: RwLock<HashMap<String, Room>>,
    port: u16,
}

impl Default for SignalingRoomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalingRoomManager {
    pub fn new() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1530);
        Self {
            rooms: RwLock::new(HashMap::new()),
            port,
        }
    }

    pub fn is_available(&self) -> bool {
        true
    }

    pub fn dev_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Add a peer to a room. Returns the list of existing peer IDs before this peer joined.
    /// Evicts any prior connection with the same peer_id (sends nothing — caller handles close).
    pub fn add_peer(
        &self,
        room_id: &str,
        connection_id: &str,
        peer_id: &str,
        tx: mpsc::UnboundedSender<String>,
    ) -> (Vec<String>, Vec<mpsc::UnboundedSender<String>>) {
        let mut rooms = self.rooms.write();
        let room = rooms.entry(room_id.to_string()).or_insert_with(|| Room {
            peers: HashMap::new(),
        });

        // Evict existing connections for this peer_id
        let mut evicted_txs = Vec::new();
        let evict_keys: Vec<String> = room
            .peers
            .iter()
            .filter(|(cid, p)| p.peer_id == peer_id && *cid != connection_id)
            .map(|(cid, _)| cid.clone())
            .collect();
        for key in evict_keys {
            if let Some(old) = room.peers.remove(&key) {
                evicted_txs.push(old.tx);
            }
        }

        // Collect existing peer IDs (deduplicated)
        let mut seen = std::collections::HashSet::new();
        let existing: Vec<String> = room
            .peers
            .values()
            .filter(|p| seen.insert(p.peer_id.clone()))
            .map(|p| p.peer_id.clone())
            .collect();

        // Collect existing peer tx channels for broadcasting peer-joined
        let broadcast_txs: Vec<mpsc::UnboundedSender<String>> =
            room.peers.values().map(|p| p.tx.clone()).collect();

        room.peers.insert(
            connection_id.to_string(),
            Peer {
                peer_id: peer_id.to_string(),
                tx,
            },
        );

        (existing, broadcast_txs)
    }

    /// Remove a peer from a room. Returns the peer_id and tx channels of remaining peers.
    pub fn remove_peer(
        &self,
        room_id: &str,
        connection_id: &str,
    ) -> Option<(String, Vec<mpsc::UnboundedSender<String>>)> {
        let mut rooms = self.rooms.write();
        let room = rooms.get_mut(room_id)?;
        let removed = room.peers.remove(connection_id)?;

        let remaining_txs: Vec<mpsc::UnboundedSender<String>> =
            room.peers.values().map(|p| p.tx.clone()).collect();

        if room.peers.is_empty() {
            rooms.remove(room_id);
        }

        Some((removed.peer_id, remaining_txs))
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

    /// Get the list of peer IDs in a room.
    pub fn get_room_peers(&self, room_id: &str) -> Vec<String> {
        let rooms = self.rooms.read();
        match rooms.get(room_id) {
            Some(room) => {
                let mut seen = std::collections::HashSet::new();
                room.peers
                    .values()
                    .filter(|p| seen.insert(p.peer_id.clone()))
                    .map(|p| p.peer_id.clone())
                    .collect()
            }
            None => Vec::new(),
        }
    }
}
