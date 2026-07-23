#[cfg(windows)]
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    // Declare `tarpaulin_include` and `tarpaulin` as known cfg names so
    // `#[cfg(not(tarpaulin_include))]` (used to exclude interactive-only code from
    // coverage) and `#[cfg_attr(tarpaulin, inline(never))]` (used to prevent the
    // inliner from collapsing coverage attribution) do not trip `unexpected_cfgs`.
    println!("cargo::rustc-check-cfg=cfg(tarpaulin_include)");
    println!("cargo::rustc-check-cfg=cfg(tarpaulin)");

    // wasm 타겟으로 빌드할 때는 build.rs를 건너뜀
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("wasm32") {
        #[cfg(windows)]
        {
            embed_manifest(new_manifest("Braillify.Braillify"))
                .expect("unable to embed manifest file");
        }
    }
}
