// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use veehiicuul::program::Veehiicuul;

fn main() -> windows_core::Result<()> {
    user_interface::run::<Veehiicuul>(Path::new(env!("CARGO_MANIFEST_DIR")))
}
