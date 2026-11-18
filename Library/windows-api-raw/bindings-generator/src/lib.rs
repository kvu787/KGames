// Writes the Windows API bindings that windows-bindgen generates from the API
// names listed in Filters.txt. main.rs writes them to WindowsApiRaw's
// src/bindings.rs, and Test/WindowsApi writes them somewhere else, to check
// that the committed file is still exactly what this writes.

use std::path::Path;

pub fn generate(output: &Path) {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"));
    windows_bindgen::builder()
        .filter_file(generator.join("Filters.txt"))
        .output(output)
        .flat()
        .write();
}
