use crate::state::CloudState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const MUSICBRAINZ_BASE: &str = "https://musicbrainz.org/ws/2";
const COVERART_BASE: &str = "https://coverartarchive.org";
const OPENLIBRARY_BASE: &str = "https://openlibrary.org";
const OPENLIBRARY_COVERS: &str = "https://covers.openlibrary.org";
const RA_BASE: &str = "https://retroachievements.org/API";
const PIPED_BASE: &str = "https://pipedapi.kavin.rocks";
const IPTV_ORG_BASE: &str = "https://iptv-org.github.io/api";
const RADIO_BROWSER_BASE: &str = "https://de1.api.radio-browser.info/json";
const USER_AGENT: &str = "Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)";

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/sources", get(list_sources))
        .route("/{addon}/popular", get(popular))
        .route("/{addon}/genres", get(genres))
}

#[derive(Serialize)]
struct CatalogItem {
    id: String,
    title: String,
    year: Option<i32>,
    description: Option<String>,
    #[serde(rename = "posterUrl")]
    poster_url: Option<String>,
    #[serde(rename = "backdropUrl")]
    backdrop_url: Option<String>,
}

#[derive(Serialize)]
struct CatalogPage {
    items: Vec<CatalogItem>,
    page: i64,
    #[serde(rename = "totalPages")]
    total_pages: i64,
}

