use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(get_settings).put(put_settings))
}

const SMART_SEARCH_SETTINGS_KEYS: &[&str] = &[
    "smart-search.movies.preferredLanguage",
    "smart-search.movies.preferredQuality",
    "smart-search.tv.preferredLanguage",
    "smart-search.tv.preferredQuality",
    "smart-search.music.preferredQuality",
    "smart-search.games.preferredConsole",
    "smart-search.books.preferredFormat",
];

fn defaults() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("smart-search.movies.preferredLanguage", "English");
    m.insert("smart-search.movies.preferredQuality", "1080p");
    m.insert("smart-search.tv.preferredLanguage", "English");
    m.insert("smart-search.tv.preferredQuality", "1080p");
    m.insert("smart-search.music.preferredQuality", "FLAC");
    m.insert("smart-search.games.preferredConsole", "");
    m.insert("smart-search.books.preferredFormat", "EPUB");
    m
}

#[derive(Serialize)]
struct MediaTypeSettings {
    #[serde(rename = "preferredLanguage", skip_serializing_if = "Option::is_none")]
    preferred_language: Option<String>,
    #[serde(rename = "preferredQuality", skip_serializing_if = "Option::is_none")]
    preferred_quality: Option<String>,
    #[serde(rename = "preferredConsole", skip_serializing_if = "Option::is_none")]
    preferred_console: Option<String>,
    #[serde(rename = "preferredFormat", skip_serializing_if = "Option::is_none")]
    preferred_format: Option<String>,
}

#[derive(Serialize)]
struct SmartSearchSettings {
    movies: MediaTypeSettings,
    tv: MediaTypeSettings,
    music: MediaTypeSettings,
    games: MediaTypeSettings,
    books: MediaTypeSettings,
}

async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.settings.get_by_prefix("smart-search.");
    let existing: HashMap<String, String> = rows.into_iter().map(|r| (r.key, r.value)).collect();
    let defs = defaults();

    // Seed missing keys
    let mut missing = HashMap::new();
    for key in SMART_SEARCH_SETTINGS_KEYS {
        if !existing.contains_key(*key) {
            missing.insert(key.to_string(), defs[key].to_string());
        }
    }
    if !missing.is_empty() {
        state.settings.set_many(&missing);
    }

    let get_val = |key: &str| -> String {
        existing
            .get(key)
            .cloned()
            .unwrap_or_else(|| defs[key].to_string())
    };

    let get_opt = |key: &str| -> Option<String> {
        if defs.contains_key(key) {
            Some(get_val(key))
        } else {
            None
        }
    };

    Json(SmartSearchSettings {
        movies: MediaTypeSettings {
            preferred_language: get_opt("smart-search.movies.preferredLanguage"),
            preferred_quality: get_opt("smart-search.movies.preferredQuality"),
            preferred_console: None,
            preferred_format: None,
        },
        tv: MediaTypeSettings {
            preferred_language: get_opt("smart-search.tv.preferredLanguage"),
            preferred_quality: get_opt("smart-search.tv.preferredQuality"),
            preferred_console: None,
            preferred_format: None,
        },
        music: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: get_opt("smart-search.music.preferredQuality"),
            preferred_console: None,
            preferred_format: None,
        },
        games: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: None,
            preferred_console: get_opt("smart-search.games.preferredConsole"),
            preferred_format: None,
        },
        books: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: None,
            preferred_console: None,
            preferred_format: get_opt("smart-search.books.preferredFormat"),
        },
    })
}

#[derive(Deserialize)]
struct UpdateSettings {
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

async fn put_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettings>,
) -> impl IntoResponse {
    let defs = defaults();
    let mut entries = HashMap::new();

    for (key, value) in &body.fields {
        let full_key = format!("smart-search.{}", key);
        if defs.contains_key(full_key.as_str()) {
            entries.insert(full_key, value.clone());
        }
    }

    if entries.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No valid settings provided" })),
        );
    }

    state.settings.set_many(&entries);
    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}
