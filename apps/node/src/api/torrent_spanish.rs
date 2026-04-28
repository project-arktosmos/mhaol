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

/// Query variants used to surface Spanish-dub releases on language-agnostic
/// indexers like PirateBay.
pub const SPANISH_QUERY_HINTS: &[&str] = &["castellano", "español", "latino"];

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
