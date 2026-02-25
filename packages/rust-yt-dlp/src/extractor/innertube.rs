use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;

use super::clients::InnertubeClient;
use super::player::PlayerResponse;

const INNERTUBE_BASE_URL: &str = "https://www.youtube.com/youtubei/v1";

/// Client for making YouTube Innertube API requests.
pub struct InnertubeApi {
    http: reqwest::Client,
}

impl InnertubeApi {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build HTTP client");
        Self { http }
    }

    /// Get a reference to the underlying HTTP client (shares cookie store).
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }

    /// Fetch player response for a video using the specified client context.
    pub async fn player(
        &self,
        video_id: &str,
        client: &InnertubeClient,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<PlayerResponse> {
        let url = format!(
            "{}/player?key={}&prettyPrint=false",
            INNERTUBE_BASE_URL, client.api_key
        );

        let body = client.build_player_request(video_id, None, po_token, visitor_data);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(client.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Mozilla/5.0")),
        );
        headers.insert(
            "X-YouTube-Client-Name",
            HeaderValue::from_str(&client.client_id.to_string()).unwrap(),
        );
        headers.insert(
            "X-YouTube-Client-Version",
            HeaderValue::from_str(client.client_version).unwrap(),
        );
        headers.insert("Origin", HeaderValue::from_static("https://www.youtube.com"));

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Innertube player API returned {}: {}", status, text);
        }

        let player_response: PlayerResponse = response.json().await?;
        Ok(player_response)
    }

    /// Fetch playlist data using the browse API.
    pub async fn browse_playlist(&self, playlist_id: &str) -> Result<Value> {
        let url = format!(
            "{}/browse?key={}&prettyPrint=false",
            INNERTUBE_BASE_URL,
            super::clients::WEB.api_key
        );

        let body = serde_json::json!({
            "context": super::clients::WEB.build_context(None),
            "browseId": format!("VL{}", playlist_id)
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(super::clients::WEB.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Mozilla/5.0")),
        );

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Innertube browse API returned {}: {}", status, text);
        }

        let data: Value = response.json().await?;
        Ok(data)
    }

    /// Fetch the YouTube watch page HTML to extract player.js URL and other metadata.
    pub async fn fetch_watch_page(&self, video_id: &str) -> Result<String> {
        let url = format!("https://www.youtube.com/watch?v={}&bpctr=9999999999&has_verified=1", video_id);

        let response = self
            .http
            .get(&url)
            .header(
                USER_AGENT,
                super::clients::WEB.user_agent,
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await?;

        let html = response.text().await?;
        Ok(html)
    }

    /// Fetch the player.js source code given its URL.
    pub async fn fetch_player_js(&self, player_js_url: &str) -> Result<String> {
        let full_url = if player_js_url.starts_with("//") {
            format!("https:{}", player_js_url)
        } else if player_js_url.starts_with('/') {
            format!("https://www.youtube.com{}", player_js_url)
        } else {
            player_js_url.to_string()
        };

        let response = self.http.get(&full_url).send().await?;
        let source = response.text().await?;
        Ok(source)
    }

    /// Fetch playlist continuation data for pagination.
    pub async fn browse_continuation(&self, continuation_token: &str) -> Result<Value> {
        let url = format!(
            "{}/browse?key={}&prettyPrint=false",
            INNERTUBE_BASE_URL,
            super::clients::WEB.api_key
        );

        let body = serde_json::json!({
            "context": super::clients::WEB.build_context(None),
            "continuation": continuation_token
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(super::clients::WEB.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Mozilla/5.0")),
        );

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data)
    }
}
