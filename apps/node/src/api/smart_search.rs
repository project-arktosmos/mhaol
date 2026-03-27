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
    "smart-search.movies.smartSearchPrompt",
    "smart-search.tv.preferredLanguage",
    "smart-search.tv.preferredQuality",
    "smart-search.tv.smartSearchPrompt",
    "smart-search.music.preferredQuality",
    "smart-search.music.smartSearchPrompt",
    "smart-search.games.preferredConsole",
    "smart-search.games.smartSearchPrompt",
    "smart-search.books.preferredFormat",
    "smart-search.books.smartSearchPrompt",
];

fn defaults() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // Movies
    m.insert("smart-search.movies.preferredLanguage", "English");
    m.insert("smart-search.movies.preferredQuality", "1080p");
    m.insert("smart-search.movies.smartSearchPrompt", "You are a media search assistant. Given a torrent name and a target movie, determine whether the torrent is actually for the target movie. Respond in JSON format only.\n\nCRITICAL: First verify the torrent is genuinely for the target title. Torrents often match search keywords but are actually different media entirely. If the torrent is NOT for the target title, set relevance to 0.\n\nTarget: {{title}} ({{year}})\nTorrent: {{torrentName}}\n\nRespond with:\n{\"quality\": \"<resolution>\", \"languages\": \"<detected languages>\", \"subs\": \"<subtitle languages or none>\", \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}\n\nRelevance guidelines:\n- 0: Wrong media entirely\n- 1-49: Likely wrong (wrong cut, cam rip, etc.)\n- 50-74: Correct title but poor quality or wrong version\n- 75-100: Correct title, good match");

    // TV
    m.insert("smart-search.tv.preferredLanguage", "English");
    m.insert("smart-search.tv.preferredQuality", "1080p");
    m.insert("smart-search.tv.smartSearchPrompt", "You are a media search assistant. Given a torrent name and a target TV show, determine whether the torrent is for the correct show. Extract season and episode information to classify the torrent. Respond in JSON format only.\n\nCRITICAL: First verify the torrent is genuinely for the target show. If the torrent is NOT for the target show, set relevance to 0.\n\nTarget: {{title}} ({{year}})\nTorrent: {{torrentName}}\n\nRespond with:\n{\"quality\": \"<resolution>\", \"languages\": \"<detected languages>\", \"subs\": \"<subtitle languages or none>\", \"seasons\": \"<ALL | season number | season range | null>\", \"episode\": <episode number | null>, \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}\n\nSeason/episode guidelines:\n- Complete series (e.g. \"Complete Series\", \"S01-S08\"): seasons=\"ALL\", episode=null\n- Season pack (e.g. \"S03\", \"Season 3\"): seasons=\"3\", episode=null\n- Season range (e.g. \"S01-S03\", \"Seasons 1-3\"): seasons=\"1-3\", episode=null\n- Single episode (e.g. \"S03E05\"): seasons=\"3\", episode=5\n- Cannot determine: seasons=null, episode=null\n\nRelevance guidelines:\n- 0: Wrong show entirely\n- 1-49: Wrong season or unrelated episode\n- 50-74: Correct show but wrong season/quality\n- 75-100: Correct show, good match");

    // Music
    m.insert("smart-search.music.preferredQuality", "FLAC");
    m.insert("smart-search.music.smartSearchPrompt", "You are a media search assistant. Given a torrent name and a target album, determine whether the torrent is for the correct artist and album. Respond in JSON format only.\n\nCRITICAL: First verify the torrent is genuinely for the target artist and album. If it is a different album or artist, set relevance to 0.\n\nTarget: {{artist}} - {{title}} ({{year}})\nTorrent: {{torrentName}}\n\nRespond with:\n{\"quality\": \"<audio quality>\", \"languages\": \"<N/A>\", \"subs\": \"<none>\", \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}\n\nRelevance guidelines:\n- 0: Wrong artist or album\n- 1-49: Might be related but likely wrong\n- 50-74: Correct artist/album but poor quality\n- 75-100: Correct artist and album, good match");

    // Games
    m.insert("smart-search.games.preferredConsole", "");
    m.insert("smart-search.games.smartSearchPrompt", "You are a media search assistant. Given a torrent name and a target game ROM, determine whether the torrent contains the correct game for the correct console. Respond in JSON format only.\n\nCRITICAL: First verify the torrent is genuinely for the target game and console platform. If it is a different game or wrong platform, set relevance to 0.\n\nTarget: {{title}} [{{console}}] ({{year}})\nTorrent: {{torrentName}}\n\nRespond with:\n{\"quality\": \"<ROM format>\", \"languages\": \"<N/A>\", \"subs\": \"<none>\", \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}\n\nRelevance guidelines:\n- 0: Wrong game or wrong console\n- 1-49: Might be related but likely wrong\n- 50-74: Correct game but wrong region or format\n- 75-100: Correct game and console, good match");

    // Books
    m.insert("smart-search.books.preferredFormat", "EPUB");
    m.insert("smart-search.books.smartSearchPrompt", "You are a media search assistant. Given a torrent name and a target book, determine whether the torrent contains the correct book. Respond in JSON format only.\n\nCRITICAL: First verify the torrent is genuinely for the target book by the correct author. If it is a different book or wrong author, set relevance to 0.\n\nTarget: {{author}} - {{title}} ({{year}})\nTorrent: {{torrentName}}\n\nRespond with:\n{\"quality\": \"<format: EPUB/PDF/MOBI/AZW3>\", \"languages\": \"<detected languages>\", \"subs\": \"<none>\", \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}\n\nRelevance guidelines:\n- 0: Wrong book or wrong author\n- 1-49: Might be related but likely wrong\n- 50-74: Correct book but poor format or quality\n- 75-100: Correct book and author, good match");

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
    #[serde(rename = "smartSearchPrompt")]
    smart_search_prompt: String,
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
            smart_search_prompt: get_val("smart-search.movies.smartSearchPrompt"),
        },
        tv: MediaTypeSettings {
            preferred_language: get_opt("smart-search.tv.preferredLanguage"),
            preferred_quality: get_opt("smart-search.tv.preferredQuality"),
            preferred_console: None,
            preferred_format: None,
            smart_search_prompt: get_val("smart-search.tv.smartSearchPrompt"),
        },
        music: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: get_opt("smart-search.music.preferredQuality"),
            preferred_console: None,
            preferred_format: None,
            smart_search_prompt: get_val("smart-search.music.smartSearchPrompt"),
        },
        games: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: None,
            preferred_console: get_opt("smart-search.games.preferredConsole"),
            preferred_format: None,
            smart_search_prompt: get_val("smart-search.games.smartSearchPrompt"),
        },
        books: MediaTypeSettings {
            preferred_language: None,
            preferred_quality: None,
            preferred_console: None,
            preferred_format: get_opt("smart-search.books.preferredFormat"),
            smart_search_prompt: get_val("smart-search.books.smartSearchPrompt"),
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
