// Library/game-input-raw holds the bindings for the part of GameInput's API
// that reads keyboards, mice and gamepads, in the shape of windows-bindgen's
// output, and nothing else, and Library/game-input what KGames adds to them:
// the committed bindings are exactly what bindings-generator writes now, what
// GameInput's header declares is bound if it's in bindings-generator's scope
// and isn't otherwise, every item in the raw package is of a kind that
// windows-bindgen writes, every method calls its own vtable slot, and no
// bindings are in the other package.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// How each item that windows-bindgen writes starts, on a line of its own:
// constants and their types, structs, callbacks, interfaces with their methods
// and vtables, and functions.
const ITEMS: [&str; 17] = [
    "#![allow(",
    "pub type ",
    "pub const ",
    "#[repr(",
    "#[derive(",
    "pub struct ",
    "impl Default for ",
    "windows_core::imp::define_interface!(",
    "windows_core::imp::interface_hierarchy!(",
    "impl IGameInput",
    "unsafe impl Send for ",
    "unsafe impl Sync for ",
    "impl windows_core::RuntimeName for ",
    "#[inline]",
    "pub unsafe fn ",
    "where",
    ")]",
];

// What only bindings have: declarations of the functions that they call,
// COM interfaces, and structs laid out as C lays them out.
const BINDINGS: [&str; 4] = ["extern \"", "link!", "define_interface!", "#[repr(C)]"];

#[test]
fn the_bindings_are_what_bindings_generator_writes() {
    let generated = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("game_input{}.rs", std::process::id()));
    game_input_bindings_generator::generate(&game_input_bindings_generator::header(), &generated);
    let fresh = read(&generated);
    let _ = std::fs::remove_file(&generated);
    let committed = read(&library("game-input-raw").join("src/lib.rs"));
    assert!(
        fresh == committed,
        "Library/game-input-raw/src/lib.rs isn't what bindings-generator writes: regenerate it with \
         `cargo run -p game-input-bindings-generator`, and don't edit it by hand"
    );
}

#[test]
fn the_raw_package_is_only_the_bindings() {
    let raw = library("game-input-raw");
    assert_eq!(names(&raw.join("src")), ["lib.rs"]);
    // The bindings need only windows-core. The build script links
    // GameInput.lib, and has no dependencies.
    assert_eq!(dependencies(&read(&raw.join("Cargo.toml"))), ["windows-core"]);

    let text = read(&raw.join("src/lib.rs"));
    let mut lines = text.lines().enumerate().map(|(index, line)| (index + 1, line));
    while let Some((number, line)) = lines.next() {
        // An item starts at the line's start. Comments, and what closes an
        // item, are what else is there.
        if line.is_empty() || line.starts_with(' ') || line.starts_with("//") || ["}", ")", ");", ">;", "{"].contains(&line) {
            continue;
        }
        assert!(
            ITEMS.iter().any(|item| line.starts_with(item)),
            "src/lib.rs:{number} isn't an item that windows-bindgen writes: {line}"
        );
        // A function declares the function that it calls, and calls it.
        if let Some(signature) = line.strip_prefix("pub unsafe fn ") {
            let name = signature.split(['(', '<']).next().unwrap();
            let body: Vec<&str> = lines.by_ref().map(|(_, line)| line).take_while(|&line| line != "}").collect();
            let declared = body.iter().position(|line| line.contains("extern \"system\"") || line.contains("link!("));
            assert!(
                declared.is_some_and(|declared| body[declared..].iter().any(|line| line.contains(&format!("fn {name}(")))),
                "src/lib.rs:{number}: {name} doesn't declare the function that it binds"
            );
        }
        // An interface's methods each call its vtable's slot of the same name.
        // Its vtable's own impl makes a vtable for an implementation of it.
        if line.starts_with("impl IGameInput") && !line.contains("_Vtbl") {
            let mut method: Option<(usize, String)> = None;
            let mut called = true;
            for (number, line) in lines.by_ref() {
                if line == "}" || line.starts_with("    pub unsafe fn ") {
                    let (start, name) = method.take().unwrap_or_default();
                    assert!(called, "src/lib.rs:{start}: {name} doesn't call its own vtable slot");
                }
                if line == "}" {
                    break;
                }
                if let Some(signature) = line.strip_prefix("    pub unsafe fn ") {
                    let name = signature.split(['(', '<']).next().unwrap().to_string();
                    called = false;
                    method = Some((number, name));
                } else if line.starts_with("    ") && !line.starts_with("     ") && !line.trim().starts_with("//") {
                    // Only methods are at the impl's own indent: no helpers,
                    // no safe functions, and no constants.
                    let trimmed = line.trim();
                    assert!(
                        ["where", "{", "}"].contains(&trimmed) || trimmed.starts_with(')'),
                        "src/lib.rs:{number} isn't a method that calls GameInput: {line}"
                    );
                }
                if let Some((_, name)) = &method
                    && line.contains(&format!("vtable(self).{name})"))
                {
                    called = true;
                }
            }
        }
    }
}

