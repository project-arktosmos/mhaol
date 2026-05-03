use crate::artists::UpsertArtistRequest;
use crate::firkins::{FileEntry, ImageMeta};
use crate::state::CloudState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const PIRATEBAY_BASE: &str = "https://apibay.org";
const LRCLIB_BASE: &str = "https://lrclib.net/api";
/// Stremio's public OpenSubtitles v3 addon. No API key required and no
/// rate-limit gating. Endpoint shape:
///   /subtitles/movie/<imdb_id>.json
///   /subtitles/series/<imdb_id>:<season>:<episode>.json
/// Returns `{ subtitles: [{ id, url, lang }, ...] }` where `lang` is a
/// 3-letter ISO 639-2 code and `url` is a UTF-8-encoded SRT.
const STREMIO_OPENSUBS_BASE: &str = "https://opensubtitles-v3.strem.io";
const TPB_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://opentracker.i2p.rocks:6969/announce",
];

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/tmdb", post(search_tmdb))
        .route("/tmdb/episodes", post(tmdb_episodes))
        .route("/torrents", post(search_torrents))
        .route("/subs-lyrics", post(search_subs_lyrics))
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// The addon id whose source we're searching. The addon implies the
    /// content kind (e.g. `tmdb-movie` searches movies, `tmdb-tv` searches
    /// TV shows), so callers no longer need a separate `type` parameter.
    pub addon: String,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub description: String,
    pub artists: Vec<UpsertArtistRequest>,
    pub images: Vec<ImageMeta>,
    pub files: Vec<FileEntry>,
    pub year: Option<i32>,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    pub raw: serde_json::Value,
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn search_tmdb(
    State(_state): State<CloudState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResultItem>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    if query.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }

    let is_tv = matches!(req.addon.as_str(), "tmdb-tv" | "wyzie-subs-tv" | "local-tv");
    let endpoint = if is_tv { "/search/tv" } else { "/search/movie" };

    let url = format!(
        "{}{}?api_key={}&query={}&include_adult=false",
        TMDB_BASE,
        endpoint,
        api_key,
        urlencoding(query)
    );

    let payload =
        crate::catalog::http_get_json(&url, &[("Accept", "application/json")]).await?;

    let raw_results: Vec<serde_json::Value> = payload
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut tasks = Vec::with_capacity(raw_results.len());
    for r in &raw_results {
        let tmdb_id = r.get("id").and_then(|v| v.as_i64());
        let api_key = api_key.clone();
        tasks.push(tokio::spawn(async move {
            match tmdb_id {
                Some(id) => fetch_tmdb_credits(is_tv, id, &api_key).await,
                None => Vec::new(),
            }
        }));
    }

    let mut items: Vec<SearchResultItem> = Vec::with_capacity(raw_results.len());
    for (i, r) in raw_results.iter().enumerate() {
        let mut item = build_tmdb_item(r);
        if let Some(handle) = tasks.get_mut(i) {
            if let Ok(artists) = handle.await {
                item.artists = artists;
            }
        }
        items.push(item);
    }

    Ok(Json(items))
}

async fn fetch_tmdb_credits(
    is_tv: bool,
    tmdb_id: i64,
    api_key: &str,
) -> Vec<UpsertArtistRequest> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!("{}/{}/{}/credits?api_key={}", TMDB_BASE, kind, tmdb_id, api_key);
    let payload = match crate::catalog::http_get_json(&url, &[("Accept", "application/json")]).await
    {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    build_tmdb_artists(&payload)
}

const TMDB_RELEVANT_CREW_DEPARTMENTS: &[&str] = &["Directing", "Writing", "Production"];
const TMDB_CAST_LIMIT: usize = 20;
const TMDB_CREW_LIMIT: usize = 15;

