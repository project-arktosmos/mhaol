//! Per-file TMDB match for library scans of `local-movie` libraries. Given a
//! video file's relative path, parse a `(title, year)` query out of its
//! filename / parent directory, hit TMDB's `/search/movie` endpoint, and
//! score the results by token overlap + year proximity to pick the best
//! candidate. The result is attached to the scan entry so the WebUI can
//! show "this video file matched to <TMDB title> (year)" in the libraries
//! table.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";

/// Bounded concurrency for the per-file matching loop. TMDB is generous
/// (~50 req/s) but there's no point hammering it with hundreds of parallel
/// requests for a large library; 5 keeps latency low and well under any
/// per-IP cap.
const MATCH_CONCURRENCY: usize = 5;

/// Words that are filename noise but not part of the movie's actual name —
/// dropped from both the query side and the candidate side before token
/// matching so a release tag like `1080p` doesn't drown out the real title.
const NOISE_WORDS: &[&str] = &[
    "1080p", "2160p", "720p", "480p", "4k", "uhd", "hdr", "dv", "bluray", "brrip", "bdrip",
    "webrip", "web", "dl", "webdl", "hdrip", "dvdrip", "x264", "x265", "h264", "h265", "hevc",
    "aac", "ac3", "dts", "ddp", "atmos", "remux", "proper", "repack", "extended", "directors",
    "cut", "uncut", "imax", "remastered", "edition",
];

static YEAR_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((\d{4})\)").unwrap());
static BARE_YEAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|[\s._-])(\d{4})(?:[\s._-]|$)").unwrap());

