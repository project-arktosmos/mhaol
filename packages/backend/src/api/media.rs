use crate::AppState;
use axum::{
    extract::{Path as AxumPath, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_media))
        .route("/library-items/{id}/related", get(get_library_item_related))
}

#[derive(Serialize)]
struct MappedLink {
    #[serde(rename = "serviceId")]
    service_id: String,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
    #[serde(rename = "episodeNumber")]
    episode_number: Option<i64>,
}

#[derive(Serialize)]
struct MappedItem {
    id: String,
    #[serde(rename = "libraryId")]
    library_id: String,
    name: String,
    extension: String,
    path: String,
    #[serde(rename = "categoryId")]
    category_id: Option<String>,
    #[serde(rename = "mediaTypeId")]
    media_type_id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    links: HashMap<String, MappedLink>,
}

#[derive(Serialize)]
struct MappedMediaType {
    id: String,
    label: String,
}

#[derive(Serialize)]
struct MappedCategory {
    id: String,
    #[serde(rename = "mediaTypeId")]
    media_type_id: String,
    label: String,
}

#[derive(Serialize)]
struct MappedLinkSource {
    id: String,
    service: String,
    label: String,
    #[serde(rename = "mediaTypeId")]
    media_type_id: String,
    #[serde(rename = "categoryId")]
    category_id: Option<String>,
}

#[derive(Serialize)]
struct MappedMediaList {
    id: String,
    #[serde(rename = "libraryId")]
    library_id: String,
    title: String,
    description: Option<String>,
    #[serde(rename = "coverImage")]
    cover_image: Option<String>,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(rename = "libraryType")]
    library_type: String,
    source: String,
    #[serde(rename = "itemCount")]
    item_count: usize,
    #[serde(rename = "createdAt")]
    created_at: String,
    links: HashMap<String, MappedMediaListLink>,
    items: Vec<MappedItem>,
}

#[derive(Serialize)]
struct MappedMediaListLink {
    #[serde(rename = "serviceId")]
    service_id: String,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
}

#[derive(Serialize)]
struct MediaResponse {
    #[serde(rename = "mediaTypes")]
    media_types: Vec<MappedMediaType>,
    categories: Vec<MappedCategory>,
    #[serde(rename = "linkSources")]
    link_sources: Vec<MappedLinkSource>,
    #[serde(rename = "itemsByCategory")]
    items_by_category: HashMap<String, Vec<MappedItem>>,
    #[serde(rename = "itemsByType")]
    items_by_type: HashMap<String, Vec<MappedItem>>,
    lists: Vec<MappedMediaList>,
    /// Map of library id → library info (name + type)
    libraries: HashMap<String, MappedLibraryInfo>,
}

#[derive(Serialize)]
struct MappedLibraryInfo {
    name: String,
    #[serde(rename = "type")]
    library_type: String,
}

