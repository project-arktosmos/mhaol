//! Background TV-show firkin builder for `local-tv` libraries. Given a
//! parsed scan group from the WebUI — show name, optional year, and a
//! flat list of `{ path, season, episode }` files — match the show to
//! TMDB, fetch every season + episode, mint a `tmdb-tv` firkin with
//! every episode as a `url` placeholder, then roll the firkin forward
//! once per local-file IPFS pin so each `(season, episode)` entry
//! becomes an `ipfs` CID as soon as that file's hash lands.
//!
//! The firkin's `files` array carries one entry per TMDB episode:
//!   - `ipfs` with the local CID when the library has a file for that
//!     episode and the pin has landed,
//!   - `url` pointing at the per-episode TMDB URL otherwise.
//!
//! That way the resulting firkin records both what's available locally
//! and what's missing, while the TV-show detail page can re-fetch the
//! full season / episode metadata from `/api/catalog/tmdb-tv/...` since
//! the firkin's canonical `https://www.themoviedb.org/tv/<id>` URL
//! identifies the show. The firkin is created up-front (right after the
//! metadata phase) so the WebUI can navigate to it before pins arrive;
//! every subsequent pin promotion rolls the version forward via
//! `firkins::rollforward_firkin`, pushing the prior CID onto
//! `version_hashes` and re-pinning the new body to IPFS.
//!
//! All side-effects run inside a `tokio::spawn`ed task so the HTTP
//! response returns immediately. Live state lives in
//! `state.tv_build_progress` (`TvBuildProgressMap`); the WebUI polls
//! `GET /api/libraries/:id/tv-builds` to render progress and
//! re-hydrate it across page reloads.

#![cfg(not(target_os = "android"))]

use crate::catalog::{
    fetch_tmdb_tv_season_episodes, fetch_tmdb_tv_seasons, tmdb_metadata, tmdb_search,
    CatalogEpisode, CatalogItem, CatalogSeason,
};
use crate::firkins::{
    create_firkin_record, rollforward_firkin, CreateFirkinRequest, FileEntry, Firkin, ImageMeta,
    Review, Trailer, TABLE as FIRKIN_TABLE,
};
use crate::ipfs_pins::IpfsPin;
use crate::state::CloudState;
use crate::tv_build_progress::{TvBuildPhase, TvBuildProgress};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use std::time::Duration;

/// Sliding window for the per-season episode fetch. TMDB tolerates
/// modest concurrency comfortably; 5 keeps a 30-season show well under
/// any per-IP cap while finishing the fan-out in a few hundred ms.
const SEASON_FETCH_CONCURRENCY: usize = 5;

/// How long a single per-file pin wait can take before the build gives
/// up on that episode and falls through to the URL placeholder. The
/// library scan task pins on a background loop, so this just ensures
/// that an unrecoverable pin failure doesn't block the whole build
/// indefinitely.
const PIN_WAIT_TIMEOUT: Duration = Duration::from_secs(120);

/// Polling interval while waiting for a pin to land. Cheap (a single
/// SurrealDB select against the `ipfs_pin` table).
const PIN_POLL_INTERVAL: Duration = Duration::from_millis(750);

#[derive(Debug, Deserialize)]
pub struct TvBuildFile {
    pub path: String,
    pub season: u32,
    pub episode: u32,
}

