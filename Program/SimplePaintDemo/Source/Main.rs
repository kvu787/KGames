// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use simple_paint_demo::demo::Demo;
use windows_core::Result;

fn main() -> Result<()> {
    user_interface::run::<Demo>(Path::new(env!("CARGO_MANIFEST_DIR")))
}
