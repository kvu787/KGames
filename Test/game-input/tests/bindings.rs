// Library/game-input-raw: that its bindings are laid out as the header lays out
// GameInput's structs and vtables, and that they read what GameInput's devices
// say about themselves, through the shapes that windows-bindgen's rules give
// them, as Library/user-interface's devices read it. KGames doesn't build
// without GameInput 4.0.306, so these run where it's installed.

use core::ffi::c_void;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use game_input_raw::*;

// Whether a type has the size and alignment that MSVC gives it.
const fn laid_out<T>(size: usize, align: usize) -> bool {
    size_of::<T>() == size && align_of::<T>() == align
}

// The size and alignment of every struct and union in the v4 header, and in
// the Windows SDK's windef.h for APP_LOCAL_DEVICE_ID, as MSVC compiles them for
// x64, and the interfaces' method counts, with IUnknown's three, so that a
// field or slot missed or added fails the tests' build.
const _: () = {
    assert!(laid_out::<APP_LOCAL_DEVICE_ID>(32, 1));
    assert!(laid_out::<GameInputArcadeStickInfo>(72, 4));
    assert!(laid_out::<GameInputArcadeStickState>(4, 4));
    assert!(laid_out::<GameInputAxisMapping>(20, 4));
    assert!(laid_out::<GameInputButtonMapping>(16, 4));
    assert!(laid_out::<GameInputColor>(3, 1));
    assert!(laid_out::<GameInputControllerSwitchInfo>(36, 4));
    assert!(laid_out::<GameInputFlightStickInfo>(80, 4));
    assert!(laid_out::<GameInputFlightStickState>(24, 4));
    assert!(laid_out::<GameInputForceFeedbackConditionParams>(52, 4));
    assert!(laid_out::<GameInputForceFeedbackConstantParams>(80, 8));
    assert!(laid_out::<GameInputForceFeedbackEnvelope>(48, 8));
    assert!(laid_out::<GameInputForceFeedbackMagnitude>(28, 4));
    assert!(laid_out::<GameInputForceFeedbackMotorInfo>(16, 4));
    assert!(laid_out::<GameInputForceFeedbackParams>(112, 8));
    assert!(laid_out::<GameInputForceFeedbackParams_0>(104, 8));
    assert!(laid_out::<GameInputForceFeedbackPeriodicParams>(88, 8));
    assert!(laid_out::<GameInputForceFeedbackRampParams>(104, 8));
    assert!(laid_out::<GameInputGamepadInfo>(116, 4));
    assert!(laid_out::<GameInputGamepadState>(28, 4));
    assert!(laid_out::<GameInputHapticInfo>(644, 4));
    assert!(laid_out::<GameInputKeyState>(12, 4));
    assert!(laid_out::<GameInputKeyboardInfo>(28, 4));
    assert!(laid_out::<GameInputMouseInfo>(12, 4));
    assert!(laid_out::<GameInputMouseState>(56, 8));
    assert!(laid_out::<GameInputRacingWheelInfo>(100, 4));
    assert!(laid_out::<GameInputRacingWheelState>(28, 4));
    assert!(laid_out::<GameInputRawDeviceReportInfo>(12, 4));
    assert!(laid_out::<GameInputRumbleParams>(16, 4));
    assert!(laid_out::<GameInputSensorsInfo>(4, 4));
    assert!(laid_out::<GameInputSensorsState>(48, 4));
    assert!(laid_out::<GameInputTouchContact>(24, 8));
    assert!(laid_out::<GameInputTouchSurfaceInfo>(16, 4));
    assert!(laid_out::<GameInputUsage>(4, 2));
    assert!(laid_out::<GameInputVersion>(8, 2));
    assert!(size_of::<IGameInput_Vtbl>() == (3 + 13) * size_of::<usize>());
    assert!(size_of::<IGameInputDevice_Vtbl>() == (3 + 60) * size_of::<usize>());
    assert!(size_of::<IGameInputDispatcher_Vtbl>() == (3 + 2) * size_of::<usize>());
    assert!(size_of::<IGameInputForceFeedbackEffect_Vtbl>() == (3 + 8) * size_of::<usize>());
    assert!(size_of::<IGameInputMapper_Vtbl>() == (3 + 7) * size_of::<usize>());
    assert!(size_of::<IGameInputRawDeviceReport_Vtbl>() == (3 + 5) * size_of::<usize>());
    assert!(size_of::<IGameInputReading_Vtbl>() == (3 + 21) * size_of::<usize>());
};

// A name or a path, which a method writes as UTF-8 with a terminating zero,
// into a buffer of the size that it's given.
type Text = unsafe fn(&IGameInputDevice, u32, Option<*mut u8>) -> u32;

#[test]
fn devices_say_what_they_are() {
    static CONNECTED: Mutex<Vec<IGameInputDevice>> = Mutex::new(Vec::new());
    unsafe extern "system" fn connected(
        _: GameInputCallbackToken,
        _: *const c_void,
        event: GameInputDeviceEvent,
        device: windows_core::Ref<IGameInputDevice>,
    ) {
        if event == GameInputDeviceEventConnected
            && let Some(device) = device.cloned()
        {
            CONNECTED.lock().unwrap().push(device);
        }
    }

    // The callback reports the devices that are already connected, about
    // 600 ms after GameInput starts.
    let (game_input, _runtime) = game_input::create().unwrap();
    let kinds = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;
    let mut token = GameInputCallbackToken::default();
    unsafe { game_input.RegisterDeviceCallback(None::<&IGameInputDevice>, kinds, None, Some(connected), Some(&mut token)) }
        .ok()
        .unwrap();
    assert_ne!(token, GameInputCallbackToken::default());
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
                let size = get(device, 0, None);
                if size == 0 {
                    continue;
                }
                let mut buffer = vec![0xFF; size as usize];
                assert_eq!(get(device, size, Some(buffer.as_mut_ptr())), size, "the {text}'s size");
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
            // Slots further down the device's vtable, and the GameInput
            // object's: a connected device's ID finds the same device.
            assert!(device.IsConnected());
            let mut id = APP_LOCAL_DEVICE_ID::default();
            device.GetDeviceId(&mut id);
            assert_ne!(id, APP_LOCAL_DEVICE_ID::default());
            assert_eq!(game_input.FindDeviceFromId(&id).unwrap(), *device);
        }
    }
    // Keyboards and mice are on a bus, so they have a path in Windows' device
    // tree.
    assert!(paths > 0, "no device of {} has a PnP path", devices.len());
}
