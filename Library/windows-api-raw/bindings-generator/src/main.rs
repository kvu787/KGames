// Regenerates src/bindings.rs from the API names listed in Filters.txt.
// Run it from anywhere in the repository after changing Filters.txt:
//
//     cargo run -p bindings-generator
//
// The generated file is committed, so building the programs doesn't need this tool.

use std::path::Path;

fn main() {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"));
    bindings_generator::generate(&generator.parent().unwrap().join("src/bindings.rs"));
}
