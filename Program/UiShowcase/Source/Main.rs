// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use ui_showcase::showcase::Showcase;
use windows_core::Result;

fn main() -> Result<()> {
    user_interface::run::<Showcase>(Path::new(env!("CARGO_MANIFEST_DIR")))
}
