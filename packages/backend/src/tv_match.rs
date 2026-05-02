//! Per-file TV episode extractor for library scans of `local-tv`
//! libraries. Given a video file's relative path, parse a
//! `(show, season, episode, year?)` query out of its filename and parent
//! directories so the WebUI can show "this video file would search TMDB
//! for <show> season <n> episode <m>" in the libraries table. No upstream
//! call is made yet — the actual TMDB lookup will be wired in once the
//! extraction shape is settled.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Words that are filename noise but not part of the show name — dropped
/// when humanising the extracted show string so a release tag like
/// `1080p` doesn't end up in the TMDB query. Mirrors the list in
/// `tmdb_match::NOISE_WORDS`.
const NOISE_WORDS: &[&str] = &[
    // resolution / quality
    "1080p", "2160p", "720p", "480p", "360p", "4k", "uhd", "hdr", "hdr10", "dv",
    // sources
    "bluray", "brrip", "bdrip", "webrip", "web", "dl", "webdl", "hdrip", "dvdrip", "dvd",
    "hdtv", "pdtv", "cam", "ts", "screener", "dvdscr",
    // codecs
    "x264", "x265", "h264", "h265", "hevc", "avc", "xvid", "divx", "vp9", "av1",
    // audio
    "aac", "ac3", "dts", "ddp", "ddp5", "atmos", "truehd", "flac",
    // misc release tags
    "remux", "proper", "repack", "extended", "uncut", "unrated", "internal", "limited",
    "complete",
    // languages / subs
    "multi", "dual", "dubbed", "subbed", "vostfr",
    // file containers leaking from dir names
    "mp4", "mkv", "avi", "m4v",
];

static SXX_EXX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)s(\d{1,3})\s*[\._\-]?\s*e(\d{1,3})").unwrap());
static NXM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:^|[\s\._\-])(\d{1,2})x(\d{1,3})(?:[\s\._\-]|$)").unwrap()
});
static SEASON_DIR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^\s*(?:season|s)\s*(\d{1,3})\s*$").unwrap());
static EPISODE_FILE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:^|[\s\._\-])(?:episode|ep|e)\s*(\d{1,3})(?:[\s\._\-]|$)").unwrap()
});
static YEAR_TAG_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\(\[\{](\d{4})[\)\]\}]").unwrap());
static PAREN_GROUP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\(\[\{][^\)\]\}]*[\)\]\}]").unwrap());
static INLINE_SEASON_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:season\s*\d{1,3}|s\d{1,3})\b").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvQuery {
    pub show: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    pub season: u32,
    pub episode: u32,
}

fn parse_season_episode(stem: &str) -> Option<(u32, u32)> {
    if let Some(c) = SXX_EXX_RE.captures(stem) {
        let s: u32 = c.get(1)?.as_str().parse().ok()?;
        let e: u32 = c.get(2)?.as_str().parse().ok()?;
        return Some((s, e));
    }
    if let Some(c) = NXM_RE.captures(stem) {
        let s: u32 = c.get(1)?.as_str().parse().ok()?;
        let e: u32 = c.get(2)?.as_str().parse().ok()?;
        return Some((s, e));
    }
    None
}

fn parse_season_dir(name: &str) -> Option<u32> {
    let cleaned = name.replace(['.', '_'], " ");
    let c = SEASON_DIR_RE.captures(cleaned.trim())?;
    c.get(1)?.as_str().parse().ok()
}

fn parse_episode_filename(stem: &str) -> Option<u32> {
    let humanized = stem.replace(['.', '_'], " ");
    let c = EPISODE_FILE_RE.captures(&humanized)?;
    c.get(1)?.as_str().parse().ok()
}

fn is_season_dir_name(name: &str) -> bool {
    let cleaned = name.replace(['.', '_'], " ");
    SEASON_DIR_RE.is_match(cleaned.trim())
}

