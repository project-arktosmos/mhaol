use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist_dir = manifest_dir.join("../frontend/dist-static");
    fs::create_dir_all(&dist_dir).ok();

    let index = dist_dir.join("index.html");
    if !index.exists() {
        fs::write(
            &index,
            "<!doctype html><html><body>cloud web frontend has not been built yet — run `pnpm --filter frontend build`</body></html>\n",
        )
        .ok();
    }

    // Bake any build-time API keys into a generated source file so the
    // values end up as plain string literals in the binary. Using a
    // generated source file (rather than `option_env!`) means cargo's
    // change detection works through the file's mtime/contents, which
    // is robust against `Swatinem/rust-cache` restoring an older
    // `.fingerprint/` directory that thinks the env var is unchanged.
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let embedded = out_dir.join("embedded_keys.rs");
    let tmdb = env::var("TMDB_API_KEY").unwrap_or_default();
    let generated = format!("pub const TMDB_API_KEY: &str = {tmdb:?};\n");
    fs::write(&embedded, generated).expect("failed to write embedded_keys.rs");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../frontend/dist-static");
    println!("cargo:rerun-if-env-changed=TMDB_API_KEY");
}