#[derive(Debug, Deserialize)]
pub struct StartTvBuildRequest {
    pub show: String,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub files: Vec<TvBuildFile>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/{id}/tv-build", post(start))
        .route("/{id}/tv-builds", get(list).delete(clear_terminal))
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

/// Stable per-show key, matching the shape the WebUI uses on its side.
/// `library_id::lowercase_show::year_or_empty` so the same show in the
/// same library can't have two concurrent build jobs.
fn job_key(library_id: &str, show: &str, year: Option<i32>) -> String {
    format!(
        "{}::{}::{}",
        library_id,
        show.trim().to_ascii_lowercase(),
        year.map(|y| y.to_string()).unwrap_or_default()
    )
}

async fn start(
    State(state): State<CloudState>,
    Path(library_id): Path<String>,
    Json(req): Json<StartTvBuildRequest>,
) -> Result<(StatusCode, Json<TvBuildProgress>), (StatusCode, Json<serde_json::Value>)> {
    let show = req.show.trim();
    if show.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "show is required"));
    }
    let key = job_key(&library_id, show, req.year);

    // Guard against concurrent kicks of the same show in the same library.
    // If a job is already running, surface its current progress as 200 so
    // the WebUI can re-hydrate without bouncing the user through an error.
    if let Some(existing) = state.tv_build_progress.get(&key) {
        if !existing.is_terminal() {
            return Ok((StatusCode::OK, Json(existing)));
        }
        // Terminal entry — clear it so the new run replaces it cleanly.
        state.tv_build_progress.remove(&key);
    }

    let now = Utc::now();
    let initial = TvBuildProgress {
        library_id: library_id.clone(),
        job_key: key.clone(),
        show: show.to_string(),
        year: req.year,
        phase: TvBuildPhase::Searching,
        message: Some("Searching TMDB…".to_string()),
        current: None,
        total: None,
        tmdb_id: None,
        tmdb_title: None,
        error: None,
        completed_firkin_id: None,
        started_at: now,
        updated_at: now,
    };
    state.tv_build_progress.insert(initial.clone());

    let task_state = state.clone();
    let task_show = show.to_string();
    let task_year = req.year;
    let task_files = req.files;
    let task_key = key.clone();
    let task_lib = library_id.clone();
    tokio::spawn(async move {
        run_job(
            task_state, task_lib, task_key, task_show, task_year, task_files,
        )
        .await;
    });

    Ok((StatusCode::ACCEPTED, Json(initial)))
}

async fn list(
    State(state): State<CloudState>,
    Path(library_id): Path<String>,
) -> Json<Vec<TvBuildProgress>> {
    state.tv_build_progress.gc(chrono::Duration::hours(1));
    Json(state.tv_build_progress.list_for_library(&library_id))
}

async fn clear_terminal(
    State(state): State<CloudState>,
    Path(library_id): Path<String>,
) -> StatusCode {
    let to_clear: Vec<String> = state
        .tv_build_progress
        .list_for_library(&library_id)
        .into_iter()
        .filter(|p| p.is_terminal())
        .map(|p| p.job_key)
        .collect();
    for k in to_clear {
        state.tv_build_progress.remove(&k);
    }
    StatusCode::NO_CONTENT
}

/// Pick the best TMDB-search candidate. TMDB orders results by relevance,
/// so we lean on that and only override when the parsed query carries a
/// year hint that picks out one specific candidate (a show + a reboot of
/// the same name share the title but never the year).
fn pick_best_tv_match(items: &[CatalogItem], year: Option<i32>) -> Option<&CatalogItem> {
    if items.is_empty() {
        return None;
    }
    if let Some(y) = year {
        if let Some(exact) = items.iter().find(|it| it.year == Some(y)) {
            return Some(exact);
        }
        if let Some(close) = items
            .iter()
            .find(|it| it.year.map(|cy| (cy - y).abs() <= 1).unwrap_or(false))
        {
            return Some(close);
        }
    }
    items.first()
}

/// Find the recorded CID for `path` in the `ipfs_pin` table. Returns
/// `None` when no pin exists yet — the caller polls on a fixed interval
/// until either a pin appears or the timeout elapses.
async fn pin_for_path(state: &CloudState, path: &str) -> Option<String> {
    let pins: Vec<IpfsPin> = match state.db.select(crate::ipfs_pins::TABLE).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("[tv-build] failed to read ipfs_pin table: {e}");
            return None;
        }
    };
    pins.into_iter().find(|p| p.path == path).map(|p| p.cid)
}

async fn wait_for_pin(state: &CloudState, path: &str) -> Option<String> {
    if let Some(cid) = pin_for_path(state, path).await {
        return Some(cid);
    }
    let start = std::time::Instant::now();
    while start.elapsed() < PIN_WAIT_TIMEOUT {
        tokio::time::sleep(PIN_POLL_INTERVAL).await;
        if let Some(cid) = pin_for_path(state, path).await {
            return Some(cid);
        }
    }
    None
}

fn fail(state: &CloudState, key: &str, message: impl Into<String>) {
    let msg = message.into();
    state.tv_build_progress.update(key, |p| {
        p.phase = TvBuildPhase::Error;
        p.error = Some(msg.clone());
        p.message = Some(msg);
    });
}

