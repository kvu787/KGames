// Hand-written bindings for the GameInput v4 API, transcribed from
// native/include/v4/GameInput.h in the Microsoft.GameInput NuGet package
// (Build.rs pins the version). GameInput isn't in the Windows metadata that
// windows-bindgen reads, so this file copies the shape of its output instead,
// and keeps GameInput's native names.
//
// Only what KGames uses is wrapped in methods. Every vtable slot is still
// listed, in header order, so the layout can be checked against the header;
// unused slots are usize placeholders, the way windows-bindgen writes them.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

use core::ffi::c_void;
use std::path::{Path, PathBuf};

use windows_api::*;
use windows_core::{Error, GUID, HRESULT, Interface, PCWSTR, Result, s, w};

pub type GameInputKind = i32;
pub const GameInputKindUnknown: GameInputKind = 0x00000000;
pub const GameInputKindRawDeviceReport: GameInputKind = 0x00000001;
pub const GameInputKindControllerAxis: GameInputKind = 0x00000002;
pub const GameInputKindControllerButton: GameInputKind = 0x00000004;
pub const GameInputKindControllerSwitch: GameInputKind = 0x00000008;
pub const GameInputKindController: GameInputKind = 0x0000000E;
pub const GameInputKindKeyboard: GameInputKind = 0x00000010;
pub const GameInputKindMouse: GameInputKind = 0x00000020;
pub const GameInputKindSensors: GameInputKind = 0x00000040;
pub const GameInputKindArcadeStick: GameInputKind = 0x00010000;
pub const GameInputKindFlightStick: GameInputKind = 0x00020000;
pub const GameInputKindGamepad: GameInputKind = 0x00040000;
pub const GameInputKindRacingWheel: GameInputKind = 0x00080000;
pub const GameInputKindLighting: GameInputKind = 0x00100000;
pub const GameInputKindSystemButton: GameInputKind = 0x00200000;
pub const GameInputKindTouch: GameInputKind = 0x00400000;

pub type GameInputDeviceEvent = i32;
pub const GameInputDeviceEventConnected: GameInputDeviceEvent = 0;
pub const GameInputDeviceEventDisconnected: GameInputDeviceEvent = 1;
pub const GameInputDeviceEventKeyboardLayoutUpdated: GameInputDeviceEvent = 2;
pub const GameInputDeviceEventHapticInfoUpdated: GameInputDeviceEvent = 3;
pub const GameInputDeviceEventXInputIndexUpdated: GameInputDeviceEvent = 4;

pub type GameInputMouseButtons = i32;
pub const GameInputMouseNone: GameInputMouseButtons = 0x00000000;
pub const GameInputMouseLeftButton: GameInputMouseButtons = 0x00000001;
pub const GameInputMouseRightButton: GameInputMouseButtons = 0x00000002;
pub const GameInputMouseMiddleButton: GameInputMouseButtons = 0x00000004;
pub const GameInputMouseButton4: GameInputMouseButtons = 0x00000008;
pub const GameInputMouseButton5: GameInputMouseButtons = 0x00000010;
pub const GameInputMouseWheelTiltLeft: GameInputMouseButtons = 0x00000020;
pub const GameInputMouseWheelTiltRight: GameInputMouseButtons = 0x00000040;

pub type GameInputMousePositions = i32;
pub const GameInputMouseNoPosition: GameInputMousePositions = 0x00000000;
pub const GameInputMouseAbsolutePosition: GameInputMousePositions = 0x00000001;
pub const GameInputMouseRelativePosition: GameInputMousePositions = 0x00000002;

