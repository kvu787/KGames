// Every 3D model is drawn with SimplePaint.hlsl or Unlit.hlsl, as the
// AGENTS.md at the root says. This lists every shader in the repository,
// with what it draws, and fails on one that isn't listed, and on HLSL
// written into Rust, so that a shader for models can't come in beside those
// two without the rule being read. A shader that draws something that isn't
// a model goes on the list, with what it draws.

use std::path::{Path, PathBuf};

// Each shader, from the root of the repository, and what it draws.
const SHADERS: [(&str, &str); 7] = [
    ("Library/simple-paint/Shaders/SimplePaint.hlsl", "3D models, painted with SimplePaint"),
    ("Library/simple-paint/Shaders/SimplePaintCore.hlsli", "SimplePaint's math, which SimplePaint.hlsl includes"),
    ("Library/simple-paint/Shaders/Unlit.hlsl", "3D models that aren't painted, unlit"),
    ("Library/user-interface/Shaders/Interface.hlsl", "the interface's shapes and glyphs"),
    ("Program/mesh-modeler/Shaders/Overlay.hlsl", "the modeler's overlay of edges, dots, tints and grid over its mesh"),
    ("Program/shadow-testing/Shaders/Common.hlsli", "what ShadowTesting's 2D shadows share, which each includes"),
    ("Program/shadow-testing/Shaders/Linear.hlsl", "ShadowTesting's 2D box and its linear shadow"),
];
// The extensions of shader sources.
const EXTENSIONS: [&str; 6] = ["hlsl", "hlsli", "fx", "fxh", "glsl", "wgsl"];

// Every file under `folder`, but not in what Git and Cargo keep of their own,
// or in a worktree, which are in folders whose names start with a period.
fn files(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().unwrap().is_dir() {
            if !name.starts_with('.') && !["target", "bin", "obj"].contains(&name.as_str()) {
                files(&path, found);
            }
        } else {
            found.push(path);
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let mut found = Vec::new();
    files(&root, &mut found);
    let relative = |path: &Path| path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");

    let mut shaders: Vec<String> = found
        .iter()
        .filter(|path| path.extension().is_some_and(|extension| EXTENSIONS.contains(&extension.to_string_lossy().to_lowercase().as_str())))
        .map(|path| relative(path))
        .collect();
    shaders.sort();
    let mut listed: Vec<String> = SHADERS.iter().map(|(path, _)| path.to_string()).collect();
    listed.sort();
    assert_eq!(
        shaders, listed,
        "the shaders aren't the ones listed. Every 3D model is drawn with SimplePaint.hlsl or Unlit.hlsl, as the AGENTS.md at the root says"
    );

    // A pixel shader's output, written into Rust instead of a shader file.
    let output = ["SV_", "TARGET"].concat();
    let this = Path::new(file!()).file_name().unwrap();
    for path in found.iter().filter(|path| path.extension().is_some_and(|extension| extension == "rs") && path.file_name() != Some(this)) {
        let text = String::from_utf8_lossy(&std::fs::read(path).unwrap()).to_uppercase();
        assert!(!text.contains(&output), "{} has a shader in it", relative(path));
    }
    println!("The repository's {} shaders are the ones listed, and no Rust has one in it.", shaders.len());
}
