// What every KGames GUI program shares besides drawing: preparing the
// process, the window and its messages, borderless fullscreen, the command
// line, the settings file, the check that the window is in a supported
// setup, and the frame counter.

#[path = "Arguments.rs"]
mod arguments;
#[path = "FrameCounter.rs"]
mod frame_counter;
#[path = "Process.rs"]
mod process;
#[path = "Settings.rs"]
mod settings;
#[path = "Window.rs"]
mod window;
#[path = "WindowCheck.rs"]
pub mod window_check;

pub use arguments::Arguments;
pub use frame_counter::FrameCounter;
pub use process::prepare_process;
pub use settings::{ProgramSettings, Settings, WindowMode};
pub use window::{Event, Window};
