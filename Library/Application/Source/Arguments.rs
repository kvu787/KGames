// The command line that every program takes. It overrides the settings file.

use std::path::{Path, PathBuf};

pub struct Arguments {
    // The display from --display, as written: an identifier or "Primary".
    // It must be connected.
    pub display: Option<String>,
    // Whether the window takes focus when it opens. --no-activate opens it
    // behind the window that has focus, so typing there stays there.
    pub activate: bool,
    // With --script, a script to run instead of the user's input, for
    // testing, and with --output, the folder for what it captures. See
    // Script.rs.
    pub script: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

impl Arguments {
    // --display and an identifier that IdentifyDisplays shows, or "Primary",
    // picks the display. --no-activate keeps the window from taking focus.
    // --script and --output run a script. Problems go in `notices`.
    pub fn parse(notices: &mut Vec<String>) -> Self {
        Self::parse_from(std::env::args().skip(1), notices)
    }

    // Like parse, from these arguments.
    pub fn parse_from(arguments: impl IntoIterator<Item = String>, notices: &mut Vec<String>) -> Self {
        let mut result = Self {
            display: None,
            activate: true,
            script: None,
            output: None,
        };
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--display" => match arguments.next() {
                    Some(display) => result.display = Some(display),
                    None => {
                        notices.push("--display needs a display identifier after it".to_string())
                    }
                },
                "--no-activate" => result.activate = false,
                "--script" | "--output" => match arguments.next() {
                    Some(path) if argument == "--script" => result.script = Some(path.into()),
                    Some(path) => result.output = Some(path.into()),
                    None => notices.push(format!("{argument} needs a path after it")),
                },
                _ => notices.push(format!("Unknown command-line argument: {argument}")),
            }
        }
        result
    }

    // Whether a script runs instead of the user's input.
    pub fn scripted(&self) -> bool {
        self.script.is_some()
    }

    // Where the program keeps a file that it writes, called `name`: in
    // `folder`, usually the program's own, or in a script's output folder,
    // so that a scripted run starts without it, and leaves the program's
    // folder alone.
    pub fn file(&self, folder: &Path, name: &str) -> PathBuf {
        match &self.output {
            Some(output) if self.scripted() => output.join(name),
            _ => folder.join(name),
        }
    }
}
