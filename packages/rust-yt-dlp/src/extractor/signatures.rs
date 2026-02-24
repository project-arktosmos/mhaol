use anyhow::Result;
use parking_lot::Mutex;
use regex::Regex;
use std::sync::Arc;

use crate::js::engine::JsEngine;

/// Resolves YouTube signature ciphers and n-parameter challenges.
/// Extracts the relevant functions from player.js and executes them via boa_engine.
pub struct SignatureResolver {
    /// Cached player.js URL (used for cache invalidation).
    player_js_url: Option<String>,
    /// Extracted signature function code from player.js.
    sig_function: Option<String>,
    /// Extracted n-parameter function code from player.js.
    n_function: Option<String>,
    /// Full helper code that the signature function depends on.
    helper_code: Option<String>,
}

impl SignatureResolver {
    pub fn new() -> Self {
        Self {
            player_js_url: None,
            sig_function: None,
            n_function: None,
            helper_code: None,
        }
    }

    /// Load and parse a player.js source, extracting the signature and n-param functions.
    pub fn load_player_js(&mut self, player_js_url: &str, source: &str) -> Result<()> {
        let sig_func = extract_signature_function(source)?;
        let n_func = extract_n_function(source)?;
        let helper = extract_helper_object(source, &sig_func)?;

        self.player_js_url = Some(player_js_url.to_string());
        self.sig_function = Some(sig_func);
        self.n_function = Some(n_func);
        self.helper_code = Some(helper);

        log::info!(
            "Loaded player.js signature functions from {}",
            player_js_url
        );
        Ok(())
    }

    /// Check if we already have functions loaded for a given player.js URL.
    pub fn is_loaded_for(&self, player_js_url: &str) -> bool {
        self.player_js_url.as_deref() == Some(player_js_url)
    }

    /// Decrypt a signature using the extracted function.
    pub fn decrypt_signature(&self, encrypted_sig: &str) -> Result<String> {
        let sig_func = self
            .sig_function
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Signature function not loaded"))?;
        let helper = self
            .helper_code
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Helper code not loaded"))?;

        let mut engine = JsEngine::new();

        // Load helper object first (contains swap, splice, reverse operations)
        engine.load(helper)?;

        // Execute the signature function
        let result = engine.call_function(sig_func, encrypted_sig)?;
        Ok(result)
    }

    /// Transform the n-parameter to bypass throttling.
    pub fn transform_n_param(&self, n_value: &str) -> Result<String> {
        let n_func = self
            .n_function
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("N-parameter function not loaded"))?;

        let mut engine = JsEngine::new();
        let result = engine.call_function(n_func, n_value)?;

        // If the result looks like the original, it might have failed silently
        if result == n_value {
            log::warn!("N-parameter transformation returned the same value, may be broken");
        }

        Ok(result)
    }

    /// Invalidate the cached functions (e.g., when decryption fails).
    pub fn invalidate(&mut self) {
        self.player_js_url = None;
        self.sig_function = None;
        self.n_function = None;
        self.helper_code = None;
    }
}

/// Create a thread-safe shared SignatureResolver.
pub fn shared_resolver() -> Arc<Mutex<SignatureResolver>> {
    Arc::new(Mutex::new(SignatureResolver::new()))
}

/// Extract the initial signature decryption function from player.js.
///
/// YouTube's player.js contains a function that decrypts the signature. It typically:
/// 1. Splits the input string into an array
/// 2. Calls a helper object's methods (swap, splice, reverse)
/// 3. Joins the array back into a string
fn extract_signature_function(source: &str) -> Result<String> {
    // Pattern 1: Find the function that's assigned to the signature decryption variable.
    // Looks for patterns like:
    //   var Xy={...}; function Xz(a){a=a.split("");Xy.ab(a,3);Xy.cd(a,2);...;return a.join("")}
    let patterns = [
        // \b[cs]\s*&&\s*[adf]\.set\([^,]+\s*,\s*encodeURIComponent\(([a-zA-Z0-9$]+)\(
        r#"\b[cs]\s*&&\s*[adf]\.set\([^,]+\s*,\s*encodeURIComponent\(([a-zA-Z0-9$]+)\("#,
        // \bm=([a-zA-Z0-9$]{2,})\(decodeURIComponent\(h\.s\)\)
        r#"\bm=([a-zA-Z0-9$]{2,})\(decodeURIComponent\(h\.s\)\)"#,
        // \bc\s*&&\s*d\.set\([^,]+\s*,\s*(?:encodeURIComponent\s*\()([a-zA-Z0-9$]+)\(
        r#"\bc\s*&&\s*d\.set\([^,]+\s*,\s*(?:encodeURIComponent\s*\()([a-zA-Z0-9$]+)\("#,
        // \bc\s*&&\s*[a-z]\.set\([^,]+\s*,\s*([a-zA-Z0-9$]+)\(
        r#"\bc\s*&&\s*[a-z]\.set\([^,]+\s*,\s*([a-zA-Z0-9$]+)\("#,
        // \bc\s*&&\s*[a-z]\.set\([^,]+\s*,\s*encodeURIComponent\(([a-zA-Z0-9$]+)\(
        r#"\bc\s*&&\s*[a-z]\.set\([^,]+\s*,\s*encodeURIComponent\(([a-zA-Z0-9$]+)\("#,
    ];

    let mut func_name = None;
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(source) {
                func_name = Some(cap[1].to_string());
                break;
            }
        }
    }

    let func_name = func_name.ok_or_else(|| {
        anyhow::anyhow!("Could not find signature function name in player.js")
    })?;

    // Now extract the function body
    let escaped_name = regex::escape(&func_name);
    let func_pattern = format!(
        r#"(?:function\s+{name}\s*|[{{;,]]\s*{name}\s*=\s*function\s*|var\s+{name}\s*=\s*function\s*)\(([^)]*)\)\s*\{{([^}}]+)\}}"#,
        name = escaped_name
    );

    if let Ok(re) = Regex::new(&func_pattern) {
        if let Some(cap) = re.captures(source) {
            let params = &cap[1];
            let body = &cap[2];
            return Ok(format!("function({}) {{ {} }}", params, body));
        }
    }

    // Fallback: try a more lenient extraction with brace counting
    extract_function_by_name(source, &func_name)
}

