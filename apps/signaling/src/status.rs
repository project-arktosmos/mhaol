pub async fn check(url: &str) -> Result<(), String> {
    let health_url = format!("{}/api/health", url.trim_end_matches('/'));
    println!("Checking signaling server at {health_url}...");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {e}"))?;
            println!("Signaling server: OK");
            if let Some(status) = body.get("status").and_then(|v| v.as_str()) {
                println!("  status: {status}");
            }
            Ok(())
        }
        Ok(resp) => Err(format!("Server returned HTTP {}", resp.status())),
        Err(e) => Err(format!("Connection failed: {e}")),
    }
}
