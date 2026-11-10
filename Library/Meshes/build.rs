// Makes the car, car.rs in OUT_DIR, which src/lib.rs includes as the
// car module, from the model that Blender exported for Veehiicuul,
// Program/Veehiicuul/Models/SlopeCar.glb: its vertices, its triangles'
// indices, and a constant for each of its parts' materials. So the car here
// is always the one in that file.

use std::fmt::Write;
use std::path::PathBuf;

// From the repository's root.
const MODEL: &str = "Program/Veehiicuul/Models/SlopeCar.glb";

// Each part's constant, and its material's name in the file. A constant's
// value is its place here, which the vertices' material indices use. All
// four wheels are one part, and so are both headlights.
const PARTS: [(&str, &str); 5] = [
    ("AXLES", "SlopeCarAxles"),
    ("BODY", "SlopeCarBodyBlue"),
    ("CABIN", "SlopeCarCabinBlue"),
    ("HEADLIGHTS", "SlopeCarHeadlights"),
    ("WHEELS", "SlopeCarWheels"),
];

// Fails the build with the message, which Cargo shows as an error, after
// the script ends.
fn fail(message: &str) -> ! {
    println!("cargo::error={message}");
    std::process::exit(0);
}

fn main() {
    let manifest_directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = MODEL.split('/').fold(manifest_directory.join("..").join(".."), |path, part| path.join(part));
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", path.display());
    let bytes = std::fs::read(&path).unwrap_or_else(|error| fail(&format!("Couldn't read {MODEL}: {error}")));
    // Git LFS keeps the model, and leaves a small text file in its place until
    // it checks the model out, as in a new worktree.
    if bytes.starts_with(b"version https://git-lfs.github.com") {
        fail(&format!(
            "{MODEL} is a Git LFS pointer file, not the model. \
             Run `git lfs checkout` to put the model there, then build again."
        ));
    }
    let model = gltf::parse(&bytes).unwrap_or_else(|problem| fail(&format!("{MODEL}: {problem}")));

    // Each of the file's materials' part.
    let mut parts = Vec::new();
    for material in &model.material_list {
        match PARTS.iter().position(|&(_, name)| name == material.name) {
            Some(part) => parts.push(part as u32),
            None => {
                let name = &material.name;
                fail(&format!("{MODEL} has a material, {name}, that isn't one of the car's parts"))
            }
        }
    }
    for (part, (_, name)) in PARTS.iter().enumerate() {
        if !parts.contains(&(part as u32)) {
            fail(&format!("{MODEL} has no material {name}"));
        }
    }

    let mut code = String::new();
    let mut line = |text: &str| {
        code.push_str(text);
        code.push('\n');
    };
    line("// Made by Library/Meshes/build.rs from Program/Veehiicuul/Models/SlopeCar.glb.");
    line("");
    line("use simple_paint::Vertex;");
    line("");
    for (part, (constant, _)) in PARTS.iter().enumerate() {
        line(&format!("pub const {constant}: u32 = {part};"));
    }
    line(&format!("pub const MATERIAL_COUNT: u32 = {};", PARTS.len()));
    line("");
    line("const fn vertex(position: [f32; 3], normal: [f32; 3], material_index: u32) -> Vertex {");
    line("    Vertex { position, normal, material_index }");
    line("}");
    line("");
    // Debug writes each number with the fewest digits that read back as the
    // same f32.
    line(&format!("pub static VERTICES: [Vertex; {}] = [", model.positions.len()));
    for ((position, normal), material) in model.positions.iter().zip(&model.normals).zip(&model.materials) {
        line(&format!("    vertex({position:?}, {normal:?}, {}),", parts[*material as usize]));
    }
    line("];");
    line("");
    line(&format!("pub static INDICES: [u32; {}] = [", model.indices.len()));
    for row in model.indices.chunks(16) {
        let mut text = String::from("   ");
        for index in row {
            write!(text, " {index},").unwrap();
        }
        line(&text);
    }
    line("];");

    let output = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("car.rs");
    std::fs::write(&output, code)
        .unwrap_or_else(|error| fail(&format!("Couldn't write {}: {error}", output.display())));
}
