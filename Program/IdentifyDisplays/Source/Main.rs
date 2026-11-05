// Shows each display's identifier on that display, and writes the
// identifiers to the console. Set "Display" in a program's settings file to
// an identifier, or run the program with --display and the identifier, to
// open it on that display.
//
// Each display gets a box in its middle, drawn with Library/UserInterface:
// its identifier, its model and its mode. The boxes close when one is
// clicked, or when Esc is pressed while one has focus. With --list, it only
// writes to the console.

use std::process::ExitCode;

use displays::Display;
use identify_displays::Identity;
use user_interface::{Fonts, Overlay, ThemeKind, run_overlays};
use windows_api::RECT;

fn print(displays: &[Display], boxes: bool) {
    let count = displays.len();
    println!("{count} {}, from left to right:", if count == 1 { "display" } else { "displays" });
    for display in displays {
        let primary = if display.primary { ", the primary display" } else { "" };
        let indent = " ".repeat(display.identifier.len() + 4);
        println!();
        println!("  {}  {}{primary}", display.identifier, display.name);
        println!("{indent}{}", display.details());
        println!("{indent}At {}, {} on the desktop", display.area.left, display.area.top);
        println!("{indent}{}", display.device_path);
    }
    println!();
    println!("To open a program on a display, set \"Display\" in the Settings.json in its");
    println!("folder to the display's identifier, or run it with --display and the identifier.");
    if boxes {
        println!();
        println!("Click an identifier on screen, or press Esc, to close them.");
    }
}

fn run() -> Result<(), String> {
    let mut boxes = true;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--list" => boxes = false,
            _ => return Err(format!("Unknown command-line argument: {argument}. The only one is --list.")),
        }
    }
    let displays = displays::enumerate().map_err(|error| format!("Couldn't find the displays: {error}"))?;
    print(&displays, boxes);
    if displays.is_empty() || !boxes {
        return Ok(());
    }

    // Clones share a monitor handle, and enumerate keeps them next to each
    // other.
    let mut groups: Vec<Vec<&Display>> = Vec::new();
    for display in &displays {
        match groups.last_mut() {
            Some(group) if group[0].monitor == display.monitor => group.push(display),
            _ => groups.push(vec![display]),
        }
    }
    let mut fonts = Fonts::new().map_err(|error| format!("Couldn't start DirectWrite: {error}"))?;
    let mut contents = Vec::new();
    let mut overlays = Vec::new();
    for group in &groups {
        let content = Identity::new(group);
        let (width, height) = content.size(&mut fonts);
        let (width, height) = (width as i32, height as i32);
        let area = group[0].area;
        let left = area.left + (area.right - area.left - width) / 2;
        let top = area.top + (area.bottom - area.top - height) / 2;
        overlays.push(Overlay {
            title: "Display identifier".to_string(),
            rect: RECT {
                left,
                top,
                right: left + width,
                bottom: top + height,
            },
        });
        contents.push(content);
    }
    run_overlays(&overlays, ThemeKind::Light, |ui, index| contents[index].draw(ui))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("IdentifyDisplays: {message}");
            ExitCode::FAILURE
        }
    }
}
