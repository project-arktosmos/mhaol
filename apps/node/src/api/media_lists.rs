use crate::api::smart_pair::{determine_confidence, parse_tv_candidate, score_match};
use crate::api::tmdb::tmdb_fetch_json;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{post, put},
    Json, Router,
};
use mhaol_queue::QueueEvent;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/{list_id}/tmdb",
            put(link_tmdb).delete(unlink_tmdb),
        )
        .route(
            "/{list_id}/musicbrainz",
            put(link_musicbrainz).delete(unlink_musicbrainz),
        )
        .route("/auto-match", post(auto_match))
}

#[derive(Deserialize)]
struct LinkTmdbBody {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
}

async fn link_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkTmdbBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "tmdb",
        &body.tmdb_id.to_string(),
        body.season_number,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "tmdb");
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct LinkMusicbrainzBody {
    #[serde(rename = "musicbrainzId")]
    musicbrainz_id: String,
}

async fn link_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkMusicbrainzBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    let mb_id = body.musicbrainz_id.trim();
    if mb_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "musicbrainzId must be a non-empty string" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "musicbrainz",
        mb_id,
        None,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "musicbrainz");
    Json(serde_json::json!({ "ok": true })).into_response()
}

// --- Auto-match endpoint ---

#[derive(Deserialize)]
struct AutoMatchRequest {
    lists: Vec<AutoMatchItem>,
}

#[derive(Deserialize)]
struct AutoMatchItem {
    #[serde(rename = "listId")]
    list_id: String,
    title: String,
}

#[derive(Serialize)]
struct AutoMatchResult {
    #[serde(rename = "listId")]
    list_id: String,
    matched: bool,
    #[serde(rename = "tmdbId")]
    tmdb_id: Option<i64>,
    #[serde(rename = "tmdbTitle")]
    tmdb_title: Option<String>,
    #[serde(rename = "tmdbYear")]
    tmdb_year: Option<String>,
    #[serde(rename = "tmdbPosterPath")]
    tmdb_poster_path: Option<String>,
    confidence: String,
}

struct ParsedFolderName {
    show_name: String,
    year: Option<String>,
}

/// Regex-based extraction of show name and year from torrent/folder names.
/// Handles patterns like:
///   "Archer (2009) Season 7 S07 (1080p NF WEB-DL ...)"
///   "Community.S01-S06.COMPLETE.SERIES.REPACK.1080p.Bluray.x265-HiQVE"
///   "[Anime Time] Attack On Titan (Complete Series) ..."
///   "Glee 2009 Season 1 Complete TVRip x264 [i_c]"
///   "evangelion_renewal_hd"
fn parse_folder_name(raw: &str) -> ParsedFolderName {
    static RE_LEADING_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[([^\]]*)\]\s*").unwrap());
    // Matches season markers: S01, Season 1, T01 (Spanish), Seasons 1-8, S01-S06, S01-04
    static RE_SEASON: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)(?:\b(?:seasons?\s*\d|s\d{2}|t\d{2})\b)").unwrap()
    });
    // Matches year in parens: (2009) or (2019)
    static RE_YEAR_PAREN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((\d{4})\)").unwrap());
    // Matches standalone year preceded by word boundary: "Glee 2009 Season"
    static RE_YEAR_BARE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b((?:19|20)\d{2})\b").unwrap());
    // Quality/codec/source markers that signal end of the show name
    static RE_QUALITY: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)\b(?:2160p|1080p|720p|480p|360p|4K|WEB[.-]?DL|WEBRip|BluRay|BDRip|DVDRip|HDTV|PDTV|HDRip|HEVC|x26[45]|H\.?26[45]|10bit|AAC|AC3|EAC3|DD5|DTS|FLAC|Mp4|MKV|REPACK|COMPLETE|Complete)\b").unwrap()
    });
    // Trailing release group: -GroupName or -GroupName[tag]
    static RE_TRAILING_GROUP: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"-[A-Za-z0-9]+(?:\[[^\]]*\])?$").unwrap());

    static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[[^\]]*\]").unwrap());
    static RE_PAREN_META: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)\((?:Complete[^)]*|Anime[^)]*|OVA[^)]*|Dual Audio|BD|Movie[^)]*)\)").unwrap()
    });

    let mut s = raw.to_string();

    // 1. Strip leading [Group] tag
    s = RE_LEADING_TAG.replace(&s, "").to_string();

    // 2. Extract year from parens before normalizing
    let year = RE_YEAR_PAREN
        .captures(&s)
        .map(|c| c[1].to_string());

    // 3. Remove parenthesized year from string
    if year.is_some() {
        s = RE_YEAR_PAREN.replace(&s, "").to_string();
    }

    // 4. Strip all remaining [bracket] metadata and known paren metadata
    s = RE_BRACKETS.replace_all(&s, "").to_string();
    s = RE_PAREN_META.replace_all(&s, "").to_string();

    // 5. Replace dots and underscores with spaces
    s = s.replace('.', " ").replace('_', " ");

    // 5. Find the earliest "cut point" — season marker or quality marker
    let season_pos = RE_SEASON.find(&s).map(|m| m.start());
    let quality_pos = RE_QUALITY.find(&s).map(|m| m.start());

    let cut = match (season_pos, quality_pos) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    let name_part = match cut {
        Some(pos) if pos > 0 => &s[..pos],
        _ => &s,
    };

    // 6. Clean up the name
    let mut name = name_part.trim().to_string();

    // Remove trailing group tag if still present
    name = RE_TRAILING_GROUP.replace(&name, "").trim().to_string();

    // Remove trailing brackets/parens content
    name = name.trim_end_matches(|c: char| c == '(' || c == '[' || c == ' ').to_string();

    // If no parenthesized year found, try bare year (but only if it's clearly a year, not part of the title)
    let year = year.or_else(|| {
        // Look for a bare year in the extracted name portion — only take it if it's at the end
        let re_year_end = Regex::new(r"\b((?:19|20)\d{2})\s*$").unwrap();
        if let Some(caps) = re_year_end.captures(&name) {
            let y = caps[1].to_string();
            name = name[..caps.get(0).unwrap().start()].trim().to_string();
            Some(y)
        } else {
            // Also check the original cut-off portion for a bare year
            RE_YEAR_BARE.captures(raw).map(|c| c[1].to_string())
        }
    });

    // Final cleanup — collapse multiple spaces
    let show_name = name.split_whitespace().collect::<Vec<_>>().join(" ");

    // If we ended up with nothing useful, return the raw string
    if show_name.is_empty() {
        return ParsedFolderName {
            show_name: raw.to_string(),
            year: None,
        };
    }

    ParsedFolderName {
        show_name,
        year,
    }
}