/// Drive a single build job through every phase, updating the shared
/// progress map at each transition. This is the function the spawned
/// task runs to completion; it never returns an error to a caller —
/// failures land on the progress map as `phase == Error`.
async fn run_job(
    state: CloudState,
    library_id: String,
    key: String,
    show: String,
    year: Option<i32>,
    files: Vec<TvBuildFile>,
) {
    let _ = library_id;

    // Phase 1 — search TMDB.
    let search_page = match tmdb_search(true, &show, 1).await {
        Ok(page) => page,
        Err((status, body)) => {
            tracing::warn!("[tv-build] tmdb search failed: {status} {body:?}");
            fail(&state, &key, format!("TMDB search failed: {status}"));
            return;
        }
    };
    let Some(matched) = pick_best_tv_match(&search_page.items, year).cloned() else {
        fail(&state, &key, format!("No TMDB result for \"{show}\""));
        return;
    };
    let tmdb_id = matched.id.clone();
    let tmdb_title = matched.title.clone();
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::FetchingSeasons;
        p.tmdb_id = Some(tmdb_id.clone());
        p.tmdb_title = Some(tmdb_title.clone());
        p.message = Some(format!(
            "Matched {}{} — fetching seasons…",
            tmdb_title,
            matched.year.map(|y| format!(" ({y})")).unwrap_or_default()
        ));
    });

    // Phase 2 — fetch the show's season list.
    let seasons = match fetch_tmdb_tv_seasons(&tmdb_id).await {
        Ok(s) => s,
        Err((status, _)) => {
            fail(&state, &key, format!("TMDB seasons fetch failed: {status}"));
            return;
        }
    };
    if seasons.is_empty() {
        fail(&state, &key, "TMDB returned no seasons for this show");
        return;
    }

    // Phase 3 — fetch episodes for every season, bounded concurrency.
    let total_seasons = seasons.len() as u32;
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::FetchingEpisodes;
        p.current = Some(0);
        p.total = Some(total_seasons);
        p.message = Some(format!("Fetching episodes (0/{total_seasons})…"));
    });

    let episodes_by_season =
        fetch_episodes_for_seasons(&state, &key, &tmdb_id, &seasons, total_seasons).await;
    if episodes_by_season.is_empty() {
        fail(
            &state,
            &key,
            "Failed to fetch episodes for any season from TMDB",
        );
        return;
    }

    // Phase 4 — fetch artists / trailers / reviews via the /metadata helper.
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::FetchingMetadata;
        p.current = None;
        p.total = None;
        p.message = Some("Fetching artists, trailers & reviews…".to_string());
    });
    let (artists, trailers, reviews) = match tmdb_metadata(true, &tmdb_id).await {
        Ok(t) => t,
        Err(_) => {
            // Metadata is non-essential — log and keep going so the firkin
            // still gets created with files + title.
            tracing::warn!("[tv-build] metadata fetch failed for {tmdb_id}; skipping");
            (Vec::new(), Vec::new(), Vec::new())
        }
    };

    // Phase 5 — build the initial firkin body with every episode as a
    // `url` placeholder pointing at the per-episode TMDB page. Local files
    // get promoted to `ipfs` entries one-by-one in the next phase as their
    // pins land. Building the body up-front lets us mint the firkin record
    // now (so the WebUI can navigate to it immediately) and roll the
    // version forward incrementally as hashes arrive.
    let mut firkin_files: Vec<FileEntry> = Vec::new();
    firkin_files.push(FileEntry {
        kind: "url".to_string(),
        value: format!("https://www.themoviedb.org/tv/{tmdb_id}"),
        title: Some("TMDB TV".to_string()),
    });
    for season in &seasons {
        let eps = match episodes_by_season.get(&season.season_number) {
            Some(e) => e,
            None => continue,
        };
        for ep in eps {
            let s = season.season_number.max(0) as u32;
            let e = ep.episode_number.max(0) as u32;
            let label = format!("S{:02}E{:02} — {}", s, e, ep.name);
            firkin_files.push(FileEntry {
                kind: "url".to_string(),
                value: format!(
                    "https://www.themoviedb.org/tv/{tmdb_id}/season/{}/episode/{}",
                    season.season_number, ep.episode_number
                ),
                title: Some(label),
            });
        }
    }

    // Phase 6 — create the firkin now. The body carries every episode as a
    // url placeholder; pin promotions in the next phase roll the version
    // forward per local file.
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::CreatingFirkin;
        p.current = None;
        p.total = None;
        p.message = Some("Creating firkin…".to_string());
    });

    let images: Vec<ImageMeta> = match matched.poster_url.clone() {
        Some(url) if !url.is_empty() => vec![ImageMeta {
            url,
            mime_type: "image/jpeg".to_string(),
            file_size: 0,
            width: 0,
            height: 0,
        }],
        _ => Vec::new(),
    };
    let upsert_artists = artists
        .into_iter()
        .map(|a| crate::artists::UpsertArtistRequest {
            name: a.name,
            role: a.role,
            image_url: a.image_url,
        })
        .collect();
    let create_trailers: Vec<Trailer> = trailers
        .into_iter()
        .map(|t| Trailer {
            youtube_url: t.youtube_url,
            label: t.label,
            language: t.language,
        })
        .collect();
    let create_reviews: Vec<Review> = reviews
        .into_iter()
        .map(|r| Review {
            label: r.label,
            score: r.score,
            max_score: r.max_score,
            vote_count: r.vote_count,
        })
        .collect();
    let req = CreateFirkinRequest {
        title: matched.title.clone(),
        artists: upsert_artists,
        description: Some(matched.description.clone().unwrap_or_default()),
        images,
        files: firkin_files,
        year: matched.year,
        addon: "tmdb-tv".to_string(),
        creator: None,
        trailers: create_trailers,
        reviews: create_reviews,
        bookmarked: Some(true),
    };

    let firkin_id = match create_firkin_record(&state, req).await {
        Ok((_, dto)) => dto.id,
        Err((status, body)) => {
            let msg = body
                .0
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("HTTP {status}"));
            tracing::warn!("[tv-build] firkin create failed: {msg}");
            fail(&state, &key, format!("Firkin create failed: {msg}"));
            return;
        }
    };
    tracing::info!(
        tmdb_id = %tmdb_id,
        firkin_id = %firkin_id,
        "[tv-build] firkin created — rolling forward as pins arrive",
    );

    // Surface the firkin id immediately so the WebUI can render a "View
    // firkin →" link during the pin window. The phase stays non-terminal
    // (`WaitingPins`) so the existing auto-nav-on-completion semantics
    // continue to wait until every pin has been processed.
    state.tv_build_progress.update(&key, |p| {
        p.completed_firkin_id = Some(firkin_id.clone());
    });

    // Phase 7 — wait for IPFS pins to settle, rolling the firkin forward
    // once per file so each (season, episode) entry flips from a `url`
    // placeholder to an `ipfs` CID as soon as the local hash lands. Pins
    // are awaited sequentially so the rollforwards never race; each one
    // pushes the prior CID onto `version_hashes` and re-pins the new body.
    let total_files = files.len() as u32;
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::WaitingPins;
        p.current = Some(0);
        p.total = Some(total_files);
        p.message = Some(format!("Waiting for IPFS pins (0/{total_files})…"));
    });
    for (i, file) in files.iter().enumerate() {
        if let Some(cid) = wait_for_pin(&state, &file.path).await {
            if let Err(e) = apply_pin_to_firkin(&state, &firkin_id, file, &cid).await {
                tracing::warn!(
                    "[tv-build] failed to roll firkin {} forward for {}: {}",
                    firkin_id,
                    file.path,
                    e,
                );
            }
        } else {
            tracing::warn!(
                "[tv-build] pin never landed for {}: leaving url placeholder",
                file.path
            );
        }
        let done = (i + 1) as u32;
        state.tv_build_progress.update(&key, |p| {
            p.current = Some(done);
            p.message = Some(format!("Waiting for IPFS pins ({done}/{total_files})…"));
        });
    }

    tracing::info!(
        tmdb_id = %tmdb_id,
        firkin_id = %firkin_id,
        "[tv-build] completed",
    );
    state.tv_build_progress.update(&key, |p| {
        p.phase = TvBuildPhase::Completed;
        p.message = Some(format!("Firkin built: {firkin_id}"));
    });
}

