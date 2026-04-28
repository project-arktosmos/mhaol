use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ed2kFileInfo {
    pub id: usize,
    pub name: String,
    /// MD4 file hash, lowercase hex (32 chars).
    pub file_hash: String,
    pub size: u64,
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers: u32,
    pub seeds: u32,
    pub state: Ed2kState,
    pub added_at: i64,
    pub eta: Option<u64>,
    pub output_path: Option<String>,
    pub source_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Ed2kState {
    Initializing,
    Searching,
    Downloading,
    Seeding,
    Paused,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddEd2kRequest {
    pub source: String,
    pub download_path: Option<String>,
    pub paused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ed2kStats {
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub active_files: u32,
    pub server_connected: bool,
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ed2kSearchResult {
    pub name: String,
    pub file_hash: String,
    pub size: u64,
    pub sources: u32,
    pub complete_sources: u32,
    pub ed2k_link: String,
    pub media_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ed2k_state_serde_round_trip() {
        let states = [
            Ed2kState::Initializing,
            Ed2kState::Searching,
            Ed2kState::Downloading,
            Ed2kState::Seeding,
            Ed2kState::Paused,
            Ed2kState::Error,
        ];
        for s in states {
            let json = serde_json::to_string(&s).unwrap();
            let back: Ed2kState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn ed2k_state_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&Ed2kState::Downloading).unwrap(),
            "\"downloading\""
        );
        assert_eq!(
            serde_json::to_string(&Ed2kState::Initializing).unwrap(),
            "\"initializing\""
        );
    }

    #[test]
    fn ed2k_file_info_serializes_camel_case() {
        let info = Ed2kFileInfo {
            id: 1,
            name: "file.mkv".to_string(),
            file_hash: "aabbccdd".to_string(),
            size: 1024,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            seeds: 0,
            state: Ed2kState::Initializing,
            added_at: 1700000000,
            eta: None,
            output_path: None,
            source_uri: "ed2k://|file|file.mkv|1024|aabbccdd|/".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"fileHash\""));
        assert!(json.contains("\"sourceUri\""));
        assert!(json.contains("\"downloadSpeed\""));
    }

    #[test]
    fn ed2k_stats_serializes_camel_case() {
        let stats = Ed2kStats {
            total_downloaded: 0,
            total_uploaded: 0,
            download_speed: 0,
            upload_speed: 0,
            active_files: 0,
            server_connected: false,
            server_name: "".to_string(),
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"serverConnected\""));
        assert!(json.contains("\"serverName\""));
        assert!(json.contains("\"activeFiles\""));
    }

    #[test]
    fn ed2k_search_result_round_trip() {
        let r = Ed2kSearchResult {
            name: "hello".to_string(),
            file_hash: "00112233445566778899aabbccddeeff".to_string(),
            size: 999,
            sources: 12,
            complete_sources: 7,
            ed2k_link: "ed2k://|file|hello|999|00112233445566778899aabbccddeeff|/"
                .to_string(),
            media_type: Some("Video".to_string()),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: Ed2kSearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn add_ed2k_request_camel_case() {
        let req = AddEd2kRequest {
            source: "ed2k://|file|x|1|abc|/".to_string(),
            download_path: Some("/tmp".to_string()),
            paused: Some(false),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"downloadPath\""));
    }
}
