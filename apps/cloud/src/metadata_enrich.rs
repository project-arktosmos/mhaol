//! Library-scan metadata enrichment.
//!
//! Each `MediaGroup` produced by [`crate::library_scan`] starts with bare
//! filename-derived metadata (title, optional year, file list). Before the
//! group is persisted as a firkin we hand it to this module, which:
//!
//! 1. Maps the group's `local-*` addon to the matching catalog source
//!    (`local-movie` → `tmdb-movie`, `local-album` → `musicbrainz`, …).
//! 2. Issues a search against that source using the extracted query.
//! 3. Scores every returned result against the same fields we extracted
//!    from the filename — title similarity (token-overlap on lowercased
//!    alphanumerics) plus year proximity — and keeps the best match if
//!    its score clears a confidence threshold.
//! 4. Returns the winning catalog item so the firkin can be persisted with
//!    the official title, description, year, and poster/backdrop images.
//!
//! Failures (no API key, upstream error, no matches above threshold) are
//! non-fatal: the caller falls back to the bare filename metadata.

#![cfg(not(target_os = "android"))]

use crate::catalog::{
    self, musicbrainz_search, retroachievements_search, tmdb_search, CatalogItem,
};
use crate::firkins::ImageMeta;

/// Search input extracted from filenames / directory names by the
/// individual detectors. `artist_hint` only matters for albums, where the
/// directory layout often encodes the artist (`<artist>/<album>/track.mp3`).
#[derive(Debug, Clone)]
pub struct ExtractedQuery {
    pub title: String,
    pub year: Option<i32>,
    pub artist_hint: Option<String>,
}

/// What the matched catalog result contributes to the firkin record.
#[derive(Debug, Clone, Default)]
pub struct EnrichedMetadata {
    pub title: Option<String>,
    pub year: Option<i32>,
    pub description: Option<String>,
    pub images: Vec<ImageMeta>,
    pub external_id: Option<String>,
}

const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const MIN_TITLE_SCORE: f32 = 0.5;

/// Map a `local-*` addon id to the catalog source we should search.
fn remote_addon_for(local_addon: &str) -> Option<&'static str> {
    match local_addon {
        "local-movie" => Some("tmdb-movie"),
        "local-tv" => Some("tmdb-tv"),
        "local-album" => Some("musicbrainz"),
        "local-game" => Some("retroachievements"),
        _ => None,
    }
}

/// Build the actual search string handed to the catalog. Albums prepend
/// the artist hint when we have one, since MusicBrainz's release-group
/// search is far more precise with `Artist - Album` than with the album
/// alone.
fn build_query(query: &ExtractedQuery, local_addon: &str) -> String {
    if local_addon == "local-album" {
        if let Some(artist) = query.artist_hint.as_ref().filter(|s| !s.is_empty()) {
            return format!("{artist} {}", query.title);
        }
    }
    query.title.clone()
}

/// Lowercase + keep only alphanumeric runs as tokens. Strips release-tag
/// noise (`1080p`, `BluRay`, `x264`, etc.) by intersecting with the other
/// side: tokens unique to a release-tag set will simply not match
/// anything in the catalog title.
fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Token-overlap similarity in [0.0, 1.0]. Symmetric Jaccard-ish: we count
/// how many of the *query* tokens appear in the candidate tokens, divided
/// by the size of the query token set. Catalog titles often carry extra
/// words ("The Lord of the Rings: The Fellowship of the Ring"), so we
/// penalise unmatched query tokens and ignore unmatched catalog tokens.
fn title_similarity(query_tokens: &[String], candidate: &str) -> f32 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let candidate_tokens = tokenize(candidate);
    if candidate_tokens.is_empty() {
        return 0.0;
    }
    let mut matched = 0usize;
    for q in query_tokens {
        if candidate_tokens.iter().any(|c| c == q) {
            matched += 1;
        }
    }
    matched as f32 / query_tokens.len() as f32
}

/// Year-proximity bonus: exact match → +0.3, off-by-one → +0.1, otherwise
/// 0.0. Off-by-one tolerance covers TMDB's release-date vs. user's "year
/// from filename" disagreements (theatrical vs. limited release, etc.).
fn year_bonus(query_year: Option<i32>, candidate_year: Option<i32>) -> f32 {
    match (query_year, candidate_year) {
        (Some(q), Some(c)) if q == c => 0.3,
        (Some(q), Some(c)) if (q - c).abs() == 1 => 0.1,
        _ => 0.0,
    }
}

fn score(query_tokens: &[String], query_year: Option<i32>, candidate: &CatalogItem) -> f32 {
    title_similarity(query_tokens, &candidate.title) + year_bonus(query_year, candidate.year)
}

fn pick_best<'a>(
    query: &ExtractedQuery,
    items: &'a [CatalogItem],
) -> Option<&'a CatalogItem> {
    let query_tokens = tokenize(&query.title);
    if query_tokens.is_empty() || items.is_empty() {
        return None;
    }
    items
        .iter()
        .map(|c| (c, score(&query_tokens, query.year, c)))
        .filter(|(_, s)| *s >= MIN_TITLE_SCORE)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| c)
}

