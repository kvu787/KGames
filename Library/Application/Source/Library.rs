// What every KGames GUI program shares besides drawing: preparing the
// process, the window and its messages, borderless fullscreen, the command
// line, the settings file, the check that the window is in a supported
// setup, the frame counter, and scripts of input for testing programs.

#[path = "Arguments.rs"]
mod arguments;
#[path = "FrameCounter.rs"]
mod frame_counter;
#[path = "Process.rs"]
mod process;
#[path = "Script.rs"]
pub mod script;
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
pub use script::virtual_key;
pub use window::{Event, Frame, Window};
