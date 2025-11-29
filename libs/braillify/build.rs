#[cfg(windows)]
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    // wasm 타겟으로 빌드할 때는 build.rs를 건너뜀
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm32") {
        return;
    }

    #[cfg(windows)]
    {
        embed_manifest(new_manifest("Braillify.Braillify")).expect("unable to embed manifest file");
    }
}