fn build_tmdb_artists(credits: &serde_json::Value) -> Vec<UpsertArtistRequest> {
    let mut out: Vec<UpsertArtistRequest> = Vec::new();
    let mut seen_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();

    if let Some(cast) = credits.get("cast").and_then(|v| v.as_array()) {
        let mut cast_sorted = cast.iter().collect::<Vec<_>>();
        cast_sorted.sort_by_key(|c| c.get("order").and_then(|v| v.as_i64()).unwrap_or(i64::MAX));
        for c in cast_sorted.into_iter().take(TMDB_CAST_LIMIT) {
            if let Some(id) = c.get("id").and_then(|v| v.as_i64()) {
                if !seen_ids.insert(id) {
                    continue;
                }
            }
            if let Some(artist) = build_tmdb_artist_from_person(c) {
                out.push(artist);
            }
        }
    }

    if let Some(crew) = credits.get("crew").and_then(|v| v.as_array()) {
        let filtered: Vec<&serde_json::Value> = crew
            .iter()
            .filter(|c| {
                c.get("department")
                    .and_then(|v| v.as_str())
                    .map(|d| TMDB_RELEVANT_CREW_DEPARTMENTS.contains(&d))
                    .unwrap_or(false)
            })
            .collect();
        for c in filtered.into_iter().take(TMDB_CREW_LIMIT) {
            if let Some(id) = c.get("id").and_then(|v| v.as_i64()) {
                if !seen_ids.insert(id) {
                    continue;
                }
            }
            if let Some(artist) = build_tmdb_artist_from_person(c) {
                out.push(artist);
            }
        }
    }

    out
}