// What GameInput's v4 header declares is bound if it's in bindings-generator's
// scope, and isn't if it's not: its enums with their values, structs,
// callbacks, typedef, constants and function, and its interfaces. An interface
// that's bound has every method's slot, in the header's order: the method's
// own if it's in scope, and a placeholder if it's not, which has no method.
// Everything in the scope is in the header. The header's IID_IGameInput is
// IGameInput::IID, and its inline GameInputCreate calls GameInputInitialize.
#[test]
fn the_bindings_are_the_scope_of_the_header() {
    use game_input_bindings_generator::{CONSTANTS, FUNCTIONS, INTERFACES, TYPES};

    let header = read(&game_input_bindings_generator::header());
    let bindings = read(&library("game-input-raw").join("src/lib.rs"));
    let interfaces: Vec<&str> = INTERFACES.iter().map(|(name, _)| *name).collect();
    // What the header declares of the scope, to compare with the scope.
    let mut declared: BTreeSet<&str> = BTreeSet::new();
    // Whether a declaration is bound, which is whether it's in the scope.
    let mut bound = |name: &str, scope: &[&'static str], binding: String| {
        let scoped = scope.iter().find(|scoped| **scoped == name);
        declared.extend(scoped);
        let found = bindings.contains(&binding);
        assert_eq!(found, scoped.is_some(), "src/lib.rs's `{binding}`, and {name} in the scope");
        found
    };

    let mut enumeration: Option<(String, bool)> = None;
    let mut interface: Option<(String, Vec<String>)> = None;
    for line in header.lines().map(|line| line.split("//").next().unwrap().trim_end()) {
        let words: Vec<&str> = line.split([' ', '\t', '(', ')', ',', ';', '*']).filter(|word| !word.is_empty()).collect();
        if line.starts_with("};") {
            enumeration = None;
            let Some((name, methods)) = interface.take() else { continue };
            if !bound(&name, &interfaces, format!("pub struct {name}_Vtbl {{")) {
                continue;
            }
            let scope = INTERFACES.iter().find(|(interface, _)| *interface == name).unwrap().1;
            for method in scope {
                assert!(methods.contains(&method.to_string()), "GameInput.h's {name} has no {method}");
            }
            let slots: Vec<(String, bool)> = methods.iter().map(|method| (method.clone(), scope.contains(&method.as_str()))).collect();
            assert_eq!(vtable(&bindings, &name), slots, "{name}'s vtable");
            let scoped: Vec<&String> = slots.iter().filter(|(_, scoped)| *scoped).map(|(method, _)| method).collect();
            assert_eq!(methods_of(&bindings, &name).iter().collect::<Vec<_>>(), scoped, "{name}'s methods");
        } else if let Some((name, scoped)) = &enumeration {
            if let Some((member, _)) = line.trim().split_once('=') {
                let binding = format!("pub const {}: {name} = ", member.trim());
                assert_eq!(bindings.contains(&binding), *scoped, "src/lib.rs's `{binding}`");
            }
        } else if let Some((_, methods)) = &mut interface {
            if line.trim_start().starts_with("IFACEMETHOD") {
                // IFACEMETHOD(Name) returns an HRESULT, IFACEMETHOD_(Type, Name) a type.
                methods.push(words[if words[0] == "IFACEMETHOD_" { 2 } else { 1 }].to_string());
            }
        } else {
            match words.as_slice() {
                ["enum", name] => {
                    let scoped = bound(name, &TYPES, format!("pub type {name} = i32;"));
                    enumeration = Some((name.to_string(), scoped));
                }
                ["struct", name] => _ = bound(name, &TYPES, format!("pub struct {name} {{")),
                ["typedef", "uint64_t", name] => _ = bound(name, &TYPES, format!("pub struct {name}(")),
                ["typedef", "void", "CALLBACK", name, ..] => _ = bound(name, &TYPES, format!("pub type {name} = Option<")),
                ["DECLARE_INTERFACE_IID_", name, ..] => interface = Some((name.to_string(), Vec::new())),
                ["const", "IID", name, ..] => assert_eq!(*name, "IID_IGameInput"),
                ["const" | "constexpr", _, name, "=", ..] | ["#define", name @ "GAMEINPUT_API_VERSION", _] => {
                    bound(name, &CONSTANTS, format!("pub const {name}: "));
                }
                ["STDAPI", name, ..] => _ = bound(name, &FUNCTIONS, format!("pub unsafe fn {name}<")),
                ["inline", "HRESULT", name, ..] => assert_eq!(*name, "GameInputCreate"),
                _ => {}
            }
        }
    }
    let scope: BTreeSet<&str> = TYPES.iter().chain(&CONSTANTS).chain(&FUNCTIONS).chain(&interfaces).copied().collect();
    assert_eq!(declared, scope, "what GameInput.h declares of the scope");
}