/// Submit an LLM extraction task and wait for the result via the broadcast channel.
/// Returns (showName, year_string) or None on failure/timeout.
#[cfg(not(target_os = "android"))]
async fn extract_show_info_via_llm(
    state: &AppState,
    folder_name: &str,
) -> Option<(String, Option<String>)> {
    if !state.llm_engine.is_model_loaded() {
        return None;
    }

    let task = state.queue.enqueue(
        "llm:extract-show-info",
        serde_json::json!({ "folderName": folder_name }),
    );
    let task_id = task.id.clone();

    let mut rx = state.queue.subscribe();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

    let result = loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(QueueEvent::TaskCompleted { task })) if task.id == task_id => {
                break task.result;
            }
            Ok(Ok(QueueEvent::TaskFailed { task })) if task.id == task_id => {
                warn!("[auto-match] LLM extraction failed for {:?}: {:?}", folder_name, task.error);
                break None;
            }
            Ok(Ok(_)) => continue,
            Ok(Err(_)) => break None,
            Err(_) => {
                warn!("[auto-match] LLM extraction timed out for {:?}", folder_name);
                break None;
            }
        }
    };

    let result = result?;
    let show_name = result.get("showName")?.as_str()?.to_string();
    if show_name.is_empty() {
        return None;
    }
    let year = result
        .get("year")
        .and_then(|v| v.as_u64())
        .map(|y| y.to_string());
    Some((show_name, year))
}