/// Roll a TV-show firkin forward to reflect a single local-file pin
/// landing. Loads the current firkin doc (so concurrent updates would be
/// preserved — though the caller awaits pins sequentially so there are
/// none in practice), rewrites the matching `(season, episode)` `url`
/// placeholder to an `ipfs` entry, then hands off to
/// `firkins::rollforward_firkin` which recomputes the CID, appends the
/// prior CID to `version_hashes`, persists the doc, and re-pins the new
/// body JSON. The `url` placeholder is matched by the title prefix
/// `S{season:02}E{episode:02} —` set when the initial firkin was built.
async fn apply_pin_to_firkin(
    state: &CloudState,
    firkin_id: &str,
    file: &TvBuildFile,
    cid: &str,
) -> Result<(), String> {
    // Hold the per-firkin lock across read-modify-write so a concurrent
    // mutation (subtitle attach, magnet pick, manual `PUT /api/firkins/:id`)
    // can't slip in between the load and the rollforward and have its
    // change silently overwritten.
    let _firkin_guard = state.firkin_lock(firkin_id).lock_owned().await;
    let current: Option<Firkin> = state
        .db
        .select((FIRKIN_TABLE, firkin_id))
        .await
        .map_err(|e| format!("db select failed: {e}"))?;
    let mut current =
        current.ok_or_else(|| format!("firkin {firkin_id} disappeared before pin update"))?;

    let prefix = format!("S{:02}E{:02} —", file.season, file.episode);
    let mut promoted = false;
    for entry in current.files.iter_mut() {
        let matches = entry.kind == "url"
            && entry
                .title
                .as_deref()
                .map(|t| t.starts_with(&prefix))
                .unwrap_or(false);
        if matches {
            entry.kind = "ipfs".to_string();
            entry.value = cid.to_string();
            promoted = true;
            break;
        }
    }
    if !promoted {
        // Either the episode entry was already an ipfs entry (re-run after
        // a previous build), or the (season, episode) numbers don't line
        // up with anything TMDB returned. Either way, no rollforward.
        return Ok(());
    }

    current.id = None;
    current.updated_at = Utc::now();
    rollforward_firkin(state, firkin_id, current)
        .await
        .map_err(|(status, body)| {
            body.0
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("HTTP {status}"))
        })?;
    Ok(())
}