fn build_tmdb_artist_from_person(person: &serde_json::Value) -> Option<UpsertArtistRequest> {
    let name = person.get("name").and_then(|v| v.as_str())?;
    if name.is_empty() {
        return None;
    }
    let image_url = person
        .get("profile_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|p| format!("{}/w185{}", TMDB_IMG_BASE, p));
    let job = person
        .get("job")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let character = person
        .get("character")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    // Three-field artist doc: bake the character into the role for cast
    // members ("Actor as Forrest Gump") so the `as <character>` cue is
    // preserved without needing a separate `description` field.
    let role = match (job, character) {
        (Some(j), _) => Some(j.to_string()),
        (None, Some(c)) => Some(format!("Actor as {c}")),
        (None, None) => None,
    };
    Some(UpsertArtistRequest {
        name: name.to_string(),
        role,
        image_url,
    })
}

fn build_tmdb_item(r: &serde_json::Value) -> SearchResultItem {
    let title = r
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("name").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let description = r
        .get("overview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let year = r
        .get("release_date")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("first_air_date").and_then(|v| v.as_str()))
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
    let external_id = r.get("id").map(|v| v.to_string());

    let mut images: Vec<ImageMeta> = Vec::new();
    if let Some(poster) = r.get("poster_path").and_then(|v| v.as_str()) {
        if !poster.is_empty() {
            images.push(ImageMeta {
                url: format!("{}/w500{}", TMDB_IMG_BASE, poster),
                mime_type: "image/jpeg".to_string(),
                file_size: 0,
                width: 500,
                height: 750,
            });
        }
    }
    if let Some(backdrop) = r.get("backdrop_path").and_then(|v| v.as_str()) {
        if !backdrop.is_empty() {
            images.push(ImageMeta {
                url: format!("{}/w1280{}", TMDB_IMG_BASE, backdrop),
                mime_type: "image/jpeg".to_string(),
                file_size: 0,
                width: 1280,
                height: 720,
            });
        }
    }

    SearchResultItem {
        title,
        description,
        artists: Vec::new(),
        images,
        files: Vec::new(),
        year,
        external_id,
        raw: r.clone(),
    }
}

#[derive(Debug, Deserialize)]
pub struct EpisodesRequest {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct EpisodeView {
    pub title: String,
}

async fn tmdb_episodes(
    State(_state): State<CloudState>,
    Json(req): Json<EpisodesRequest>,
) -> Result<Json<Vec<EpisodeView>>, (StatusCode, Json<serde_json::Value>)> {
    let id = req.id.trim();
    if id.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }

    let detail_url = format!("{}/tv/{}?api_key={}", TMDB_BASE, urlencoding(id), api_key);
    let detail =
        crate::catalog::http_get_json(&detail_url, &[("Accept", "application/json")]).await?;

    let seasons = detail
        .get("seasons")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut episodes: Vec<EpisodeView> = Vec::new();
    for season in seasons {
        let n = season
            .get("season_number")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let url = format!(
            "{}/tv/{}/season/{}?api_key={}",
            TMDB_BASE,
            urlencoding(id),
            n,
            api_key
        );
        let payload =
            match crate::catalog::http_get_json(&url, &[("Accept", "application/json")]).await {
                Ok(v) => v,
                Err(_) => continue,
            };
        if let Some(eps) = payload.get("episodes").and_then(|e| e.as_array()) {
            for ep in eps {
                let name = ep
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let s = ep
                    .get("season_number")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(n);
                let e = ep
                    .get("episode_number")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let title = if name.is_empty() {
                    format!("S{:02}E{:02}", s, e)
                } else {
                    format!("S{:02}E{:02} – {}", s, e, name)
                };
                episodes.push(EpisodeView { title });
            }
        }
    }

    Ok(Json(episodes))
}

#[derive(Debug, Deserialize)]
pub struct TorrentSearchRequest {
    pub query: String,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentResult {
    pub title: String,
    pub description: String,
    pub seeders: u64,
    pub leechers: u64,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: u64,
    #[serde(rename = "magnetLink")]
    pub magnet_link: String,
    #[serde(rename = "infoHash")]
    pub info_hash: String,
    pub raw: serde_json::Value,
}

fn build_magnet_link(info_hash: &str, name: &str) -> String {
    let encoded_name = urlencoding(name);
    let trackers = TPB_TRACKERS
        .iter()
        .map(|t| format!("&tr={}", urlencoding(t)))
        .collect::<String>();
    format!("magnet:?xt=urn:btih:{info_hash}&dn={encoded_name}{trackers}")
}

/// Run a single PirateBay query and return the parsed result list.
/// Shared between the public `/api/search/torrents` route and the
/// firkin-scoped torrent search persistence in `crate::torrent`.
pub async fn run_torrent_query(query: &str, category: &str) -> Result<Vec<TorrentResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let category = if category.is_empty() { "0" } else { category };

    let url = format!(
        "{}/q.php?q={}&cat={}",
        PIRATEBAY_BASE,
        urlencoding(query),
        urlencoding(category)
    );

    let raw = crate::catalog::http_get_json(
        &url,
        &[
            ("User-Agent", "Mozilla/5.0 (compatible; Mhaol/1.0)"),
            ("Accept", "application/json"),
        ],
    )
    .await
    .map_err(|(s, b)| format!("piratebay request failed: {s} {:?}", b.0))?;

    let arr = match raw.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    if arr.len() == 1
        && arr[0].get("id").and_then(|v| v.as_str()) == Some("0")
        && arr[0].get("name").and_then(|v| v.as_str()) == Some("No results returned")
    {
        return Ok(Vec::new());
    }

    let parsed: Vec<TorrentResult> = arr
        .iter()
        .map(|r| {
            let name = r
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let info_hash = r
                .get("info_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let seeders = r
                .get("seeders")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let leechers = r
                .get("leechers")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let size_bytes = r
                .get("size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let magnet_link = if info_hash.is_empty() {
                String::new()
            } else {
                build_magnet_link(&info_hash, &name)
            };
            TorrentResult {
                title: name,
                description: format!(
                    "{seeders} seeders · {leechers} leechers · {size_bytes} bytes"
                ),
                seeders,
                leechers,
                size_bytes,
                magnet_link,
                info_hash,
                raw: r.clone(),
            }
        })
        .collect();

    Ok(parsed)
}

async fn search_torrents(
    State(_state): State<CloudState>,
    Json(req): Json<TorrentSearchRequest>,
) -> Result<Json<Vec<TorrentResult>>, (StatusCode, Json<serde_json::Value>)> {
    let category = req.category.as_deref().unwrap_or("0").to_string();
    run_torrent_query(&req.query, &category)
        .await
        .map(Json)
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ============================================================================
// Subs / Lyrics search — used by the right-side `SubsLyricsFinder` panel in
// the cloud webui (and mirrors `/api/search/subs-lyrics` on the node).
// LRCLIB for music tracks/albums, Wyzie for movies/TV episodes.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncedLine {
    pub time: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubsLyrics {
    pub kind: String,
    pub source: String,
    #[serde(rename = "externalId", default)]
    pub external_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "trackName", default, skip_serializing_if = "Option::is_none")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName", default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    #[serde(rename = "albumName", default, skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "plainLyrics", default, skip_serializing_if = "Option::is_none")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics", default, skip_serializing_if = "Option::is_none")]
    pub synced_lyrics: Option<Vec<SyncedLine>>,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(rename = "isHearingImpaired", default)]
    pub is_hearing_impaired: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(rename = "sourceExternalId", default, skip_serializing_if = "Option::is_none")]
    pub source_external_id: Option<String>,
    /// Release / file name reported by the upstream subtitle host's
    /// `Content-Disposition` header (e.g.
    /// `Captain.America.Civil.WAR.2016.1080p.HD.TC.AC3.x264-ETRG.srt`).
    /// Populated by `search_stremio_opensubs` via per-URL HEAD requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubsLyricsRequest {
    /// The addon id whose subs/lyrics we're looking up. `lrclib` and
    /// `musicbrainz`/`local-album` use LRCLIB; `tmdb-movie` / `tmdb-tv`
    /// (and `local-movie` / `local-tv`) route through Stremio's public
    /// OpenSubtitles v3 addon, which keys off IMDb id — the backend
    /// converts the supplied TMDB id via TMDB's `external_ids` endpoint.
    pub addon: String,
    pub query: String,
    #[serde(rename = "externalIds", default)]
    pub external_ids: Vec<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    /// TV-only. Stremio's OpenSubtitles v3 addon scopes subtitles to a
    /// specific episode. When omitted, the backend defaults to season 1
    /// episode 1 so the catalog detail page at least returns something
    /// for the show as a whole.
    #[serde(default)]
    pub season: Option<u32>,
    #[serde(default)]
    pub episode: Option<u32>,
}

