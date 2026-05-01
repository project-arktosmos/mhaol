use std::sync::Arc;

use mhaol_ipfs_core::IpfsManager;

use crate::rooms::RoomManager;
use crate::turn::TurnConfig;

#[derive(Clone)]
pub struct RendezvousState {
    pub ipfs: Arc<IpfsManager>,
    pub rooms: Arc<RoomManager>,
    pub turn: Arc<TurnConfig>,
}
