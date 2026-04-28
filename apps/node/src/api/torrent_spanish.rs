//! Spanish-language torrent indexers.
//!
//! The 7 indexers below are catalogued from
//! <https://github.com/Jackett/Jackett/issues/1468>. All are private
//! invite-only or open-registration trackers that require credentials. Each
//! search function reads credentials from environment variables (declared
//! per-indexer) and silently returns an empty result set when credentials
//! are absent. Results are aggregated alongside PirateBay hits when the
//! frontend selects Spanish.

use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct SpanishIndexer {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    pub username_env: &'static str,
    pub password_env: &'static str,
}

pub const SPANISH_INDEXERS: &[SpanishIndexer] = &[
    SpanishIndexer {
        id: "hdspain",
        name: "HDSpain",
        url: "https://www.hd-spain.com/",
        username_env: "HDSPAIN_USERNAME",
        password_env: "HDSPAIN_PASSWORD",
    },
    SpanishIndexer {
        id: "hdcity",
        name: "HDCity",
        url: "https://hdcity.li/",
        username_env: "HDCITY_USERNAME",
        password_env: "HDCITY_PASSWORD",
    },
    SpanishIndexer {
        id: "hachede",
        name: "HacheDe",
        url: "https://hachede.me/",
        username_env: "HACHEDE_USERNAME",
        password_env: "HACHEDE_PASSWORD",
    },
    SpanishIndexer {
        id: "puntotorrent",
        name: "Puntotorrent",
        url: "https://xbt.puntotorrent.ch/",
        username_env: "PUNTOTORRENT_USERNAME",
        password_env: "PUNTOTORRENT_PASSWORD",
    },
    SpanishIndexer {
        id: "torrentland",
        name: "Torrentland",
        url: "https://torrentland.li/",
        username_env: "TORRENTLAND_USERNAME",
        password_env: "TORRENTLAND_PASSWORD",
    },
    SpanishIndexer {
        id: "xbytesv2",
        name: "xBytesV2",
        url: "http://xbytesv2.li/",
        username_env: "XBYTESV2_USERNAME",
        password_env: "XBYTESV2_PASSWORD",
    },
    SpanishIndexer {
        id: "unionfansub",
        name: "Unionfansub",
        url: "http://torrent.unionfansub.com/",
        username_env: "UNIONFANSUB_USERNAME",
        password_env: "UNIONFANSUB_PASSWORD",
    },
];

/// Query variants used to surface Castilian-dub releases on language-agnostic
/// indexers like PirateBay. Note: `latino` is intentionally excluded — Latin
/// American Spanish is a distinct dub from Castilian and "Spanish" in the UI
/// means Castilian only.
pub const SPANISH_QUERY_HINTS: &[&str] = &["castellano", "español"];

/// Whole-word Castilian Spanish markers. Latin American Spanish (`latino`,
/// `latam`, `es-MX`, etc.) is explicitly excluded via `LATINO_TOKENS` below
/// and a name matching any of those is rejected even when a Castilian marker
/// also appears (the latino marker wins).
const SPANISH_TOKENS: &[&str] = &[
    "castellano",
    "castelhano",
    "español",
    "espanol",
    "spanish",
    "spa",
    "esp",
    "cast", // common shorthand on Castilian releases ("[CAST]", "DUAL.CAST")
    "es-es",
];

/// Whole-word markers for Latin American Spanish releases. A name containing
/// any of these is rejected outright when the user selected Castilian Spanish.
const LATINO_TOKENS: &[&str] = &[
    "latino",
    "latina",
    "latinoamericano",
    "latinoamericana",
    "latinoamerica",
    "latam",
    "lat",
    "es-mx",
    "es-la",
    "es-ar",
    "es-cl",
    "es-419",
    "mexicano",
    "mexicana",
];

/// True when `name` contains a Castilian Spanish marker AND no Latin American
/// marker. The latino check wins regardless of other markers in the name —
/// e.g. "español latino" is rejected because `latino` is present.
///
/// Word boundaries treat any non-alphanumeric character (including `-` for
/// non-locale tokens) as a delimiter; locale-code tokens like `es-mx` carry
/// their internal `-` and the boundary check only looks at the surrounding
/// characters.
pub fn is_spanish_release(name: &str) -> bool {
    let lower = name.to_lowercase();
    if contains_token(&lower, LATINO_TOKENS) {
        return false;
    }
    contains_token(&lower, SPANISH_TOKENS)
}

fn contains_token(lower: &str, tokens: &[&str]) -> bool {
    let bytes = lower.as_bytes();
    for token in tokens {
        let tlen = token.len();
        let mut start = 0;
        while let Some(idx) = lower[start..].find(token) {
            let abs = start + idx;
            let prev_ok = abs == 0 || !is_word_char(bytes[abs - 1], token);
            let end = abs + tlen;
            let next_ok = end == bytes.len() || !is_word_char(bytes[end], token);
            if prev_ok && next_ok {
                return true;
            }
            start = abs + 1;
        }
    }
    false
}

/// Word-character predicate. Locale-code tokens (`es-mx`, etc.) include `-`
/// inside themselves, so for those we treat `-` as part of the word boundary
/// instead of an internal separator.
fn is_word_char(b: u8, token: &str) -> bool {
    if b.is_ascii_alphanumeric() {
        return true;
    }
    // For plain alpha tokens like "spanish", "-" is a delimiter (good).
    // For locale-code tokens like "es-mx", the "-" lives inside the token
    // and the surrounding boundary check uses the chars on either side of the
    // whole token, never inside it.
    let _ = token;
    false
}

