// What IdentifyDisplays prints: each display's identifier, model, mode,
// place on the desktop and device path, as a library that
// Test/IdentifyDisplays tests without a console.

use displays::Display;

pub const USAGE: &str = "IdentifyDisplays";

// IdentifyDisplays takes no arguments.
pub fn check_arguments(arguments: &[String]) -> Result<(), String> {
    match arguments.first() {
        None => Ok(()),
        Some(argument) => Err(format!("Unknown argument: {argument}. IdentifyDisplays takes none.")),
    }
}

// The displays, from left to right as Library/Displays lists them, and how to
// open a program on one.
pub fn report(displays: &[Display]) -> String {
    if displays.is_empty() {
        return "Windows reports no displays.\n".to_string();
    }
    let count = displays.len();
    let mut text = format!("{count} {}, from left to right:\n", if count == 1 { "display" } else { "displays" });
    for display in displays {
        let primary = if display.primary { ", the primary display" } else { "" };
        let indent = " ".repeat(display.identifier.len() + 4);
        text += "\n";
        text += &format!("  {}  {}{primary}\n", display.identifier, display.name);
        text += &format!("{indent}{}\n", display.details());
        text += &format!("{indent}At {}, {} on the desktop\n", display.area.left, display.area.top);
        text += &format!("{indent}{}\n", display.device_path);
    }
    text += "\n";
    text += "To open a program on a display, set \"Display\" in the Settings.json in its\n";
    text += "folder to the display's identifier, or run it with --display and the identifier.\n";
    text
}