/// Returns the matched substring of the first `BARE_YEAR_RE` group, or
/// `None` if no four-digit year is present. Used so we can both extract the
/// year *and* trim everything after it (typical scene-release filenames put
/// the year right where the title ends).
fn first_year(s: &str) -> Option<(usize, i32)> {
    let caps = BARE_YEAR_RE.captures(s)?;
    let m = caps.get(1)?;
    let year: i32 = m.as_str().parse().ok()?;
    if !(1888..=2100).contains(&year) {
        return None;
    }
    Some((m.start(), year))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieQuery {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
}

/// Parse a movie title and year out of a video file's relative path. Tries
/// the parent directory name first (when present), falling back to the
/// filename stem. Handles both the canonical `Movie Name (2023)` form and
/// the messy scene-release form `Movie.Name.2023.1080p.BluRay.x264-GROUP`.
pub fn extract_movie_query(relative_path: &str) -> Option<MovieQuery> {
    let path = PathBuf::from(relative_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // Prefer the parent dir when it carries a `(YYYY)` tag — that's the
    // canonical "Movie Name (2023)/movie.mkv" layout. Otherwise fall back
    // to the filename stem, which is what scene releases use.
    let raw = if !parent.is_empty() && YEAR_TAG_RE.is_match(&parent) {
        parent
    } else if !stem.is_empty() {
        stem
    } else {
        return None;
    };

    let humanized = raw.replace(['.', '_'], " ");

    if let Some(c) = YEAR_TAG_RE.captures(&humanized) {
        let year = c.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
        let stripped = YEAR_TAG_RE.replace(&humanized, "").trim().to_string();
        let cleaned = stripped
            .trim_end_matches(|c: char| matches!(c, '.' | '-' | '_' | ' '))
            .trim()
            .to_string();
        if cleaned.is_empty() {
            return None;
        }
        return Some(MovieQuery {
            title: cleaned,
            year,
        });
    }

    if let Some((idx, year)) = first_year(&humanized) {
        let title = humanized[..idx]
            .trim_end_matches(|c: char| matches!(c, '.' | '-' | '_' | ' '))
            .trim()
            .to_string();
        if !title.is_empty() {
            return Some(MovieQuery {
                title,
                year: Some(year),
            });
        }
    }

    let trimmed = humanized.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(MovieQuery {
            title: trimmed,
            year: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMatch {
    #[serde(rename = "tmdbId")]
    pub tmdb_id: i64,
    pub title: String,
    #[serde(rename = "originalTitle", skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(rename = "posterUrl", skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(rename = "voteAverage", skip_serializing_if = "Option::is_none")]
    pub vote_average: Option<f64>,
    /// The score the picker assigned to this candidate. Useful for
    /// debugging / surfacing low-confidence matches in the UI.
    pub score: f64,
}

fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth: i32 = 0;
    for c in s.chars() {
        match c {
            '(' | '[' => {
                depth += 1;
                out.push(' ');
            }
            ')' | ']' => {
                depth = (depth - 1).max(0);
                out.push(' ');
            }
            _ if depth > 0 => out.push(' '),
            _ => {
                let lower = c.to_ascii_lowercase();
                if lower.is_ascii_alphanumeric() {
                    out.push(lower);
                } else {
                    out.push(' ');
                }
            }
        }
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .split(' ')
        .filter(|w| !NOISE_WORDS.contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokens(s: &str) -> Vec<String> {
    normalize(s)
        .split(' ')
        .filter(|w| w.len() > 1)
        .map(|w| w.to_string())
        .collect()
}

fn urlencode(s: &str) -> String {
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

#[derive(Debug)]
struct Candidate {
    tmdb_id: i64,
    title: String,
    original_title: Option<String>,
    year: Option<i32>,
    overview: Option<String>,
    poster_url: Option<String>,
    vote_average: Option<f64>,
}

fn parse_year(release_date: &str) -> Option<i32> {
    if release_date.len() < 4 {
        return None;
    }
    release_date[..4].parse::<i32>().ok()
}

fn build_candidates(payload: &serde_json::Value) -> Vec<Candidate> {
    let arr = match payload.get("results").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };
    arr.iter()
        .filter_map(|r| {
            let tmdb_id = r.get("id").and_then(|v| v.as_i64())?;
            let title = r
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if title.is_empty() {
                return None;
            }
            let original_title = r
                .get("original_title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty() && *s != title)
                .map(|s| s.to_string());
            let year = r
                .get("release_date")
                .and_then(|v| v.as_str())
                .and_then(parse_year);
            let overview = r
                .get("overview")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let poster_url = r
                .get("poster_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| format!("{TMDB_IMG_BASE}/w342{s}"));
            let vote_average = r.get("vote_average").and_then(|v| v.as_f64());
            Some(Candidate {
                tmdb_id,
                title,
                original_title,
                year,
                overview,
                poster_url,
                vote_average,
            })
        })
        .collect()
}

/// Score a TMDB candidate against the parsed query. Hard gate: title-token
/// overlap ≥ 50% of the query's tokens. Then track the title overlap (×10),
/// year proximity (+6 exact, +3 ±1y, 0 otherwise — when the query has a
/// year), original-title overlap as a tiebreaker (×2), and a small bias
/// toward popular results via `vote_average` (×0.1).
fn score_candidate(query_tokens: &[String], query_year: Option<i32>, c: &Candidate) -> Option<f64> {
    if query_tokens.is_empty() {
        return None;
    }
    let title_norm = normalize(&c.title);
    let original_norm = c
        .original_title
        .as_deref()
        .map(normalize)
        .unwrap_or_default();
    let combined = format!("{title_norm} {original_norm}");

    let title_hits = query_tokens
        .iter()
        .filter(|t| title_norm.contains(t.as_str()))
        .count() as f64;
    let title_ratio = title_hits / query_tokens.len() as f64;
    if title_ratio < 0.5 {
        return None;
    }
    let mut score = title_ratio * 10.0;

    if !original_norm.is_empty() {
        let hits = query_tokens
            .iter()
            .filter(|t| combined.contains(t.as_str()))
            .count() as f64;
        score += (hits / query_tokens.len() as f64) * 2.0;
    }

    if let Some(qy) = query_year {
        if let Some(cy) = c.year {
            let delta = (cy - qy).abs();
            if delta == 0 {
                score += 6.0;
            } else if delta == 1 {
                score += 3.0;
            } else if delta <= 2 {
                score += 1.0;
            }
        }
    }

    if let Some(va) = c.vote_average {
        score += va * 0.1;
    }

    Some(score)
}

fn pick_best(candidates: Vec<Candidate>, query: &MovieQuery) -> Option<TmdbMatch> {
    let qt = tokens(&query.title);
    let mut best: Option<(Candidate, f64)> = None;
    for cand in candidates {
        let Some(score) = score_candidate(&qt, query.year, &cand) else {
            continue;
        };
        match best {
            Some((_, bs)) if bs >= score => {}
            _ => best = Some((cand, score)),
        }
    }
    best.map(|(c, score)| TmdbMatch {
        tmdb_id: c.tmdb_id,
        title: c.title,
        original_title: c.original_title,
        year: c.year,
        overview: c.overview,
        poster_url: c.poster_url,
        vote_average: c.vote_average,
        score,
    })
}

/// Run a single TMDB `/search/movie` request and pick the best match. When
/// the query carries a year hint, try the year-filtered upstream first and
/// fall back to a no-year search if that returns no candidates above the
/// 50% gate. Returns `None` for any non-fatal failure (network error, bad
/// response, no candidates) so the caller can keep going for other files.
pub async fn match_movie(client: &reqwest::Client, api_key: &str, query: &MovieQuery) -> Option<TmdbMatch> {
    if api_key.is_empty() || query.title.trim().is_empty() {
        return None;
    }

    if let Some(year) = query.year {
        let url = format!(
            "{}/search/movie?api_key={}&query={}&year={}&include_adult=false",
            TMDB_BASE,
            api_key,
            urlencode(&query.title),
            year
        );
        if let Some(m) = fetch_and_pick(client, &url, query).await {
            return Some(m);
        }
    }

    let url = format!(
        "{}/search/movie?api_key={}&query={}&include_adult=false",
        TMDB_BASE,
        api_key,
        urlencode(&query.title)
    );
    fetch_and_pick(client, &url, query).await
}

async fn fetch_and_pick(
    client: &reqwest::Client,
    url: &str,
    query: &MovieQuery,
) -> Option<TmdbMatch> {
    let res = match client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!("[tmdb-match] tmdb returned {} for {}", r.status(), query.title);
            return None;
        }
        Err(e) => {
            tracing::warn!("[tmdb-match] tmdb request failed for {}: {e}", query.title);
            return None;
        }
    };
    let payload: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("[tmdb-match] tmdb parse failed for {}: {e}", query.title);
            return None;
        }
    };
    let candidates = build_candidates(&payload);
    pick_best(candidates, query)
}

/// Match a batch of `(index, MovieQuery)` pairs against TMDB in parallel
/// (bounded by `MATCH_CONCURRENCY`). Returns a `Vec<(index, Option<TmdbMatch>)>`
/// preserving the original indices so the caller can stitch results back
/// onto whatever flat collection it owns. Reads `TMDB_API_KEY` from the
/// environment; if it's missing, returns an empty result list (no error,
/// scans still complete).
pub async fn match_movies_parallel(queries: Vec<(usize, MovieQuery)>) -> Vec<(usize, TmdbMatch)> {
    if queries.is_empty() {
        return Vec::new();
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        tracing::info!("[tmdb-match] TMDB_API_KEY not set — skipping per-file matching");
        return Vec::new();
    }
    let client = reqwest::Client::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(MATCH_CONCURRENCY));
    let mut handles = Vec::with_capacity(queries.len());
    for (idx, query) in queries {
        let sem = semaphore.clone();
        let client = client.clone();
        let api_key = api_key.clone();
        handles.push(tokio::spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return (idx, None),
            };
            (idx, match_movie(&client, &api_key, &query).await)
        }));
    }
    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok((idx, Some(m))) = h.await {
            out.push((idx, m));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_directory_form() {
        let q = extract_movie_query("The Matrix (1999)/the.matrix.mkv").unwrap();
        assert_eq!(q.title, "The Matrix");
        assert_eq!(q.year, Some(1999));
    }

    #[test]
    fn parses_scene_release_filename() {
        let q =
            extract_movie_query("Inception.2010.1080p.BluRay.x264-GROUP.mkv").unwrap();
        assert_eq!(q.title, "Inception");
        assert_eq!(q.year, Some(2010));
    }

    #[test]
    fn parses_filename_without_year() {
        let q = extract_movie_query("Some.Random.Movie.mkv").unwrap();
        assert_eq!(q.title, "Some Random Movie");
        assert_eq!(q.year, None);
    }

    #[test]
    fn parses_year_in_filename_with_parent() {
        let q =
            extract_movie_query("Movies/The Matrix (1999).mkv").unwrap();
        assert_eq!(q.title, "The Matrix");
        assert_eq!(q.year, Some(1999));
    }
}