async fn get_media(State(state): State<AppState>) -> impl IntoResponse {
    let media_types: Vec<MappedMediaType> = state
        .media_types
        .get_all()
        .into_iter()
        .map(|mt| MappedMediaType {
            id: mt.id,
            label: mt.label,
        })
        .collect();

    let categories: Vec<MappedCategory> = state
        .categories
        .get_all()
        .into_iter()
        .map(|c| MappedCategory {
            id: c.id,
            media_type_id: c.media_type_id,
            label: c.label,
        })
        .collect();

    let link_sources: Vec<MappedLinkSource> = state
        .link_sources
        .get_all()
        .into_iter()
        .map(|ls| MappedLinkSource {
            id: ls.id,
            service: ls.service,
            label: ls.label,
            media_type_id: ls.media_type_id,
            category_id: ls.category_id,
        })
        .collect();


    let map_rows = |rows: Vec<crate::db::repo::library_item::LibraryItemRow>,
                    media_type_id: &str|
     -> Vec<MappedItem> {
        rows.into_iter()
            .map(|r| {
                let link_rows = state.library_item_links.get_by_item(&r.id);
                let mut links = HashMap::new();
                for link in link_rows {
                    links.insert(
                        link.service,
                        MappedLink {
                            service_id: link.service_id,
                            season_number: link.season_number,
                            episode_number: link.episode_number,
                        },
                    );
                }
                let name = Path::new(&r.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                MappedItem {
                    id: r.id,
                    library_id: r.library_id,
                    name,
                    extension: r.extension,
                    path: r.path,
                    category_id: r.category_id,
                    media_type_id: media_type_id.to_string(),
                    created_at: r.created_at,
                    links,
                }
            })
            .collect()
    };

    let mut items_by_category: HashMap<String, Vec<MappedItem>> = HashMap::new();
    for cat in &categories {
        let mut rows = state.library_items.get_by_category(&cat.id);
        if cat.id.ends_with("-uncategorized") {
            let uncategorized = state
                .library_items
                .get_uncategorized_by_media_type(&cat.media_type_id);
            rows.extend(uncategorized);
        }
        items_by_category.insert(cat.id.clone(), map_rows(rows, &cat.media_type_id));
    }

    let mut items_by_type: HashMap<String, Vec<MappedItem>> = HashMap::new();
    for mt in &media_types {
        let rows = state.library_items.get_by_media_type(&mt.id);
        items_by_type.insert(mt.id.clone(), map_rows(rows, &mt.id));
    }

    // Build library info map: id → { name, type }
    let lib_type_map: HashMap<String, MappedLibraryInfo> = state
        .libraries
        .get_all()
        .into_iter()
        .map(|lib| {
            let types: Vec<String> = serde_json::from_str(&lib.media_types).unwrap_or_default();
            let raw = types.into_iter().next().unwrap_or_else(|| "movies".to_string());
            // Normalize "video" → "movies" so the flix UI filters work correctly
            let lt = if raw == "video" { "movies".to_string() } else { raw };
            (lib.id, MappedLibraryInfo { name: lib.name, library_type: lt })
        })
        .collect();

    let all_lists = state.media_lists.get_all();
    let lists: Vec<MappedMediaList> = all_lists
        .into_iter()
        .map(|list| {
            let list_items = state.media_list_items.get_by_list(&list.id);
            let items: Vec<MappedItem> = list_items
                .iter()
                .filter_map(|li| {
                    let r = state.library_items.get(&li.library_item_id)?;
                    let link_rows = state.library_item_links.get_by_item(&r.id);
                    let mut links = HashMap::new();
                    for link in link_rows {
                        links.insert(
                            link.service,
                            MappedLink {
                                service_id: link.service_id,
                                season_number: link.season_number,
                                episode_number: link.episode_number,
                            },
                        );
                    }
                    let name = Path::new(&r.path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    Some(MappedItem {
                        id: r.id,
                        library_id: r.library_id,
                        name,
                        extension: r.extension,
                        path: r.path,
                        category_id: r.category_id,
                        media_type_id: list.media_type.clone(),
                        created_at: r.created_at,
                        links,
                    })
                })
                .collect();
            let item_count = items.len();
            let list_link_rows = state.media_list_links.get_by_list(&list.id);
            let mut list_links = HashMap::new();
            for ll in list_link_rows {
                list_links.insert(
                    ll.service,
                    MappedMediaListLink {
                        service_id: ll.service_id,
                        season_number: ll.season_number,
                    },
                );
            }
            let library_type = lib_type_map
                .get(&list.library_id)
                .map(|info| info.library_type.clone())
                .unwrap_or_else(|| "movies".to_string());
            MappedMediaList {
                id: list.id,
                library_id: list.library_id,
                title: list.title,
                description: list.description,
                cover_image: list.cover_image,
                media_type: list.media_type,
                library_type,
                source: list.source,
                item_count,
                created_at: list.created_at,
                links: list_links,
                items,
            }
        })
        .collect();

    Json(MediaResponse {
        media_types,
        categories,
        link_sources,
        items_by_category,
        items_by_type,
        lists,
        libraries: lib_type_map,
    })
}

#[derive(Serialize)]
struct RelatedLibrary {
    id: String,
    name: String,
    path: String,
    #[serde(rename = "mediaTypes")]
    media_types: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Serialize)]
struct RelatedLink {
    id: String,
    service: String,
    #[serde(rename = "serviceId")]
    service_id: String,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
    #[serde(rename = "episodeNumber")]
    episode_number: Option<i64>,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Serialize)]
struct RelatedTorrentDownload {
    #[serde(rename = "infoHash")]
    info_hash: String,
    name: String,
    size: i64,
    progress: f64,
    state: String,
    #[serde(rename = "downloadSpeed")]
    download_speed: i64,
    #[serde(rename = "uploadSpeed")]
    upload_speed: i64,
    peers: i64,
    seeds: i64,
    #[serde(rename = "addedAt")]
    added_at: i64,
    eta: Option<i64>,
    #[serde(rename = "outputPath")]
    output_path: Option<String>,
    source: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Serialize)]
