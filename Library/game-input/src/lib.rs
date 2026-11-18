// What KGames adds to the GameInput bindings in Library/game-input-raw, which
// are only the API: create, which starts GameInput, once it has checked that
// the runtime that KGames needs is installed (see installation.rs), and says
// which runtime it started. Bindings go there, never here.

pub mod installation;

use core::ffi::c_void;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};

use game_input_raw::{GameInputInitialize, IGameInput};
use windows_api_raw::*;
use windows_core::{Error, Interface, PCWSTR, PWSTR};

// The GameInput runtime that GameInput.lib's loader loaded, for display.
pub struct Runtime {
    pub path: PathBuf,
    pub version: [u16; 4],
}

// Starts GameInput the way Microsoft's v4 header does for C++ games: with
// GameInputInitialize from GameInput.lib, and the IID of v4's IGameInput. It
// returns the runtime that the loader loaded, or says what's wrong, and how to
// install GameInput 4.0.306.
//
// The loader takes the runtime with the newest file version of Windows'
// GameInput.dll, the installed GameInputRedist.dll, from System32, and a
// GameInputRedist.dll beside the exe, and the one beside the exe on a tie,
// then the installed one. With 4.0.306 installed and nothing beside the exe,
// it's System32's, which COM also loads to carry calls to GameInput's service,
// so the process runs one copy of the runtime. It loads the runtime with
// LoadLibraryW, so the DLLs that the runtime imports, like HID.DLL, are looked
// for beside the exe first. Test/GameInput pins all three.
//
// It checks that GameInput 4.0.306 is installed first, so that a build or a
// program without it says how to install it, instead of failing to start
// GameInput.
//
// The loader keeps the runtime it loaded in globals that it doesn't guard, so
// one thread at a time calls it. GameInput 4.0.306's runtime crashes when one
// thread releases the last IGameInput while another creates one, so the first
// one that this creates is kept for the life of the process, like the runtime,
// and the last one is never released. Test/GameInput pins it.
pub fn create() -> core::result::Result<(IGameInput, Runtime), String> {
    static KEPT: OnceLock<IGameInput> = OnceLock::new();
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    installation::installed_runtime()?;
    let game_input: IGameInput = {
        let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
        unsafe { GameInputInitialize::<IGameInput>() }
            .map_err(|error| format!("Couldn't start GameInput: {} ({})", error.message(), error.code()))?
    };
    KEPT.get_or_init(|| game_input.clone());
    let runtime = runtime_of(&game_input)?;
    Ok((game_input, runtime))
}

// The runtime whose code an object of GameInput's is: the DLL that holds its
// vtable, which is the first thing in a COM object, and that DLL's version.
fn runtime_of(object: &impl Interface) -> core::result::Result<Runtime, String> {
    let path = unsafe {
        let vtable = *(object.as_raw() as *const *const c_void);
        let mut module = HMODULE::default();
        let flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        GetModuleHandleExW(flags as u32, PCWSTR(vtable as *const u16), &mut module)
            .ok()
            .map_err(|error| format!("Couldn't find GameInput's runtime: {} ({})", error.message(), error.code()))?;
        let mut name = vec![0u16; 32768];
        let length = GetModuleFileNameW(Some(module), PWSTR(name.as_mut_ptr()), name.len() as u32) as usize;
        if length == 0 || length == name.len() {
            let error = Error::from_thread();
            return Err(format!("Couldn't find GameInput's runtime: {} ({})", error.message(), error.code()));
        }
        PathBuf::from(OsString::from_wide(&name[..length]))
    };
    let bytes = std::fs::read(&path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
    let version = installation::version_in(&bytes).ok_or_else(|| format!("{} has no version", path.display()))?;
    Ok(Runtime { path, version })
}
