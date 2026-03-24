use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mhaol_node::{api::build_router, AppState};
use tower::ServiceExt;

/// Build a test app with an in-memory database and initialized modules.
fn test_app() -> axum::Router {
    let state = AppState::new(None).expect("Failed to create in-memory AppState");
    state.initialize_modules();
    state.seed_default_libraries();
    build_router(state)
}

/// Build a test app without module initialization (lighter, for DB-only tests).
fn test_app_bare() -> (axum::Router, AppState) {
    let state = AppState::new(None).expect("Failed to create in-memory AppState");
    let router = build_router(state.clone());
    (router, state)
}

/// Helper to read response body as a string.
async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Helper to parse response body as JSON.
async fn body_json(body: Body) -> serde_json::Value {
    let s = body_string(body).await;
    serde_json::from_str(&s).expect("Response body is not valid JSON")
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["status"], "ok");
}

// ---------------------------------------------------------------------------
// Database — tables
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_database_list_tables() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/database/tables")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    let tables = json.as_array().expect("tables should be an array");
    assert!(!tables.is_empty(), "should have at least one table");

    // Check that expected tables exist
    let table_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(table_names.contains(&"settings"), "should have settings table");
    assert!(table_names.contains(&"libraries"), "should have libraries table");
    assert!(table_names.contains(&"metadata"), "should have metadata table");
}

#[tokio::test]
async fn test_database_get_table_settings() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/database/tables/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["table"], "settings");
    assert!(json["columns"].is_array());
    assert!(json["pagination"].is_object());
    assert!(json["pagination"]["page"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_database_get_nonexistent_table() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/database/tables/nonexistent_table_xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_database_get_table_with_pagination() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/database/tables/settings?page=1&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["pagination"]["page"], 1);
    assert_eq!(json["pagination"]["limit"], 5);
}

// ---------------------------------------------------------------------------
// Libraries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_libraries() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/libraries")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    let libs = json.as_array().expect("libraries should be an array");
    // seed_default_libraries creates one
    assert!(!libs.is_empty(), "should have at least the default library");
}

#[tokio::test]
async fn test_create_and_delete_library() {
    let (app, _state) = test_app_bare();

    // Create library
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/libraries")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Test Library",
                        "path": "/tmp/test-lib",
                        "libraryType": "movies"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response.into_body()).await;
    let lib_id = json["id"].as_str().expect("should have id");
    assert_eq!(json["name"], "Test Library");
    assert_eq!(json["path"], "/tmp/test-lib");

    // Delete library
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/libraries/{}", lib_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// Media types and categories
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_media_types() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/libraries/media-types")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "media types should be an array");
}

#[tokio::test]
async fn test_get_categories() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/libraries/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "categories should be an array");
}

// ---------------------------------------------------------------------------
// Addons
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_addons() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/addons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "addons should be an array");
}

#[tokio::test]
async fn test_update_addon_setting_missing_fields() {
    let app = test_app();

    // Missing addon field
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/addons/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "key": "some_key", "value": "some_value" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Missing key field
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/addons/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "addon": "tmdb", "value": "some_value" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Missing value field
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/addons/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "addon": "tmdb", "key": "apiKey" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_addon_setting_not_found() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/addons/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "addon": "nonexistent_addon",
                        "key": "apiKey",
                        "value": "abc123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Plugins
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_plugins() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/plugins")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "plugins should be an array");
    // With modules initialized, there should be several plugins
    let plugins = json.as_array().unwrap();
    assert!(!plugins.is_empty(), "should have registered plugins after initialize_modules");
}

#[tokio::test]
async fn test_update_plugin_setting_missing_fields() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/plugins/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "key": "k", "value": "v" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Downloads
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_downloads_empty() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/downloads")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    let downloads = json.as_array().expect("downloads should be an array");
    assert!(downloads.is_empty(), "should have no downloads initially");
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_media() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json["mediaTypes"].is_array(), "should have mediaTypes");
    assert!(json["categories"].is_array(), "should have categories");
    assert!(json["linkSources"].is_array(), "should have linkSources");
    assert!(json["itemsByCategory"].is_object(), "should have itemsByCategory");
    assert!(json["itemsByType"].is_object(), "should have itemsByType");
    assert!(json["lists"].is_array(), "should have lists");
    assert!(json["libraries"].is_object(), "should have libraries");
}

// ---------------------------------------------------------------------------
// Network info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_network_info() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/network/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    // local_ip may be null in CI but the field should exist
    assert!(json.get("local_ip").is_some(), "should have local_ip field");
}

// ---------------------------------------------------------------------------
// Signaling status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_signaling_status() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/signaling/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.get("devAvailable").is_some(), "should have devAvailable");
    assert!(json.get("devUrl").is_some(), "should have devUrl");
}

// ---------------------------------------------------------------------------
// Hub — list apps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hub_list_apps() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/hub")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    let apps = json.as_array().expect("hub apps should be an array");
    assert!(!apps.is_empty(), "should have registered apps");

    // Each app entry should have name, port, and status fields
    let first = &apps[0];
    assert!(first.get("name").is_some());
    assert!(first.get("port").is_some());
    assert!(first.get("status").is_some());
}

// ---------------------------------------------------------------------------
// Player — playable files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_player_list_playable() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/player/playable")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "playable should be an array");
}

// ---------------------------------------------------------------------------
// Library item creation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_library_item() {
    let (app, _state) = test_app_bare();

    // First create a library
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/libraries")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Items Test Lib",
                        "path": "/tmp/items-test",
                        "libraryType": "movies"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let lib_json = body_json(response.into_body()).await;
    let lib_id = lib_json["id"].as_str().unwrap();

    // Create an item in that library
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/libraries/{}/items", lib_id))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Test Movie",
                        "path": "/tmp/items-test/movie.mp4",
                        "mediaType": "video"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let item_json = body_json(response.into_body()).await;
    assert_eq!(item_json["name"], "Test Movie");
    assert_eq!(item_json["libraryId"], lib_id);

    // Get library files and verify item is there
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/libraries/{}/files", lib_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let files_json = body_json(response.into_body()).await;
    assert_eq!(files_json["libraryId"], lib_id);
    let files = files_json["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
}

// ---------------------------------------------------------------------------
// Library files for nonexistent library
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_library_files_not_found() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/libraries/nonexistent-id/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Database reset
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_database_reset() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/database/reset")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["ok"], true);
    assert!(json["tables"].is_array());
}

// ---------------------------------------------------------------------------
// Browse directory
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_browse_directory() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/libraries/browse?path=/tmp")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["path"], "/tmp");
    assert!(json["directories"].is_array());
}

// ---------------------------------------------------------------------------
// Identities
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_identities() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/identities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert!(json.is_array(), "identities should be an array");
}

#[tokio::test]
async fn test_create_identity_missing_name() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/identities")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