/// Build an `images` list from a catalog item's poster/backdrop URLs.
/// The catalog already produces full URLs (TMDB's `w500` poster, MB's
/// cover-art-archive `front-500`, RA badge), so we just wrap them in
/// `ImageMeta` records — width/height come from the known TMDB sizes
/// when applicable.
fn images_from(catalog_item: &CatalogItem) -> Vec<ImageMeta> {
    let mut out: Vec<ImageMeta> = Vec::new();
    if let Some(url) = catalog_item.poster_url.as_ref().filter(|s| !s.is_empty()) {
        let (w, h) = if url.starts_with(&format!("{TMDB_IMG_BASE}/w500")) {
            (500, 750)
        } else {
            (0, 0)
        };
        out.push(ImageMeta {
            url: url.clone(),
            mime_type: "image/jpeg".to_string(),
            file_size: 0,
            width: w,
            height: h,
        });
    }
    if let Some(url) = catalog_item.backdrop_url.as_ref().filter(|s| !s.is_empty()) {
        let (w, h) = if url.starts_with(&format!("{TMDB_IMG_BASE}/w1280")) {
            (1280, 720)
        } else {
            (0, 0)
        };
        out.push(ImageMeta {
            url: url.clone(),
            mime_type: "image/jpeg".to_string(),
            file_size: 0,
            width: w,
            height: h,
        });
    }
    out
}

async fn search_catalog(
    remote_addon: &str,
    query: &str,
) -> Option<Vec<CatalogItem>> {
    let result = match remote_addon {
        "tmdb-movie" => tmdb_search(false, query, 1).await,
        "tmdb-tv" => tmdb_search(true, query, 1).await,
        "musicbrainz" => musicbrainz_search(query, 1).await,
        "retroachievements" => retroachievements_search(query, None, 1).await,
        _ => return None,
    };
    match result {
        Ok(page) => Some(page.items),
        Err((status, body)) => {
            tracing::warn!(
                "[metadata-enrich] {remote_addon} search failed: {status} {body:?}"
            );
            None
        }
    }
}

/// Given a media group's extracted query and its `local-*` addon id, ask
/// the matching catalog source for matches and return the best one
/// (title-similarity ≥ {MIN_TITLE_SCORE} + optional year bonus). Returns
/// `None` when there's no remote addon for the kind, when the search
/// fails, or when no candidate clears the threshold.
pub async fn enrich(
    query: &ExtractedQuery,
    local_addon: &str,
) -> Option<EnrichedMetadata> {
    let remote_addon = remote_addon_for(local_addon)?;
    if !catalog::is_known_addon(remote_addon) {
        return None;
    }
    let q = build_query(query, local_addon);
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return None;
    }
    let items = search_catalog(remote_addon, trimmed).await?;
    let best = pick_best(query, &items)?;
    Some(EnrichedMetadata {
        title: Some(best.title.clone()).filter(|s| !s.is_empty()),
        year: best.year.or(query.year),
        description: best
            .description
            .clone()
            .filter(|s: &String| !s.is_empty()),
        images: images_from(best),
        external_id: Some(best.id.clone()).filter(|s| !s.is_empty()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, title: &str, year: Option<i32>) -> CatalogItem {
        CatalogItem {
            id: id.to_string(),
            title: title.to_string(),
            year,
            description: Some(format!("about {title}")),
            poster_url: None,
            backdrop_url: None,
        }
    }

    #[test]
    fn title_similarity_matches_release_tag_noise() {
        // "Spider-Man.No.Way.Home.2021.1080p.BluRay.x264" against TMDB's
        // "Spider-Man: No Way Home" should score very high — the query
        // tokens that exist in the candidate (spider, man, no, way, home)
        // are the load-bearing ones; release tags (1080p, bluray, x264,
        // 2021) simply don't match catalog tokens.
        let q = tokenize("Spider-Man.No.Way.Home.2021.1080p.BluRay.x264");
        let s = title_similarity(&q, "Spider-Man: No Way Home");
        // 5 of the 9 query tokens land (spider, man, no, way, home).
        assert!(s >= 0.5, "got {s}");
    }

    #[test]
    fn pick_best_prefers_year_match() {
        let q = ExtractedQuery {
            title: "Spider-Man".to_string(),
            year: Some(2021),
            artist_hint: None,
        };
        let items = vec![
            item("1", "Spider-Man", Some(2002)),
            item("2", "Spider-Man: No Way Home", Some(2021)),
            item("3", "Spider-Man: Homecoming", Some(2017)),
        ];
        let best = pick_best(&q, &items).expect("a match");
        assert_eq!(best.id, "2");
    }

    #[test]
    fn pick_best_rejects_low_similarity() {
        let q = ExtractedQuery {
            title: "Foobarbaz".to_string(),
            year: None,
            artist_hint: None,
        };
        let items = vec![item("1", "Totally Different", Some(2020))];
        assert!(pick_best(&q, &items).is_none());
    }

    #[test]
    fn build_query_includes_album_artist_hint() {
        let q = ExtractedQuery {
            title: "The Wall".to_string(),
            year: Some(1979),
            artist_hint: Some("Pink Floyd".to_string()),
        };
        assert_eq!(build_query(&q, "local-album"), "Pink Floyd The Wall");
        assert_eq!(build_query(&q, "local-movie"), "The Wall");
    }
}
