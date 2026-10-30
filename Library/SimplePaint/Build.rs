// Tells the build scripts of packages that depend on this one where
// SimplePaint's shaders are, as the DEP_SIMPLE_PAINT_SHADERS environment
// variable. Hosts compile SimplePaint.hlsl themselves, with their own
// SIMPLE_PAINT_MATERIAL_COUNT. See Usage.md.

fn main() {
    let manifest_directory = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let shaders = manifest_directory.join("Shaders");
    println!("cargo::metadata=shaders={}", shaders.display());
    println!("cargo::rerun-if-changed=Build.rs");
}