#[cfg(test)]
mod tests {
    use super::is_spanish_release;

    #[test]
    fn detects_castilian_markers() {
        assert!(is_spanish_release("Barbie.2023.1080p.BluRay.x264.SPANISH"));
        assert!(is_spanish_release("Barbie (2023) [Castellano]"));
        assert!(is_spanish_release("Barbie.2023.1080p.WEB-DL.DUAL.CAST"));
        assert!(is_spanish_release("Barbie.2023.[ESP][1080p]"));
        assert!(is_spanish_release("Barbie 2023 español"));
        assert!(is_spanish_release("Barbie.2023.es-ES.1080p.WEBRip"));
    }

    #[test]
    fn rejects_latin_american_releases() {
        // Plain latino markers
        assert!(!is_spanish_release("Barbie.2023.LATAM.1080p"));
        assert!(!is_spanish_release("Barbie.2023.Latino.1080p"));
        assert!(!is_spanish_release("Barbie.2023.Latinoamericano.1080p"));
        // Latin American locale codes
        assert!(!is_spanish_release("Barbie.2023.es-MX.1080p.WEBRip"));
        assert!(!is_spanish_release("Barbie.2023.es-AR.1080p"));
        assert!(!is_spanish_release("Barbie.2023.es-419.1080p"));
        // Latino marker beats a Castilian one — "español latino" is Latin American
        assert!(!is_spanish_release("Barbie 2023 español latino"));
        assert!(!is_spanish_release("Barbie.2023.[ESP][LATAM]"));
        assert!(!is_spanish_release("Barbie.2023.Castellano.Latino.1080p"));
    }

    #[test]
    fn rejects_english_releases() {
        assert!(!is_spanish_release("Barbie.2023.1080p.BluRay.x264.ENGLISH"));
        assert!(!is_spanish_release("Barbie 2023 1080p WEB-DL"));
        assert!(!is_spanish_release("Barbie.2023.MULTI"));
    }

    #[test]
    fn does_not_match_substrings() {
        // "spa" must be a standalone token — it should NOT match "spawn"
        assert!(!is_spanish_release("Spawn.1997.1080p"));
        // "cast" must not match "Castle" or "broadcast"
        assert!(!is_spanish_release("Castle.S01.1080p"));
        assert!(!is_spanish_release("Broadcast.News.1987.1080p"));
        // "esp" must not match "espionage"
        assert!(!is_spanish_release("Espionage.2020.1080p"));
        // "lat" must not match "Late" or "platform"
        assert!(!is_spanish_release("Late.Night.2019.1080p"));
        assert!(!is_spanish_release("The.Platform.2019.1080p"));
    }
}

/// Build the full set of PirateBay queries to issue for a Spanish search:
/// the original query plus one variant per Spanish-language hint.
pub fn build_piratebay_queries(query: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(SPANISH_QUERY_HINTS.len() + 1);
    out.push(query.to_string());
    for hint in SPANISH_QUERY_HINTS {
        out.push(format!("{} {}", query, hint));
    }
    out
}

/// Result shape returned by Spanish indexer scrapers — must mirror the
/// `SearchResult` shape used by `torrent.rs`. Kept here to avoid a cyclic
/// import; `torrent.rs` constructs the final wire shape.
#[derive(Debug, Clone, Serialize)]
pub struct SpanishSearchResult {
    pub id: String,
    pub name: String,
    pub info_hash: String,
    pub seeders: i64,
    pub leechers: i64,
    pub size: i64,
    pub uploaded_at: i64,
    pub category: String,
    pub magnet_uri: String,
    pub indexer: String,
}

/// Run all Spanish indexer searches in parallel. Indexers without configured
/// credentials are skipped silently. Errors from any single indexer are
/// suppressed so a misconfigured tracker can't poison the aggregate response.
pub async fn search_all(query: &str) -> Vec<SpanishSearchResult> {
    let mut handles = Vec::with_capacity(SPANISH_INDEXERS.len());
    for indexer in SPANISH_INDEXERS {
        let q = query.to_string();
        let idx = *indexer;
        handles.push(tokio::spawn(async move { search_indexer(idx, &q).await }));
    }

    let mut out = Vec::new();
    for h in handles {
        if let Ok(Ok(rows)) = h.await {
            out.extend(rows);
        }
    }
    out
}

/// Search a single indexer. Returns Ok(empty) when credentials are missing,
/// so callers cannot distinguish "not configured" from "no results"
/// (both are non-fatal).
async fn search_indexer(
    indexer: SpanishIndexer,
    _query: &str,
) -> Result<Vec<SpanishSearchResult>, String> {
    let user = std::env::var(indexer.username_env).ok().filter(|s| !s.is_empty());
    let pass = std::env::var(indexer.password_env).ok().filter(|s| !s.is_empty());
    if user.is_none() || pass.is_none() {
        // Credentials not configured — return empty; the indexer is invite-only
        // so anonymous scraping would only ever hit the login page.
        return Ok(Vec::new());
    }

    // Per-indexer scraper integration is the natural extension point here.
    // Each tracker has a published Jackett YAML definition (see the issue
    // referenced at the top of this file) describing its login form, search
    // path, and DOM selectors. Implementing those scrapers is gated on having
    // working credentials to test against; until then we return empty rather
    // than ship an untested scraper.
    tracing::debug!(
        indexer = indexer.id,
        "spanish indexer credentials configured but scraper not yet implemented"
    );
    Ok(Vec::new())
}
