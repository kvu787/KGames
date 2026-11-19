// Regenerates GameInputRaw's src/lib.rs from GameInput.h. Run it from anywhere
// in the repository after changing this generator or the package's version:
//
//     cargo run -p game-input-bindings-generator
//
// It reads the header from the NuGet package that GameInputRaw's build script
// restores. The generated file is committed, so building the programs doesn't
// need this tool.

use std::path::Path;

fn main() {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"));
    let header = game_input_bindings_generator::header();
    game_input_bindings_generator::generate(&header, &generator.parent().unwrap().join("src/lib.rs"));
}