#[derive(Serialize)]
struct CatalogGenre {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct CatalogTypeInfo {
    id: &'static str,
    label: &'static str,
}

#[derive(Serialize)]
struct CatalogSource {
    id: &'static str,
    label: &'static str,
    types: Vec<CatalogTypeInfo>,
    #[serde(rename = "filterLabel")]
    filter_label: &'static str,
    #[serde(rename = "hasFilter")]
    has_filter: bool,
}

async fn list_sources() -> Json<Vec<CatalogSource>> {
    Json(vec![
        CatalogSource {
            id: "tmdb",
            label: "TMDB",
            types: vec![
                CatalogTypeInfo {
                    id: "movie",
                    label: "Movies",
                },
                CatalogTypeInfo {
                    id: "tv",
                    label: "TV Shows",
                },
                CatalogTypeInfo {
                    id: "image",
                    label: "Images",
                },
            ],
            filter_label: "Genre",
            has_filter: true,
        },
        CatalogSource {
            id: "musicbrainz",
            label: "MusicBrainz",
            types: vec![CatalogTypeInfo {
                id: "album",
                label: "Albums",
            }],
            filter_label: "Genre",
            has_filter: true,
        },
        CatalogSource {
            id: "retroachievements",
            label: "RetroAchievements",
            types: vec![CatalogTypeInfo {
                id: "game",
                label: "Games",
            }],
            filter_label: "Console",
            has_filter: true,
        },
        CatalogSource {
            id: "youtube",
            label: "YouTube",
            types: vec![
                CatalogTypeInfo {
                    id: "video",
                    label: "Videos",
                },
                CatalogTypeInfo {
                    id: "channel",
                    label: "Channels",
                },
            ],
            filter_label: "Region",
            has_filter: true,
        },
        CatalogSource {
            id: "lrclib",
            label: "LRCLIB",
            types: vec![CatalogTypeInfo {
                id: "album",
                label: "Albums",
            }],
            filter_label: "Filter",
            has_filter: false,
        },
        CatalogSource {
            id: "openlibrary",
            label: "OpenLibrary",
            types: vec![CatalogTypeInfo {
                id: "book",
                label: "Books",
            }],
            filter_label: "Subject",
            has_filter: true,
        },
        CatalogSource {
            id: "wyzie-subs",
            label: "Wyzie Subs",
            types: vec![
                CatalogTypeInfo {
                    id: "movie",
                    label: "Movies",
                },
                CatalogTypeInfo {
                    id: "tv",
                    label: "TV Shows",
                },
            ],
            filter_label: "Filter",
            has_filter: false,
        },
        CatalogSource {
            id: "iptv",
            label: "IPTV",
            types: vec![CatalogTypeInfo {
                id: "channel",
                label: "Channels",
            }],
            filter_label: "Category",
            has_filter: true,
        },
        CatalogSource {
            id: "radio",
            label: "Radio",
            types: vec![CatalogTypeInfo {
                id: "station",
                label: "Stations",
            }],
            filter_label: "Tag",
            has_filter: true,
        },
    ])
}

#[derive(Debug, Deserialize)]
pub struct PopularQuery {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GenresQuery {
    #[serde(default)]
    pub r#type: Option<String>,
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn popular(
    State(_state): State<CloudState>,
    Path(addon): Path<String>,
    Query(q): Query<PopularQuery>,
) -> Result<Json<CatalogPage>, (StatusCode, Json<serde_json::Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    match addon.as_str() {
        "tmdb" => tmdb_popular(q.r#type.as_deref(), q.filter.as_deref(), page).await,
        "musicbrainz" => musicbrainz_popular(q.r#type.as_deref(), q.filter.as_deref(), page).await,
        "openlibrary" => openlibrary_popular(q.filter.as_deref(), page).await,
        "retroachievements" => retroachievements_popular(q.filter.as_deref(), page).await,
        "youtube" => youtube_popular(q.r#type.as_deref(), q.filter.as_deref(), page).await,
        "iptv" => iptv_popular(q.filter.as_deref(), page).await,
        "radio" => radio_popular(q.filter.as_deref(), page).await,
        "lrclib" | "wyzie-subs" => Ok(empty_page(page)),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
}

async fn genres(
    State(_state): State<CloudState>,
    Path(addon): Path<String>,
    Query(q): Query<GenresQuery>,
) -> Result<Json<Vec<CatalogGenre>>, (StatusCode, Json<serde_json::Value>)> {
    match addon.as_str() {
        "tmdb" => tmdb_genres(q.r#type.as_deref()).await,
        "musicbrainz" => Ok(static_music_genres()),
        "openlibrary" => Ok(static_openlibrary_subjects()),
        "retroachievements" => Ok(static_ra_consoles()),
        "youtube" => Ok(static_youtube_regions()),
        "iptv" => Ok(static_iptv_categories()),
        "radio" => Ok(static_radio_tags()),
        "lrclib" | "wyzie-subs" => Ok(Vec::new()),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
}

fn empty_page(page: i64) -> CatalogPage {
    CatalogPage {
        items: Vec::new(),
        page,
        total_pages: 1,
    }
}

// ---------- TMDB ----------

async fn tmdb_popular(
    media_type: Option<&str>,
    genre: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let raw_kind = media_type.unwrap_or("movie");
    // image reuses popular movies.
    let kind = match raw_kind {
        "tv" => "tv",
        _ => "movie",
    };
    let endpoint = match kind {
        "tv" => {
            if let Some(g) = genre.filter(|s| !s.is_empty()) {
                format!(
                    "/discover/tv?api_key={}&page={}&with_genres={}&include_adult=false",
                    api_key,
                    page,
                    urlencoding(g)
                )
            } else {
                format!("/tv/popular?api_key={}&page={}", api_key, page)
            }
        }
        _ => {
            if let Some(g) = genre.filter(|s| !s.is_empty()) {
                format!(
                    "/discover/movie?api_key={}&page={}&with_genres={}&include_adult=false",
                    api_key,
                    page,
                    urlencoding(g)
                )
            } else {
                format!("/movie/popular?api_key={}&page={}", api_key, page)
            }
        }
    };
    let url = format!("{}{}", TMDB_BASE, endpoint);
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;

    let total_pages = payload
        .get("total_pages")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let items = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(tmdb_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages,
    })
}

fn tmdb_to_item(r: &serde_json::Value) -> CatalogItem {
    let title = r
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("name").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let description = r
        .get("overview")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let year = r
        .get("release_date")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("first_air_date").and_then(|v| v.as_str()))
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
    let id = r
        .get("id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());
    let poster_url = r
        .get("poster_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|p| format!("{}/w500{}", TMDB_IMG_BASE, p));
    let backdrop_url = r
        .get("backdrop_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|p| format!("{}/w1280{}", TMDB_IMG_BASE, p));
    CatalogItem {
        id,
        title,
        year,
        description,
        poster_url,
        backdrop_url,
    }
}

async fn tmdb_genres(
    media_type: Option<&str>,
) -> Result<Vec<CatalogGenre>, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let raw_kind = media_type.unwrap_or("movie");
    let kind = match raw_kind {
        "tv" => "tv",
        _ => "movie",
    };
    let path = if kind == "tv" {
        "/genre/tv/list"
    } else {
        "/genre/movie/list"
    };
    let url = format!("{}{}?api_key={}", TMDB_BASE, path, api_key);
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let genres = payload
        .get("genres")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    let id = g.get("id")?.as_i64()?.to_string();
                    let name = g.get("name")?.as_str()?.to_string();
                    Some(CatalogGenre { id, name })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(genres)
}

// ---------- MusicBrainz ----------

const MUSICBRAINZ_GENRES: &[&str] = &[
    "rock",
    "pop",
    "electronic",
    "hip hop",
    "jazz",
    "classical",
    "r&b",
    "metal",
    "folk",
    "soul",
    "punk",
    "blues",
    "country",
    "ambient",
    "indie",
    "alternative",
    "reggae",
    "latin",
];

fn static_music_genres() -> Vec<CatalogGenre> {
    MUSICBRAINZ_GENRES
        .iter()
        .map(|g| CatalogGenre {
            id: (*g).to_string(),
            name: capitalize_words(g),
        })
        .collect()
}

async fn musicbrainz_popular(
    _media_type: Option<&str>,
    genre: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 20;
    let offset = (page - 1) * limit;
    let tag = genre.filter(|s| !s.is_empty()).unwrap_or("rock");
    let query = format!("tag:\"{}\"", tag);
    let url = format!(
        "{}/release-group?query={}&fmt=json&limit={}&offset={}",
        MUSICBRAINZ_BASE,
        urlencoding(&query),
        limit,
        offset
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let count = payload
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(limit);
    let total_pages = ((count as f64) / (limit as f64)).ceil() as i64;
    let items = payload
        .get("release-groups")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(musicbrainz_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn musicbrainz_to_item(rg: &serde_json::Value) -> CatalogItem {
    let id = rg
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = rg
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let year = rg
        .get("first-release-date")
        .and_then(|v| v.as_str())
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
    let credits = rg
        .get("artist-credit")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    c.get("name")
                        .and_then(|v| v.as_str())
                        .or_else(|| c.get("artist").and_then(|a| a.get("name")?.as_str()))
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let primary_type = rg
        .get("primary-type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut description = String::new();
    if !credits.is_empty() {
        description.push_str(&credits);
    }
    if !primary_type.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(primary_type);
    }
    let poster_url = if id.is_empty() {
        None
    } else {
        Some(format!("{}/release-group/{}/front", COVERART_BASE, id))
    };
    CatalogItem {
        id,
        title,
        year,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        poster_url,
        backdrop_url: None,
    }
}

// ---------- OpenLibrary ----------

const OPENLIBRARY_SUBJECTS: &[(&str, &str)] = &[
    ("fiction", "Fiction"),
    ("science_fiction", "Science Fiction"),
    ("fantasy", "Fantasy"),
    ("mystery", "Mystery"),
    ("romance", "Romance"),
    ("history", "History"),
    ("science", "Science"),
    ("philosophy", "Philosophy"),
    ("biography", "Biography"),
    ("poetry", "Poetry"),
    ("children", "Children"),
    ("art", "Art"),
];

fn static_openlibrary_subjects() -> Vec<CatalogGenre> {
    OPENLIBRARY_SUBJECTS
        .iter()
        .map(|(id, name)| CatalogGenre {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

async fn openlibrary_popular(
    subject: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 20;
    let offset = (page - 1) * limit;
    let subj = subject.filter(|s| !s.is_empty()).unwrap_or("fiction");
    let url = format!(
        "{}/subjects/{}.json?limit={}&offset={}",
        OPENLIBRARY_BASE,
        urlencoding(subj),
        limit,
        offset
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let work_count = payload
        .get("work_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(limit);
    let total_pages = ((work_count as f64) / (limit as f64)).ceil() as i64;
    let items = payload
        .get("works")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(openlibrary_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn openlibrary_to_item(w: &serde_json::Value) -> CatalogItem {
    let key = w
        .get("key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_start_matches("/works/")
        .to_string();
    let title = w
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let year = w
        .get("first_publish_year")
        .and_then(|v| v.as_i64())
        .map(|n| n as i32);
    let authors = w
        .get("authors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let cover_id = w.get("cover_id").and_then(|v| v.as_i64());
    let poster_url =
        cover_id.map(|id| format!("{}/b/id/{}-L.jpg", OPENLIBRARY_COVERS, id));
    CatalogItem {
        id: key,
        title,
        year,
        description: if authors.is_empty() {
            None
        } else {
            Some(authors)
        },
        poster_url,
        backdrop_url: None,
    }
}

// ---------- RetroAchievements ----------

const RA_CONSOLES: &[(i64, &str)] = &[
    (1, "Genesis/Mega Drive"),
    (2, "Nintendo 64"),
    (3, "SNES/Super Famicom"),
    (4, "Game Boy"),
    (5, "Game Boy Advance"),
    (6, "Game Boy Color"),
    (7, "NES/Famicom"),
    (8, "PC Engine/TurboGrafx-16"),
    (9, "Atari 2600"),
    (10, "Atari 7800"),
    (11, "Master System"),
    (12, "PlayStation"),
    (13, "Atari Lynx"),
    (14, "Neo Geo Pocket"),
    (17, "Atari Jaguar"),
    (18, "Nintendo DS"),
    (21, "PlayStation 2"),
    (47, "Nintendo GameCube"),
];

fn static_ra_consoles() -> Vec<CatalogGenre> {
    RA_CONSOLES
        .iter()
        .map(|(id, name)| CatalogGenre {
            id: id.to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

async fn retroachievements_popular(
    console_id: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let user = std::env::var("RA_API_USER")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("RA_USERNAME").ok().filter(|s| !s.is_empty()))
        .or_else(|| std::env::var("RA_USER").ok().filter(|s| !s.is_empty()));
    let key = std::env::var("RA_API_KEY").ok().filter(|s| !s.is_empty());
    let (Some(user), Some(key)) = (user, key) else {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "RA_API_USER and RA_API_KEY env vars must be set on the cloud server",
        ));
    };
    let console = console_id
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);
    let url = format!(
        "{}/API_GetGameList.php?z={}&y={}&i={}&h=1",
        RA_BASE,
        urlencoding(&user),
        urlencoding(&key),
        console
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload.as_array().cloned().unwrap_or_default();
    let mut games: Vec<&serde_json::Value> = arr
        .iter()
        .filter(|g| {
            g.get("NumAchievements")
                .and_then(|v| v.as_i64())
                .map(|n| n > 0)
                .unwrap_or(false)
        })
        .collect();
    games.sort_by(|a, b| {
        let ap = a
            .get("NumDistinctPlayersHardcore")
            .or_else(|| a.get("NumDistinctPlayers"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let bp = b
            .get("NumDistinctPlayersHardcore")
            .or_else(|| b.get("NumDistinctPlayers"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        bp.cmp(&ap)
    });
    let limit: usize = 20;
    let total_pages = ((games.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let items: Vec<CatalogItem> = games
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(retroachievements_to_item)
        .collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn retroachievements_to_item(g: &serde_json::Value) -> CatalogItem {
    let id = g
        .get("ID")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());
    let title = g
        .get("Title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let year = g
        .get("Released")
        .and_then(|v| v.as_str())
        .and_then(|s| s.get(s.len().saturating_sub(4)..))
        .and_then(|s| s.parse::<i32>().ok());
    let console_name = g
        .get("ConsoleName")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let genre = g
        .get("Genre")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let mut description = console_name;
    if !genre.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(genre);
    }
    let poster_url = g
        .get("ImageBoxArt")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| g.get("ImageIcon").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty())
        .map(|p| {
            if p.starts_with("http") {
                p.to_string()
            } else {
                format!("https://media.retroachievements.org{}", p)
            }
        });
    CatalogItem {
        id,
        title,
        year,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        poster_url,
        backdrop_url: None,
    }
}

// ---------- YouTube ----------

const YOUTUBE_REGIONS: &[(&str, &str)] = &[
    ("US", "United States"),
    ("GB", "United Kingdom"),
    ("CA", "Canada"),
    ("AU", "Australia"),
    ("DE", "Germany"),
    ("FR", "France"),
    ("ES", "Spain"),
    ("IT", "Italy"),
    ("BR", "Brazil"),
    ("MX", "Mexico"),
    ("JP", "Japan"),
    ("KR", "South Korea"),
    ("IN", "India"),
];

fn static_youtube_regions() -> Vec<CatalogGenre> {
    YOUTUBE_REGIONS
        .iter()
        .map(|(id, name)| CatalogGenre {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

async fn youtube_popular(
    media_type: Option<&str>,
    region: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let region = region
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "US".to_string());
    let url = format!(
        "{}/trending?region={}",
        PIPED_BASE,
        urlencoding(&region)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload.as_array().cloned().unwrap_or_default();
    let limit: usize = 24;
    let total_pages = ((arr.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let kind = media_type.unwrap_or("video");
    let items: Vec<CatalogItem> = arr
        .iter()
        .skip(offset)
        .take(limit)
        .map(|item| youtube_to_item(item, kind))
        .collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn youtube_to_item(item: &serde_json::Value, kind: &str) -> CatalogItem {
    let want_channel = kind == "channel";
    let raw_id = if want_channel {
        item.get("uploaderUrl").and_then(|v| v.as_str()).map(|s| {
            s.trim_start_matches('/')
                .trim_start_matches("channel/")
                .to_string()
        })
    } else {
        item.get("url").and_then(|v| v.as_str()).map(|s| {
            s.split_once("v=")
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_else(|| s.trim_start_matches('/').to_string())
        })
    };
    let id = raw_id.unwrap_or_default();
    let title = if want_channel {
        item.get("uploaderName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let description = if want_channel {
        item.get("uploaderDescription")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        let uploader = item
            .get("uploaderName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let views = item.get("views").and_then(|v| v.as_i64());
        let mut parts: Vec<String> = Vec::new();
        if !uploader.is_empty() {
            parts.push(uploader.to_string());
        }
        if let Some(v) = views {
            parts.push(format!("{} views", v));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" · "))
        }
    };
    let poster_url = if want_channel {
        item.get("uploaderAvatar")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        item.get("thumbnail")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    CatalogItem {
        id,
        title,
        year: None,
        description,
        poster_url,
        backdrop_url: None,
    }
}

// ---------- IPTV ----------

const IPTV_CATEGORIES: &[&str] = &[
    "general",
    "news",
    "sports",
    "movies",
    "music",
    "kids",
    "documentary",
    "entertainment",
    "education",
    "religious",
    "lifestyle",
    "business",
    "weather",
    "travel",
    "comedy",
    "auto",
    "outdoor",
    "animation",
    "cooking",
    "science",
    "family",
    "classic",
    "legislative",
    "culture",
];

fn static_iptv_categories() -> Vec<CatalogGenre> {
    IPTV_CATEGORIES
        .iter()
        .map(|c| CatalogGenre {
            id: (*c).to_string(),
            name: capitalize_words(c),
        })
        .collect()
}

async fn iptv_popular(
    category: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let channels_url = format!("{}/channels.json", IPTV_ORG_BASE);
    let payload: serde_json::Value = http_get_json(
        &channels_url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload.as_array().cloned().unwrap_or_default();
    let category = category
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());
    let mut filtered: Vec<&serde_json::Value> = arr
        .iter()
        .filter(|ch| {
            let cats = ch
                .get("categories")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_lowercase()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            match &category {
                Some(c) => cats.iter().any(|x| x == c),
                None => true,
            }
        })
        .collect();
    filtered.sort_by(|a, b| {
        let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        an.cmp(bn)
    });
    let limit: usize = 24;
    let total_pages = ((filtered.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let items: Vec<CatalogItem> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(iptv_to_item)
        .collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn iptv_to_item(ch: &serde_json::Value) -> CatalogItem {
    let id = ch
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = ch
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let country = ch
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let cats = ch
        .get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let mut description = String::new();
    if !country.is_empty() {
        description.push_str(&country);
    }
    if !cats.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(&cats);
    }
    let poster_url = ch
        .get("logo")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    CatalogItem {
        id,
        title,
        year: None,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        poster_url,
        backdrop_url: None,
    }
}

// ---------- Radio ----------

const RADIO_TAGS: &[&str] = &[
    "pop",
    "rock",
    "news",
    "classical",
    "jazz",
    "country",
    "electronic",
    "hip hop",
    "talk",
    "dance",
    "metal",
    "indie",
    "folk",
    "blues",
    "reggae",
    "latin",
    "ambient",
    "oldies",
    "alternative",
    "sports",
];

fn static_radio_tags() -> Vec<CatalogGenre> {
    RADIO_TAGS
        .iter()
        .map(|t| CatalogGenre {
            id: (*t).to_string(),
            name: capitalize_words(t),
        })
        .collect()
}

async fn radio_popular(
    tag: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 24;
    let offset = (page - 1).max(0) * limit;
    let mut params: Vec<(&str, String)> = vec![
        ("limit", limit.to_string()),
        ("offset", offset.to_string()),
        ("order", "clickcount".to_string()),
        ("reverse", "true".to_string()),
        ("hidebroken", "true".to_string()),
    ];
    if let Some(t) = tag.filter(|s| !s.is_empty()) {
        params.push(("tag", t.to_string()));
    }
    let qs = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding(v)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!("{}/stations/search?{}", RADIO_BROWSER_BASE, qs);
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload.as_array().cloned().unwrap_or_default();
    // Radio Browser doesn't return a total count for /stations/search; assume more pages
    // exist while the current page is full.
    let total_pages = if (arr.len() as i64) < limit {
        page
    } else {
        page + 1
    };
    let items: Vec<CatalogItem> = arr.iter().map(radio_to_item).collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn radio_to_item(s: &serde_json::Value) -> CatalogItem {
    let id = s
        .get("stationuuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = s
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let country = s
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags = s
        .get("tags")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut description = String::new();
    if !country.is_empty() {
        description.push_str(&country);
    }
    if !tags.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(&tags);
    }
    let poster_url = s
        .get("favicon")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    CatalogItem {
        id,
        title,
        year: None,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        poster_url,
        backdrop_url: None,
    }
}

// ---------- helpers ----------

async fn http_get_json(
    url: &str,
    headers: &[(&str, &str)],
) -> Result<serde_json::Value, (StatusCode, Json<serde_json::Value>)> {
    let mut req = reqwest::Client::new().get(url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let res = req
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("upstream request failed: {e}")))?;
    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("upstream returned {}", res.status()),
        ));
    }
    res.json::<serde_json::Value>()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("upstream parse failed: {e}")))
}

fn capitalize_words(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