/// Convert a raw show string (filename stem or directory name) into the
/// human-readable show title plus optional year. Drops `(YYYY)` tags,
/// dots/underscores, and known release-tag noise words.
fn humanize_show_name(raw: &str) -> (String, Option<i32>) {
    let humanized = raw.replace(['.', '_'], " ");
    let year: Option<i32> = YEAR_TAG_RE
        .captures(&humanized)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok());
    // Drop the entire parenthesised / bracketed group (year or otherwise)
    // — release tags like `(1080p H265 Joy)` and `[GROUP]` go away with it.
    let no_parens = PAREN_GROUP_RE.replace_all(&humanized, " ").to_string();
    // Drop inline season tags (`Season 2`, `S02`) so a dir like
    // "Andor Season 2 Mp4 1080p" reduces to just the show name.
    let no_seasons = INLINE_SEASON_RE.replace_all(&no_parens, " ").to_string();
    let cleaned = no_seasons
        .split_whitespace()
        .filter(|w| !NOISE_WORDS.contains(&w.to_ascii_lowercase().as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    let trimmed = cleaned
        .trim_matches(|c: char| matches!(c, '.' | '-' | '_' | ' '))
        .to_string();
    (trimmed, year)
}

/// Strip the season-episode marker (and everything after it) from a
/// filename stem so what remains is the show name part of a scene-release
/// filename.
fn show_from_filename_stem(stem: &str) -> String {
    if let Some(m) = SXX_EXX_RE.find(stem) {
        return stem[..m.start()].to_string();
    }
    if let Some(m) = NXM_RE.find(stem) {
        return stem[..m.start()].to_string();
    }
    stem.to_string()
}

/// Parse a TV episode query out of a video file's relative path. Tries
/// the canonical `<Show>/<Season N>/<Show> SxxEyy` layout first, then
/// falls back to scene-release filenames (`Show.Name.S01E01.…`) and the
/// `1x01` short form. Returns `None` when no season/episode markers can
/// be found anywhere in the path.
pub fn extract_tv_query(relative_path: &str) -> Option<TvQuery> {
    let path = PathBuf::from(relative_path);
    let stem = path.file_stem().and_then(|s| s.to_str())?.to_string();

    let components: Vec<String> = path
        .iter()
        .filter_map(|c| c.to_str().map(String::from))
        .collect();
    if components.is_empty() {
        return None;
    }
    let dir_components: Vec<&String> = if components.len() > 1 {
        components[..components.len() - 1].iter().collect()
    } else {
        Vec::new()
    };

    let (season, episode) = if let Some(p) = parse_season_episode(&stem) {
        p
    } else {
        let season_from_dir = dir_components
            .iter()
            .rev()
            .find_map(|c| parse_season_dir(c))?;
        let ep = parse_episode_filename(&stem)?;
        (season_from_dir, ep)
    };

    // Show name: prefer the topmost directory component that isn't a
    // "Season N" / "S01" folder. If no useful directory exists, derive it
    // from the filename stem by trimming everything from the season-
    // episode marker onwards.
    let show_dir = dir_components
        .iter()
        .find(|c| !is_season_dir_name(c))
        .map(|s| s.to_string());

    let show_raw = match show_dir {
        Some(d) => d,
        None => show_from_filename_stem(&stem),
    };

    let (show, year) = humanize_show_name(&show_raw);
    if show.is_empty() {
        return None;
    }

    Some(TvQuery {
        show,
        year,
        season,
        episode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_layout() {
        let q = extract_tv_query("Breaking Bad/Season 01/Breaking Bad S01E01.mkv").unwrap();
        assert_eq!(q.show, "Breaking Bad");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 1);
        assert_eq!(q.year, None);
    }

    #[test]
    fn parses_scene_release_filename() {
        let q = extract_tv_query("Breaking.Bad.S01E01.1080p.mkv").unwrap();
        assert_eq!(q.show, "Breaking Bad");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 1);
    }

    #[test]
    fn parses_year_in_show_dir() {
        let q = extract_tv_query("Breaking Bad (2008)/Season 01/Breaking Bad S01E01.mkv").unwrap();
        assert_eq!(q.show, "Breaking Bad");
        assert_eq!(q.year, Some(2008));
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 1);
    }

    #[test]
    fn parses_nxm_short_form() {
        let q = extract_tv_query("Breaking Bad/Breaking Bad - 1x01 - Pilot.mkv").unwrap();
        assert_eq!(q.show, "Breaking Bad");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 1);
    }

    #[test]
    fn parses_season_dir_with_episode_filename() {
        let q = extract_tv_query("Breaking Bad/Season 1/Episode 01.mkv").unwrap();
        assert_eq!(q.show, "Breaking Bad");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 1);
    }

    #[test]
    fn parses_three_digit_episode() {
        let q = extract_tv_query("One Piece/Season 1/One Piece S01E105.mkv").unwrap();
        assert_eq!(q.show, "One Piece");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 105);
    }

    #[test]
    fn returns_none_without_episode_marker() {
        assert!(extract_tv_query("random_file.mkv").is_none());
        assert!(extract_tv_query("Movies/The Matrix (1999).mkv").is_none());
    }

    #[test]
    fn strips_inline_season_and_container_from_show_dir() {
        let q = extract_tv_query("Andor Season 2 Mp4 1080p/Andor S02E02.mp4").unwrap();
        assert_eq!(q.show, "Andor");
        assert_eq!(q.season, 2);
        assert_eq!(q.episode, 2);
    }

    #[test]
    fn strips_paren_release_tag_from_show_dir() {
        let q = extract_tv_query(
            "Archer Season 1  (1080p H265 Joy)/Archer S01E02 Training Day (1080p H265 Joy).m4v",
        )
        .unwrap();
        assert_eq!(q.show, "Archer");
        assert_eq!(q.season, 1);
        assert_eq!(q.episode, 2);
    }
}
