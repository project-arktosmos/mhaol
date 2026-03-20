use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // Read .env.app from workspace root and pass API keys as compile-time env vars.
    // This ensures they are baked into the binary for Android where .env.app is unavailable.
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env.app");
        }
        if !dir.pop() {
            break PathBuf::from(".env.app");
        }
    };

    if let Ok(content) = std::fs::read_to_string(&env_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(eq_idx) = trimmed.find('=') {
                let key = trimmed[..eq_idx].trim();
                let value = trimmed[eq_idx + 1..].trim();
                if !key.is_empty() {
                    println!("cargo:rustc-env={}={}", key, value);
                }
            }
        }
        println!("cargo:rerun-if-changed={}", env_path.display());
    }
}
