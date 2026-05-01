use mhaol_ipfs_stream::manager::IpfsStreamManager;
use mhaol_ipfs_stream::session::SessionState;
use std::path::PathBuf;

#[test]
fn manager_records_base_dir() {
    let mgr = IpfsStreamManager::new("/tmp/mhaol-ipfs-stream-test-base");
    assert_eq!(
        mgr.base_dir(),
        std::path::Path::new("/tmp/mhaol-ipfs-stream-test-base")
    );
}

#[test]
fn manager_starts_with_no_sessions() {
    let mgr = IpfsStreamManager::new("/tmp/mhaol-ipfs-stream-test-empty");
    assert!(mgr.list_sessions().is_empty());
}

#[test]
fn start_session_rejects_missing_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mgr = IpfsStreamManager::new(dir.path().to_path_buf());
    let result = mgr.start_session(
        "bafy-test".to_string(),
        PathBuf::from("/nonexistent/path/source.mp4"),
    );
    assert!(result.is_err());
    assert!(mgr.list_sessions().is_empty());
}

#[test]
fn stop_unknown_session_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mgr = IpfsStreamManager::new(dir.path().to_path_buf());
    let result = mgr.stop_session("does-not-exist");
    assert!(result.is_err());
}

#[test]
fn session_state_round_trips_via_serde() {
    let s = SessionState::Running;
    let json = serde_json::to_string(&s).expect("ser");
    let back: SessionState = serde_json::from_str(&json).expect("de");
    assert_eq!(s, back);
}