/// Run the per-season episode fetches in a sliding-window loop bounded by
/// `SEASON_FETCH_CONCURRENCY`. Updates the progress map after each
/// completion so the WebUI sees a live "Fetching episodes (M/N)…" tick.
/// Returns the assembled `season_number → episodes` map; missing entries
/// are silently dropped — they re-surface as the "no episodes" gap when
/// the file builder iterates the seasons.
async fn fetch_episodes_for_seasons(
    state: &CloudState,
    key: &str,
    tmdb_id: &str,
    seasons: &[CatalogSeason],
    total: u32,
) -> std::collections::HashMap<i64, Vec<CatalogEpisode>> {
    use tokio::sync::Semaphore;
    let sem = std::sync::Arc::new(Semaphore::new(SEASON_FETCH_CONCURRENCY));
    let mut handles = Vec::with_capacity(seasons.len());
    for season in seasons {
        let permit_sem = sem.clone();
        let id = tmdb_id.to_string();
        let season_number = season.season_number;
        handles.push(tokio::spawn(async move {
            let _permit = match permit_sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return (season_number, None::<Vec<CatalogEpisode>>),
            };
            match fetch_tmdb_tv_season_episodes(&id, season_number).await {
                Ok(eps) => (season_number, Some(eps)),
                Err((status, _)) => {
                    tracing::warn!(
                        "[tv-build] season {season_number} episodes fetch failed: {status}"
                    );
                    (season_number, None)
                }
            }
        }));
    }

    let mut out: std::collections::HashMap<i64, Vec<CatalogEpisode>> =
        std::collections::HashMap::with_capacity(seasons.len());
    let mut done: u32 = 0;
    for h in handles {
        match h.await {
            Ok((season_number, Some(eps))) => {
                out.insert(season_number, eps);
            }
            Ok((_, None)) | Err(_) => {
                // skipped season; still counts toward progress
            }
        }
        done += 1;
        state.tv_build_progress.update(key, |p| {
            p.current = Some(done);
            p.message = Some(format!("Fetching episodes ({done}/{total})…"));
        });
    }
    out
}
