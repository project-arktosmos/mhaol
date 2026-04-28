use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist_dir = manifest_dir.join("../cloud-web/dist-static");
    fs::create_dir_all(&dist_dir).ok();

    let index = dist_dir.join("index.html");
    if !index.exists() {
        fs::write(
            &index,
            "<!doctype html><html><body>cloud-web has not been built yet — run `pnpm --filter cloud-web build`</body></html>\n",
        )
        .ok();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../cloud-web/dist-static");
}