async fn auto_match(
    State(state): State<AppState>,
    Json(body): Json<AutoMatchRequest>,
) -> Response {
    let has_key = state
        .settings
        .get("tmdb.apiKey")
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    if !has_key {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "TMDB API key not configured" })),
        )
            .into_response();
    }

    let total = body.lists.len();
    info!("[auto-match] Matching {} TV show lists", total);

    let stream = async_stream::stream! {
        for (idx, item) in body.lists.iter().enumerate() {
            // 1. Regex-based extraction (always available, instant)
            let parsed = parse_folder_name(&item.title);
            let (mut search_query, mut year_filter) = (parsed.show_name, parsed.year);

            // 2. Try LLM enhancement if available (overrides regex result)
            #[cfg(not(target_os = "android"))]
            if let Some((llm_name, llm_year)) = extract_show_info_via_llm(&state, &item.title).await {
                search_query = llm_name;
                if llm_year.is_some() {
                    year_filter = llm_year;
                }
            }

            let match_title = search_query.clone();
            let mut tv_params: Vec<(&str, &str)> = vec![("query", &search_query), ("page", "1")];
            let yf_owned;
            if let Some(ref y) = year_filter {
                yf_owned = y.clone();
                tv_params.push(("first_air_date_year", &yf_owned));
            }

            let tv_res = tmdb_fetch_json(&state, "/search/tv", &tv_params).await;

            let mut best = None;
            let mut best_score: f64 = 0.0;

            if let Ok(data) = &tv_res {
                if let Some(results_arr) = data.get("results").and_then(|r| r.as_array()) {
                    for r in results_arr.iter().take(3) {
                        if let Some(candidate) = parse_tv_candidate(r) {
                            let s = score_match(&match_title, &candidate.title, candidate.popularity, candidate.vote_count);
                            if s > best_score {
                                best_score = s;
                                best = Some(candidate);
                            }
                        }
                    }
                }
            }

            let result = match &best {
                Some(c) => {
                    let confidence = determine_confidence(&match_title, &c.title, c.vote_count);
                    let should_link = confidence == "high" || confidence == "medium";

                    if should_link {
                        info!(
                            "[auto-match] ({}/{}) \"{}\" (cleaned: \"{}\") -> {} (id={}, confidence={})",
                            idx + 1, total, item.title, match_title, c.title, c.id, confidence
                        );
                        state.media_list_links.upsert(
                            &uuid::Uuid::new_v4().to_string(),
                            &item.list_id,
                            "tmdb",
                            &c.id.to_string(),
                            None,
                        );
                    } else {
                        info!(
                            "[auto-match] ({}/{}) \"{}\" (cleaned: \"{}\") -> {} (id={}, confidence={}, skipped)",
                            idx + 1, total, item.title, match_title, c.title, c.id, confidence
                        );
                    }

                    AutoMatchResult {
                        list_id: item.list_id.clone(),
                        matched: should_link,
                        tmdb_id: Some(c.id),
                        tmdb_title: Some(c.title.clone()),
                        tmdb_year: Some(c.year.clone()),
                        tmdb_poster_path: c.poster_path.clone(),
                        confidence,
                    }
                }
                None => {
                    info!("[auto-match] ({}/{}) \"{}\" (cleaned: \"{}\") -> no match", idx + 1, total, item.title, match_title);
                    AutoMatchResult {
                        list_id: item.list_id.clone(),
                        matched: false,
                        tmdb_id: None,
                        tmdb_title: None,
                        tmdb_year: None,
                        tmdb_poster_path: None,
                        confidence: "none".to_string(),
                    }
                }
            };

            let mut line = serde_json::to_string(&result).unwrap_or_default();
            line.push('\n');
            yield Ok::<_, std::convert::Infallible>(line);

            // Rate-limit every 15 items to stay under TMDB API limits
            if (idx + 1) % 15 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }

        info!("[auto-match] Done matching {} lists", total);
    };

    Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .body(Body::from_stream(stream))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse(input: &str, expected_name: &str, expected_year: Option<&str>) {
        let parsed = parse_folder_name(input);
        assert_eq!(
            parsed.show_name, expected_name,
            "show_name mismatch for input: {input}"
        );
        assert_eq!(
            parsed.year.as_deref(),
            expected_year,
            "year mismatch for input: {input}"
        );
    }

    #[test]
    fn test_parse_show_with_year_and_season() {
        assert_parse(
            "Archer (2009) Season 7 S07 (1080p NF WEB-DL x265 HEVC 10bit AC3 5.1 RZeroX)",
            "Archer",
            Some("2009"),
        );
    }

    #[test]
    fn test_parse_show_season_no_year() {
        assert_parse("Archer Season 1  (1080p H265 Joy)", "Archer", None);
    }

    #[test]
    fn test_parse_dotted_with_season_range() {
        assert_parse(
            "Community.S01-S06.COMPLETE.SERIES.REPACK.1080p.Bluray.x265-HiQVE",
            "Community",
            None,
        );
    }

    #[test]
    fn test_parse_dotted_single_season() {
        assert_parse(
            "Cunk.On.Earth.S01.1080p.WEBRip.x265[eztv.re]",
            "Cunk On Earth",
            None,
        );
    }

    #[test]
    fn test_parse_dotted_rick_and_morty() {
        assert_parse(
            "Rick.and.Morty.S07.COMPLETE.1080p.HMAX.WEB-DL.DD5.1.x264-NTb[TGx]",
            "Rick and Morty",
            None,
        );
    }

    #[test]
    fn test_parse_anime_with_group_tag() {
        assert_parse(
            "[Anime Time] Attack On Titan (Complete Series) (S01-S04+OVA) [Dual Audio][BD][1080p][HEVC 10bit x265][AAC][Eng Sub]",
            "Attack On Titan",
            None,
        );
    }

    #[test]
    fn test_parse_anime_with_year() {
        assert_parse(
            "[Anime Time] Fullmetal Alchemist (2003) (Anime+Movie) [Dual Audio][BD][1080p][HEVC 10bit x265][AAC][Eng Sub]",
            "Fullmetal Alchemist",
            Some("2003"),
        );
    }

    #[test]
    fn test_parse_bare_year_before_season() {
        assert_parse(
            "Glee 2009 Season 1 Complete TVRip x264 [i_c]",
            "Glee",
            Some("2009"),
        );
    }

    #[test]
    fn test_parse_bare_year_multi_season() {
        assert_parse(
            "Brooklyn Nine Nine 2013 Seasons 1 to 8 Complete 1080p BluRay x264 [i_c]",
            "Brooklyn Nine Nine",
            Some("2013"),
        );
    }

    #[test]
    fn test_parse_underscore_name() {
        assert_parse("evangelion_renewal_hd", "evangelion renewal hd", None);
    }

    #[test]
    fn test_parse_mr_robot() {
        assert_parse(
            "Mr.Robot.Season.1-4.S01-04.COMPLETE.1080p.BluRay.WEB.10bit.DD5.1.x265-POIASD",
            "Mr Robot",
            None,
        );
    }

    #[test]
    fn test_parse_show_with_year_and_season_abbott() {
        assert_parse(
            "Abbott Elementary (2021) Season 2 S02 (1080p AMZN WEB-DL x265 HEVC 10bit EAC3 5.1 Silence)",
            "Abbott Elementary",
            Some("2021"),
        );
    }

    #[test]
    fn test_parse_simple_with_season_and_mp4() {
        assert_parse("Andor Season 2 Mp4 1080p", "Andor", None);
    }

    #[test]
    fn test_parse_spanish_show() {
        assert_parse(
            "Aqui no hay quien viva T01-T05 Spanish DVDRip XviD",
            "Aqui no hay quien viva",
            None,
        );
    }

    #[test]
    fn test_parse_the_boys() {
        assert_parse("The Boys Season 1 Mp4 1080p", "The Boys", None);
    }

    #[test]
    fn test_parse_wwdits_year_and_season() {
        assert_parse(
            "What We Do in the Shadows (2019) Season 2 S02 (1080p HULU WEB-DL x265 HEVC 10bit EAC3 5.1 Ghost)",
            "What We Do in the Shadows",
            Some("2019"),
        );
    }

    #[test]
    fn test_parse_wwdits_dotted() {
        assert_parse(
            "What.We.Do.In.the.Shadows.Season.6.COMPLETE.1080p.DSNP.WEB-DL.x264.ESubs.[4GB].[MP4].[S06.Full]-[y2flix]",
            "What We Do In the Shadows",
            None,
        );
    }

    #[test]
    fn test_parse_fleabag_multiseason() {
        assert_parse(
            "Fleabag (2016) Season 1-2 S01-S02 (1080p BluRay x265 HEVC 10bit AAC 2.0 RZeroX)",
            "Fleabag",
            Some("2016"),
        );
    }

    #[test]
    fn test_parse_severance() {
        assert_parse("Severance Season 1 Mp4 1080p", "Severance", None);
    }

    #[test]
    fn test_parse_reaktor_group() {
        assert_parse(
            "[Reaktor] Fullmetal Alchemist Brotherhood + OVA Complete v2 [1080p][x265][10-bit][Dual-Audio]",
            "Fullmetal Alchemist Brotherhood + OVA",
            None,
        );
    }

    #[test]
    fn test_parse_bojack() {
        assert_parse("BoJack Horseman", "BoJack Horseman", None);
    }

    #[test]
    fn test_parse_superstore_dotted() {
        assert_parse(
            "Superstore.S01-S06.Season.01-06.COMPLETE.WEBRip.720p.x265.HEVC",
            "Superstore",
            None,
        );
    }
}