struct RelatedFetchCache {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    #[serde(rename = "mediaType")]
    media_type: String,
    candidate: serde_json::Value,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Serialize)]
struct RelatedTmdbCache {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    data: serde_json::Value,
    #[serde(rename = "fetchedAt")]
    fetched_at: String,
}

#[derive(Serialize)]
struct LibraryItemRelatedResponse {
    library: Option<RelatedLibrary>,
    links: Vec<RelatedLink>,
    #[serde(rename = "fetchCache")]
    fetch_cache: Option<RelatedFetchCache>,
    #[serde(rename = "torrentDownload")]
    torrent_download: Option<RelatedTorrentDownload>,
    #[serde(rename = "tmdbCache")]
    tmdb_cache: Option<RelatedTmdbCache>,
}

async fn get_library_item_related(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    // Get the library item
    let item = match state.library_items.get(&id) {
        Some(item) => item,
        None => return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response(),
    };

    // Library
    let library = state.libraries.get(&item.library_id).map(|lib| RelatedLibrary {
        id: lib.id,
        name: lib.name,
        path: lib.path,
        media_types: lib.media_types,
        created_at: lib.created_at,
    });

    // Links
    let link_rows = state.library_item_links.get_by_item(&id);
    let links: Vec<RelatedLink> = link_rows
        .into_iter()
        .map(|l| RelatedLink {
            id: l.id,
            service: l.service,
            service_id: l.service_id,
            season_number: l.season_number,
            episode_number: l.episode_number,
            created_at: l.created_at,
        })
        .collect();

    // Find TMDB ID from links
    let tmdb_id: Option<i64> = links
        .iter()
        .find(|l| l.service == "tmdb")
        .and_then(|l| l.service_id.parse().ok());

    // Fetch cache (by TMDB ID)
    let fetch_cache = tmdb_id.and_then(|tid| {
        state.torrent_fetch_cache.get(tid).map(|fc| {
            let candidate: serde_json::Value =
                serde_json::from_str(&fc.candidate_json).unwrap_or(serde_json::Value::Null);
            RelatedFetchCache {
                tmdb_id: fc.tmdb_id,
                media_type: fc.media_type,
                candidate,
                created_at: fc.created_at,
            }
        })
    });

    // Torrent download (by info_hash from fetch cache candidate)
    let torrent_download = fetch_cache.as_ref().and_then(|fc| {
        fc.candidate
            .get("infoHash")
            .and_then(|v| v.as_str())
            .map(|h| h.to_lowercase())
            .and_then(|hash| {
                state.torrent_downloads.get(&hash).map(|td| RelatedTorrentDownload {
                    info_hash: td.info_hash,
                    name: td.name,
                    size: td.size,
                    progress: td.progress,
                    state: td.state,
                    download_speed: td.download_speed,
                    upload_speed: td.upload_speed,
                    peers: td.peers,
                    seeds: td.seeds,
                    added_at: td.added_at,
                    eta: td.eta,
                    output_path: td.output_path,
                    source: td.source,
                    created_at: td.created_at,
                    updated_at: td.updated_at,
                })
            })
    });

    // TMDB cache
    let tmdb_cache = tmdb_id.and_then(|tid| {
        let conn = state.db.lock();
        // Try movies first, then TV shows
        conn.query_row(
            "SELECT tmdb_id, data, fetched_at FROM tmdb_movies WHERE tmdb_id = ?1",
            rusqlite::params![tid],
            |row| {
                let tmdb_id: i64 = row.get(0)?;
                let data_str: String = row.get(1)?;
                let fetched_at: String = row.get(2)?;
                Ok((tmdb_id, data_str, fetched_at))
            },
        )
        .ok()
        .or_else(|| {
            conn.query_row(
                "SELECT tmdb_id, data, fetched_at FROM tmdb_tv_shows WHERE tmdb_id = ?1",
                rusqlite::params![tid],
                |row| {
                    let tmdb_id: i64 = row.get(0)?;
                    let data_str: String = row.get(1)?;
                    let fetched_at: String = row.get(2)?;
                    Ok((tmdb_id, data_str, fetched_at))
                },
            )
            .ok()
        })
        .map(|(tmdb_id, data_str, fetched_at)| {
            let data: serde_json::Value =
                serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            RelatedTmdbCache {
                tmdb_id,
                data,
                fetched_at,
            }
        })
    });

    Json(LibraryItemRelatedResponse {
        library,
        links,
        fetch_cache,
        torrent_download,
        tmdb_cache,
    })
    .into_response()
}
