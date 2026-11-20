// The bindings for the part of GameInput's v4 API that reads keyboards, mice
// and gamepads, from native/include/v4/GameInput.h in the Microsoft.GameInput
// NuGet package that NuGetPackages.csproj declares, and nothing else: what
// windows-bindgen generates for bindings-generator's scope of the header,
// which bindings-generator describes as Windows metadata for it. A usize in a
// vtable is the slot of a method that's out of scope. Don't edit it by hand:
// regenerate it with `cargo run -p game-input-bindings-generator`.

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]

#[inline]
pub unsafe fn GameInputInitialize<T>() -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    unsafe extern "system" {
        pub fn GameInputInitialize(
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT;
    }
    let mut result__ = core::ptr::null_mut();
    unsafe {
        GameInputInitialize(&T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct APP_LOCAL_DEVICE_ID {
    pub value: [u8; 32],
}
impl Default for APP_LOCAL_DEVICE_ID {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const GAMEINPUT_API_VERSION: i32 = 4;
pub const GAMEINPUT_E_CALLBACK_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0009_u32 as _);
pub const GAMEINPUT_E_DEVICE_DISCONNECTED: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0001_u32 as _);
pub const GAMEINPUT_E_DEVICE_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0002_u32 as _);
pub const GAMEINPUT_E_INPUT_KIND_NOT_PRESENT: windows_core::HRESULT =
    windows_core::HRESULT(0x838A000C_u32 as _);
pub const GAMEINPUT_E_OBJECT_NO_LONGER_EXISTS: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0008_u32 as _);
pub const GAMEINPUT_E_READING_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0003_u32 as _);
pub const GAMEINPUT_E_REFERENCE_READING_TOO_OLD: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0004_u32 as _);
pub const GAMEINPUT_FACILITY: i32 = 906;
pub const GameInputAbntKeyboard: GameInputKeyboardKind = 3;
pub const GameInputAnsiKeyboard: GameInputKeyboardKind = 0;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GameInputCallbackToken(pub u64);
pub type GameInputDeviceCallback = Option<
    unsafe extern "system" fn(
        callbacktoken: GameInputCallbackToken,
        context: *const core::ffi::c_void,
        deviceevent: GameInputDeviceEvent,
        device: windows_core::Ref<IGameInputDevice>,
    ),
