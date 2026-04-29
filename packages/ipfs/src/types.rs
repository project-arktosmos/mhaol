use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpfsState {
    Stopped,
    Starting,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddIpfsRequest {
    /// Filesystem path to add to IPFS.
    pub source: String,
    /// Whether to pin the resulting CID (recursive).
    pub pin: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IpfsFileInfo {
    pub cid: String,
    pub name: String,
    pub size: u64,
    pub pinned: bool,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IpfsStats {
    pub state: IpfsState,
    pub peer_id: Option<String>,
    pub agent_version: String,
    pub connected_peers: u32,
    pub pinned_count: u32,
    pub repo_size_bytes: u64,
    pub listen_addrs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IpfsPeerInfo {
    pub peer_id: String,
    pub addr: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_serializes_snake_case() {
        assert_eq!(serde_json::to_string(&IpfsState::Stopped).unwrap(), "\"stopped\"");
        assert_eq!(serde_json::to_string(&IpfsState::Running).unwrap(), "\"running\"");
        assert_eq!(serde_json::to_string(&IpfsState::Starting).unwrap(), "\"starting\"");
        assert_eq!(serde_json::to_string(&IpfsState::Error).unwrap(), "\"error\"");
    }

    #[test]
    fn add_request_camel_case() {
        let r = AddIpfsRequest { source: "/x".to_string(), pin: Some(true) };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"source\""));
        assert!(json.contains("\"pin\""));
    }

    #[test]
    fn file_info_roundtrip() {
        let f = IpfsFileInfo {
            cid: "bafy...".to_string(),
            name: "foo".to_string(),
            size: 42,
            pinned: true,
            added_at: 1700000000,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: IpfsFileInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(f, back);
    }

    #[test]
    fn stats_roundtrip() {
        let s = IpfsStats {
            state: IpfsState::Running,
            peer_id: Some("12D3...".to_string()),
            agent_version: "mhaol-ipfs/0.1.0".to_string(),
            connected_peers: 3,
            pinned_count: 0,
            repo_size_bytes: 1024,
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: IpfsStats = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