pub type GameInputGamepadButtons = i32;
pub const GameInputGamepadNone: GameInputGamepadButtons = 0x00000000;
pub const GameInputGamepadMenu: GameInputGamepadButtons = 0x00000001;
pub const GameInputGamepadView: GameInputGamepadButtons = 0x00000002;
pub const GameInputGamepadA: GameInputGamepadButtons = 0x00000004;
pub const GameInputGamepadB: GameInputGamepadButtons = 0x00000008;
pub const GameInputGamepadC: GameInputGamepadButtons = 0x00004000;
pub const GameInputGamepadX: GameInputGamepadButtons = 0x00000010;
pub const GameInputGamepadY: GameInputGamepadButtons = 0x00000020;
pub const GameInputGamepadZ: GameInputGamepadButtons = 0x00008000;
pub const GameInputGamepadDPadUp: GameInputGamepadButtons = 0x00000040;
pub const GameInputGamepadDPadDown: GameInputGamepadButtons = 0x00000080;
pub const GameInputGamepadDPadLeft: GameInputGamepadButtons = 0x00000100;
pub const GameInputGamepadDPadRight: GameInputGamepadButtons = 0x00000200;
pub const GameInputGamepadLeftShoulder: GameInputGamepadButtons = 0x00000400;
pub const GameInputGamepadRightShoulder: GameInputGamepadButtons = 0x00000800;
pub const GameInputGamepadLeftTriggerButton: GameInputGamepadButtons = 0x00010000;
pub const GameInputGamepadRightTriggerButton: GameInputGamepadButtons = 0x00020000;
pub const GameInputGamepadLeftThumbstick: GameInputGamepadButtons = 0x00001000;
pub const GameInputGamepadLeftThumbstickUp: GameInputGamepadButtons = 0x00040000;
pub const GameInputGamepadLeftThumbstickDown: GameInputGamepadButtons = 0x00080000;
pub const GameInputGamepadLeftThumbstickLeft: GameInputGamepadButtons = 0x00100000;
pub const GameInputGamepadLeftThumbstickRight: GameInputGamepadButtons = 0x00200000;
pub const GameInputGamepadRightThumbstick: GameInputGamepadButtons = 0x00002000;
pub const GameInputGamepadRightThumbstickUp: GameInputGamepadButtons = 0x00400000;
pub const GameInputGamepadRightThumbstickDown: GameInputGamepadButtons = 0x00800000;
pub const GameInputGamepadRightThumbstickLeft: GameInputGamepadButtons = 0x01000000;
pub const GameInputGamepadRightThumbstickRight: GameInputGamepadButtons = 0x02000000;
pub const GameInputGamepadPaddleLeft1: GameInputGamepadButtons = 0x04000000;
pub const GameInputGamepadPaddleLeft2: GameInputGamepadButtons = 0x08000000;
pub const GameInputGamepadPaddleRight1: GameInputGamepadButtons = 0x10000000;
pub const GameInputGamepadPaddleRight2: GameInputGamepadButtons = 0x20000000;

pub type GameInputDeviceFamily = i32;
pub const GameInputFamilyVirtual: GameInputDeviceFamily = -1;
pub const GameInputFamilyUnknown: GameInputDeviceFamily = 0;
pub const GameInputFamilyXboxOne: GameInputDeviceFamily = 1;
pub const GameInputFamilyXbox360: GameInputDeviceFamily = 2;
pub const GameInputFamilyHid: GameInputDeviceFamily = 3;
pub const GameInputFamilyI8042: GameInputDeviceFamily = 4;
pub const GameInputFamilyAggregate: GameInputDeviceFamily = 5;

pub type GameInputRumbleMotors = i32;
pub const GameInputRumbleNone: GameInputRumbleMotors = 0x00000000;
pub const GameInputRumbleLowFrequency: GameInputRumbleMotors = 0x00000001;
pub const GameInputRumbleHighFrequency: GameInputRumbleMotors = 0x00000002;
pub const GameInputRumbleLeftTrigger: GameInputRumbleMotors = 0x00000004;
pub const GameInputRumbleRightTrigger: GameInputRumbleMotors = 0x00000008;

// An enum of button and axis names, like GameInputLabelXboxA.
// The demo doesn't read labels, so their values aren't listed.
pub type GameInputLabel = i32;

pub type GameInputCallbackToken = u64;

pub type GameInputDeviceCallback = Option<
    unsafe extern "system" fn(
        callbacktoken: GameInputCallbackToken,
        context: *mut c_void,
        deviceevent: GameInputDeviceEvent,
        device: *mut c_void,
    ),
