use serde_json::{json, Value};

/// An Innertube API client configuration.
#[derive(Debug, Clone)]
pub struct InnertubeClient {
    pub name: &'static str,
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub api_key: &'static str,
    pub user_agent: &'static str,
    pub requires_js: bool,
    pub client_id: u32,
    /// Whether this is a browser-based client. Browser clients expect Origin/Referer
    /// headers on stream requests; native app clients (Android, iOS) must not send them.
    pub is_browser: bool,
}

impl InnertubeClient {
    /// Build the Innertube context JSON for API requests.
    pub fn build_context(&self) -> Value {
        json!({
            "client": {
                "clientName": self.client_name,
                "clientVersion": self.client_version,
                "hl": "en",
                "timeZone": "UTC",
                "utcOffsetMinutes": 0
            }
        })
    }

    /// Build a full player request body.
    pub fn build_player_request(
        &self,
        video_id: &str,
        sts: Option<u64>,
        po_token: Option<&str>,
    ) -> Value {
        let mut body = json!({
            "context": self.build_context(),
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true
        });

        if let Some(sts_val) = sts {
            body["playbackContext"] = json!({
                "contentPlaybackContext": {
                    "html5Preference": "HTML5_PREF_WANTS",
                    "signatureTimestamp": sts_val
                }
            });
        }

        if let Some(token) = po_token {
            body["serviceIntegrityDimensions"] = json!({
                "poToken": token
            });
        }

        body
    }
}

/// Android client - doesn't require JS player for signatures in many cases.
pub const ANDROID: InnertubeClient = InnertubeClient {
    name: "android",
    client_name: "ANDROID",
    client_version: "20.03.02",
    api_key: "AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w",
    user_agent: "com.google.android.youtube/20.03.02 (Linux; U; Android 14; en_US; sdk_gphone64_arm64 Build/UE1A.230829.036.A1) gzip",
    requires_js: false,
    client_id: 3,
    is_browser: false,
};

/// Web client - primary client, may require JS for signature decryption.
pub const WEB: InnertubeClient = InnertubeClient {
    name: "web",
    client_name: "WEB",
    client_version: "2.20250117.01.00",
    api_key: "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
    user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    requires_js: true,
    client_id: 1,
    is_browser: true,
};

/// Web embedded player - useful for bypassing age restrictions.
pub const WEB_EMBEDDED: InnertubeClient = InnertubeClient {
    name: "web_embedded",
    client_name: "WEB_EMBEDDED_PLAYER",
    client_version: "2.20250117.01.00",
    api_key: "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
    user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
    requires_js: true,
    client_id: 56,
    is_browser: true,
};

/// iOS client - returns HLS manifests.
pub const IOS: InnertubeClient = InnertubeClient {
    name: "ios",
    client_name: "IOS",
    client_version: "20.03.2",
    api_key: "AIzaSyB-63vPrdThhKuerbB2N_l7Kwwcxj6yUAc",
    user_agent: "com.google.ios.youtube/20.03.2 (iPhone16,2; U; CPU iOS 18_3 like Mac OS X;)",
    requires_js: false,
    client_id: 5,
    is_browser: false,
};

/// TV HTML5 client.
pub const TV: InnertubeClient = InnertubeClient {
    name: "tv",
    client_name: "TVHTML5",
    client_version: "7.20250117.12.00",
    api_key: "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
    user_agent: "Mozilla/5.0 (ChromiumStylePlatform) Cobalt/Version",
    requires_js: true,
    client_id: 7,
    is_browser: true,
};

/// The ordered list of clients to try. ANDROID first (no JS needed), then WEB, then fallbacks.
pub const CLIENT_PRIORITY: &[&InnertubeClient] = &[&ANDROID, &WEB, &WEB_EMBEDDED, &IOS, &TV];
