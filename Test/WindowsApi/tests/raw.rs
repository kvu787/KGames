// Library/WindowsApiRaw holds only what windows-bindgen generates, and
// Library/WindowsApi what KGames adds to it: the committed bindings are
// exactly what BindingsGenerator writes now, nothing else is in the raw
// package, and no bindings are in the other one.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// What only bindings have: declarations of the functions that they call,
// COM interfaces, and structs laid out as C lays them out.
const BINDINGS: [&str; 4] = ["extern \"", "link!", "define_interface!", "#[repr(C)]"];

#[test]
fn the_bindings_are_what_windows_bindgen_generates() {
    let generated = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("bindings{}.rs", std::process::id()));
    bindings_generator::generate(&generated);
    let fresh = read(&generated);
    let _ = std::fs::remove_file(&generated);
    let committed = read(&library("WindowsApiRaw").join("src/bindings.rs"));
    assert!(
        fresh == committed,
        "Library/WindowsApiRaw/src/bindings.rs isn't what BindingsGenerator writes: regenerate it with \
         `cargo run -p bindings_generator`, and don't edit it by hand"
    );
}

#[test]
fn the_raw_package_is_only_the_bindings() {
    let raw = library("WindowsApiRaw");
    // Its source is the generated file, and the file that exports it.
    assert_eq!(names(&raw.join("src")), ["bindings.rs", "lib.rs"]);
    assert_eq!(code(&read(&raw.join("src/lib.rs"))), "mod bindings; pub use bindings::*;");
    // It builds nothing else, and needs only windows-core, as the bindings do.
    assert!(!raw.join("build.rs").exists());
    assert_eq!(dependencies(&read(&raw.join("Cargo.toml"))), ["windows-core"]);
}

#[test]
fn what_goes_beyond_the_bindings_has_no_bindings() {
    let package = library("WindowsApi");
    for file in rust_files(&package) {
        let text = read(&file);
        for binding in BINDINGS {
            assert!(!text.contains(binding), "{} has {binding}: bindings go in Library/WindowsApiRaw", file.display());
        }
    }
    assert!(!dependencies(&read(&package.join("Cargo.toml"))).contains(&"bindings_generator".to_string()));
}

fn library(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Library").join(name)
}

// A text file, with the line endings that Git may give it on checkout as
// they're committed.
fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())).replace("\r\n", "\n")
}

// The names in a folder, in order.
fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// The package's Rust files: its build script, if it has one, and its source.
fn rust_files(package: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut folders = vec![package.join("src")];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.extend(Some(package.join("build.rs")).filter(|build| build.exists()));
    files
}

// A Rust file's code, without its comments and attributes, with its words
// one space apart.
fn code(text: &str) -> String {
    let joined = text.lines().map(|line| line.split("//").next().unwrap()).collect::<Vec<_>>().join(" ");
    let mut code = String::new();
    let mut characters = joined.chars();
    while let Some(character) = characters.next() {
        if character == '#' {
            // An attribute, to its closing bracket.
            let mut depth = 0;
            for inside in characters.by_ref() {
                match inside {
                    '[' => depth += 1,
                    ']' if depth == 1 => break,
                    ']' => depth -= 1,
                    _ => {}
                }
            }
        } else {
            code.push(character);
        }
    }
    code.split_whitespace().collect::<Vec<_>>().join(" ")
}

// The names of what a package's manifest depends on, of every kind: to build,
// to run its build script, and to test.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    let mut table = String::new();
    for line in manifest.lines().map(str::trim).filter(|line| !line.starts_with('#')) {
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            table = name.to_string();
            if let Some((kind, dependency)) = table.split_once('.')
                && kind.ends_with("dependencies")
            {
                names.insert(dependency.to_string());
            }
        } else if table.ends_with("dependencies")
            && let Some((name, _)) = line.split_once('=')
        {
            names.insert(name.trim().to_string());
        }
    }
    names.into_iter().collect()
}