>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputKeyState {
    pub scanCode: u32,
    pub codePoint: u32,
    pub virtualKey: u8,
    pub isDeadKey: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputMouseState {
    pub buttons: GameInputMouseButtons,
    pub positions: GameInputMousePositions,
    pub positionX: i64,
    pub positionY: i64,
    pub absolutePositionX: i64,
    pub absolutePositionY: i64,
    pub wheelX: i64,
    pub wheelY: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputGamepadState {
    pub buttons: GameInputGamepadButtons,
    pub leftTrigger: f32,
    pub rightTrigger: f32,
    pub leftThumbstickX: f32,
    pub leftThumbstickY: f32,
    pub rightThumbstickX: f32,
    pub rightThumbstickY: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputMouseInfo {
    pub supportedButtons: GameInputMouseButtons,
    pub sampleRate: u32,
    pub hasWheelX: bool,
    pub hasWheelY: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputGamepadInfo {
    pub supportedLayout: GameInputGamepadButtons,
    pub menuButtonLabel: GameInputLabel,
    pub viewButtonLabel: GameInputLabel,
    pub aButtonLabel: GameInputLabel,
    pub bButtonLabel: GameInputLabel,
    pub cButtonLabel: GameInputLabel,
    pub xButtonLabel: GameInputLabel,
    pub yButtonLabel: GameInputLabel,
    pub zButtonLabel: GameInputLabel,
    pub dpadUpLabel: GameInputLabel,
    pub dpadDownLabel: GameInputLabel,
    pub dpadLeftLabel: GameInputLabel,
    pub dpadRightLabel: GameInputLabel,
    pub leftShoulderButtonLabel: GameInputLabel,
    pub rightShoulderButtonLabel: GameInputLabel,
    pub leftThumbstickButtonLabel: GameInputLabel,
    pub rightThumbstickButtonLabel: GameInputLabel,
    pub paddleLeft1Label: GameInputLabel,
    pub paddleLeft2Label: GameInputLabel,
    pub paddleRight1Label: GameInputLabel,
    pub paddleRight2Label: GameInputLabel,
    pub leftTriggerLabel: GameInputLabel,
    pub rightTriggerLabel: GameInputLabel,
    pub leftThumbstickXLabel: GameInputLabel,
    pub leftThumbstickYLabel: GameInputLabel,
    pub rightThumbstickXLabel: GameInputLabel,
    pub rightThumbstickYLabel: GameInputLabel,
    pub extraButtonCount: u32,
    pub extraAxisCount: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputRumbleParams {
    pub lowFrequency: f32,
    pub highFrequency: f32,
    pub leftTrigger: f32,
    pub rightTrigger: f32,
}

pub const GAMEINPUT_E_DEVICE_DISCONNECTED: HRESULT = HRESULT(0x838A0001_u32 as i32);
pub const GAMEINPUT_E_DEVICE_NOT_FOUND: HRESULT = HRESULT(0x838A0002_u32 as i32);
pub const GAMEINPUT_E_READING_NOT_FOUND: HRESULT = HRESULT(0x838A0003_u32 as i32);
pub const GAMEINPUT_E_REFERENCE_READING_TOO_OLD: HRESULT = HRESULT(0x838A0004_u32 as i32);

fn device_abi(device: Option<&IGameInputDevice>) -> *mut c_void {
    device.map_or(core::ptr::null_mut(), |device| device.as_raw())
}

windows_core::imp::define_interface!(
    IGameInput,
    IGameInput_Vtbl,
    0xf2daae4c_4d21_4250_8953_4f791e6279ab
);
windows_core::imp::interface_hierarchy!(IGameInput, windows_core::IUnknown);
impl IGameInput {
    pub unsafe fn GetCurrentTimestamp(&self) -> u64 {
        unsafe { (Interface::vtable(self).GetCurrentTimestamp)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetCurrentReading(
        &self,
        inputkind: GameInputKind,
        device: Option<&IGameInputDevice>,
    ) -> Result<IGameInputReading> {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (Interface::vtable(self).GetCurrentReading)(
                Interface::as_raw(self),
                inputkind,
                device_abi(device),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetNextReading(
        &self,
        referencereading: &IGameInputReading,
        inputkind: GameInputKind,
        device: Option<&IGameInputDevice>,
    ) -> Result<IGameInputReading> {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (Interface::vtable(self).GetNextReading)(
                Interface::as_raw(self),
                referencereading.as_raw(),
                inputkind,
                device_abi(device),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetPreviousReading(
        &self,
        referencereading: &IGameInputReading,
        inputkind: GameInputKind,
        device: Option<&IGameInputDevice>,
    ) -> Result<IGameInputReading> {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (Interface::vtable(self).GetPreviousReading)(
                Interface::as_raw(self),
                referencereading.as_raw(),
                inputkind,
                device_abi(device),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn RegisterDeviceCallback(
        &self,
        device: Option<&IGameInputDevice>,
        inputkind: GameInputKind,
        context: Option<*const c_void>,
        callbackfunc: GameInputDeviceCallback,
    ) -> Result<GameInputCallbackToken> {
        let mut result__ = 0;
        unsafe {
            (Interface::vtable(self).RegisterDeviceCallback)(
                Interface::as_raw(self),
                device_abi(device),
                inputkind,
                context.unwrap_or(core::ptr::null()),
                callbackfunc,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn UnregisterCallback(&self, callbacktoken: GameInputCallbackToken) -> bool {
        unsafe {
            (Interface::vtable(self).UnregisterCallback)(Interface::as_raw(self), callbacktoken)
        }
    }
}
#[repr(C)]
pub struct IGameInput_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetCurrentTimestamp: unsafe extern "system" fn(*mut c_void) -> u64,
    pub GetCurrentReading: unsafe extern "system" fn(
        *mut c_void,
        GameInputKind,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetNextReading: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        GameInputKind,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetPreviousReading: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        GameInputKind,
        *mut c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    RegisterReadingCallback: usize,
    pub RegisterDeviceCallback: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        GameInputKind,
        *const c_void,
        GameInputDeviceCallback,
        *mut GameInputCallbackToken,
    ) -> HRESULT,
    pub UnregisterCallback: unsafe extern "system" fn(*mut c_void, GameInputCallbackToken) -> bool,
    CreateDispatcher: usize,
    FindDeviceFromId: usize,
    FindDeviceFromPlatformString: usize,
    EnableBackgroundInput: usize,
    CreateAggregateDevice: usize,
    DisableAggregateDevice: usize,
}

windows_core::imp::define_interface!(
    IGameInputReading,
    IGameInputReading_Vtbl,
    0x0dfaa2a2_688b_4a9c_a3a9_0feb4f90ee12
);
windows_core::imp::interface_hierarchy!(IGameInputReading, windows_core::IUnknown);
impl IGameInputReading {
    pub unsafe fn GetInputKind(&self) -> GameInputKind {
        unsafe { (Interface::vtable(self).GetInputKind)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetTimestamp(&self) -> u64 {
        unsafe { (Interface::vtable(self).GetTimestamp)(Interface::as_raw(self)) }
    }
    // The device comes back with a reference already added, so it's owned.
    pub unsafe fn GetDevice(&self) -> Option<IGameInputDevice> {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (Interface::vtable(self).GetDevice)(Interface::as_raw(self), &mut result__);
            windows_core::imp::Type::from_abi(result__).ok()
        }
    }
    pub unsafe fn GetKeyCount(&self) -> u32 {
        unsafe { (Interface::vtable(self).GetKeyCount)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetKeyState(&self, statearray: &mut [GameInputKeyState]) -> u32 {
        unsafe {
            (Interface::vtable(self).GetKeyState)(
                Interface::as_raw(self),
                statearray.len() as u32,
                statearray.as_mut_ptr(),
            )
        }
    }
    pub unsafe fn GetMouseState(&self, state: &mut GameInputMouseState) -> bool {
        unsafe { (Interface::vtable(self).GetMouseState)(Interface::as_raw(self), state) }
    }
    pub unsafe fn GetGamepadState(&self, state: &mut GameInputGamepadState) -> bool {
        unsafe { (Interface::vtable(self).GetGamepadState)(Interface::as_raw(self), state) }
    }
}
#[repr(C)]
pub struct IGameInputReading_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetInputKind: unsafe extern "system" fn(*mut c_void) -> GameInputKind,
    pub GetTimestamp: unsafe extern "system" fn(*mut c_void) -> u64,
    pub GetDevice: unsafe extern "system" fn(*mut c_void, *mut *mut c_void),
    GetControllerAxisCount: usize,
    GetControllerAxisState: usize,
    GetControllerButtonCount: usize,
    GetControllerButtonState: usize,
    GetControllerSwitchCount: usize,
    GetControllerSwitchState: usize,
    pub GetKeyCount: unsafe extern "system" fn(*mut c_void) -> u32,
    pub GetKeyState: unsafe extern "system" fn(*mut c_void, u32, *mut GameInputKeyState) -> u32,
    pub GetMouseState: unsafe extern "system" fn(*mut c_void, *mut GameInputMouseState) -> bool,
    GetSensorsState: usize,
    GetArcadeStickState: usize,
    GetFlightStickState: usize,
    pub GetGamepadState: unsafe extern "system" fn(*mut c_void, *mut GameInputGamepadState) -> bool,
    GetRacingWheelState: usize,
    GetRawReport: usize,
    GetSystemButtonState: usize,
    GetTouchContactCount: usize,
    GetTouchSurfaceState: usize,
}

windows_core::imp::define_interface!(
    IGameInputDevice,
    IGameInputDevice_Vtbl,
    0x4420f01d_8918_49d4_b302_dbead986105f
);
windows_core::imp::interface_hierarchy!(IGameInputDevice, windows_core::IUnknown);
// GameInput's objects are free-threaded: device callbacks hand devices over
// from GameInput's own threads, and the main thread then calls into them.
unsafe impl Send for IGameInputDevice {}
unsafe impl Sync for IGameInputDevice {}
impl IGameInputDevice {
    pub unsafe fn GetVendorId(&self) -> u16 {
        unsafe { (Interface::vtable(self).GetVendorId)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetProductId(&self) -> u16 {
        unsafe { (Interface::vtable(self).GetProductId)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetDeviceFamily(&self) -> GameInputDeviceFamily {
        unsafe { (Interface::vtable(self).GetDeviceFamily)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetSupportedInput(&self) -> GameInputKind {
        unsafe { (Interface::vtable(self).GetSupportedInput)(Interface::as_raw(self)) }
    }
    pub unsafe fn GetSupportedRumbleMotors(&self) -> GameInputRumbleMotors {
        unsafe { (Interface::vtable(self).GetSupportedRumbleMotors)(Interface::as_raw(self)) }
    }
    // Returns the size in bytes of the UTF-8 name, including its terminating
    // zero. Pass an empty buffer to ask for the size.
    pub unsafe fn GetDisplayName(&self, buffer: &mut [u8]) -> u32 {
        unsafe {
            (Interface::vtable(self).GetDisplayName)(
                Interface::as_raw(self),
                buffer.len() as u32,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
            )
        }
    }
    pub unsafe fn GetXInputDeviceIndex(&self, index: &mut u32) -> bool {
        unsafe { (Interface::vtable(self).GetXInputDeviceIndex)(Interface::as_raw(self), index) }
    }
    pub unsafe fn GetMouseInfo(&self, info: &mut GameInputMouseInfo) -> bool {
        unsafe { (Interface::vtable(self).GetMouseInfo)(Interface::as_raw(self), info) }
    }
    pub unsafe fn GetGamepadInfo(&self, info: &mut GameInputGamepadInfo) -> bool {
        unsafe { (Interface::vtable(self).GetGamepadInfo)(Interface::as_raw(self), info) }
    }
    pub unsafe fn SetRumbleState(&self, params: Option<&GameInputRumbleParams>) {
        unsafe {
            (Interface::vtable(self).SetRumbleState)(
                Interface::as_raw(self),
                params.map_or(core::ptr::null(), |params| params),
            )
        }
    }
}
#[repr(C)]
pub struct IGameInputDevice_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetVendorId: unsafe extern "system" fn(*mut c_void) -> u16,
    pub GetProductId: unsafe extern "system" fn(*mut c_void) -> u16,
    GetRevisionNumber: usize,
    GetUsage: usize,
    GetHardwareVersion: usize,
    GetFirmwareVersion: usize,
    GetDeviceId: usize,
    GetDeviceRootId: usize,
    pub GetDeviceFamily: unsafe extern "system" fn(*mut c_void) -> GameInputDeviceFamily,
    pub GetSupportedInput: unsafe extern "system" fn(*mut c_void) -> GameInputKind,
    pub GetSupportedRumbleMotors: unsafe extern "system" fn(*mut c_void) -> GameInputRumbleMotors,
    GetSupportedSystemButtons: usize,
    GetSupportedLightingGroups: usize,
    GetSupportedLightingGroupSize: usize,
    GetContainerId: usize,
    pub GetDisplayName: unsafe extern "system" fn(*mut c_void, u32, *mut u8) -> u32,
    GetPnpPath: usize,
    pub GetXInputDeviceIndex: unsafe extern "system" fn(*mut c_void, *mut u32) -> bool,
    GetKeyboardInfo: usize,
    pub GetMouseInfo: unsafe extern "system" fn(*mut c_void, *mut GameInputMouseInfo) -> bool,
    GetSensorsInfo: usize,
    GetControllerAxisCount: usize,
    GetControllerAxisLabels: usize,
    GetControllerButtonCount: usize,
    GetControllerButtonLabels: usize,
    GetControllerSwitchCount: usize,
    GetControllerSwitchInfo: usize,
    GetArcadeStickInfo: usize,
    GetFlightStickInfo: usize,
    pub GetGamepadInfo: unsafe extern "system" fn(*mut c_void, *mut GameInputGamepadInfo) -> bool,
    GetRacingWheelInfo: usize,
    GetForceFeedbackMotorCount: usize,
    GetForceFeedbackMotorInfo: usize,
    GetInputReportCount: usize,
    GetInputReportInfo: usize,
    GetOutputReportCount: usize,
    GetOutputReportInfo: usize,
    GetFeatureReportCount: usize,
    GetFeatureReportInfo: usize,
    GetHapticInfo: usize,
    IsConnected: usize,
    CreateForceFeedbackEffect: usize,
    IsForceFeedbackMotorPoweredOn: usize,
    SetForceFeedbackMotorGain: usize,
    pub SetRumbleState: unsafe extern "system" fn(*mut c_void, *const GameInputRumbleParams),
    DirectInputEscape: usize,
    CreateInputMapper: usize,
    GetExtraAxisCount: usize,
    GetExtraButtonCount: usize,
    GetExtraAxisIndexes: usize,
    GetExtraButtonIndexes: usize,
    CreateRawDeviceReport: usize,
    GetRawDeviceFeature: usize,
    SetRawDeviceFeature: usize,
    SendRawDeviceOutput: usize,
    SetLightingColor: usize,
    ApplyLightingColors: usize,
    ApplyLightingSystemDefault: usize,
    GetTouchSurfaceCount: usize,
    GetTouchSurfaceInfo: usize,
}

// The GameInput runtime DLL that was loaded, for display.
pub struct Runtime {
    pub path: PathBuf,
    pub version: [u16; 4],
}

// A Rust port of GameInputInitialize from native/src/GameInput.cpp in the
// NuGet package, which C++ games get by linking GameInput.lib. There can be
// three copies of the runtime, and the newest one wins:
// - GameInput.dll, built into Windows. Windows 11 ships an old v0-only one.
// - GameInputRedist.dll, installed by GameInputRedist.msi.
// - GameInputRedist.dll in the app's own folder, which Build.rs puts there.
pub fn create() -> Result<(IGameInput, Runtime)> {
    let system = system_directory()?;
    let mut candidates = vec![system.join("GameInput.dll")];
    let redist = system.join("GameInputRedist.dll");
    if redist.is_file() {
        candidates.push(redist);
    } else if let Some(directory) = redist_directory() {
        candidates.push(directory.join("GameInputRedist.dll"));
    }
    // Like in GameInput.cpp, not finding the app's folder only skips it.
    if let Ok(exe) = std::env::current_exe()
        && let Some(directory) = exe.parent()
    {
        candidates.push(directory.join("GameInputRedist.dll"));
    }

    // max_by_key picks the last of equal versions, so ties go to the later
    // candidate, like in GameInput.cpp.
    let (path, version) = candidates
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| {
            let version = file_version(&path).unwrap_or(0);
            (path, version)
        })
        .max_by_key(|(_, version)| *version)
        .ok_or_else(|| Error::new(HRESULT(0x80070002_u32 as i32), "no GameInput runtime found"))?;

    unsafe {
        let module = LoadLibraryW(PCWSTR(wide(&path).as_ptr()));
        if module.0.is_null() {
            return Err(Error::from_thread());
        }
        // The module stays loaded for the life of the process.
        let Some(initialize) = GetProcAddress(module, s!("GameInputInitialize")) else {
            // Only runtimes too old for anything but the v0 API lack this export.
            return Err(Error::new(
                HRESULT(0x80004002_u32 as i32),
                format!("{} only supports GameInput v0", path.display()),
            ));
        };
        let initialize: unsafe extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT =
            core::mem::transmute(initialize);

        let mut result = core::ptr::null_mut();
        let game_input: IGameInput = initialize(&IGameInput::IID, &mut result)
            .and_then(|| windows_core::imp::Type::from_abi(result))?;

        let runtime = Runtime {
            path,
            version: [
                (version >> 48) as u16,
                (version >> 32) as u16,
                (version >> 16) as u16,
                version as u16,
            ],
        };
        Ok((game_input, runtime))
    }
}

fn system_directory() -> Result<PathBuf> {
    unsafe {
        // The size, including the terminating zero.
        let length = GetSystemDirectoryW(None, 0);
        if length == 0 {
            return Err(Error::from_thread());
        }
        let mut buffer = vec![0u16; length as usize];
        let length = GetSystemDirectoryW(Some(windows_core::PWSTR(buffer.as_mut_ptr())), length);
        if length == 0 {
            return Err(Error::from_thread());
        }
        Ok(PathBuf::from(String::from_utf16_lossy(
            &buffer[..length as usize],
        )))
    }
}

// Where GameInputRedist.msi installed the runtime, if it did.
fn redist_directory() -> Option<PathBuf> {
    let key = w!(r"SOFTWARE\Microsoft\GameInput");
    let value = w!("RedistDir");
    let flags = (RRF_RT_REG_SZ | RRF_SUBKEY_WOW6432KEY) as u32;
    unsafe {
        let mut size = 0u32;
        if RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key,
            value,
            flags,
            None,
            None,
            Some(&raw mut size),
        )
        .0 != 0
        {
            return None;
        }
        let mut buffer = vec![0u16; size as usize / 2];
        if RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key,
            value,
            flags,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&raw mut size),
        )
        .0 != 0
        {
            return None;
        }
        let text = String::from_utf16_lossy(&buffer);
        Some(PathBuf::from(text.trim_end_matches('\0')))
    }
}

// The file version from the DLL's version resource, packed as major.minor.build.revision
// in 16 bits each, so that versions compare as numbers.
fn file_version(path: &Path) -> Option<u64> {
    let path = wide(path);
    let path = PCWSTR(path.as_ptr());
    unsafe {
        let size = GetFileVersionInfoSizeW(path, None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if !GetFileVersionInfoW(path, None, size, data.as_mut_ptr().cast()).as_bool() {
            return None;
        }
        let mut info: *mut c_void = core::ptr::null_mut();
        let mut length = 0u32;
        if !VerQueryValueW(data.as_ptr().cast(), w!(r"\"), &mut info, &mut length).as_bool() {
            return None;
        }
        let info = &*(info as *const VS_FIXEDFILEINFO);
        Some(((info.dwFileVersionMS as u64) << 32) | info.dwFileVersionLS as u64)
    }
}

fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str().encode_wide().chain([0]).collect()
}