/// Extract the n-parameter transformation function from player.js.
fn extract_n_function(source: &str) -> Result<String> {
    // Pattern to find n-param function reference
    let patterns = [
        r#"\.get\("n"\)\)&&\(b=([a-zA-Z0-9$]+)(?:\[(\d+)\])?\([a-zA-Z0-9]\)"#,
        r#"([a-zA-Z0-9$]+)\s*=\s*function\([a-zA-Z]\)\s*\{[^}]*?join\(""\)"#,
        r#"\b([a-zA-Z0-9$]{2,})\s*=\s*function\(a\)\s*\{a=a\.split\(""\)"#,
    ];

    let mut func_name = None;
    let mut array_index = None;

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(source) {
                func_name = Some(cap[1].to_string());
                array_index = cap.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
                break;
            }
        }
    }

    let func_name =
        func_name.ok_or_else(|| anyhow::anyhow!("Could not find n-param function in player.js"))?;

    // If it's an array reference like b=arr[0](a), find the array and get the function
    if let Some(idx) = array_index {
        let escaped_name = regex::escape(&func_name);
        let array_pattern = format!(r#"var\s+{}\s*=\s*\[([^\]]+)\]"#, escaped_name);
        if let Ok(re) = Regex::new(&array_pattern) {
            if let Some(cap) = re.captures(source) {
                let elements: Vec<&str> = cap[1].split(',').collect();
                if let Some(actual_name) = elements.get(idx) {
                    let actual_name = actual_name.trim();
                    return extract_function_by_name(source, actual_name);
                }
            }
        }
    }

    extract_function_by_name(source, &func_name)
}

/// Extract the helper object that signature functions depend on.
/// These contain methods like: reverse, splice, swap (rotate).
fn extract_helper_object(source: &str, sig_function: &str) -> Result<String> {
    // Find what object the signature function calls (e.g., "Xy.ab(a,3)" → extract "Xy")
    let re = Regex::new(r"([a-zA-Z0-9$]+)\.[a-zA-Z0-9$]+\(a,\d+\)")?;
    let helper_name = re
        .captures(sig_function)
        .map(|cap| cap[1].to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not find helper object reference in sig function"))?;

    // Extract the helper object definition: var Xy={...};
    let escaped_name = regex::escape(&helper_name);
    let obj_pattern = format!(
        r#"var\s+{}\s*=\s*\{{([\s\S]*?)\}}\s*;"#,
        escaped_name
    );

    if let Ok(re) = Regex::new(&obj_pattern) {
        if let Some(cap) = re.captures(source) {
            return Ok(format!("var {} = {{ {} }};", helper_name, &cap[1]));
        }
    }

    // Return empty helper if not found (some functions are self-contained)
    log::warn!("Could not extract helper object '{}', sig function may fail", helper_name);
    Ok(String::new())
}

/// Extract a named function from JavaScript source by counting braces.
fn extract_function_by_name(source: &str, name: &str) -> Result<String> {
    let escaped = regex::escape(name);

    // Try: function name(params) { body }
    let patterns = [
        format!(r"function\s+{}\s*\(", escaped),
        format!(r"var\s+{}\s*=\s*function\s*\(", escaped),
        format!(r"[{{;,]\s*{}\s*=\s*function\s*\(", escaped),
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(m) = re.find(source) {
                // Find the opening paren of the function parameters
                let start_from = m.start();
                if let Some(func) = extract_function_at(source, start_from) {
                    return Ok(func);
                }
            }
        }
    }

    anyhow::bail!("Could not extract function '{}' from player.js", name)
}

/// Extract a complete function starting at a given position by counting braces.
fn extract_function_at(source: &str, start: usize) -> Option<String> {
    let rest = &source[start..];

    // Find the start of "function"
    let func_start = rest.find("function")?;
    let from_func = &rest[func_start..];

    // Find the opening brace
    let open_brace = from_func.find('{')?;
    let mut depth = 0;
    let mut end = 0;

    for (i, ch) in from_func.char_indices() {
        if i < open_brace {
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end > 0 {
        Some(from_func[..end].to_string())
    } else {
        None
    }
}

/// Apply n-parameter transformation to a URL.
pub fn apply_n_param(
    url: &str,
    resolver: &SignatureResolver,
) -> Result<String> {
    // Find the n parameter in the URL
    let mut parsed = url::Url::parse(url)?;

    let n_value: Option<String> = parsed
        .query_pairs()
        .find(|(key, _)| key == "n")
        .map(|(_, val)| val.to_string());

    if let Some(n_val) = n_value {
        let transformed = resolver.transform_n_param(&n_val)?;

        // Replace the n parameter value in the URL
        let new_pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(k, v)| {
                if k == "n" {
                    (k.to_string(), transformed.clone())
                } else {
                    (k.to_string(), v.to_string())
                }
            })
            .collect();

        parsed.query_pairs_mut().clear();
        for (k, v) in &new_pairs {
            parsed.query_pairs_mut().append_pair(k, v);
        }

        Ok(parsed.to_string())
    } else {
        Ok(url.to_string())
    }
}
