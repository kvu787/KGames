// Regenerates Source/Bindings.rs from the API names listed in Filters.txt.
// Run it from the HelloDirect3D12 folder after changing Filters.txt:
//
//     cargo run --manifest-path BindingsGenerator/Cargo.toml
//
// The generated file is committed, so building the game doesn't need this tool.

use std::path::Path;

fn main() {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = generator.parent().unwrap();

    windows_bindgen::builder()
        .filter_file(generator.join("Filters.txt"))
        .output(project.join("Source/Bindings.rs"))
        .flat()
        .write();
}