// An interface's vtable's slots, after IUnknown's, in order: each one's name,
// and whether it's bound, or a placeholder, which is a usize.
fn vtable(bindings: &str, interface: &str) -> Vec<(String, bool)> {
    let start = format!("pub struct {interface}_Vtbl {{\n");
    let body = &bindings[bindings.find(&start).unwrap_or_else(|| panic!("no {start}")) + start.len()..];
    let mut slots = Vec::new();
    for line in body.lines().take_while(|line| *line != "}") {
        // A slot's name starts a line, and its type's lines follow.
        let slot = line.strip_prefix("    pub ").or_else(|| line.strip_prefix("    ")).unwrap_or(line);
        let Some((name, ty)) = slot.split_once(':') else { continue };
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        if name != "base__" {
            slots.push((name.to_string(), ty.trim() != "usize,"));
        }
    }
    slots
}

// The names of an interface's methods, in order.
fn methods_of(bindings: &str, interface: &str) -> Vec<String> {
    let start = format!("\nimpl {interface} {{\n");
    let body = &bindings[bindings.find(&start).unwrap_or_else(|| panic!("no {start}")) + start.len()..];
    let methods = body.lines().take_while(|line| *line != "}").filter_map(|line| line.strip_prefix("    pub unsafe fn "));
    methods.map(|signature| signature.split(['(', '<']).next().unwrap().to_string()).collect()
}

#[test]
fn what_goes_beyond_the_bindings_has_no_bindings() {
    let package = library("game-input");
    for file in rust_files(&package) {
        let text = read(&file);
        for binding in BINDINGS {
            assert!(!text.contains(binding), "{} has {binding}: bindings go in Library/game-input-raw", file.display());
        }
    }
    let dependencies = dependencies(&read(&package.join("Cargo.toml")));
    assert!(!dependencies.contains(&"game-input-bindings-generator".to_string()));
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
