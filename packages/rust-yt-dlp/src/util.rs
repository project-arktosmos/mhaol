use crate::error::YtDlpError;

/// Extract video ID from various YouTube URL formats.
pub fn extract_video_id(url: &str) -> Result<String, YtDlpError> {
    // Try youtu.be short URLs
    if let Some(id) = url
        .strip_prefix("https://youtu.be/")
        .or_else(|| url.strip_prefix("http://youtu.be/"))
    {
        let id = id.split(['?', '&', '/']).next().unwrap_or(id);
        if is_valid_video_id(id) {
            return Ok(id.to_string());
        }
    }

    // Try parsing as URL and extract 'v' parameter
    if let Ok(parsed) = url::Url::parse(url) {
        // youtube.com/watch?v=ID
        for (key, value) in parsed.query_pairs() {
            if key == "v" && is_valid_video_id(&value) {
                return Ok(value.to_string());
            }
        }

        // youtube.com/embed/ID or youtube.com/v/ID or youtube.com/shorts/ID
        if let Some(segments) = parsed.path_segments() {
            let segments: Vec<&str> = segments.collect();
            if segments.len() >= 2 {
                let prefix = segments[0];
                let id = segments[1];
                if (prefix == "embed" || prefix == "v" || prefix == "shorts")
                    && is_valid_video_id(id)
                {
                    return Ok(id.to_string());
                }
            }
        }
    }

    // Try bare video ID
    if is_valid_video_id(url) {
        return Ok(url.to_string());
    }

    Err(YtDlpError::InvalidUrl(format!(
        "Could not extract video ID from: {}",
        url
    )))
}

/// Extract playlist ID from a YouTube URL.
pub fn extract_playlist_id(url: &str) -> Result<String, YtDlpError> {
    if let Ok(parsed) = url::Url::parse(url) {
        for (key, value) in parsed.query_pairs() {
            if key == "list" && !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    Err(YtDlpError::InvalidUrl(format!(
        "Could not extract playlist ID from: {}",
        url
    )))
}

/// Check if a string looks like a valid YouTube video ID (11 chars, alphanumeric + - _).
fn is_valid_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_watch_url() {
        let id = extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_short_url() {
        let id = extract_video_id("https://youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_embed_url() {
        let id = extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_shorts_url() {
        let id = extract_video_id("https://www.youtube.com/shorts/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_bare() {
        let id = extract_video_id("dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_invalid() {
        assert!(extract_video_id("not-a-url").is_err());
    }

    #[test]
    fn test_extract_playlist_id() {
        let id = extract_playlist_id(
            "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf",
        )
        .unwrap();
        assert_eq!(id, "PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf");
    }

}