>;
pub type GameInputDeviceEvent = i32;
pub const GameInputDeviceEventConnected: GameInputDeviceEvent = 0;
pub const GameInputDeviceEventDisconnected: GameInputDeviceEvent = 1;
pub const GameInputDeviceEventHapticInfoUpdated: GameInputDeviceEvent = 3;
pub const GameInputDeviceEventKeyboardLayoutUpdated: GameInputDeviceEvent = 2;
pub const GameInputDeviceEventXInputIndexUpdated: GameInputDeviceEvent = 4;
pub type GameInputDeviceFamily = i32;
pub const GameInputFamilyAggregate: GameInputDeviceFamily = 5;
pub const GameInputFamilyHid: GameInputDeviceFamily = 3;
pub const GameInputFamilyI8042: GameInputDeviceFamily = 4;
pub const GameInputFamilyUnknown: GameInputDeviceFamily = 0;
pub const GameInputFamilyVirtual: GameInputDeviceFamily = -1;
pub const GameInputFamilyXbox360: GameInputDeviceFamily = 2;
pub const GameInputFamilyXboxOne: GameInputDeviceFamily = 1;
pub const GameInputGamepadA: GameInputGamepadButtons = 4;
pub const GameInputGamepadB: GameInputGamepadButtons = 8;
pub type GameInputGamepadButtons = i32;
pub const GameInputGamepadC: GameInputGamepadButtons = 16384;
pub const GameInputGamepadDPadDown: GameInputGamepadButtons = 128;
pub const GameInputGamepadDPadLeft: GameInputGamepadButtons = 256;
pub const GameInputGamepadDPadRight: GameInputGamepadButtons = 512;
pub const GameInputGamepadDPadUp: GameInputGamepadButtons = 64;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
pub const GameInputGamepadLayoutBasic: GameInputGamepadButtons = 975;
pub const GameInputGamepadLayoutButtons: GameInputGamepadButtons = 4095;
pub const GameInputGamepadLayoutElite: GameInputGamepadButtons = 1073692671;
pub const GameInputGamepadLayoutStandard: GameInputGamepadButtons = 67059711;
pub const GameInputGamepadLeftShoulder: GameInputGamepadButtons = 1024;
pub const GameInputGamepadLeftThumbstick: GameInputGamepadButtons = 4096;
pub const GameInputGamepadLeftThumbstickDown: GameInputGamepadButtons = 524288;
pub const GameInputGamepadLeftThumbstickLeft: GameInputGamepadButtons = 1048576;
pub const GameInputGamepadLeftThumbstickRight: GameInputGamepadButtons = 2097152;
pub const GameInputGamepadLeftThumbstickUp: GameInputGamepadButtons = 262144;
pub const GameInputGamepadLeftTriggerButton: GameInputGamepadButtons = 65536;
pub const GameInputGamepadMenu: GameInputGamepadButtons = 1;
pub const GameInputGamepadModuleDpad: GameInputGamepadButtons = 960;
pub const GameInputGamepadModulePaddles2: GameInputGamepadButtons = 335544320;
pub const GameInputGamepadModulePaddles4: GameInputGamepadButtons = 1006632960;
pub const GameInputGamepadModuleShoulders: GameInputGamepadButtons = 3072;
pub const GameInputGamepadModuleSystemDuo: GameInputGamepadButtons = 3;
pub const GameInputGamepadModuleThumbsticks: GameInputGamepadButtons = 66846720;
pub const GameInputGamepadModuleTriggers: GameInputGamepadButtons = 196608;
pub const GameInputGamepadNone: GameInputGamepadButtons = 0;
pub const GameInputGamepadPaddleLeft1: GameInputGamepadButtons = 67108864;
pub const GameInputGamepadPaddleLeft2: GameInputGamepadButtons = 134217728;
pub const GameInputGamepadPaddleRight1: GameInputGamepadButtons = 268435456;
pub const GameInputGamepadPaddleRight2: GameInputGamepadButtons = 536870912;
pub const GameInputGamepadRightShoulder: GameInputGamepadButtons = 2048;
pub const GameInputGamepadRightThumbstick: GameInputGamepadButtons = 8192;
pub const GameInputGamepadRightThumbstickDown: GameInputGamepadButtons = 8388608;
pub const GameInputGamepadRightThumbstickLeft: GameInputGamepadButtons = 16777216;
pub const GameInputGamepadRightThumbstickRight: GameInputGamepadButtons = 33554432;
pub const GameInputGamepadRightThumbstickUp: GameInputGamepadButtons = 4194304;
pub const GameInputGamepadRightTriggerButton: GameInputGamepadButtons = 131072;
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
pub const GameInputGamepadView: GameInputGamepadButtons = 2;
pub const GameInputGamepadX: GameInputGamepadButtons = 16;
pub const GameInputGamepadY: GameInputGamepadButtons = 32;
pub const GameInputGamepadZ: GameInputGamepadButtons = 32768;
pub const GameInputIsoKeyboard: GameInputKeyboardKind = 1;
pub const GameInputJisKeyboard: GameInputKeyboardKind = 4;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputKeyState {
    pub scanCode: u32,
    pub codePoint: u32,
    pub virtualKey: u8,
    pub isDeadKey: bool,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputKeyboardInfo {
    pub kind: GameInputKeyboardKind,
    pub layout: u32,
    pub keyCount: u32,
    pub functionKeyCount: u32,
    pub maxSimultaneousKeys: u32,
    pub platformType: u32,
    pub platformSubtype: u32,
}
pub type GameInputKeyboardKind = i32;
pub type GameInputKind = i32;
pub const GameInputKindArcadeStick: GameInputKind = 65536;
pub const GameInputKindController: GameInputKind = 14;
pub const GameInputKindControllerAxis: GameInputKind = 2;
pub const GameInputKindControllerButton: GameInputKind = 4;
pub const GameInputKindControllerSwitch: GameInputKind = 8;
pub const GameInputKindFlightStick: GameInputKind = 131072;
pub const GameInputKindGamepad: GameInputKind = 262144;
pub const GameInputKindKeyboard: GameInputKind = 16;
pub const GameInputKindLighting: GameInputKind = 1048576;
pub const GameInputKindMouse: GameInputKind = 32;
pub const GameInputKindRacingWheel: GameInputKind = 524288;
pub const GameInputKindRawDeviceReport: GameInputKind = 1;
pub const GameInputKindSensors: GameInputKind = 64;
pub const GameInputKindSystemButton: GameInputKind = 2097152;
pub const GameInputKindTouch: GameInputKind = 4194304;
pub const GameInputKindUnknown: GameInputKind = 0;
pub const GameInputKsKeyboard: GameInputKeyboardKind = 2;
pub type GameInputLabel = i32;
pub const GameInputLabelArrowClockwise: GameInputLabel = 72;
pub const GameInputLabelArrowCounterClockwise: GameInputLabel = 73;
pub const GameInputLabelArrowDown: GameInputLabel = 65;
pub const GameInputLabelArrowDownLLeft: GameInputLabel = 66;
pub const GameInputLabelArrowDownRight: GameInputLabel = 64;
pub const GameInputLabelArrowLeft: GameInputLabel = 67;
pub const GameInputLabelArrowLeftRight: GameInputLabel = 70;
pub const GameInputLabelArrowReturn: GameInputLabel = 74;
pub const GameInputLabelArrowRight: GameInputLabel = 63;
pub const GameInputLabelArrowUp: GameInputLabel = 61;
pub const GameInputLabelArrowUpDown: GameInputLabel = 69;
pub const GameInputLabelArrowUpDownLeftRight: GameInputLabel = 71;
pub const GameInputLabelArrowUpLeft: GameInputLabel = 68;
pub const GameInputLabelArrowUpRight: GameInputLabel = 62;
pub const GameInputLabelBack: GameInputLabel = 101;
pub const GameInputLabelBrake: GameInputLabel = 135;
pub const GameInputLabelClutch: GameInputLabel = 136;
pub const GameInputLabelDown: GameInputLabel = 106;
pub const GameInputLabelFirePrimary: GameInputLabel = 139;
pub const GameInputLabelFireSecondary: GameInputLabel = 140;
pub const GameInputLabelGuide: GameInputLabel = 96;
pub const GameInputLabelHandbrake: GameInputLabel = 137;
pub const GameInputLabelHome: GameInputLabel = 95;
pub const GameInputLabelIconBranding: GameInputLabel = 75;
pub const GameInputLabelIconCircle: GameInputLabel = 79;
pub const GameInputLabelIconCross: GameInputLabel = 78;
pub const GameInputLabelIconDPadDown: GameInputLabel = 84;
pub const GameInputLabelIconDPadLeft: GameInputLabel = 85;
pub const GameInputLabelIconDPadRight: GameInputLabel = 86;
pub const GameInputLabelIconDPadUp: GameInputLabel = 83;
pub const GameInputLabelIconDialClockwise: GameInputLabel = 87;
pub const GameInputLabelIconDialCounterClockwise: GameInputLabel = 88;
pub const GameInputLabelIconHome: GameInputLabel = 76;
pub const GameInputLabelIconMenu: GameInputLabel = 77;
pub const GameInputLabelIconMinus: GameInputLabel = 93;
pub const GameInputLabelIconPlus: GameInputLabel = 92;
pub const GameInputLabelIconSliderLeftRight: GameInputLabel = 89;
pub const GameInputLabelIconSliderUpDown: GameInputLabel = 90;
pub const GameInputLabelIconSquare: GameInputLabel = 80;
pub const GameInputLabelIconStar: GameInputLabel = 82;
pub const GameInputLabelIconSuspension: GameInputLabel = 94;
pub const GameInputLabelIconTriangle: GameInputLabel = 81;
pub const GameInputLabelIconWheelUpDown: GameInputLabel = 91;
pub const GameInputLabelL1: GameInputLabel = 112;
pub const GameInputLabelL2: GameInputLabel = 113;
pub const GameInputLabelL3: GameInputLabel = 114;
pub const GameInputLabelLB: GameInputLabel = 109;
pub const GameInputLabelLSB: GameInputLabel = 111;
pub const GameInputLabelLT: GameInputLabel = 110;
pub const GameInputLabelLeft: GameInputLabel = 107;
pub const GameInputLabelLeftThumbstickX: GameInputLabel = 126;
pub const GameInputLabelLeftThumbstickY: GameInputLabel = 127;
pub const GameInputLabelLetterA: GameInputLabel = 25;
pub const GameInputLabelLetterB: GameInputLabel = 26;
pub const GameInputLabelLetterC: GameInputLabel = 27;
pub const GameInputLabelLetterD: GameInputLabel = 28;
pub const GameInputLabelLetterE: GameInputLabel = 29;
pub const GameInputLabelLetterF: GameInputLabel = 30;
pub const GameInputLabelLetterG: GameInputLabel = 31;
pub const GameInputLabelLetterH: GameInputLabel = 32;
pub const GameInputLabelLetterI: GameInputLabel = 33;
pub const GameInputLabelLetterJ: GameInputLabel = 34;
pub const GameInputLabelLetterK: GameInputLabel = 35;
pub const GameInputLabelLetterL: GameInputLabel = 36;
pub const GameInputLabelLetterM: GameInputLabel = 37;
pub const GameInputLabelLetterN: GameInputLabel = 38;
pub const GameInputLabelLetterO: GameInputLabel = 39;
pub const GameInputLabelLetterP: GameInputLabel = 40;
pub const GameInputLabelLetterQ: GameInputLabel = 41;
pub const GameInputLabelLetterR: GameInputLabel = 42;
pub const GameInputLabelLetterS: GameInputLabel = 43;
pub const GameInputLabelLetterT: GameInputLabel = 44;
pub const GameInputLabelLetterU: GameInputLabel = 45;
pub const GameInputLabelLetterV: GameInputLabel = 46;
pub const GameInputLabelLetterW: GameInputLabel = 47;
pub const GameInputLabelLetterX: GameInputLabel = 48;
pub const GameInputLabelLetterY: GameInputLabel = 49;
pub const GameInputLabelLetterZ: GameInputLabel = 50;
pub const GameInputLabelMenu: GameInputLabel = 99;
pub const GameInputLabelMode: GameInputLabel = 97;
pub const GameInputLabelNone: GameInputLabel = 0;
pub const GameInputLabelNumber0: GameInputLabel = 51;
pub const GameInputLabelNumber1: GameInputLabel = 52;
pub const GameInputLabelNumber2: GameInputLabel = 53;
pub const GameInputLabelNumber3: GameInputLabel = 54;
pub const GameInputLabelNumber4: GameInputLabel = 55;
pub const GameInputLabelNumber5: GameInputLabel = 56;
pub const GameInputLabelNumber6: GameInputLabel = 57;
pub const GameInputLabelNumber7: GameInputLabel = 58;
pub const GameInputLabelNumber8: GameInputLabel = 59;
pub const GameInputLabelNumber9: GameInputLabel = 60;
pub const GameInputLabelOptions: GameInputLabel = 103;
pub const GameInputLabelPaddleLeft1: GameInputLabel = 121;
pub const GameInputLabelPaddleLeft2: GameInputLabel = 122;
pub const GameInputLabelPaddleRight1: GameInputLabel = 123;
pub const GameInputLabelPaddleRight2: GameInputLabel = 124;
pub const GameInputLabelPatternShifter: GameInputLabel = 138;
pub const GameInputLabelPitch: GameInputLabel = 131;
pub const GameInputLabelR1: GameInputLabel = 118;
pub const GameInputLabelR2: GameInputLabel = 119;
pub const GameInputLabelR3: GameInputLabel = 120;
pub const GameInputLabelRB: GameInputLabel = 115;
pub const GameInputLabelRSB: GameInputLabel = 117;
pub const GameInputLabelRT: GameInputLabel = 116;
pub const GameInputLabelRight: GameInputLabel = 108;
pub const GameInputLabelRightThumbstickX: GameInputLabel = 128;
pub const GameInputLabelRightThumbstickY: GameInputLabel = 129;
pub const GameInputLabelRoll: GameInputLabel = 130;
pub const GameInputLabelSelect: GameInputLabel = 98;
pub const GameInputLabelShare: GameInputLabel = 104;
pub const GameInputLabelStart: GameInputLabel = 102;
pub const GameInputLabelSteering: GameInputLabel = 134;
pub const GameInputLabelThrottle: GameInputLabel = 133;
pub const GameInputLabelTouchpadButton: GameInputLabel = 125;
pub const GameInputLabelUnknown: GameInputLabel = -1;
pub const GameInputLabelUp: GameInputLabel = 105;
pub const GameInputLabelView: GameInputLabel = 100;
pub const GameInputLabelXboxA: GameInputLabel = 7;
pub const GameInputLabelXboxB: GameInputLabel = 8;
pub const GameInputLabelXboxBack: GameInputLabel = 2;
pub const GameInputLabelXboxDPadDown: GameInputLabel = 12;
pub const GameInputLabelXboxDPadLeft: GameInputLabel = 13;
pub const GameInputLabelXboxDPadRight: GameInputLabel = 14;
pub const GameInputLabelXboxDPadUp: GameInputLabel = 11;
pub const GameInputLabelXboxGuide: GameInputLabel = 1;
pub const GameInputLabelXboxLeftShoulder: GameInputLabel = 15;
pub const GameInputLabelXboxLeftStickButton: GameInputLabel = 17;
pub const GameInputLabelXboxLeftTrigger: GameInputLabel = 16;
pub const GameInputLabelXboxMenu: GameInputLabel = 4;
pub const GameInputLabelXboxPaddle1: GameInputLabel = 21;
pub const GameInputLabelXboxPaddle2: GameInputLabel = 22;
pub const GameInputLabelXboxPaddle3: GameInputLabel = 23;
pub const GameInputLabelXboxPaddle4: GameInputLabel = 24;
pub const GameInputLabelXboxRightShoulder: GameInputLabel = 18;
pub const GameInputLabelXboxRightStickButton: GameInputLabel = 20;
pub const GameInputLabelXboxRightTrigger: GameInputLabel = 19;
pub const GameInputLabelXboxStart: GameInputLabel = 3;
pub const GameInputLabelXboxView: GameInputLabel = 5;
pub const GameInputLabelXboxX: GameInputLabel = 9;
pub const GameInputLabelXboxY: GameInputLabel = 10;
pub const GameInputLabelYaw: GameInputLabel = 132;
pub const GameInputMouseAbsolutePosition: GameInputMousePositions = 1;
pub const GameInputMouseButton4: GameInputMouseButtons = 8;
pub const GameInputMouseButton5: GameInputMouseButtons = 16;
pub type GameInputMouseButtons = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputMouseInfo {
    pub supportedButtons: GameInputMouseButtons,
    pub sampleRate: u32,
    pub hasWheelX: bool,
    pub hasWheelY: bool,
}
pub const GameInputMouseLeftButton: GameInputMouseButtons = 1;
pub const GameInputMouseMiddleButton: GameInputMouseButtons = 4;
pub const GameInputMouseNoPosition: GameInputMousePositions = 0;
pub const GameInputMouseNone: GameInputMouseButtons = 0;
pub type GameInputMousePositions = i32;
pub const GameInputMouseRelativePosition: GameInputMousePositions = 2;
pub const GameInputMouseRightButton: GameInputMouseButtons = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
pub const GameInputMouseWheelTiltLeft: GameInputMouseButtons = 32;
pub const GameInputMouseWheelTiltRight: GameInputMouseButtons = 64;
pub type GameInputReadingCallback = Option<
    unsafe extern "system" fn(
        callbacktoken: GameInputCallbackToken,
        context: *const core::ffi::c_void,
        reading: windows_core::Ref<IGameInputReading>,
    ),
>;
pub const GameInputRumbleHighFrequency: GameInputRumbleMotors = 2;
pub const GameInputRumbleLeftTrigger: GameInputRumbleMotors = 4;
pub const GameInputRumbleLowFrequency: GameInputRumbleMotors = 1;
pub type GameInputRumbleMotors = i32;
pub const GameInputRumbleNone: GameInputRumbleMotors = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputRumbleParams {
    pub lowFrequency: f32,
    pub highFrequency: f32,
    pub leftTrigger: f32,
    pub rightTrigger: f32,
}
pub const GameInputRumbleRightTrigger: GameInputRumbleMotors = 8;
pub const GameInputSystemButtonGuide: GameInputSystemButtons = 1;
pub const GameInputSystemButtonNone: GameInputSystemButtons = 0;
pub const GameInputSystemButtonShare: GameInputSystemButtons = 2;
pub type GameInputSystemButtons = i32;
pub const GameInputUnknownKeyboard: GameInputKeyboardKind = -1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputUsage {
    pub page: u16,
    pub id: u16,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputVersion {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
    pub revision: u16,
}
windows_core::imp::define_interface!(
    IGameInput,
    IGameInput_Vtbl,
    0xf2daae4c_4d21_4250_8953_4f791e6279ab
);
windows_core::imp::interface_hierarchy!(IGameInput, windows_core::IUnknown);
impl IGameInput {
    pub unsafe fn GetCurrentTimestamp(&self) -> u64 {
        unsafe {
            (windows_core::Interface::vtable(self).GetCurrentTimestamp)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetCurrentReading<P1>(
        &self,
        inputkind: GameInputKind,
        device: P1,
    ) -> windows_core::Result<IGameInputReading>
    where
        P1: windows_core::Param<IGameInputDevice>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentReading)(
                windows_core::Interface::as_raw(self),
                inputkind,
                device.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetNextReading<P0, P2>(
        &self,
        referencereading: P0,
        inputkind: GameInputKind,
        device: P2,
    ) -> windows_core::Result<IGameInputReading>
    where
        P0: windows_core::Param<IGameInputReading>,
        P2: windows_core::Param<IGameInputDevice>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetNextReading)(
                windows_core::Interface::as_raw(self),
                referencereading.param().abi(),
                inputkind,
                device.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetPreviousReading<P0, P2>(
        &self,
        referencereading: P0,
        inputkind: GameInputKind,
        device: P2,
    ) -> windows_core::Result<IGameInputReading>
    where
        P0: windows_core::Param<IGameInputReading>,
        P2: windows_core::Param<IGameInputDevice>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPreviousReading)(
                windows_core::Interface::as_raw(self),
                referencereading.param().abi(),
                inputkind,
                device.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn RegisterReadingCallback<P0>(
        &self,
        device: P0,
        inputkind: GameInputKind,
        context: Option<*const core::ffi::c_void>,
        callbackfunc: GameInputReadingCallback,
        callbacktoken: Option<*mut GameInputCallbackToken>,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IGameInputDevice>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RegisterReadingCallback)(
                windows_core::Interface::as_raw(self),
                device.param().abi(),
                inputkind,
                context.unwrap_or(core::mem::zeroed()) as _,
                callbackfunc,
                callbacktoken.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn RegisterDeviceCallback<P0>(
        &self,
        device: P0,
        inputkind: GameInputKind,
        context: Option<*const core::ffi::c_void>,
        callbackfunc: GameInputDeviceCallback,
        callbacktoken: Option<*mut GameInputCallbackToken>,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IGameInputDevice>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RegisterDeviceCallback)(
                windows_core::Interface::as_raw(self),
                device.param().abi(),
                inputkind,
                context.unwrap_or(core::mem::zeroed()) as _,
                callbackfunc,
                callbacktoken.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn UnregisterCallback(&self, callbacktoken: GameInputCallbackToken) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).UnregisterCallback)(
                windows_core::Interface::as_raw(self),
                callbacktoken,
            )
        }
    }
    pub unsafe fn FindDeviceFromId(
        &self,
        value: *const APP_LOCAL_DEVICE_ID,
    ) -> windows_core::Result<IGameInputDevice> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindDeviceFromId)(
                windows_core::Interface::as_raw(self),
                value,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn FindDeviceFromPlatformString<P0>(
        &self,
        value: P0,
    ) -> windows_core::Result<IGameInputDevice>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindDeviceFromPlatformString)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn EnableBackgroundInput(&self, enable: bool) {
        unsafe {
            (windows_core::Interface::vtable(self).EnableBackgroundInput)(
                windows_core::Interface::as_raw(self),
                enable,
            );
        }
    }
}
#[repr(C)]
pub struct IGameInput_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetCurrentTimestamp: unsafe extern "system" fn(*mut core::ffi::c_void) -> u64,
    pub GetCurrentReading: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetNextReading: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        GameInputKind,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPreviousReading: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        GameInputKind,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RegisterReadingCallback: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        GameInputKind,
        *const core::ffi::c_void,
        GameInputReadingCallback,
        *mut GameInputCallbackToken,
    ) -> windows_core::HRESULT,
    pub RegisterDeviceCallback: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        GameInputKind,
        *const core::ffi::c_void,
        GameInputDeviceCallback,
        *mut GameInputCallbackToken,
    ) -> windows_core::HRESULT,
    pub UnregisterCallback:
        unsafe extern "system" fn(*mut core::ffi::c_void, GameInputCallbackToken) -> bool,
    CreateDispatcher: usize,
    pub FindDeviceFromId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const APP_LOCAL_DEVICE_ID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindDeviceFromPlatformString: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub EnableBackgroundInput: unsafe extern "system" fn(*mut core::ffi::c_void, bool),
    CreateAggregateDevice: usize,
    DisableAggregateDevice: usize,
}
unsafe impl Send for IGameInput {}
unsafe impl Sync for IGameInput {}
impl windows_core::RuntimeName for IGameInput {}
windows_core::imp::define_interface!(
    IGameInputDevice,
    IGameInputDevice_Vtbl,
    0x4420f01d_8918_49d4_b302_dbead986105f
);
windows_core::imp::interface_hierarchy!(IGameInputDevice, windows_core::IUnknown);
impl IGameInputDevice {
    pub unsafe fn GetVendorId(&self) -> u16 {
        unsafe {
            (windows_core::Interface::vtable(self).GetVendorId)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetProductId(&self) -> u16 {
        unsafe {
            (windows_core::Interface::vtable(self).GetProductId)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetRevisionNumber(&self) -> u16 {
        unsafe {
            (windows_core::Interface::vtable(self).GetRevisionNumber)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetUsage(&self) -> GameInputUsage {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetUsage)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetHardwareVersion(&self) -> GameInputVersion {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetHardwareVersion)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetFirmwareVersion(&self) -> GameInputVersion {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFirmwareVersion)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetDeviceId(&self, deviceid: *mut APP_LOCAL_DEVICE_ID) {
        unsafe {
            (windows_core::Interface::vtable(self).GetDeviceId)(
                windows_core::Interface::as_raw(self),
                deviceid as _,
            );
        }
    }
    pub unsafe fn GetDeviceRootId(&self, devicerootid: *mut APP_LOCAL_DEVICE_ID) {
        unsafe {
            (windows_core::Interface::vtable(self).GetDeviceRootId)(
                windows_core::Interface::as_raw(self),
                devicerootid as _,
            );
        }
    }
    pub unsafe fn GetDeviceFamily(&self) -> GameInputDeviceFamily {
        unsafe {
            (windows_core::Interface::vtable(self).GetDeviceFamily)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetSupportedInput(&self) -> GameInputKind {
        unsafe {
            (windows_core::Interface::vtable(self).GetSupportedInput)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetSupportedRumbleMotors(&self) -> GameInputRumbleMotors {
        unsafe {
            (windows_core::Interface::vtable(self).GetSupportedRumbleMotors)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetSupportedSystemButtons(&self) -> GameInputSystemButtons {
        unsafe {
            (windows_core::Interface::vtable(self).GetSupportedSystemButtons)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetContainerId(&self) -> windows_core::GUID {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetContainerId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetDisplayName(&self, bufferbytecount: u32, buffer: Option<*mut u8>) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetDisplayName)(
                windows_core::Interface::as_raw(self),
                bufferbytecount,
                buffer.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetPnpPath(&self, bufferbytecount: u32, buffer: Option<*mut u8>) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetPnpPath)(
                windows_core::Interface::as_raw(self),
                bufferbytecount,
                buffer.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetXInputDeviceIndex(&self, index: *mut u32) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetXInputDeviceIndex)(
                windows_core::Interface::as_raw(self),
                index as _,
            )
        }
    }
    pub unsafe fn GetKeyboardInfo(&self, info: *mut GameInputKeyboardInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetKeyboardInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn GetMouseInfo(&self, info: *mut GameInputMouseInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetMouseInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn GetGamepadInfo(&self, info: *mut GameInputGamepadInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetGamepadInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn IsConnected(&self) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).IsConnected)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn SetRumbleState(&self, params: Option<*const GameInputRumbleParams>) {
        unsafe {
            (windows_core::Interface::vtable(self).SetRumbleState)(
                windows_core::Interface::as_raw(self),
                params.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
}
#[repr(C)]
pub struct IGameInputDevice_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetVendorId: unsafe extern "system" fn(*mut core::ffi::c_void) -> u16,
    pub GetProductId: unsafe extern "system" fn(*mut core::ffi::c_void) -> u16,
    pub GetRevisionNumber: unsafe extern "system" fn(*mut core::ffi::c_void) -> u16,
    pub GetUsage: unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputUsage),
    pub GetHardwareVersion:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputVersion),
    pub GetFirmwareVersion:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputVersion),
    pub GetDeviceId: unsafe extern "system" fn(*mut core::ffi::c_void, *mut APP_LOCAL_DEVICE_ID),
    pub GetDeviceRootId:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut APP_LOCAL_DEVICE_ID),
    pub GetDeviceFamily: unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputDeviceFamily,
    pub GetSupportedInput: unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputKind,
    pub GetSupportedRumbleMotors:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputRumbleMotors,
    pub GetSupportedSystemButtons:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputSystemButtons,
    GetSupportedLightingGroups: usize,
    GetSupportedLightingGroupSize: usize,
    pub GetContainerId: unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_core::GUID),
    pub GetDisplayName: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut u8) -> u32,
    pub GetPnpPath: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut u8) -> u32,
    pub GetXInputDeviceIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> bool,
    pub GetKeyboardInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputKeyboardInfo) -> bool,
    pub GetMouseInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputMouseInfo) -> bool,
    GetSensorsInfo: usize,
    GetControllerAxisCount: usize,
    GetControllerAxisLabels: usize,
    GetControllerButtonCount: usize,
    GetControllerButtonLabels: usize,
    GetControllerSwitchCount: usize,
    GetControllerSwitchInfo: usize,
    GetArcadeStickInfo: usize,
    GetFlightStickInfo: usize,
    pub GetGamepadInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputGamepadInfo) -> bool,
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
    pub IsConnected: unsafe extern "system" fn(*mut core::ffi::c_void) -> bool,
    CreateForceFeedbackEffect: usize,
    IsForceFeedbackMotorPoweredOn: usize,
    SetForceFeedbackMotorGain: usize,
    pub SetRumbleState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const GameInputRumbleParams),
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
unsafe impl Send for IGameInputDevice {}
unsafe impl Sync for IGameInputDevice {}
impl windows_core::RuntimeName for IGameInputDevice {}
windows_core::imp::define_interface!(
    IGameInputReading,
    IGameInputReading_Vtbl,
    0x0dfaa2a2_688b_4a9c_a3a9_0feb4f90ee12
);
windows_core::imp::interface_hierarchy!(IGameInputReading, windows_core::IUnknown);
impl IGameInputReading {
    pub unsafe fn GetInputKind(&self) -> GameInputKind {
        unsafe {
            (windows_core::Interface::vtable(self).GetInputKind)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetTimestamp(&self) -> u64 {
        unsafe {
            (windows_core::Interface::vtable(self).GetTimestamp)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetDevice(&self) -> windows_core::Result<IGameInputDevice> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDevice)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub unsafe fn GetKeyCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetKeyCount)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetKeyState(
        &self,
        statearraycount: u32,
        statearray: *mut GameInputKeyState,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetKeyState)(
                windows_core::Interface::as_raw(self),
                statearraycount,
                statearray as _,
            )
        }
    }
    pub unsafe fn GetMouseState(&self, state: *mut GameInputMouseState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetMouseState)(
                windows_core::Interface::as_raw(self),
                state as _,
            )
        }
    }
    pub unsafe fn GetGamepadState(&self, state: *mut GameInputGamepadState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetGamepadState)(
                windows_core::Interface::as_raw(self),
                state as _,
            )
        }
    }
    pub unsafe fn GetSystemButtonState(&self, buttons: *mut GameInputSystemButtons) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetSystemButtonState)(
                windows_core::Interface::as_raw(self),
                buttons as _,
            )
        }
    }
}
#[repr(C)]
pub struct IGameInputReading_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetInputKind: unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputKind,
    pub GetTimestamp: unsafe extern "system" fn(*mut core::ffi::c_void) -> u64,
    pub GetDevice: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    GetControllerAxisCount: usize,
    GetControllerAxisState: usize,
    GetControllerButtonCount: usize,
    GetControllerButtonState: usize,
    GetControllerSwitchCount: usize,
    GetControllerSwitchState: usize,
    pub GetKeyCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetKeyState:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut GameInputKeyState) -> u32,
    pub GetMouseState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputMouseState) -> bool,
    GetSensorsState: usize,
    GetArcadeStickState: usize,
    GetFlightStickState: usize,
    pub GetGamepadState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputGamepadState) -> bool,
    GetRacingWheelState: usize,
    GetRawReport: usize,
    pub GetSystemButtonState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputSystemButtons) -> bool,
    GetTouchContactCount: usize,
    GetTouchSurfaceState: usize,
}
unsafe impl Send for IGameInputReading {}
unsafe impl Sync for IGameInputReading {}
impl windows_core::RuntimeName for IGameInputReading {}
