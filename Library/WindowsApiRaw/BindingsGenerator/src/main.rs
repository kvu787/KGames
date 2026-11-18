// Regenerates src/bindings.rs from the API names listed in Filters.txt.
// Run it from anywhere in the repository after changing Filters.txt:
//
//     cargo run -p bindings_generator
//
// The generated file is committed, so building the programs doesn't need this tool.

use std::path::Path;

fn main() {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = generator.parent().unwrap();

    windows_bindgen::builder()
        .filter_file(generator.join("Filters.txt"))
        .output(project.join("src/bindings.rs"))
        .flat()
        .write();
}
