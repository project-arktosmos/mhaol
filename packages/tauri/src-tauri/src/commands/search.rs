use serde::{Deserialize, Serialize};

const PIRATEBAY_API: &str = "https://apibay.org";
const USER_AGENT: &str = "Mozilla/5.0 (compatible; Mhaol/1.0)";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PirateBayResult {
    id: String,
    name: String,
    info_hash: String,
    leechers: String,
    seeders: String,
    num_files: String,
    size: String,
    username: String,
    added: String,
    status: String,
    category: String,
    imdb: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentSearchResult {
    pub id: String,
    pub name: String,
    pub info_hash: String,
    pub magnet_link: String,
    pub seeders: i64,
    pub leechers: i64,
    pub size: i64,
    pub category: String,
    pub uploaded_by: String,
    pub uploaded_at: i64,
    pub is_vip: bool,
    pub is_trusted: bool,
}

fn build_magnet(info_hash: &str, name: &str) -> String {
    let encoded_name = urlencoding::encode(name);
    format!(
        "magnet:?xt=urn:btih:{}&dn={}&tr=udp://tracker.coppersurfer.tk:6969/announce&tr=udp://tracker.openbittorrent.com:6969/announce&tr=udp://tracker.opentrackr.org:1337/announce",
        info_hash, encoded_name
    )
}

fn parse_result(r: PirateBayResult) -> TorrentSearchResult {
    let seeders = r.seeders.parse::<i64>().unwrap_or(0);
    let leechers = r.leechers.parse::<i64>().unwrap_or(0);
    let size = r.size.parse::<i64>().unwrap_or(0);
    let uploaded_at = r.added.parse::<i64>().unwrap_or(0);
    let status = r.status.as_str();

    TorrentSearchResult {
        magnet_link: build_magnet(&r.info_hash, &r.name),
        id: r.id,
        name: r.name,
        info_hash: r.info_hash,
        seeders,
        leechers,
        size,
        category: r.category,
        uploaded_by: r.username,
        uploaded_at,
        is_vip: status == "vip",
        is_trusted: status == "trusted" || status == "vip",
    }
}

#[tauri::command]
pub async fn search_torrents(
    query: String,
    category: Option<String>,
) -> Result<Vec<TorrentSearchResult>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let cat = category.unwrap_or_else(|| "0".to_string());
    let url = format!(
        "{}/q.php?q={}&cat={}",
        PIRATEBAY_API,
        urlencoding::encode(trimmed),
        urlencoding::encode(&cat)
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("PirateBay API returned {}", response.status()));
    }

    let raw_results: Vec<PirateBayResult> = response.json().await.map_err(|e| e.to_string())?;

    // PirateBay returns a single result with id "0" and name "No results..." when nothing found
    let results: Vec<TorrentSearchResult> = raw_results
        .into_iter()
        .filter(|r| r.id != "0")
        .map(parse_result)
        .collect();

    Ok(results)
}
