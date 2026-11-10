// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use mesh_modeler::modeler::Modeler;
use user_interface::Options;

fn main() -> ExitCode {
    user_interface::run(Options::new("MeshModeler", env!("CARGO_MANIFEST_DIR")), |startup| Ok(Modeler::start(startup)))
}