async fn search_subs_lyrics(
    State(_state): State<CloudState>,
    Json(req): Json<SubsLyricsRequest>,
) -> Result<Json<Vec<SubsLyrics>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    let addon = req.addon.trim();

    let is_album_addon = matches!(addon, "lrclib" | "musicbrainz" | "local-album");
    if is_album_addon {
        if query.is_empty() {
            return Ok(Json(Vec::new()));
        }
        return search_lrclib(query).await.map(Json);
    }

    let is_tv_addon = matches!(addon, "tmdb-tv" | "wyzie-subs-tv" | "local-tv");
    let is_movie_addon = matches!(addon, "tmdb-movie" | "wyzie-subs-movie" | "local-movie");
    if is_tv_addon || is_movie_addon {
        if req.external_ids.is_empty() {
            return Ok(Json(Vec::new()));
        }
        let kind = if is_tv_addon { "tv" } else { "movie" };
        let season = req.season.unwrap_or(1);
        let episode = req.episode.unwrap_or(1);
        return search_stremio_opensubs(kind, &req.external_ids, season, episode)
            .await
            .map(Json);
    }

    Ok(Json(Vec::new()))
}

async fn search_lrclib(
    query: &str,
) -> Result<Vec<SubsLyrics>, (StatusCode, Json<serde_json::Value>)> {
    lrclib_search(query)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

/// Run a free-text search against LRCLIB and return parsed results. The
/// HTTP route wraps this and converts errors to JSON responses; other
/// modules (notably the firkin track resolver) call it directly.
pub async fn lrclib_search(query: &str) -> Result<Vec<SubsLyrics>, String> {
    let url = format!("{}/search?q={}", LRCLIB_BASE, urlencoding(query));
    let payload = crate::catalog::http_get_json(
        &url,
        &[
            ("Accept", "application/json"),
            (
                "User-Agent",
                "Mhaol/1.0 (https://github.com/project-arktosmos/mhaol)",
            ),
        ],
    )
    .await
    .map_err(|(s, b)| format!("lrclib request failed: {s} {:?}", b.0))?;

    let arr = match payload.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let out = arr
        .iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string())
                .unwrap_or_default();
            let track_name = item
                .get("trackName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let artist_name = item
                .get("artistName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let album_name = item
                .get("albumName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let duration = item.get("duration").and_then(|v| v.as_f64());
            let plain_lyrics = item
                .get("plainLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let synced_raw = item
                .get("syncedLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let synced_lyrics = synced_raw.map(parse_lrc_lines);
            let instrumental = item
                .get("instrumental")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let display = if !artist_name.is_empty() {
                format!("{} — {}", artist_name, track_name)
            } else {
                track_name.clone()
            };
            SubsLyrics {
                kind: "lyrics".to_string(),
                source: "lrclib".to_string(),
                external_id: id,
                language: None,
                track_name: Some(track_name),
                artist_name: Some(artist_name),
                album_name,
                duration,
                format: Some("lrc".to_string()),
                url: None,
                plain_lyrics,
                synced_lyrics,
                instrumental,
                is_hearing_impaired: false,
                display: Some(display),
                source_external_id: None,
                release: None,
            }
        })
        .collect();
    Ok(out)
}

/// One LRCLIB hit kept in its raw shape (synced LRC text rather than
/// pre-parsed lines). Used by the firkin track resolver, which embeds the
/// raw LRC into the firkin's `files` so the body stays self-contained
/// across IPFS pins without re-fetching LRCLIB.
#[derive(Debug, Clone, Default)]
pub struct LrclibHit {
    pub id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration: Option<f64>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub instrumental: bool,
}

pub async fn lrclib_search_raw(query: &str) -> Result<Vec<LrclibHit>, String> {
    let url = format!("{}/search?q={}", LRCLIB_BASE, urlencoding(query));
    let payload = crate::catalog::http_get_json(
        &url,
        &[
            ("Accept", "application/json"),
            (
                "User-Agent",
                "Mhaol/1.0 (https://github.com/project-arktosmos/mhaol)",
            ),
        ],
    )
    .await
    .map_err(|(s, b)| format!("lrclib request failed: {s} {:?}", b.0))?;
    let arr = match payload.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };
    Ok(arr
        .iter()
        .map(|item| LrclibHit {
            id: item
                .get("id")
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string())
                .unwrap_or_default(),
            track_name: item
                .get("trackName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            artist_name: item
                .get("artistName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            album_name: item
                .get("albumName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            duration: item.get("duration").and_then(|v| v.as_f64()),
            plain_lyrics: item
                .get("plainLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            synced_lyrics: item
                .get("syncedLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            instrumental: item
                .get("instrumental")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
        .collect())
}

fn parse_lrc_lines(lrc: &str) -> Vec<SyncedLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }
        if let Some(bracket_end) = line.find(']') {
            let time_str = &line[1..bracket_end];
            let text = line[bracket_end + 1..].trim().to_string();
            if let Some(time) = parse_lrc_timestamp(time_str) {
                lines.push(SyncedLine { time, text });
            }
        }
    }
    lines.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lines
}

fn parse_lrc_timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

/// Resolve a TMDB id to its IMDb id via TMDB's `external_ids` endpoint.
/// Movies expose `imdb_id` at the root of the detail response too, but
/// TV shows only carry it under `external_ids` — going through this
/// dedicated endpoint works for both kinds with one URL shape.
async fn tmdb_to_imdb_id(kind: &str, tmdb_id: &str) -> Option<String> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return None;
    }
    let path = if kind == "tv" { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/external_ids?api_key={}",
        TMDB_BASE,
        path,
        urlencoding(tmdb_id),
        urlencoding(&api_key)
    );
    let payload = crate::catalog::http_get_json(&url, &[("Accept", "application/json")])
        .await
        .ok()?;
    payload
        .get("imdb_id")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Three-letter ISO 639-2 codes from Stremio's OpenSubtitles v3 addon
/// mapped to a human-readable display name. Covers the languages
/// OpenSubtitles actually carries; unknowns fall through to the raw
/// 3-letter code so the row still renders.
fn language_display(lang: &str) -> Option<&'static str> {
    Some(match lang {
        "eng" => "English",
        "spa" => "Spanish",
        "cat" => "Catalan",
        "por" => "Portuguese",
        "pob" => "Portuguese (Brazil)",
        "fre" | "fra" => "French",
        "ger" | "deu" => "German",
        "ita" => "Italian",
        "rus" => "Russian",
        "jpn" => "Japanese",
        "kor" => "Korean",
        "chi" | "zho" => "Chinese",
        "ara" => "Arabic",
        "dut" | "nld" => "Dutch",
        "pol" => "Polish",
        "tur" => "Turkish",
        "swe" => "Swedish",
        "nor" => "Norwegian",
        "dan" => "Danish",
        "fin" => "Finnish",
        "ell" | "gre" => "Greek",
        "heb" => "Hebrew",
        "hin" => "Hindi",
        "ind" => "Indonesian",
        "vie" => "Vietnamese",
        "tha" => "Thai",
        "ron" | "rum" => "Romanian",
        "hun" => "Hungarian",
        "cze" | "ces" => "Czech",
        "slo" | "slk" => "Slovak",
        "ukr" => "Ukrainian",
        "bul" => "Bulgarian",
        "srp" => "Serbian",
        "hrv" => "Croatian",
        "slv" => "Slovenian",
        _ => return None,
    })
}

async fn search_stremio_opensubs(
    kind: &str,
    external_ids: &[String],
    season: u32,
    episode: u32,
) -> Result<Vec<SubsLyrics>, (StatusCode, Json<serde_json::Value>)> {
    let mut out: Vec<SubsLyrics> = Vec::new();
    for tmdb_id in external_ids {
        let trimmed = tmdb_id.trim();
        if trimmed.is_empty() {
            continue;
        }
        let imdb_id = match tmdb_to_imdb_id(kind, trimmed).await {
            Some(id) => id,
            None => continue,
        };
        let url = if kind == "tv" {
            format!(
                "{}/subtitles/series/{}:{}:{}.json",
                STREMIO_OPENSUBS_BASE, imdb_id, season, episode
            )
        } else {
            format!("{}/subtitles/movie/{}.json", STREMIO_OPENSUBS_BASE, imdb_id)
        };
        let payload = match crate::catalog::http_get_json(
            &url,
            &[("Accept", "application/json"), ("User-Agent", "Mhaol/1.0")],
        )
        .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };
        let arr = match payload.get("subtitles").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };
        for item in arr {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let lang = item
                .get("lang")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // Restrict to the three languages the user actually consumes.
            // Filtering server-side avoids spending HEAD requests (and
            // payload size) on subs that the UI would never render.
            if !matches!(lang.as_str(), "eng" | "cat" | "spa") {
                continue;
            }
            let url_str = item
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display = language_display(&lang)
                .map(|s| s.to_string())
                .unwrap_or_else(|| lang.clone());
            out.push(SubsLyrics {
                kind: "subtitle".to_string(),
                source: "opensubtitles".to_string(),
                external_id: id,
                language: if lang.is_empty() { None } else { Some(lang) },
                track_name: None,
                artist_name: None,
                album_name: None,
                duration: None,
                format: Some("srt".to_string()),
                url: url_str,
                plain_lyrics: None,
                synced_lyrics: None,
                instrumental: false,
                is_hearing_impaired: false,
                display: Some(display),
                source_external_id: Some(imdb_id.clone()),
                release: None,
            });
        }
    }

    resolve_release_filenames(&mut out).await;
    Ok(out)
}

/// Resolve every subtitle's release filename in parallel by HEADing the
/// download URL and parsing `Content-Disposition: attachment; filename=…`.
/// Concurrency is bounded with a semaphore (firing ~100 parallel HEADs at
/// the same Cloudflare-fronted host produced scattered failures mid-batch),
/// and each request gets a short timeout + one retry. Per-URL failures
/// fall through silently — the row still renders, just without a filename.
async fn resolve_release_filenames(items: &mut [SubsLyrics]) {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    const CONCURRENCY: usize = 12;
    const PER_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
    let semaphore = Arc::new(Semaphore::new(CONCURRENCY));
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut set: JoinSet<Option<(usize, String)>> = JoinSet::new();
    for (idx, item) in items.iter().enumerate() {
        let Some(url) = item.url.clone() else { continue };
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        set.spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok()?;
            for attempt in 0..2 {
                let req = client
                    .head(&url)
                    .header("User-Agent", "Mhaol/1.0")
                    .timeout(PER_REQUEST_TIMEOUT)
                    .send();
                if let Ok(res) = req.await {
                    if res.status().is_success() {
                        if let Some(header) = res.headers().get(reqwest::header::CONTENT_DISPOSITION)
                        {
                            if let Ok(value) = header.to_str() {
                                if let Some(name) = parse_content_disposition_filename(value) {
                                    return Some((idx, name));
                                }
                            }
                        }
                        // No CD header on a 200 — no point retrying.
                        return None;
                    }
                }
                if attempt == 0 {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                }
            }
            None
        });
    }

    while let Some(joined) = set.join_next().await {
        if let Ok(Some((idx, name))) = joined {
            if let Some(item) = items.get_mut(idx) {
                item.release = Some(name);
            }
        }
    }
}

/// Pull the `filename="…"` (or bare `filename=…`) value out of an
/// HTTP `Content-Disposition` header. Strips surrounding quotes and any
/// stray whitespace, returns `None` if the param is absent.
fn parse_content_disposition_filename(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let key = "filename=";
    let start = lower.find(key)? + key.len();
    let rest = value.get(start..)?.trim();
    let trimmed = rest.trim_matches(|c: char| c == '"' || c == '\'');
    let end = trimmed
        .find(';')
        .map(|i| trimmed[..i].trim_end_matches(|c: char| c == '"' || c == '\''))
        .unwrap_or(trimmed);
    let cleaned = end.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

