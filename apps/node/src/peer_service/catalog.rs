use super::types::{
    CatalogMediaItem, CatalogMediaItemLink, CatalogMovie, DataChannelEnvelope, ServerCatalogMessage,
};
use crate::api::tmdb::tmdb_fetch_json;
use crate::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// Cached catalog that can be shared across peer connections.
pub type CatalogCache = Arc<RwLock<Option<Vec<CatalogMovie>>>>;

/// Create a new empty catalog cache.
pub fn new_cache() -> CatalogCache {
    Arc::new(RwLock::new(None))
}

/// Build the movie catalog from the database.
/// Always rebuilds because the streamable flag depends on live download state.
pub async fn get_or_build_catalog(state: &AppState, _cache: &CatalogCache) -> Vec<CatalogMovie> {
    let catalog = build_catalog(state).await;
    info!(count = catalog.len(), "Built movie catalog");
    catalog
}

/// Invalidate the catalog cache so it gets rebuilt on next request.
pub async fn invalidate(cache: &CatalogCache) {
    let mut guard = cache.write().await;
    *guard = None;
}

/// A path is considered a real file (not a virtual pinned entry).
fn is_real_path(path: &str) -> bool {
    !path.starts_with("pinned://")
}

/// Check whether a library item is actually available for streaming.
/// Checks in order:
/// 1. Direct torrent link → download record must be complete
/// 2. TMDB ID cross-reference → any completed download linked to the same TMDB ID
/// 3. File exists on disk at the stored path
fn is_available(
    state: &AppState,
    path: &str,
    links: &[crate::db::repo::library_item_link::LibraryItemLinkRow],
) -> bool {
    // 1. Direct torrent link check
    let torrent_link = links
        .iter()
        .find(|l| l.service == "torrent-download" || l.service == "torrent-stream");

    if let Some(link) = torrent_link {
        if let Some(row) = state.downloads.get(&link.service_id) {
            if row.state == "seeding" || row.progress >= 1.0 {
                return true;
            }
        }
    }

    // 2. Cross-reference: find any completed download linked to the same TMDB ID
    //    (handles items where the torrent link wasn't saved on the library item itself)
    let tmdb_link = links.iter().find(|l| l.service == "tmdb");
    if let Some(tmdb) = tmdb_link {
        let all_items_with_tmdb = state
            .library_item_links
            .get_by_service_id("tmdb", &tmdb.service_id);
        for sibling in &all_items_with_tmdb {
            let sibling_links = state
                .library_item_links
                .get_by_item(&sibling.library_item_id);
            for sl in &sibling_links {
                if sl.service == "torrent-download" || sl.service == "torrent-stream" {
                    if let Some(row) = state.downloads.get(&sl.service_id) {
                        if row.state == "seeding" || row.progress >= 1.0 {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: file exists on disk
    std::path::Path::new(path).exists()
}

/// Build the movie catalog from the database, including pinned movies.
/// Deduplicates by TMDB ID, preferring items with real file paths.
async fn build_catalog(state: &AppState) -> Vec<CatalogMovie> {
    // Gather movie items by category: library movies + pinned movies.
    // This avoids depending on inconsistent library type naming ("movies" vs "video")
    // and excludes pinned-tv episodes from the movie catalog.
    let mut all_items = state.library_items.get_by_category("movies");
    all_items.extend(state.library_items.get_by_category("pinned-movies"));

    // Deduplicate by TMDB ID, preferring streamable items with real file paths.
    // Items without a TMDB link are kept as-is.
    let mut seen_tmdb: HashMap<String, usize> = HashMap::new();
    let mut catalog: Vec<CatalogMovie> = Vec::new();

    for item in &all_items {
        let links = state.library_item_links.get_by_item(&item.id);
        let tmdb_link = links.iter().find(|l| l.service == "tmdb");

        if let Some(link) = tmdb_link {
            let tmdb_id = &link.service_id;
            if let Some(&existing_idx) = seen_tmdb.get(tmdb_id) {
                if !catalog[existing_idx].streamable && is_real_path(&item.path) {
                    let entry = build_catalog_entry(state, item, &links).await;
                    catalog[existing_idx] = entry;
                }
                continue;
            }
            seen_tmdb.insert(tmdb_id.clone(), catalog.len());
        }

        let entry = build_catalog_entry(state, item, &links).await;
        catalog.push(entry);
    }

    // Include completed torrent downloads that aren't already in the catalog.
    // This catches downloads that were never linked to library items.
    let torrent_downloads = state.downloads.get_by_type("torrent");
    for dl in torrent_downloads {
        if dl.state != "seeding" && dl.progress < 1.0 {
            continue;
        }
        let output_path = match &dl.output_path {
            Some(p) => p.clone(),
            None => continue,
        };
        let file_path = if !dl.name.is_empty() {
            format!("{}/{}", output_path, dl.name)
        } else {
            output_path
        };

        // Skip if already in catalog via a torrent link
        if seen_tmdb.values().any(|&idx| {
            catalog[idx]
                .item
                .links
                .get("torrent-download")
                .or(catalog[idx].item.links.get("torrent-stream"))
                .map_or(false, |l| l.service_id == dl.id)
        }) {
            continue;
        }

        let name = std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&dl.name)
            .to_string();

        let mut link_map = HashMap::new();
        link_map.insert(
            "torrent-download".to_string(),
            CatalogMediaItemLink {
                service_id: dl.id.clone(),
                service_url: None,
            },
        );

        let entry = CatalogMovie {
            item: CatalogMediaItem {
                id: format!("dl:{}", dl.id),
                library_id: String::new(),
                name,
                extension: String::new(),
                path: file_path,
                category_id: Some("movies".to_string()),
                media_type_id: "video".to_string(),
                created_at: dl.created_at.clone(),
                links: link_map,
            },
            tmdb: None,
            streamable: true,
        };
        catalog.push(entry);
    }

    catalog
}

/// Build a single catalog entry for a library item, resolving TMDB display data.
async fn build_catalog_entry(
    state: &AppState,
    item: &crate::db::repo::library_item::LibraryItemRow,
    links: &[crate::db::repo::library_item_link::LibraryItemLinkRow],
) -> CatalogMovie {
    let mut link_map = HashMap::new();
    let mut tmdb_display = None;

    for link in links {
        link_map.insert(
            link.service.clone(),
            CatalogMediaItemLink {
                service_id: link.service_id.clone(),
                service_url: None,
            },
        );

        if link.service == "tmdb" {
            if let Ok(tmdb_id) = link.service_id.parse::<i64>() {
                tmdb_display = resolve_tmdb_display(state, tmdb_id).await;
            }
        }
    }

    let name = std::path::Path::new(&item.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&item.path)
        .to_string();

    let streamable = is_real_path(&item.path) && is_available(state, &item.path, links);

    CatalogMovie {
        item: CatalogMediaItem {
            id: item.id.clone(),
            library_id: item.library_id.clone(),
            name,
            extension: item.extension.clone(),
            path: item.path.clone(),
            category_id: item.category_id.clone(),
            media_type_id: item.media_type.clone(),
            created_at: item.created_at.clone(),
            links: link_map,
        },
        tmdb: tmdb_display,
        streamable,
    }
}

/// Resolve TMDB data into the DisplayTMDBMovie format expected by the frontend.
async fn resolve_tmdb_display(state: &AppState, tmdb_id: i64) -> Option<serde_json::Value> {
    let data = tmdb_fetch_json(state, &format!("/movie/{}", tmdb_id), &[])
        .await
        .ok()?;

    let poster_path = data.get("poster_path").and_then(|p| p.as_str());
    let backdrop_path = data.get("backdrop_path").and_then(|p| p.as_str());
    let release_date = data
        .get("release_date")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let genres = data
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    g.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let release_year = release_date.split('-').next().unwrap_or("").to_string();

    Some(serde_json::json!({
        "id": tmdb_id,
        "title": data.get("title").and_then(|t| t.as_str()).unwrap_or(""),
        "originalTitle": data.get("original_title").and_then(|t| t.as_str()).unwrap_or(""),
        "releaseYear": release_year,
        "overview": data.get("overview").and_then(|o| o.as_str()).unwrap_or(""),
        "posterUrl": poster_path.map(|p| format!("{}/w342{}", TMDB_IMAGE_BASE, p)),
        "backdropUrl": backdrop_path.map(|p| format!("{}/w780{}", TMDB_IMAGE_BASE, p)),
        "voteAverage": data.get("vote_average").and_then(|v| v.as_f64()).unwrap_or(0.0),
        "voteCount": data.get("vote_count").and_then(|v| v.as_i64()).unwrap_or(0),
        "genres": genres,
    }))
}

/// Resolve a TMDB ID to a streamable file path by looking up library items in the database.
pub fn resolve_file_path_for_tmdb(state: &AppState, tmdb_id: i64) -> Option<String> {
    let links = state
        .library_item_links
        .get_by_service_id("tmdb", &tmdb_id.to_string());

    for link in &links {
        if let Some(item) = state.library_items.get(&link.library_item_id) {
            if is_real_path(&item.path) {
                return Some(item.path);
            }
        }
    }
    None
}

/// Build a catalog-start envelope that tells the client to clear its state.
pub fn build_catalog_start_envelope(count: usize) -> DataChannelEnvelope {
    let msg = ServerCatalogMessage::CatalogStart { count };
    DataChannelEnvelope {
        channel: "server-catalog".to_string(),
        payload: serde_json::to_value(msg).unwrap(),
    }
}

/// Build a catalog-movies envelope for a single movie.
pub fn build_single_movie_envelope(movie: &CatalogMovie) -> DataChannelEnvelope {
    let msg = ServerCatalogMessage::CatalogMovies {
        movies: vec![movie.clone()],
    };
    DataChannelEnvelope {
        channel: "server-catalog".to_string(),
        payload: serde_json::to_value(msg).unwrap(),
    }
}

/// Parse a server-catalog message from a data channel envelope.
pub fn parse_catalog_message(envelope: &DataChannelEnvelope) -> Option<ServerCatalogMessage> {
    if envelope.channel != "server-catalog" {
        return None;
    }
    serde_json::from_value(envelope.payload.clone()).ok()
}
