// The command line that every program takes. It overrides the settings file.

use crate::Settings;

pub struct Arguments {
    // Whether the display came from --display, which must be connected.
    pub display_required: bool,
    // Whether the window takes focus when it opens. --no-activate opens it
    // behind the window that has focus, so typing there stays there.
    pub activate: bool,
}

impl Arguments {
    // --display and an identifier that IdentifyDisplays shows, or "Primary",
    // picks the display. --no-activate keeps the window from taking focus.
    // Problems go in `notices`.
    pub fn parse(settings: &mut Settings, notices: &mut Vec<String>) -> Self {
        let mut result = Self {
            display_required: false,
            activate: true,
        };
        let mut arguments = std::env::args().skip(1);
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--display" => match arguments.next() {
                    Some(display) => {
                        settings.display = crate::settings::parse_display(&display);
                        result.display_required = true;
                    }
                    None => {
                        notices.push("--display needs a display identifier after it".to_string())
                    }
                },
                "--no-activate" => result.activate = false,
                _ => notices.push(format!("Unknown command-line argument: {argument}")),
            }
        }
        result
    }
}
