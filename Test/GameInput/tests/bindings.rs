// Library/GameInputRaw: that its bindings are laid out as the header lays out
// GameInput's structs and vtables, and that they read what GameInput's devices
// say about themselves, through the shapes that windows-bindgen's rules give
// them, as Library/UserInterface's devices read it. KGames doesn't build
// without GameInput 4.0.306, so these run where it's installed.

use core::ffi::c_void;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use game_input_raw::*;
use windows_core::Interface;

// The sizes that the v4 header gives these structs, as MSVC compiles it for
// x64, and its interfaces' method counts, with IUnknown's three, so that a
// field or slot missed or added in transcribing fails the tests' build.
const _: () = {
    assert!(size_of::<GameInputKeyState>() == 12);
    assert!(size_of::<GameInputMouseState>() == 56);
    assert!(size_of::<GameInputGamepadState>() == 28);
    assert!(size_of::<GameInputVersion>() == 8);
    assert!(size_of::<GameInputUsage>() == 4);
    assert!(size_of::<GameInputKeyboardInfo>() == 28);
    assert!(size_of::<GameInputMouseInfo>() == 12);
    assert!(size_of::<GameInputGamepadInfo>() == 116);
    assert!(size_of::<GameInputRumbleParams>() == 16);
    assert!(size_of::<IGameInput_Vtbl>() == (3 + 13) * size_of::<usize>());
    assert!(size_of::<IGameInputReading_Vtbl>() == (3 + 21) * size_of::<usize>());
    assert!(size_of::<IGameInputDevice_Vtbl>() == (3 + 60) * size_of::<usize>());
};

// A name or a path, which a method writes as UTF-8 with a terminating zero.
type Text = unsafe fn(&IGameInputDevice, Option<&mut [u8]>) -> u32;

#[test]
fn devices_say_what_they_are() {
    static CONNECTED: Mutex<Vec<IGameInputDevice>> = Mutex::new(Vec::new());
    unsafe extern "system" fn connected(_: GameInputCallbackToken, _: *mut c_void, event: GameInputDeviceEvent, device: *mut c_void) {
        if event == GameInputDeviceEventConnected
            && let Some(device) = unsafe { IGameInputDevice::from_raw_borrowed(&device) }
        {
            CONNECTED.lock().unwrap().push(device.clone());
        }
    }

    // The callback reports the devices that are already connected, about
    // 600 ms after GameInput starts.
    let (game_input, _runtime) = game_input::create().unwrap();
    let kinds = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;
    let mut token = 0;
    unsafe { game_input.RegisterDeviceCallback(None::<&IGameInputDevice>, kinds, None, Some(connected), Some(&mut token)) }
        .ok()
        .unwrap();
    assert_ne!(token, 0);
    let deadline = Instant::now() + Duration::from_secs(10);
    while CONNECTED.lock().unwrap().is_empty() {
        assert!(Instant::now() < deadline, "GameInput reported no keyboard, mouse or gamepad");
        std::thread::sleep(Duration::from_millis(1));
    }
    std::thread::sleep(Duration::from_millis(100));
    assert!(unsafe { game_input.UnregisterCallback(token) });

    let devices = std::mem::take(&mut *CONNECTED.lock().unwrap());
    let mut paths = 0;
    for device in &devices {
        unsafe {
            for (text, get) in [("name", IGameInputDevice::GetDisplayName as Text), ("PnP path", IGameInputDevice::GetPnpPath)] {
                // Without a buffer, its size, then the text in a buffer of
                // that size.
                let size = get(device, None);
                if size == 0 {
                    continue;
                }
                let mut buffer = vec![0xFF; size as usize];
                assert_eq!(get(device, Some(&mut buffer)), size, "the {text}'s size");
                assert_eq!(buffer.iter().position(|&byte| byte == 0), Some(buffer.len() - 1), "the {text}: {buffer:?}");
                assert!(std::str::from_utf8(&buffer[..buffer.len() - 1]).is_ok(), "the {text}: {buffer:?}");
                paths += usize::from(text == "PnP path");
            }
            // What the methods return, instead of writing through a pointer,
            // is the same each time.
            assert_eq!(device.GetUsage(), device.GetUsage());
            assert_eq!(device.GetHardwareVersion(), device.GetHardwareVersion());
            assert_eq!(device.GetFirmwareVersion(), device.GetFirmwareVersion());
            assert_eq!(device.GetContainerId(), device.GetContainerId());
            let supported = device.GetSupportedInput();
            let mut keyboard = GameInputKeyboardInfo::default();
            assert_eq!(device.GetKeyboardInfo(&mut keyboard), supported & GameInputKindKeyboard != 0);
            let mut mouse = GameInputMouseInfo::default();
            assert_eq!(device.GetMouseInfo(&mut mouse), supported & GameInputKindMouse != 0);
        }
    }
    // Keyboards and mice are on a bus, so they have a path in Windows' device
    // tree.
    assert!(paths > 0, "no device of {} has a PnP path", devices.len());
}
