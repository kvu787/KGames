// The tester's command line. It takes the options that RunInBackground gives
// every program, like Library/user-interface's programs do, and two of its
// own, for runs that can't type: the text to start with, and the size.

use crate::tester::{MAXIMUM_SCALE, Theme};

pub const USAGE: &str = "pixel-font-tester [--display <identifier>|Primary] [--no-activate] [--no-input] [--vsync|--no-vsync] \
                         [--theme Light|Dark] [--size <1-10>] [--text <text>]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments {
    // The display to open on, as IdentifyDisplays prints it, or Primary.
    pub display: String,
    // Opens beneath the other windows, without taking focus.
    pub no_activate: bool,
    // Ignores the keyboard.
    pub no_input: bool,
    pub vsync: bool,
    pub theme: Theme,
    pub scale: u32,
    // The text to start with, with \n for a line break and \\ for a
    // backslash.
    pub text: String,
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments {
            display: "Primary".to_string(),
            no_activate: false,
            no_input: false,
            vsync: true,
            theme: Theme::Light,
            scale: 1,
            text: String::new(),
        }
    }
}

impl Arguments {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
        let mut parsed = Arguments::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            let mut value = || arguments.next().ok_or_else(|| format!("{argument} needs a value after it."));
            match argument.as_str() {
                "--display" => parsed.display = value()?,
                "--no-activate" => parsed.no_activate = true,
                "--no-input" => parsed.no_input = true,
                "--vsync" => parsed.vsync = true,
                "--no-vsync" => parsed.vsync = false,
                "--theme" => {
                    let theme = value()?;
                    parsed.theme = match theme.to_ascii_lowercase().as_str() {
                        "light" => Theme::Light,
                        "dark" => Theme::Dark,
                        _ => return Err(format!("--theme must be Light or Dark, not {theme}.")),
                    };
                }
                "--size" => {
                    let size = value()?;
                    parsed.scale = size
                        .parse()
                        .ok()
                        .filter(|scale| (1..=MAXIMUM_SCALE).contains(scale))
                        .ok_or_else(|| format!("--size must be a whole number from 1 to {MAXIMUM_SCALE}, not {size}."))?;
                }
                "--text" => parsed.text = unescape(&value()?),
                "--script" | "--output" => return Err("PixelFontTester doesn't run scripts: test its input with Test/pixel-font-tester's tests.".to_string()),
                _ => return Err(format!("{argument} isn't an option.")),
            }
        }
        Ok(parsed)
    }
}

// \n to a line break, and \\ to a backslash. Any other backslash stays.
fn unescape(text: &str) -> String {
    let mut result = String::new();
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match (character, characters.peek()) {
            ('\\', Some('n')) => {
                result.push('\n');
                characters.next();
            }
            ('\\', Some('\\')) => {
                result.push('\\');
                characters.next();
            }
            _ => result.push(character),
        }
    }
    result
}
