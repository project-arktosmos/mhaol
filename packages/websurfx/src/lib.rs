use serde::{Deserialize, Serialize};
use tracing;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSurfxResult {
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
    pub domain: Option<String>,
    pub published_date: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone)]
enum Provider {
    DuckDuckGo,
    Google { api_key: String, cx: String },
    Tavily { api_key: String },
    Brave { api_key: String },
}

#[derive(Debug, Clone)]
pub struct WebSurfxClient {
    provider: Provider,
}

impl WebSurfxClient {
    pub fn from_env() -> Self {
        let provider_name = std::env::var("WEBSURFX_PROVIDER")
            .unwrap_or_else(|_| "duckduckgo".to_string())
            .to_lowercase();

        let provider = match provider_name.as_str() {
            "google" => {
                let api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
                let cx = std::env::var("GOOGLE_CX").unwrap_or_default();
                if api_key.is_empty() || cx.is_empty() {
                    tracing::warn!("GOOGLE_API_KEY or GOOGLE_CX not set, falling back to DuckDuckGo");
                    Provider::DuckDuckGo
                } else {
                    Provider::Google { api_key, cx }
                }
            }
            "tavily" => {
                let api_key = std::env::var("TAVILY_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    tracing::warn!("TAVILY_API_KEY not set, falling back to DuckDuckGo");
                    Provider::DuckDuckGo
                } else {
                    Provider::Tavily { api_key }
                }
            }
            "brave" => {
                let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    tracing::warn!("BRAVE_API_KEY not set, falling back to DuckDuckGo");
                    Provider::DuckDuckGo
                } else {
                    Provider::Brave { api_key }
                }
            }
            _ => Provider::DuckDuckGo,
        };

        tracing::info!("WebSurfx using provider: {}", Self::provider_name_for(&provider));
        Self { provider }
    }

    pub fn provider_name(&self) -> &str {
        Self::provider_name_for(&self.provider)
    }

    fn provider_name_for(provider: &Provider) -> &str {
        match provider {
            Provider::DuckDuckGo => "duckduckgo",
            Provider::Google { .. } => "google",
            Provider::Tavily { .. } => "tavily",
            Provider::Brave { .. } => "brave",
        }
    }

    pub async fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
    ) -> Result<Vec<WebSurfxResult>, String> {
        use websearch::{web_search, SearchOptions};

        let provider: Box<dyn websearch::SearchProvider> = match &self.provider {
            Provider::DuckDuckGo => {
                Box::new(websearch::providers::DuckDuckGoProvider::new())
            }
            Provider::Google { api_key, cx } => {
                Box::new(
                    websearch::providers::GoogleProvider::new(api_key, cx)
                        .map_err(|e| format!("Google provider error: {e}"))?,
                )
            }
            Provider::Tavily { api_key } => {
                Box::new(
                    websearch::providers::TavilyProvider::new(api_key)
                        .map_err(|e| format!("Tavily provider error: {e}"))?,
                )
            }
            Provider::Brave { api_key } => {
                Box::new(
                    websearch::providers::BraveProvider::new(api_key)
                        .map_err(|e| format!("Brave provider error: {e}"))?,
                )
            }
        };

        let options = SearchOptions {
            query: query.to_string(),
            max_results,
            provider,
            ..Default::default()
        };

        let results = web_search(options)
            .await
            .map_err(|e| format!("Search error: {e}"))?;

        Ok(results
            .into_iter()
            .map(|r| WebSurfxResult {
                url: r.url,
                title: r.title,
                snippet: r.snippet,
                domain: r.domain,
                published_date: r.published_date,
                provider: r.provider,
            })
            .collect())
    }
}
