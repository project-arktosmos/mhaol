use std::sync::Arc;

use mhaol_ipfs::IpfsManager;

#[derive(Clone)]
pub struct RendezvousState {
    pub ipfs: Arc<IpfsManager>,
}
