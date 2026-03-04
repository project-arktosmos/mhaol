use crate::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_media))
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

    // Auto-link completed YouTube downloads to library items
    let completed = state.youtube_downloads.get_by_state("completed");
    for dl in &completed {
        if let Some(ref output_path) = dl.output_path {
            if let Some(item_id) = state.library_items.exists_by_path(output_path) {
                let existing = state
                    .library_item_links
                    .get_by_item_and_service(&item_id, "youtube");
                if existing.is_none() {
                    state.library_item_links.upsert(
                        &uuid::Uuid::new_v4().to_string(),
                        &item_id,
                        "youtube",
                        &dl.video_id,
                        None,
                        None,
                    );
                }
            }
        }
    }

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
            MappedMediaList {
                id: list.id,
                library_id: list.library_id,
                title: list.title,
                description: list.description,
                cover_image: list.cover_image,
                media_type: list.media_type,
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
    })
}
