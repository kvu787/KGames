// The bindings for all of GameInput's v4 API, from native/include/v4/GameInput.h
// in the Microsoft.GameInput NuGet package that NuGetPackages.csproj declares,
// and nothing else: what windows-bindgen generates for the header, which
// bindings-generator describes as Windows metadata for it. Don't edit it by
// hand: regenerate it with `cargo run -p game-input-bindings-generator`.

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
pub const GAMEINPUT_E_AGGREGATE_OPERATION_NOT_SUPPORTED: windows_core::HRESULT =
    windows_core::HRESULT(0x838A000B_u32 as _);
pub const GAMEINPUT_E_CALLBACK_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0009_u32 as _);
pub const GAMEINPUT_E_DEVICE_DISCONNECTED: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0001_u32 as _);
pub const GAMEINPUT_E_DEVICE_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0002_u32 as _);
pub const GAMEINPUT_E_FEEDBACK_NOT_SUPPORTED: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0007_u32 as _);
pub const GAMEINPUT_E_HAPTIC_INFO_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A000A_u32 as _);
pub const GAMEINPUT_E_INPUT_KIND_NOT_PRESENT: windows_core::HRESULT =
    windows_core::HRESULT(0x838A000C_u32 as _);
pub const GAMEINPUT_E_OBJECT_NO_LONGER_EXISTS: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0008_u32 as _);
pub const GAMEINPUT_E_READING_NOT_FOUND: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0003_u32 as _);
pub const GAMEINPUT_E_REFERENCE_READING_TOO_OLD: windows_core::HRESULT =
    windows_core::HRESULT(0x838A0004_u32 as _);
pub const GAMEINPUT_FACILITY: i32 = 906;
pub const GAMEINPUT_HAPTIC_LOCATION_GRIP_LEFT: windows_core::GUID =
    windows_core::GUID::from_u128(0x08c707c2_66bb_406c_a84a_dfe085120a92);
pub const GAMEINPUT_HAPTIC_LOCATION_GRIP_RIGHT: windows_core::GUID =
    windows_core::GUID::from_u128(0x155a0b77_8bb2_40db_8690_b6d41126dfc1);
pub const GAMEINPUT_HAPTIC_LOCATION_NONE: windows_core::GUID =
    windows_core::GUID::from_u128(0x00000000_0000_0000_0000_000000000000);
pub const GAMEINPUT_HAPTIC_LOCATION_TRIGGER_LEFT: windows_core::GUID =
    windows_core::GUID::from_u128(0x8de4d896_5559_4081_86e5_1724cc07c6bc);
pub const GAMEINPUT_HAPTIC_LOCATION_TRIGGER_RIGHT: windows_core::GUID =
    windows_core::GUID::from_u128(0xff0cb557_3af5_406b_8b0f_555a2d92a220);
pub const GAMEINPUT_HAPTIC_MAX_AUDIO_ENDPOINT_ID_SIZE: u32 = 256;
pub const GAMEINPUT_HAPTIC_MAX_LOCATIONS: u32 = 8;
pub const GAMEINPUT_LIGHTING_GROUP_ORDER_INDEX_ALL: u32 = 4294967295;
pub const GAMEINPUT_MAX_SWITCH_STATES: u32 = 8;
pub const GameInput2WaySwitch: GameInputSwitchKind = 0;
pub const GameInput4WaySwitch: GameInputSwitchKind = 1;
pub const GameInput8WaySwitch: GameInputSwitchKind = 2;
pub const GameInputAbntKeyboard: GameInputKeyboardKind = 3;
pub const GameInputAnsiKeyboard: GameInputKeyboardKind = 0;
pub const GameInputArcadeStickAction1: GameInputArcadeStickButtons = 64;
pub const GameInputArcadeStickAction2: GameInputArcadeStickButtons = 128;
pub const GameInputArcadeStickAction3: GameInputArcadeStickButtons = 256;
pub const GameInputArcadeStickAction4: GameInputArcadeStickButtons = 512;
pub const GameInputArcadeStickAction5: GameInputArcadeStickButtons = 1024;
pub const GameInputArcadeStickAction6: GameInputArcadeStickButtons = 2048;
pub type GameInputArcadeStickButtons = i32;
pub const GameInputArcadeStickDown: GameInputArcadeStickButtons = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputArcadeStickInfo {
    pub menuButtonLabel: GameInputLabel,
    pub viewButtonLabel: GameInputLabel,
    pub stickUpLabel: GameInputLabel,
    pub stickDownLabel: GameInputLabel,
    pub stickLeftLabel: GameInputLabel,
    pub stickRightLabel: GameInputLabel,
    pub actionButton1Label: GameInputLabel,
    pub actionButton2Label: GameInputLabel,
    pub actionButton3Label: GameInputLabel,
    pub actionButton4Label: GameInputLabel,
    pub actionButton5Label: GameInputLabel,
    pub actionButton6Label: GameInputLabel,
    pub specialButton1Label: GameInputLabel,
    pub specialButton2Label: GameInputLabel,
    pub leftThumbstickButtonLabel: GameInputLabel,
    pub rightThumbstickButtonLabel: GameInputLabel,
    pub extraButtonCount: u32,
    pub extraAxisCount: u32,
}
pub const GameInputArcadeStickLeft: GameInputArcadeStickButtons = 16;
pub const GameInputArcadeStickLeftThumbstick: GameInputArcadeStickButtons = 16384;
pub const GameInputArcadeStickMenu: GameInputArcadeStickButtons = 1;
pub const GameInputArcadeStickNone: GameInputArcadeStickButtons = 0;
pub const GameInputArcadeStickRight: GameInputArcadeStickButtons = 32;
pub const GameInputArcadeStickRightThumbstick: GameInputArcadeStickButtons = 32768;
pub const GameInputArcadeStickSpecial1: GameInputArcadeStickButtons = 4096;
pub const GameInputArcadeStickSpecial2: GameInputArcadeStickButtons = 8192;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputArcadeStickState {
    pub buttons: GameInputArcadeStickButtons,
}
pub const GameInputArcadeStickUp: GameInputArcadeStickButtons = 4;
pub const GameInputArcadeStickView: GameInputArcadeStickButtons = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputAxisMapping {
    pub controllerElementKind: GameInputElementKind,
    pub controllerIndex: u32,
    pub isInverted: bool,
    pub fromTwoButtons: bool,
    pub buttonMinIndexValue: u32,
    pub referenceDirection: GameInputSwitchPosition,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputButtonMapping {
    pub controllerElementKind: GameInputElementKind,
    pub controllerIndex: u32,
    pub isInverted: bool,
    pub switchPosition: GameInputSwitchPosition,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GameInputCallbackToken(pub u64);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameInputControllerSwitchInfo {
    pub labels: [GameInputLabel; 8],
    pub kind: GameInputSwitchKind,
}
impl Default for GameInputControllerSwitchInfo {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
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
pub type GameInputElementKind = i32;
pub const GameInputElementKindAxis: GameInputElementKind = 1;
pub const GameInputElementKindButton: GameInputElementKind = 2;
pub const GameInputElementKindNone: GameInputElementKind = 0;
pub const GameInputElementKindSwitch: GameInputElementKind = 3;
pub const GameInputFamilyAggregate: GameInputDeviceFamily = 5;
pub const GameInputFamilyHid: GameInputDeviceFamily = 3;
pub const GameInputFamilyI8042: GameInputDeviceFamily = 4;
pub const GameInputFamilyUnknown: GameInputDeviceFamily = 0;
pub const GameInputFamilyVirtual: GameInputDeviceFamily = -1;
pub const GameInputFamilyXbox360: GameInputDeviceFamily = 2;
pub const GameInputFamilyXboxOne: GameInputDeviceFamily = 1;
pub type GameInputFeedbackAxes = i32;
pub const GameInputFeedbackAxisAngularX: GameInputFeedbackAxes = 8;
pub const GameInputFeedbackAxisAngularY: GameInputFeedbackAxes = 16;
pub const GameInputFeedbackAxisAngularZ: GameInputFeedbackAxes = 32;
pub const GameInputFeedbackAxisLinearX: GameInputFeedbackAxes = 1;
pub const GameInputFeedbackAxisLinearY: GameInputFeedbackAxes = 2;
pub const GameInputFeedbackAxisLinearZ: GameInputFeedbackAxes = 4;
pub const GameInputFeedbackAxisNone: GameInputFeedbackAxes = 0;
pub const GameInputFeedbackAxisNormal: GameInputFeedbackAxes = 64;
pub type GameInputFeedbackEffectState = i32;
pub const GameInputFeedbackPaused: GameInputFeedbackEffectState = 2;
pub const GameInputFeedbackRunning: GameInputFeedbackEffectState = 1;
pub const GameInputFeedbackStopped: GameInputFeedbackEffectState = 0;
pub const GameInputFlightStickA: GameInputFlightStickButtons = 256;
pub type GameInputFlightStickAxes = i32;
pub const GameInputFlightStickAxesNone: GameInputFlightStickAxes = 0;
pub const GameInputFlightStickB: GameInputFlightStickButtons = 512;
pub type GameInputFlightStickButtons = i32;
pub const GameInputFlightStickFirePrimary: GameInputFlightStickButtons = 4;
pub const GameInputFlightStickFireSecondary: GameInputFlightStickButtons = 8;
pub const GameInputFlightStickHatSwitchDown: GameInputFlightStickButtons = 32;
pub const GameInputFlightStickHatSwitchLeft: GameInputFlightStickButtons = 64;
pub const GameInputFlightStickHatSwitchRight: GameInputFlightStickButtons = 128;
pub const GameInputFlightStickHatSwitchUp: GameInputFlightStickButtons = 16;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputFlightStickInfo {
    pub menuButtonLabel: GameInputLabel,
    pub viewButtonLabel: GameInputLabel,
    pub firePrimaryButtonLabel: GameInputLabel,
    pub fireSecondaryButtonLabel: GameInputLabel,
    pub hatSwitchUpLabel: GameInputLabel,
    pub hatSwitchDownLabel: GameInputLabel,
    pub hatSwitchLeftLabel: GameInputLabel,
    pub hatSwitchRightLabel: GameInputLabel,
    pub aButtonLabel: GameInputLabel,
    pub bButtonLabel: GameInputLabel,
    pub xButtonLabel: GameInputLabel,
    pub yButtonLabel: GameInputLabel,
    pub leftShoulderButtonLabel: GameInputLabel,
    pub rightShoulderButtonLabel: GameInputLabel,
    pub rollLabel: GameInputLabel,
    pub pitchLabel: GameInputLabel,
    pub yawLabel: GameInputLabel,
    pub throttleLabel: GameInputLabel,
    pub extraButtonCount: u32,
    pub extraAxisCount: u32,
}
pub const GameInputFlightStickLeftShoulder: GameInputFlightStickButtons = 4096;
pub const GameInputFlightStickMenu: GameInputFlightStickButtons = 1;
pub const GameInputFlightStickNone: GameInputFlightStickButtons = 0;
pub const GameInputFlightStickPitch: GameInputFlightStickAxes = 32;
pub const GameInputFlightStickRightShoulder: GameInputFlightStickButtons = 8192;
pub const GameInputFlightStickRoll: GameInputFlightStickAxes = 16;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputFlightStickState {
    pub buttons: GameInputFlightStickButtons,
    pub hatSwitch: GameInputSwitchPosition,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub throttle: f32,
}
pub const GameInputFlightStickThrottle: GameInputFlightStickAxes = 128;
pub const GameInputFlightStickView: GameInputFlightStickButtons = 2;
pub const GameInputFlightStickX: GameInputFlightStickButtons = 1024;
pub const GameInputFlightStickY: GameInputFlightStickButtons = 2048;
pub const GameInputFlightStickYaw: GameInputFlightStickAxes = 64;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackConditionParams {
    pub magnitude: GameInputForceFeedbackMagnitude,
    pub positiveCoefficient: f32,
    pub negativeCoefficient: f32,
    pub maxPositiveMagnitude: f32,
    pub maxNegativeMagnitude: f32,
    pub deadZone: f32,
    pub bias: f32,
}
pub const GameInputForceFeedbackConstant: GameInputForceFeedbackEffectKind = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackConstantParams {
    pub envelope: GameInputForceFeedbackEnvelope,
    pub magnitude: GameInputForceFeedbackMagnitude,
}
pub const GameInputForceFeedbackDamper: GameInputForceFeedbackEffectKind = 9;
pub type GameInputForceFeedbackEffectKind = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackEnvelope {
    pub attackDuration: u64,
    pub sustainDuration: u64,
    pub releaseDuration: u64,
    pub attackGain: f32,
    pub sustainGain: f32,
    pub releaseGain: f32,
    pub playCount: u32,
    pub repeatDelay: u64,
}
pub const GameInputForceFeedbackFriction: GameInputForceFeedbackEffectKind = 8;
pub const GameInputForceFeedbackInertia: GameInputForceFeedbackEffectKind = 10;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackMagnitude {
    pub linearX: f32,
    pub linearY: f32,
    pub linearZ: f32,
    pub angularX: f32,
    pub angularY: f32,
    pub angularZ: f32,
    pub normal: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputForceFeedbackMotorInfo {
    pub supportedAxes: GameInputFeedbackAxes,
    pub isConstantEffectSupported: bool,
    pub isRampEffectSupported: bool,
    pub isSineWaveEffectSupported: bool,
    pub isSquareWaveEffectSupported: bool,
    pub isTriangleWaveEffectSupported: bool,
    pub isSawtoothUpWaveEffectSupported: bool,
    pub isSawtoothDownWaveEffectSupported: bool,
    pub isSpringEffectSupported: bool,
    pub isFrictionEffectSupported: bool,
    pub isDamperEffectSupported: bool,
    pub isInertiaEffectSupported: bool,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameInputForceFeedbackParams {
    pub kind: GameInputForceFeedbackEffectKind,
    pub data: GameInputForceFeedbackParams_0,
}
impl Default for GameInputForceFeedbackParams {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union GameInputForceFeedbackParams_0 {
    pub constant: GameInputForceFeedbackConstantParams,
    pub ramp: GameInputForceFeedbackRampParams,
    pub sineWave: GameInputForceFeedbackPeriodicParams,
    pub squareWave: GameInputForceFeedbackPeriodicParams,
    pub triangleWave: GameInputForceFeedbackPeriodicParams,
    pub sawtoothUpWave: GameInputForceFeedbackPeriodicParams,
    pub sawtoothDownWave: GameInputForceFeedbackPeriodicParams,
    pub spring: GameInputForceFeedbackConditionParams,
    pub friction: GameInputForceFeedbackConditionParams,
    pub damper: GameInputForceFeedbackConditionParams,
    pub inertia: GameInputForceFeedbackConditionParams,
}
impl Default for GameInputForceFeedbackParams_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackPeriodicParams {
    pub envelope: GameInputForceFeedbackEnvelope,
    pub magnitude: GameInputForceFeedbackMagnitude,
    pub frequency: f32,
    pub phase: f32,
    pub bias: f32,
}
pub const GameInputForceFeedbackRamp: GameInputForceFeedbackEffectKind = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputForceFeedbackRampParams {
    pub envelope: GameInputForceFeedbackEnvelope,
    pub startMagnitude: GameInputForceFeedbackMagnitude,
    pub endMagnitude: GameInputForceFeedbackMagnitude,
}
pub const GameInputForceFeedbackSawtoothDownWave: GameInputForceFeedbackEffectKind = 6;
pub const GameInputForceFeedbackSawtoothUpWave: GameInputForceFeedbackEffectKind = 5;
pub const GameInputForceFeedbackSineWave: GameInputForceFeedbackEffectKind = 2;
pub const GameInputForceFeedbackSpring: GameInputForceFeedbackEffectKind = 7;
pub const GameInputForceFeedbackSquareWave: GameInputForceFeedbackEffectKind = 3;
pub const GameInputForceFeedbackTriangleWave: GameInputForceFeedbackEffectKind = 4;
pub const GameInputGamepadA: GameInputGamepadButtons = 4;
pub type GameInputGamepadAxes = i32;
pub const GameInputGamepadAxesNone: GameInputGamepadAxes = 0;
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
pub const GameInputGamepadLeftThumbstickX: GameInputGamepadAxes = 4;
pub const GameInputGamepadLeftThumbstickY: GameInputGamepadAxes = 8;
pub const GameInputGamepadLeftTrigger: GameInputGamepadAxes = 1;
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
pub const GameInputGamepadRightThumbstickX: GameInputGamepadAxes = 16;
pub const GameInputGamepadRightThumbstickY: GameInputGamepadAxes = 32;
pub const GameInputGamepadRightTrigger: GameInputGamepadAxes = 2;
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
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameInputHapticInfo {
    pub audioEndpointId: [u16; 256],
    pub locationCount: u32,
    pub locations: [windows_core::GUID; 8],
}
impl Default for GameInputHapticInfo {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
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
pub const GameInputLightingGroupAll: GameInputLightingGroups = -1;
pub const GameInputLightingGroupCenterLight: GameInputLightingGroups = 16;
pub const GameInputLightingGroupDPad: GameInputLightingGroups = 4;
pub const GameInputLightingGroupFaceButtons: GameInputLightingGroups = 8;
pub const GameInputLightingGroupKeyboardKey: GameInputLightingGroups = 32;
pub const GameInputLightingGroupLeftHeadset: GameInputLightingGroups = 64;
pub const GameInputLightingGroupLeftThumbstick: GameInputLightingGroups = 1;
pub const GameInputLightingGroupRightHeadset: GameInputLightingGroups = 128;
pub const GameInputLightingGroupRightThumbstick: GameInputLightingGroups = 2;
pub const GameInputLightingGroupUnknown: GameInputLightingGroups = 0;
pub type GameInputLightingGroups = i32;
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
pub const GameInputRacingWheelA: GameInputRacingWheelButtons = 256;
pub type GameInputRacingWheelAxes = i32;
pub const GameInputRacingWheelAxesNone: GameInputRacingWheelAxes = 0;
pub const GameInputRacingWheelB: GameInputRacingWheelButtons = 512;
pub const GameInputRacingWheelBrake: GameInputRacingWheelAxes = 1024;
pub type GameInputRacingWheelButtons = i32;
pub const GameInputRacingWheelClutch: GameInputRacingWheelAxes = 2048;
pub const GameInputRacingWheelDpadDown: GameInputRacingWheelButtons = 32;
pub const GameInputRacingWheelDpadLeft: GameInputRacingWheelButtons = 64;
pub const GameInputRacingWheelDpadRight: GameInputRacingWheelButtons = 128;
pub const GameInputRacingWheelDpadUp: GameInputRacingWheelButtons = 16;
pub const GameInputRacingWheelHandbrake: GameInputRacingWheelAxes = 4096;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputRacingWheelInfo {
    pub menuButtonLabel: GameInputLabel,
    pub viewButtonLabel: GameInputLabel,
    pub previousGearButtonLabel: GameInputLabel,
    pub nextGearButtonLabel: GameInputLabel,
    pub dpadUpLabel: GameInputLabel,
    pub dpadDownLabel: GameInputLabel,
    pub dpadLeftLabel: GameInputLabel,
    pub dpadRightLabel: GameInputLabel,
    pub aButtonLabel: GameInputLabel,
    pub bButtonLabel: GameInputLabel,
    pub xButtonLabel: GameInputLabel,
    pub yButtonLabel: GameInputLabel,
    pub leftThumbstickButtonLabel: GameInputLabel,
    pub rightThumbstickButtonLabel: GameInputLabel,
    pub steeringLabel: GameInputLabel,
    pub throttleLabel: GameInputLabel,
    pub brakeLabel: GameInputLabel,
    pub clutchLabel: GameInputLabel,
    pub handbrakeLabel: GameInputLabel,
    pub hasClutch: bool,
    pub hasHandbrake: bool,
    pub hasPatternShifter: bool,
    pub minPatternShifterGear: i32,
    pub maxPatternShifterGear: i32,
    pub maxWheelAngle: f32,
    pub extraButtonCount: u32,
    pub extraAxisCount: u32,
}
pub const GameInputRacingWheelLeftThumbstick: GameInputRacingWheelButtons = 4096;
pub const GameInputRacingWheelMenu: GameInputRacingWheelButtons = 1;
pub const GameInputRacingWheelNextGear: GameInputRacingWheelButtons = 8;
pub const GameInputRacingWheelNone: GameInputRacingWheelButtons = 0;
pub const GameInputRacingWheelPatternShifter: GameInputRacingWheelAxes = 8192;
pub const GameInputRacingWheelPreviousGear: GameInputRacingWheelButtons = 4;
pub const GameInputRacingWheelRightThumbstick: GameInputRacingWheelButtons = 8192;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputRacingWheelState {
    pub buttons: GameInputRacingWheelButtons,
    pub patternShifterGear: i32,
    pub wheel: f32,
    pub throttle: f32,
    pub brake: f32,
    pub clutch: f32,
    pub handbrake: f32,
}
pub const GameInputRacingWheelSteering: GameInputRacingWheelAxes = 256;
pub const GameInputRacingWheelThrottle: GameInputRacingWheelAxes = 512;
pub const GameInputRacingWheelView: GameInputRacingWheelButtons = 2;
pub const GameInputRacingWheelX: GameInputRacingWheelButtons = 1024;
pub const GameInputRacingWheelY: GameInputRacingWheelButtons = 2048;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputRawDeviceReportInfo {
    pub kind: GameInputRawDeviceReportKind,
    pub id: u32,
    pub size: u32,
}
pub type GameInputRawDeviceReportKind = i32;
pub const GameInputRawFeatureReport: GameInputRawDeviceReportKind = 2;
pub const GameInputRawInputReport: GameInputRawDeviceReportKind = 0;
pub const GameInputRawOutputReport: GameInputRawDeviceReportKind = 1;
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
pub type GameInputSensorAccuracy = i32;
pub const GameInputSensorAccuracyApproximate: GameInputSensorAccuracy = 2;
pub const GameInputSensorAccuracyHigh: GameInputSensorAccuracy = 3;
pub const GameInputSensorAccuracyUnknown: GameInputSensorAccuracy = 0;
pub const GameInputSensorAccuracyUnreliable: GameInputSensorAccuracy = 1;
pub const GameInputSensorsAccelerometer: GameInputSensorsKind = 1;
pub const GameInputSensorsCompass: GameInputSensorsKind = 4;
pub const GameInputSensorsGyrometer: GameInputSensorsKind = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputSensorsInfo {
    pub supportedSensors: GameInputSensorsKind,
}
pub type GameInputSensorsKind = i32;
pub const GameInputSensorsNone: GameInputSensorsKind = 0;
pub const GameInputSensorsOrientation: GameInputSensorsKind = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInputSensorsState {
    pub accelerationInGX: f32,
    pub accelerationInGY: f32,
    pub accelerationInGZ: f32,
    pub angularVelocityInRadPerSecX: f32,
    pub angularVelocityInRadPerSecY: f32,
    pub angularVelocityInRadPerSecZ: f32,
    pub headingInDegreesFromMagneticNorth: f32,
    pub headingAccuracy: GameInputSensorAccuracy,
    pub orientationW: f32,
    pub orientationX: f32,
    pub orientationY: f32,
    pub orientationZ: f32,
}
pub const GameInputSwitchCenter: GameInputSwitchPosition = 0;
pub const GameInputSwitchDown: GameInputSwitchPosition = 5;
pub const GameInputSwitchDownLeft: GameInputSwitchPosition = 6;
pub const GameInputSwitchDownRight: GameInputSwitchPosition = 4;
pub type GameInputSwitchKind = i32;
pub const GameInputSwitchLeft: GameInputSwitchPosition = 7;
pub type GameInputSwitchPosition = i32;
pub const GameInputSwitchRight: GameInputSwitchPosition = 3;
pub const GameInputSwitchUp: GameInputSwitchPosition = 1;
pub const GameInputSwitchUpLeft: GameInputSwitchPosition = 8;
pub const GameInputSwitchUpRight: GameInputSwitchPosition = 2;
pub const GameInputSystemButtonGuide: GameInputSystemButtons = 1;
pub const GameInputSystemButtonNone: GameInputSystemButtons = 0;
pub const GameInputSystemButtonShare: GameInputSystemButtons = 2;
pub type GameInputSystemButtons = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputTouchContact {
    pub contactId: u64,
    pub screenX: i32,
    pub screenY: i32,
    pub padX: u32,
    pub padY: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameInputTouchSurfaceInfo {
    pub maxContactCount: u32,
    pub padWidth: u32,
    pub padHeight: u32,
    pub r#type: GameInputTouchSurfaceType,
}
pub type GameInputTouchSurfaceType = i32;
pub const GameInputTouchSurfaceTypeTouchpad: GameInputTouchSurfaceType = 2;
pub const GameInputTouchSurfaceTypeTouchscreen: GameInputTouchSurfaceType = 1;
pub const GameInputTouchSurfaceTypeUnknown: GameInputTouchSurfaceType = 0;
pub const GameInputUnknownKeyboard: GameInputKeyboardKind = -1;
pub const GameInputUnknownSwitchKind: GameInputSwitchKind = -1;
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
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
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
    pub unsafe fn CreateDispatcher(&self) -> windows_core::Result<IGameInputDispatcher> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDispatcher)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
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
    pub unsafe fn CreateAggregateDevice(
        &self,
        inputkind: GameInputKind,
        deviceid: *mut APP_LOCAL_DEVICE_ID,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateAggregateDevice)(
                windows_core::Interface::as_raw(self),
                inputkind,
                deviceid as _,
            )
        }
    }
    pub unsafe fn DisableAggregateDevice(
        &self,
        deviceid: *const APP_LOCAL_DEVICE_ID,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DisableAggregateDevice)(
                windows_core::Interface::as_raw(self),
                deviceid,
            )
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
    pub CreateDispatcher: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
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
    pub CreateAggregateDevice: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        *mut APP_LOCAL_DEVICE_ID,
    ) -> windows_core::HRESULT,
    pub DisableAggregateDevice: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const APP_LOCAL_DEVICE_ID,
    ) -> windows_core::HRESULT,
}
unsafe impl Send for IGameInput {}
unsafe impl Sync for IGameInput {}
pub trait IGameInput_Impl: windows_core::IUnknownImpl {
    fn GetCurrentTimestamp(&self) -> u64;
    fn GetCurrentReading(
        &self,
        inputkind: GameInputKind,
        device: windows_core::Ref<IGameInputDevice>,
    ) -> windows_core::Result<IGameInputReading>;
    fn GetNextReading(
        &self,
        referencereading: windows_core::Ref<IGameInputReading>,
        inputkind: GameInputKind,
        device: windows_core::Ref<IGameInputDevice>,
    ) -> windows_core::Result<IGameInputReading>;
    fn GetPreviousReading(
        &self,
        referencereading: windows_core::Ref<IGameInputReading>,
        inputkind: GameInputKind,
        device: windows_core::Ref<IGameInputDevice>,
    ) -> windows_core::Result<IGameInputReading>;
    fn RegisterReadingCallback(
        &self,
        device: windows_core::Ref<IGameInputDevice>,
        inputkind: GameInputKind,
        context: *const core::ffi::c_void,
        callbackfunc: GameInputReadingCallback,
        callbacktoken: *mut GameInputCallbackToken,
    ) -> windows_core::Result<()>;
    fn RegisterDeviceCallback(
        &self,
        device: windows_core::Ref<IGameInputDevice>,
        inputkind: GameInputKind,
        context: *const core::ffi::c_void,
        callbackfunc: GameInputDeviceCallback,
        callbacktoken: *mut GameInputCallbackToken,
    ) -> windows_core::Result<()>;
    fn UnregisterCallback(&self, callbacktoken: GameInputCallbackToken) -> bool;
    fn CreateDispatcher(&self) -> windows_core::Result<IGameInputDispatcher>;
    fn FindDeviceFromId(
        &self,
        value: *const APP_LOCAL_DEVICE_ID,
    ) -> windows_core::Result<IGameInputDevice>;
    fn FindDeviceFromPlatformString(
        &self,
        value: &windows_core::PCWSTR,
    ) -> windows_core::Result<IGameInputDevice>;
    fn EnableBackgroundInput(&self, enable: bool);
    fn CreateAggregateDevice(
        &self,
        inputkind: GameInputKind,
        deviceid: *mut APP_LOCAL_DEVICE_ID,
    ) -> windows_core::Result<()>;
    fn DisableAggregateDevice(
        &self,
        deviceid: *const APP_LOCAL_DEVICE_ID,
    ) -> windows_core::Result<()>;
}
impl IGameInput_Vtbl {
    pub const fn new<Identity: IGameInput_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetCurrentTimestamp<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u64 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::GetCurrentTimestamp(this)
            }
        }
        unsafe extern "system" fn GetCurrentReading<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            device: *mut core::ffi::c_void,
            reading: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::GetCurrentReading(
                    this,
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&device),
                ) {
                    Ok(ok__) => {
                        reading.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetNextReading<Identity: IGameInput_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            referencereading: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            device: *mut core::ffi::c_void,
            reading: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::GetNextReading(
                    this,
                    core::mem::transmute_copy(&referencereading),
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&device),
                ) {
                    Ok(ok__) => {
                        reading.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPreviousReading<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            referencereading: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            device: *mut core::ffi::c_void,
            reading: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::GetPreviousReading(
                    this,
                    core::mem::transmute_copy(&referencereading),
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&device),
                ) {
                    Ok(ok__) => {
                        reading.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RegisterReadingCallback<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            device: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            context: *const core::ffi::c_void,
            callbackfunc: GameInputReadingCallback,
            callbacktoken: *mut GameInputCallbackToken,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::RegisterReadingCallback(
                    this,
                    core::mem::transmute_copy(&device),
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&context),
                    core::mem::transmute_copy(&callbackfunc),
                    core::mem::transmute_copy(&callbacktoken),
                )
                .into()
            }
        }
        unsafe extern "system" fn RegisterDeviceCallback<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            device: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            context: *const core::ffi::c_void,
            callbackfunc: GameInputDeviceCallback,
            callbacktoken: *mut GameInputCallbackToken,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::RegisterDeviceCallback(
                    this,
                    core::mem::transmute_copy(&device),
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&context),
                    core::mem::transmute_copy(&callbackfunc),
                    core::mem::transmute_copy(&callbacktoken),
                )
                .into()
            }
        }
        unsafe extern "system" fn UnregisterCallback<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            callbacktoken: GameInputCallbackToken,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::UnregisterCallback(this, core::mem::transmute_copy(&callbacktoken))
            }
        }
        unsafe extern "system" fn CreateDispatcher<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            dispatcher: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::CreateDispatcher(this) {
                    Ok(ok__) => {
                        dispatcher.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindDeviceFromId<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            value: *const APP_LOCAL_DEVICE_ID,
            device: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::FindDeviceFromId(this, core::mem::transmute_copy(&value)) {
                    Ok(ok__) => {
                        device.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindDeviceFromPlatformString<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            value: windows_core::PCWSTR,
            device: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInput_Impl::FindDeviceFromPlatformString(
                    this,
                    core::mem::transmute(&value),
                ) {
                    Ok(ok__) => {
                        device.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn EnableBackgroundInput<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            enable: bool,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::EnableBackgroundInput(this, core::mem::transmute_copy(&enable));
            }
        }
        unsafe extern "system" fn CreateAggregateDevice<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            deviceid: *mut APP_LOCAL_DEVICE_ID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::CreateAggregateDevice(
                    this,
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&deviceid),
                )
                .into()
            }
        }
        unsafe extern "system" fn DisableAggregateDevice<
            Identity: IGameInput_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            deviceid: *const APP_LOCAL_DEVICE_ID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInput_Impl::DisableAggregateDevice(this, core::mem::transmute_copy(&deviceid))
                    .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetCurrentTimestamp: GetCurrentTimestamp::<Identity, OFFSET>,
            GetCurrentReading: GetCurrentReading::<Identity, OFFSET>,
            GetNextReading: GetNextReading::<Identity, OFFSET>,
            GetPreviousReading: GetPreviousReading::<Identity, OFFSET>,
            RegisterReadingCallback: RegisterReadingCallback::<Identity, OFFSET>,
            RegisterDeviceCallback: RegisterDeviceCallback::<Identity, OFFSET>,
            UnregisterCallback: UnregisterCallback::<Identity, OFFSET>,
            CreateDispatcher: CreateDispatcher::<Identity, OFFSET>,
            FindDeviceFromId: FindDeviceFromId::<Identity, OFFSET>,
            FindDeviceFromPlatformString: FindDeviceFromPlatformString::<Identity, OFFSET>,
            EnableBackgroundInput: EnableBackgroundInput::<Identity, OFFSET>,
            CreateAggregateDevice: CreateAggregateDevice::<Identity, OFFSET>,
            DisableAggregateDevice: DisableAggregateDevice::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInput as windows_core::Interface>::IID
    }
}
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
    pub unsafe fn GetSupportedLightingGroups(&self) -> GameInputLightingGroups {
        unsafe {
            (windows_core::Interface::vtable(self).GetSupportedLightingGroups)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetSupportedLightingGroupSize(&self, group: GameInputLightingGroups) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetSupportedLightingGroupSize)(
                windows_core::Interface::as_raw(self),
                group,
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
    pub unsafe fn GetSensorsInfo(&self, info: *mut GameInputSensorsInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetSensorsInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn GetControllerAxisCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerAxisCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerAxisLabels(
        &self,
        labelarraycount: u32,
        labelarray: Option<*mut GameInputLabel>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerAxisLabels)(
                windows_core::Interface::as_raw(self),
                labelarraycount,
                labelarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetControllerButtonCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerButtonCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerButtonLabels(
        &self,
        labelarraycount: u32,
        labelarray: Option<*mut GameInputLabel>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerButtonLabels)(
                windows_core::Interface::as_raw(self),
                labelarraycount,
                labelarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetControllerSwitchCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerSwitchCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerSwitchInfo(
        &self,
        infoarraycount: u32,
        infoarray: Option<*mut GameInputControllerSwitchInfo>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerSwitchInfo)(
                windows_core::Interface::as_raw(self),
                infoarraycount,
                infoarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetArcadeStickInfo(&self, info: *mut GameInputArcadeStickInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetArcadeStickInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn GetFlightStickInfo(&self, info: *mut GameInputFlightStickInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetFlightStickInfo)(
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
    pub unsafe fn GetRacingWheelInfo(&self, info: *mut GameInputRacingWheelInfo) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetRacingWheelInfo)(
                windows_core::Interface::as_raw(self),
                info as _,
            )
        }
    }
    pub unsafe fn GetForceFeedbackMotorCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetForceFeedbackMotorCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetForceFeedbackMotorInfo(
        &self,
        infoarraycount: u32,
        infoarray: Option<*mut GameInputForceFeedbackMotorInfo>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetForceFeedbackMotorInfo)(
                windows_core::Interface::as_raw(self),
                infoarraycount,
                infoarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetInputReportCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetInputReportCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetInputReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: Option<*mut GameInputRawDeviceReportInfo>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetInputReportInfo)(
                windows_core::Interface::as_raw(self),
                infoarraycount,
                infoarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetOutputReportCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetOutputReportCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetOutputReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: Option<*mut GameInputRawDeviceReportInfo>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetOutputReportInfo)(
                windows_core::Interface::as_raw(self),
                infoarraycount,
                infoarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetFeatureReportCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetFeatureReportCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetFeatureReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: Option<*mut GameInputRawDeviceReportInfo>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetFeatureReportInfo)(
                windows_core::Interface::as_raw(self),
                infoarraycount,
                infoarray.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn GetHapticInfo(&self, info: *mut GameInputHapticInfo) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetHapticInfo)(
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
    pub unsafe fn CreateForceFeedbackEffect(
        &self,
        motorindex: u32,
        params: *const GameInputForceFeedbackParams,
    ) -> windows_core::Result<IGameInputForceFeedbackEffect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateForceFeedbackEffect)(
                windows_core::Interface::as_raw(self),
                motorindex,
                params,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn IsForceFeedbackMotorPoweredOn(&self, motorindex: u32) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).IsForceFeedbackMotorPoweredOn)(
                windows_core::Interface::as_raw(self),
                motorindex,
            )
        }
    }
    pub unsafe fn SetForceFeedbackMotorGain(&self, motorindex: u32, mastergain: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetForceFeedbackMotorGain)(
                windows_core::Interface::as_raw(self),
                motorindex,
                mastergain,
            );
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
    pub unsafe fn DirectInputEscape(
        &self,
        command: u32,
        bufferin: &[u8],
        bufferout: *mut u8,
        bufferoutsize: u32,
        bufferoutsizewritten: Option<*mut u32>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DirectInputEscape)(
                windows_core::Interface::as_raw(self),
                command,
                bufferin.as_ptr(),
                bufferin.len().try_into().unwrap(),
                bufferout as _,
                bufferoutsize,
                bufferoutsizewritten.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub unsafe fn CreateInputMapper(&self) -> windows_core::Result<IGameInputMapper> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateInputMapper)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetExtraAxisCount(&self, inputkind: GameInputKind) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetExtraAxisCount)(
                windows_core::Interface::as_raw(self),
                inputkind,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn GetExtraButtonCount(
        &self,
        inputkind: GameInputKind,
    ) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetExtraButtonCount)(
                windows_core::Interface::as_raw(self),
                inputkind,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn GetExtraAxisIndexes(
        &self,
        inputkind: GameInputKind,
        extraaxiscount: u32,
        extraaxisindexes: *mut u8,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetExtraAxisIndexes)(
                windows_core::Interface::as_raw(self),
                inputkind,
                extraaxiscount,
                extraaxisindexes as _,
            )
        }
    }
    pub unsafe fn GetExtraButtonIndexes(
        &self,
        inputkind: GameInputKind,
        extrabuttoncount: u32,
        extrabuttonindexes: *mut u8,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetExtraButtonIndexes)(
                windows_core::Interface::as_raw(self),
                inputkind,
                extrabuttoncount,
                extrabuttonindexes as _,
            )
        }
    }
    pub unsafe fn CreateRawDeviceReport(
        &self,
        reportid: u32,
        reportkind: GameInputRawDeviceReportKind,
    ) -> windows_core::Result<IGameInputRawDeviceReport> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateRawDeviceReport)(
                windows_core::Interface::as_raw(self),
                reportid,
                reportkind,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetRawDeviceFeature(
        &self,
        reportid: u32,
    ) -> windows_core::Result<IGameInputRawDeviceReport> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRawDeviceFeature)(
                windows_core::Interface::as_raw(self),
                reportid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn SetRawDeviceFeature<P0>(&self, report: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IGameInputRawDeviceReport>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetRawDeviceFeature)(
                windows_core::Interface::as_raw(self),
                report.param().abi(),
            )
        }
    }
    pub unsafe fn SendRawDeviceOutput<P0>(&self, report: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IGameInputRawDeviceReport>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SendRawDeviceOutput)(
                windows_core::Interface::as_raw(self),
                report.param().abi(),
            )
        }
    }
    pub unsafe fn SetLightingColor(
        &self,
        color: GameInputColor,
        group: GameInputLightingGroups,
        grouporderindex: Option<u32>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetLightingColor)(
                windows_core::Interface::as_raw(self),
                color,
                group,
                grouporderindex.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub unsafe fn ApplyLightingColors(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).ApplyLightingColors)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
    pub unsafe fn ApplyLightingSystemDefault(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).ApplyLightingSystemDefault)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
    pub unsafe fn GetTouchSurfaceCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetTouchSurfaceCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetTouchSurfaceInfo(
        &self,
        surfaceindex: u32,
    ) -> windows_core::Result<GameInputTouchSurfaceInfo> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTouchSurfaceInfo)(
                windows_core::Interface::as_raw(self),
                surfaceindex,
                &mut result__,
            )
            .map(|| result__)
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
    pub GetSupportedLightingGroups:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputLightingGroups,
    pub GetSupportedLightingGroupSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, GameInputLightingGroups) -> u32,
    pub GetContainerId: unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_core::GUID),
    pub GetDisplayName: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut u8) -> u32,
    pub GetPnpPath: unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut u8) -> u32,
    pub GetXInputDeviceIndex: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> bool,
    pub GetKeyboardInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputKeyboardInfo) -> bool,
    pub GetMouseInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputMouseInfo) -> bool,
    pub GetSensorsInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputSensorsInfo) -> bool,
    pub GetControllerAxisCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerAxisLabels:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut GameInputLabel) -> u32,
    pub GetControllerButtonCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerButtonLabels:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut GameInputLabel) -> u32,
    pub GetControllerSwitchCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerSwitchInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputControllerSwitchInfo,
    ) -> u32,
    pub GetArcadeStickInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputArcadeStickInfo) -> bool,
    pub GetFlightStickInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputFlightStickInfo) -> bool,
    pub GetGamepadInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputGamepadInfo) -> bool,
    pub GetRacingWheelInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputRacingWheelInfo) -> bool,
    pub GetForceFeedbackMotorCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetForceFeedbackMotorInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputForceFeedbackMotorInfo,
    ) -> u32,
    pub GetInputReportCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetInputReportInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputRawDeviceReportInfo,
    ) -> u32,
    pub GetOutputReportCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetOutputReportInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputRawDeviceReportInfo,
    ) -> u32,
    pub GetFeatureReportCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetFeatureReportInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputRawDeviceReportInfo,
    ) -> u32,
    pub GetHapticInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut GameInputHapticInfo,
    ) -> windows_core::HRESULT,
    pub IsConnected: unsafe extern "system" fn(*mut core::ffi::c_void) -> bool,
    pub CreateForceFeedbackEffect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const GameInputForceFeedbackParams,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub IsForceFeedbackMotorPoweredOn:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> bool,
    pub SetForceFeedbackMotorGain: unsafe extern "system" fn(*mut core::ffi::c_void, u32, f32),
    pub SetRumbleState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const GameInputRumbleParams),
    pub DirectInputEscape: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const u8,
        u32,
        *mut u8,
        u32,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub CreateInputMapper: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetExtraAxisCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetExtraButtonCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetExtraAxisIndexes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        u32,
        *mut u8,
    ) -> windows_core::HRESULT,
    pub GetExtraButtonIndexes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputKind,
        u32,
        *mut u8,
    ) -> windows_core::HRESULT,
    pub CreateRawDeviceReport: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        GameInputRawDeviceReportKind,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetRawDeviceFeature: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetRawDeviceFeature: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SendRawDeviceOutput: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetLightingColor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputColor,
        GameInputLightingGroups,
        u32,
    ),
    pub ApplyLightingColors: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub ApplyLightingSystemDefault: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub GetTouchSurfaceCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetTouchSurfaceInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut GameInputTouchSurfaceInfo,
    ) -> windows_core::HRESULT,
}
unsafe impl Send for IGameInputDevice {}
unsafe impl Sync for IGameInputDevice {}
pub trait IGameInputDevice_Impl: windows_core::IUnknownImpl {
    fn GetVendorId(&self) -> u16;
    fn GetProductId(&self) -> u16;
    fn GetRevisionNumber(&self) -> u16;
    fn GetUsage(&self, usage: *mut GameInputUsage);
    fn GetHardwareVersion(&self, version: *mut GameInputVersion);
    fn GetFirmwareVersion(&self, version: *mut GameInputVersion);
    fn GetDeviceId(&self, deviceid: *mut APP_LOCAL_DEVICE_ID);
    fn GetDeviceRootId(&self, devicerootid: *mut APP_LOCAL_DEVICE_ID);
    fn GetDeviceFamily(&self) -> GameInputDeviceFamily;
    fn GetSupportedInput(&self) -> GameInputKind;
    fn GetSupportedRumbleMotors(&self) -> GameInputRumbleMotors;
    fn GetSupportedSystemButtons(&self) -> GameInputSystemButtons;
    fn GetSupportedLightingGroups(&self) -> GameInputLightingGroups;
    fn GetSupportedLightingGroupSize(&self, group: GameInputLightingGroups) -> u32;
    fn GetContainerId(&self, containerid: *mut windows_core::GUID);
    fn GetDisplayName(&self, bufferbytecount: u32, buffer: *mut u8) -> u32;
    fn GetPnpPath(&self, bufferbytecount: u32, buffer: *mut u8) -> u32;
    fn GetXInputDeviceIndex(&self, index: *mut u32) -> bool;
    fn GetKeyboardInfo(&self, info: *mut GameInputKeyboardInfo) -> bool;
    fn GetMouseInfo(&self, info: *mut GameInputMouseInfo) -> bool;
    fn GetSensorsInfo(&self, info: *mut GameInputSensorsInfo) -> bool;
    fn GetControllerAxisCount(&self) -> u32;
    fn GetControllerAxisLabels(&self, labelarraycount: u32, labelarray: *mut GameInputLabel)
    -> u32;
    fn GetControllerButtonCount(&self) -> u32;
    fn GetControllerButtonLabels(
        &self,
        labelarraycount: u32,
        labelarray: *mut GameInputLabel,
    ) -> u32;
    fn GetControllerSwitchCount(&self) -> u32;
    fn GetControllerSwitchInfo(
        &self,
        infoarraycount: u32,
        infoarray: *mut GameInputControllerSwitchInfo,
    ) -> u32;
    fn GetArcadeStickInfo(&self, info: *mut GameInputArcadeStickInfo) -> bool;
    fn GetFlightStickInfo(&self, info: *mut GameInputFlightStickInfo) -> bool;
    fn GetGamepadInfo(&self, info: *mut GameInputGamepadInfo) -> bool;
    fn GetRacingWheelInfo(&self, info: *mut GameInputRacingWheelInfo) -> bool;
    fn GetForceFeedbackMotorCount(&self) -> u32;
    fn GetForceFeedbackMotorInfo(
        &self,
        infoarraycount: u32,
        infoarray: *mut GameInputForceFeedbackMotorInfo,
    ) -> u32;
    fn GetInputReportCount(&self) -> u32;
    fn GetInputReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: *mut GameInputRawDeviceReportInfo,
    ) -> u32;
    fn GetOutputReportCount(&self) -> u32;
    fn GetOutputReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: *mut GameInputRawDeviceReportInfo,
    ) -> u32;
    fn GetFeatureReportCount(&self) -> u32;
    fn GetFeatureReportInfo(
        &self,
        infoarraycount: u32,
        infoarray: *mut GameInputRawDeviceReportInfo,
    ) -> u32;
    fn GetHapticInfo(&self, info: *mut GameInputHapticInfo) -> windows_core::Result<()>;
    fn IsConnected(&self) -> bool;
    fn CreateForceFeedbackEffect(
        &self,
        motorindex: u32,
        params: *const GameInputForceFeedbackParams,
    ) -> windows_core::Result<IGameInputForceFeedbackEffect>;
    fn IsForceFeedbackMotorPoweredOn(&self, motorindex: u32) -> bool;
    fn SetForceFeedbackMotorGain(&self, motorindex: u32, mastergain: f32);
    fn SetRumbleState(&self, params: *const GameInputRumbleParams);
    fn DirectInputEscape(
        &self,
        command: u32,
        bufferin: *const u8,
        bufferinsize: u32,
        bufferout: *mut u8,
        bufferoutsize: u32,
        bufferoutsizewritten: *mut u32,
    ) -> windows_core::Result<()>;
    fn CreateInputMapper(&self) -> windows_core::Result<IGameInputMapper>;
    fn GetExtraAxisCount(&self, inputkind: GameInputKind) -> windows_core::Result<u32>;
    fn GetExtraButtonCount(&self, inputkind: GameInputKind) -> windows_core::Result<u32>;
    fn GetExtraAxisIndexes(
        &self,
        inputkind: GameInputKind,
        extraaxiscount: u32,
        extraaxisindexes: *mut u8,
    ) -> windows_core::Result<()>;
    fn GetExtraButtonIndexes(
        &self,
        inputkind: GameInputKind,
        extrabuttoncount: u32,
        extrabuttonindexes: *mut u8,
    ) -> windows_core::Result<()>;
    fn CreateRawDeviceReport(
        &self,
        reportid: u32,
        reportkind: GameInputRawDeviceReportKind,
    ) -> windows_core::Result<IGameInputRawDeviceReport>;
    fn GetRawDeviceFeature(&self, reportid: u32)
    -> windows_core::Result<IGameInputRawDeviceReport>;
    fn SetRawDeviceFeature(
        &self,
        report: windows_core::Ref<IGameInputRawDeviceReport>,
    ) -> windows_core::Result<()>;
    fn SendRawDeviceOutput(
        &self,
        report: windows_core::Ref<IGameInputRawDeviceReport>,
    ) -> windows_core::Result<()>;
    fn SetLightingColor(
        &self,
        color: &GameInputColor,
        group: GameInputLightingGroups,
        grouporderindex: u32,
    );
    fn ApplyLightingColors(&self);
    fn ApplyLightingSystemDefault(&self);
    fn GetTouchSurfaceCount(&self) -> u32;
    fn GetTouchSurfaceInfo(
        &self,
        surfaceindex: u32,
    ) -> windows_core::Result<GameInputTouchSurfaceInfo>;
}
impl IGameInputDevice_Vtbl {
    pub const fn new<Identity: IGameInputDevice_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetVendorId<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u16 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetVendorId(this)
            }
        }
        unsafe extern "system" fn GetProductId<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u16 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetProductId(this)
            }
        }
        unsafe extern "system" fn GetRevisionNumber<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u16 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetRevisionNumber(this)
            }
        }
        unsafe extern "system" fn GetUsage<Identity: IGameInputDevice_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            usage: *mut GameInputUsage,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetUsage(this, core::mem::transmute_copy(&usage));
            }
        }
        unsafe extern "system" fn GetHardwareVersion<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            version: *mut GameInputVersion,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetHardwareVersion(
                    this,
                    core::mem::transmute_copy(&version),
                );
            }
        }
        unsafe extern "system" fn GetFirmwareVersion<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            version: *mut GameInputVersion,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetFirmwareVersion(
                    this,
                    core::mem::transmute_copy(&version),
                );
            }
        }
        unsafe extern "system" fn GetDeviceId<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            deviceid: *mut APP_LOCAL_DEVICE_ID,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetDeviceId(this, core::mem::transmute_copy(&deviceid));
            }
        }
        unsafe extern "system" fn GetDeviceRootId<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            devicerootid: *mut APP_LOCAL_DEVICE_ID,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetDeviceRootId(
                    this,
                    core::mem::transmute_copy(&devicerootid),
                );
            }
        }
        unsafe extern "system" fn GetDeviceFamily<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputDeviceFamily {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetDeviceFamily(this)
            }
        }
        unsafe extern "system" fn GetSupportedInput<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputKind {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSupportedInput(this)
            }
        }
        unsafe extern "system" fn GetSupportedRumbleMotors<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputRumbleMotors {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSupportedRumbleMotors(this)
            }
        }
        unsafe extern "system" fn GetSupportedSystemButtons<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputSystemButtons {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSupportedSystemButtons(this)
            }
        }
        unsafe extern "system" fn GetSupportedLightingGroups<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputLightingGroups {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSupportedLightingGroups(this)
            }
        }
        unsafe extern "system" fn GetSupportedLightingGroupSize<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            group: GameInputLightingGroups,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSupportedLightingGroupSize(
                    this,
                    core::mem::transmute_copy(&group),
                )
            }
        }
        unsafe extern "system" fn GetContainerId<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            containerid: *mut windows_core::GUID,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetContainerId(
                    this,
                    core::mem::transmute_copy(&containerid),
                );
            }
        }
        unsafe extern "system" fn GetDisplayName<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            bufferbytecount: u32,
            buffer: *mut u8,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetDisplayName(
                    this,
                    core::mem::transmute_copy(&bufferbytecount),
                    core::mem::transmute_copy(&buffer),
                )
            }
        }
        unsafe extern "system" fn GetPnpPath<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            bufferbytecount: u32,
            buffer: *mut u8,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetPnpPath(
                    this,
                    core::mem::transmute_copy(&bufferbytecount),
                    core::mem::transmute_copy(&buffer),
                )
            }
        }
        unsafe extern "system" fn GetXInputDeviceIndex<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: *mut u32,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetXInputDeviceIndex(this, core::mem::transmute_copy(&index))
            }
        }
        unsafe extern "system" fn GetKeyboardInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputKeyboardInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetKeyboardInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetMouseInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputMouseInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetMouseInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetSensorsInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputSensorsInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetSensorsInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetControllerAxisCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerAxisCount(this)
            }
        }
        unsafe extern "system" fn GetControllerAxisLabels<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            labelarraycount: u32,
            labelarray: *mut GameInputLabel,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerAxisLabels(
                    this,
                    core::mem::transmute_copy(&labelarraycount),
                    core::mem::transmute_copy(&labelarray),
                )
            }
        }
        unsafe extern "system" fn GetControllerButtonCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerButtonCount(this)
            }
        }
        unsafe extern "system" fn GetControllerButtonLabels<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            labelarraycount: u32,
            labelarray: *mut GameInputLabel,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerButtonLabels(
                    this,
                    core::mem::transmute_copy(&labelarraycount),
                    core::mem::transmute_copy(&labelarray),
                )
            }
        }
        unsafe extern "system" fn GetControllerSwitchCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerSwitchCount(this)
            }
        }
        unsafe extern "system" fn GetControllerSwitchInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            infoarraycount: u32,
            infoarray: *mut GameInputControllerSwitchInfo,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetControllerSwitchInfo(
                    this,
                    core::mem::transmute_copy(&infoarraycount),
                    core::mem::transmute_copy(&infoarray),
                )
            }
        }
        unsafe extern "system" fn GetArcadeStickInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputArcadeStickInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetArcadeStickInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetFlightStickInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputFlightStickInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetFlightStickInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetGamepadInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputGamepadInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetGamepadInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetRacingWheelInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputRacingWheelInfo,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetRacingWheelInfo(this, core::mem::transmute_copy(&info))
            }
        }
        unsafe extern "system" fn GetForceFeedbackMotorCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetForceFeedbackMotorCount(this)
            }
        }
        unsafe extern "system" fn GetForceFeedbackMotorInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            infoarraycount: u32,
            infoarray: *mut GameInputForceFeedbackMotorInfo,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetForceFeedbackMotorInfo(
                    this,
                    core::mem::transmute_copy(&infoarraycount),
                    core::mem::transmute_copy(&infoarray),
                )
            }
        }
        unsafe extern "system" fn GetInputReportCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetInputReportCount(this)
            }
        }
        unsafe extern "system" fn GetInputReportInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            infoarraycount: u32,
            infoarray: *mut GameInputRawDeviceReportInfo,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetInputReportInfo(
                    this,
                    core::mem::transmute_copy(&infoarraycount),
                    core::mem::transmute_copy(&infoarray),
                )
            }
        }
        unsafe extern "system" fn GetOutputReportCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetOutputReportCount(this)
            }
        }
        unsafe extern "system" fn GetOutputReportInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            infoarraycount: u32,
            infoarray: *mut GameInputRawDeviceReportInfo,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetOutputReportInfo(
                    this,
                    core::mem::transmute_copy(&infoarraycount),
                    core::mem::transmute_copy(&infoarray),
                )
            }
        }
        unsafe extern "system" fn GetFeatureReportCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetFeatureReportCount(this)
            }
        }
        unsafe extern "system" fn GetFeatureReportInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            infoarraycount: u32,
            infoarray: *mut GameInputRawDeviceReportInfo,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetFeatureReportInfo(
                    this,
                    core::mem::transmute_copy(&infoarraycount),
                    core::mem::transmute_copy(&infoarray),
                )
            }
        }
        unsafe extern "system" fn GetHapticInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            info: *mut GameInputHapticInfo,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetHapticInfo(this, core::mem::transmute_copy(&info)).into()
            }
        }
        unsafe extern "system" fn IsConnected<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::IsConnected(this)
            }
        }
        unsafe extern "system" fn CreateForceFeedbackEffect<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            motorindex: u32,
            params: *const GameInputForceFeedbackParams,
            effect: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::CreateForceFeedbackEffect(
                    this,
                    core::mem::transmute_copy(&motorindex),
                    core::mem::transmute_copy(&params),
                ) {
                    Ok(ok__) => {
                        effect.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn IsForceFeedbackMotorPoweredOn<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            motorindex: u32,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::IsForceFeedbackMotorPoweredOn(
                    this,
                    core::mem::transmute_copy(&motorindex),
                )
            }
        }
        unsafe extern "system" fn SetForceFeedbackMotorGain<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            motorindex: u32,
            mastergain: f32,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::SetForceFeedbackMotorGain(
                    this,
                    core::mem::transmute_copy(&motorindex),
                    core::mem::transmute_copy(&mastergain),
                );
            }
        }
        unsafe extern "system" fn SetRumbleState<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            params: *const GameInputRumbleParams,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::SetRumbleState(this, core::mem::transmute_copy(&params));
            }
        }
        unsafe extern "system" fn DirectInputEscape<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            command: u32,
            bufferin: *const u8,
            bufferinsize: u32,
            bufferout: *mut u8,
            bufferoutsize: u32,
            bufferoutsizewritten: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::DirectInputEscape(
                    this,
                    core::mem::transmute_copy(&command),
                    core::mem::transmute_copy(&bufferin),
                    core::mem::transmute_copy(&bufferinsize),
                    core::mem::transmute_copy(&bufferout),
                    core::mem::transmute_copy(&bufferoutsize),
                    core::mem::transmute_copy(&bufferoutsizewritten),
                )
                .into()
            }
        }
        unsafe extern "system" fn CreateInputMapper<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputmapper: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::CreateInputMapper(this) {
                    Ok(ok__) => {
                        inputmapper.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetExtraAxisCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            extraaxiscount: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::GetExtraAxisCount(
                    this,
                    core::mem::transmute_copy(&inputkind),
                ) {
                    Ok(ok__) => {
                        extraaxiscount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetExtraButtonCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            extrabuttoncount: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::GetExtraButtonCount(
                    this,
                    core::mem::transmute_copy(&inputkind),
                ) {
                    Ok(ok__) => {
                        extrabuttoncount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetExtraAxisIndexes<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            extraaxiscount: u32,
            extraaxisindexes: *mut u8,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetExtraAxisIndexes(
                    this,
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&extraaxiscount),
                    core::mem::transmute_copy(&extraaxisindexes),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetExtraButtonIndexes<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            inputkind: GameInputKind,
            extrabuttoncount: u32,
            extrabuttonindexes: *mut u8,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetExtraButtonIndexes(
                    this,
                    core::mem::transmute_copy(&inputkind),
                    core::mem::transmute_copy(&extrabuttoncount),
                    core::mem::transmute_copy(&extrabuttonindexes),
                )
                .into()
            }
        }
        unsafe extern "system" fn CreateRawDeviceReport<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            reportid: u32,
            reportkind: GameInputRawDeviceReportKind,
            report: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::CreateRawDeviceReport(
                    this,
                    core::mem::transmute_copy(&reportid),
                    core::mem::transmute_copy(&reportkind),
                ) {
                    Ok(ok__) => {
                        report.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetRawDeviceFeature<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            reportid: u32,
            report: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::GetRawDeviceFeature(
                    this,
                    core::mem::transmute_copy(&reportid),
                ) {
                    Ok(ok__) => {
                        report.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetRawDeviceFeature<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            report: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::SetRawDeviceFeature(this, core::mem::transmute_copy(&report))
                    .into()
            }
        }
        unsafe extern "system" fn SendRawDeviceOutput<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            report: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::SendRawDeviceOutput(this, core::mem::transmute_copy(&report))
                    .into()
            }
        }
        unsafe extern "system" fn SetLightingColor<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            color: GameInputColor,
            group: GameInputLightingGroups,
            grouporderindex: u32,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::SetLightingColor(
                    this,
                    core::mem::transmute(&color),
                    core::mem::transmute_copy(&group),
                    core::mem::transmute_copy(&grouporderindex),
                );
            }
        }
        unsafe extern "system" fn ApplyLightingColors<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::ApplyLightingColors(this);
            }
        }
        unsafe extern "system" fn ApplyLightingSystemDefault<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::ApplyLightingSystemDefault(this);
            }
        }
        unsafe extern "system" fn GetTouchSurfaceCount<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDevice_Impl::GetTouchSurfaceCount(this)
            }
        }
        unsafe extern "system" fn GetTouchSurfaceInfo<
            Identity: IGameInputDevice_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            surfaceindex: u32,
            info: *mut GameInputTouchSurfaceInfo,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDevice_Impl::GetTouchSurfaceInfo(
                    this,
                    core::mem::transmute_copy(&surfaceindex),
                ) {
                    Ok(ok__) => {
                        info.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetVendorId: GetVendorId::<Identity, OFFSET>,
            GetProductId: GetProductId::<Identity, OFFSET>,
            GetRevisionNumber: GetRevisionNumber::<Identity, OFFSET>,
            GetUsage: GetUsage::<Identity, OFFSET>,
            GetHardwareVersion: GetHardwareVersion::<Identity, OFFSET>,
            GetFirmwareVersion: GetFirmwareVersion::<Identity, OFFSET>,
            GetDeviceId: GetDeviceId::<Identity, OFFSET>,
            GetDeviceRootId: GetDeviceRootId::<Identity, OFFSET>,
            GetDeviceFamily: GetDeviceFamily::<Identity, OFFSET>,
            GetSupportedInput: GetSupportedInput::<Identity, OFFSET>,
            GetSupportedRumbleMotors: GetSupportedRumbleMotors::<Identity, OFFSET>,
            GetSupportedSystemButtons: GetSupportedSystemButtons::<Identity, OFFSET>,
            GetSupportedLightingGroups: GetSupportedLightingGroups::<Identity, OFFSET>,
            GetSupportedLightingGroupSize: GetSupportedLightingGroupSize::<Identity, OFFSET>,
            GetContainerId: GetContainerId::<Identity, OFFSET>,
            GetDisplayName: GetDisplayName::<Identity, OFFSET>,
            GetPnpPath: GetPnpPath::<Identity, OFFSET>,
            GetXInputDeviceIndex: GetXInputDeviceIndex::<Identity, OFFSET>,
            GetKeyboardInfo: GetKeyboardInfo::<Identity, OFFSET>,
            GetMouseInfo: GetMouseInfo::<Identity, OFFSET>,
            GetSensorsInfo: GetSensorsInfo::<Identity, OFFSET>,
            GetControllerAxisCount: GetControllerAxisCount::<Identity, OFFSET>,
            GetControllerAxisLabels: GetControllerAxisLabels::<Identity, OFFSET>,
            GetControllerButtonCount: GetControllerButtonCount::<Identity, OFFSET>,
            GetControllerButtonLabels: GetControllerButtonLabels::<Identity, OFFSET>,
            GetControllerSwitchCount: GetControllerSwitchCount::<Identity, OFFSET>,
            GetControllerSwitchInfo: GetControllerSwitchInfo::<Identity, OFFSET>,
            GetArcadeStickInfo: GetArcadeStickInfo::<Identity, OFFSET>,
            GetFlightStickInfo: GetFlightStickInfo::<Identity, OFFSET>,
            GetGamepadInfo: GetGamepadInfo::<Identity, OFFSET>,
            GetRacingWheelInfo: GetRacingWheelInfo::<Identity, OFFSET>,
            GetForceFeedbackMotorCount: GetForceFeedbackMotorCount::<Identity, OFFSET>,
            GetForceFeedbackMotorInfo: GetForceFeedbackMotorInfo::<Identity, OFFSET>,
            GetInputReportCount: GetInputReportCount::<Identity, OFFSET>,
            GetInputReportInfo: GetInputReportInfo::<Identity, OFFSET>,
            GetOutputReportCount: GetOutputReportCount::<Identity, OFFSET>,
            GetOutputReportInfo: GetOutputReportInfo::<Identity, OFFSET>,
            GetFeatureReportCount: GetFeatureReportCount::<Identity, OFFSET>,
            GetFeatureReportInfo: GetFeatureReportInfo::<Identity, OFFSET>,
            GetHapticInfo: GetHapticInfo::<Identity, OFFSET>,
            IsConnected: IsConnected::<Identity, OFFSET>,
            CreateForceFeedbackEffect: CreateForceFeedbackEffect::<Identity, OFFSET>,
            IsForceFeedbackMotorPoweredOn: IsForceFeedbackMotorPoweredOn::<Identity, OFFSET>,
            SetForceFeedbackMotorGain: SetForceFeedbackMotorGain::<Identity, OFFSET>,
            SetRumbleState: SetRumbleState::<Identity, OFFSET>,
            DirectInputEscape: DirectInputEscape::<Identity, OFFSET>,
            CreateInputMapper: CreateInputMapper::<Identity, OFFSET>,
            GetExtraAxisCount: GetExtraAxisCount::<Identity, OFFSET>,
            GetExtraButtonCount: GetExtraButtonCount::<Identity, OFFSET>,
            GetExtraAxisIndexes: GetExtraAxisIndexes::<Identity, OFFSET>,
            GetExtraButtonIndexes: GetExtraButtonIndexes::<Identity, OFFSET>,
            CreateRawDeviceReport: CreateRawDeviceReport::<Identity, OFFSET>,
            GetRawDeviceFeature: GetRawDeviceFeature::<Identity, OFFSET>,
            SetRawDeviceFeature: SetRawDeviceFeature::<Identity, OFFSET>,
            SendRawDeviceOutput: SendRawDeviceOutput::<Identity, OFFSET>,
            SetLightingColor: SetLightingColor::<Identity, OFFSET>,
            ApplyLightingColors: ApplyLightingColors::<Identity, OFFSET>,
            ApplyLightingSystemDefault: ApplyLightingSystemDefault::<Identity, OFFSET>,
            GetTouchSurfaceCount: GetTouchSurfaceCount::<Identity, OFFSET>,
            GetTouchSurfaceInfo: GetTouchSurfaceInfo::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputDevice as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputDevice {}
windows_core::imp::define_interface!(
    IGameInputDispatcher,
    IGameInputDispatcher_Vtbl,
    0x415eed2e_98cb_42c2_8f28_b94601074e31
);
windows_core::imp::interface_hierarchy!(IGameInputDispatcher, windows_core::IUnknown);
impl IGameInputDispatcher {
    pub unsafe fn Dispatch(&self, quotainmicroseconds: u64) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).Dispatch)(
                windows_core::Interface::as_raw(self),
                quotainmicroseconds,
            )
        }
    }
    pub unsafe fn OpenWaitHandle(&self) -> windows_core::Result<HANDLE> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).OpenWaitHandle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IGameInputDispatcher_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Dispatch: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> bool,
    pub OpenWaitHandle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut HANDLE) -> windows_core::HRESULT,
}
unsafe impl Send for IGameInputDispatcher {}
unsafe impl Sync for IGameInputDispatcher {}
pub trait IGameInputDispatcher_Impl: windows_core::IUnknownImpl {
    fn Dispatch(&self, quotainmicroseconds: u64) -> bool;
    fn OpenWaitHandle(&self) -> windows_core::Result<HANDLE>;
}
impl IGameInputDispatcher_Vtbl {
    pub const fn new<Identity: IGameInputDispatcher_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Dispatch<
            Identity: IGameInputDispatcher_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            quotainmicroseconds: u64,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputDispatcher_Impl::Dispatch(
                    this,
                    core::mem::transmute_copy(&quotainmicroseconds),
                )
            }
        }
        unsafe extern "system" fn OpenWaitHandle<
            Identity: IGameInputDispatcher_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            waithandle: *mut HANDLE,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IGameInputDispatcher_Impl::OpenWaitHandle(this) {
                    Ok(ok__) => {
                        waithandle.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Dispatch: Dispatch::<Identity, OFFSET>,
            OpenWaitHandle: OpenWaitHandle::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputDispatcher as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputDispatcher {}
windows_core::imp::define_interface!(
    IGameInputForceFeedbackEffect,
    IGameInputForceFeedbackEffect_Vtbl,
    0xbb3500c0_4b43_44ca_81a2_382fc63ae0e9
);
windows_core::imp::interface_hierarchy!(IGameInputForceFeedbackEffect, windows_core::IUnknown);
impl IGameInputForceFeedbackEffect {
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
    pub unsafe fn GetMotorIndex(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetMotorIndex)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetGain(&self) -> f32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetGain)(windows_core::Interface::as_raw(self))
        }
    }
    pub unsafe fn SetGain(&self, gain: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetGain)(
                windows_core::Interface::as_raw(self),
                gain,
            );
        }
    }
    pub unsafe fn GetParams(&self, params: *mut GameInputForceFeedbackParams) {
        unsafe {
            (windows_core::Interface::vtable(self).GetParams)(
                windows_core::Interface::as_raw(self),
                params as _,
            );
        }
    }
    pub unsafe fn SetParams(&self, params: *const GameInputForceFeedbackParams) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).SetParams)(
                windows_core::Interface::as_raw(self),
                params,
            )
        }
    }
    pub unsafe fn GetState(&self) -> GameInputFeedbackEffectState {
        unsafe {
            (windows_core::Interface::vtable(self).GetState)(windows_core::Interface::as_raw(self))
        }
    }
    pub unsafe fn SetState(&self, state: GameInputFeedbackEffectState) {
        unsafe {
            (windows_core::Interface::vtable(self).SetState)(
                windows_core::Interface::as_raw(self),
                state,
            );
        }
    }
}
#[repr(C)]
pub struct IGameInputForceFeedbackEffect_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetDevice: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub GetMotorIndex: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetGain: unsafe extern "system" fn(*mut core::ffi::c_void) -> f32,
    pub SetGain: unsafe extern "system" fn(*mut core::ffi::c_void, f32),
    pub GetParams:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputForceFeedbackParams),
    pub SetParams: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const GameInputForceFeedbackParams,
    ) -> bool,
    pub GetState: unsafe extern "system" fn(*mut core::ffi::c_void) -> GameInputFeedbackEffectState,
    pub SetState: unsafe extern "system" fn(*mut core::ffi::c_void, GameInputFeedbackEffectState),
}
unsafe impl Send for IGameInputForceFeedbackEffect {}
unsafe impl Sync for IGameInputForceFeedbackEffect {}
pub trait IGameInputForceFeedbackEffect_Impl: windows_core::IUnknownImpl {
    fn GetDevice(&self, device: windows_core::OutRef<IGameInputDevice>);
    fn GetMotorIndex(&self) -> u32;
    fn GetGain(&self) -> f32;
    fn SetGain(&self, gain: f32);
    fn GetParams(&self, params: *mut GameInputForceFeedbackParams);
    fn SetParams(&self, params: *const GameInputForceFeedbackParams) -> bool;
    fn GetState(&self) -> GameInputFeedbackEffectState;
    fn SetState(&self, state: GameInputFeedbackEffectState);
}
impl IGameInputForceFeedbackEffect_Vtbl {
    pub const fn new<Identity: IGameInputForceFeedbackEffect_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetDevice<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            device: *mut *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::GetDevice(
                    this,
                    core::mem::transmute_copy(&device),
                );
            }
        }
        unsafe extern "system" fn GetMotorIndex<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::GetMotorIndex(this)
            }
        }
        unsafe extern "system" fn GetGain<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> f32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::GetGain(this)
            }
        }
        unsafe extern "system" fn SetGain<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            gain: f32,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::SetGain(this, core::mem::transmute_copy(&gain));
            }
        }
        unsafe extern "system" fn GetParams<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            params: *mut GameInputForceFeedbackParams,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::GetParams(
                    this,
                    core::mem::transmute_copy(&params),
                );
            }
        }
        unsafe extern "system" fn SetParams<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            params: *const GameInputForceFeedbackParams,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::SetParams(
                    this,
                    core::mem::transmute_copy(&params),
                )
            }
        }
        unsafe extern "system" fn GetState<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputFeedbackEffectState {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::GetState(this)
            }
        }
        unsafe extern "system" fn SetState<
            Identity: IGameInputForceFeedbackEffect_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: GameInputFeedbackEffectState,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputForceFeedbackEffect_Impl::SetState(
                    this,
                    core::mem::transmute_copy(&state),
                );
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetDevice: GetDevice::<Identity, OFFSET>,
            GetMotorIndex: GetMotorIndex::<Identity, OFFSET>,
            GetGain: GetGain::<Identity, OFFSET>,
            SetGain: SetGain::<Identity, OFFSET>,
            GetParams: GetParams::<Identity, OFFSET>,
            SetParams: SetParams::<Identity, OFFSET>,
            GetState: GetState::<Identity, OFFSET>,
            SetState: SetState::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputForceFeedbackEffect as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputForceFeedbackEffect {}
windows_core::imp::define_interface!(
    IGameInputMapper,
    IGameInputMapper_Vtbl,
    0x3c600700_f16c_49ce_9be6_6a2ef752ed5e
);
windows_core::imp::interface_hierarchy!(IGameInputMapper, windows_core::IUnknown);
impl IGameInputMapper {
    pub unsafe fn GetArcadeStickButtonMappingInfo(
        &self,
        buttonelement: GameInputArcadeStickButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetArcadeStickButtonMappingInfo)(
                windows_core::Interface::as_raw(self),
                buttonelement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetFlightStickAxisMappingInfo(
        &self,
        axiselement: GameInputFlightStickAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetFlightStickAxisMappingInfo)(
                windows_core::Interface::as_raw(self),
                axiselement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetFlightStickButtonMappingInfo(
        &self,
        buttonelement: GameInputFlightStickButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetFlightStickButtonMappingInfo)(
                windows_core::Interface::as_raw(self),
                buttonelement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetGamepadAxisMappingInfo(
        &self,
        axiselement: GameInputGamepadAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetGamepadAxisMappingInfo)(
                windows_core::Interface::as_raw(self),
                axiselement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetGamepadButtonMappingInfo(
        &self,
        buttonelement: GameInputGamepadButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetGamepadButtonMappingInfo)(
                windows_core::Interface::as_raw(self),
                buttonelement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetRacingWheelAxisMappingInfo(
        &self,
        axiselement: GameInputRacingWheelAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetRacingWheelAxisMappingInfo)(
                windows_core::Interface::as_raw(self),
                axiselement,
                mapping as _,
            )
        }
    }
    pub unsafe fn GetRacingWheelButtonMappingInfo(
        &self,
        buttonelement: GameInputRacingWheelButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetRacingWheelButtonMappingInfo)(
                windows_core::Interface::as_raw(self),
                buttonelement,
                mapping as _,
            )
        }
    }
}
#[repr(C)]
pub struct IGameInputMapper_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetArcadeStickButtonMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputArcadeStickButtons,
        *mut GameInputButtonMapping,
    ) -> bool,
    pub GetFlightStickAxisMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputFlightStickAxes,
        *mut GameInputAxisMapping,
    ) -> bool,
    pub GetFlightStickButtonMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputFlightStickButtons,
        *mut GameInputButtonMapping,
    ) -> bool,
    pub GetGamepadAxisMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputGamepadAxes,
        *mut GameInputAxisMapping,
    ) -> bool,
    pub GetGamepadButtonMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputGamepadButtons,
        *mut GameInputButtonMapping,
    ) -> bool,
    pub GetRacingWheelAxisMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputRacingWheelAxes,
        *mut GameInputAxisMapping,
    ) -> bool,
    pub GetRacingWheelButtonMappingInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GameInputRacingWheelButtons,
        *mut GameInputButtonMapping,
    ) -> bool,
}
unsafe impl Send for IGameInputMapper {}
unsafe impl Sync for IGameInputMapper {}
pub trait IGameInputMapper_Impl: windows_core::IUnknownImpl {
    fn GetArcadeStickButtonMappingInfo(
        &self,
        buttonelement: GameInputArcadeStickButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool;
    fn GetFlightStickAxisMappingInfo(
        &self,
        axiselement: GameInputFlightStickAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool;
    fn GetFlightStickButtonMappingInfo(
        &self,
        buttonelement: GameInputFlightStickButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool;
    fn GetGamepadAxisMappingInfo(
        &self,
        axiselement: GameInputGamepadAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool;
    fn GetGamepadButtonMappingInfo(
        &self,
        buttonelement: GameInputGamepadButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool;
    fn GetRacingWheelAxisMappingInfo(
        &self,
        axiselement: GameInputRacingWheelAxes,
        mapping: *mut GameInputAxisMapping,
    ) -> bool;
    fn GetRacingWheelButtonMappingInfo(
        &self,
        buttonelement: GameInputRacingWheelButtons,
        mapping: *mut GameInputButtonMapping,
    ) -> bool;
}
impl IGameInputMapper_Vtbl {
    pub const fn new<Identity: IGameInputMapper_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetArcadeStickButtonMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buttonelement: GameInputArcadeStickButtons,
            mapping: *mut GameInputButtonMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetArcadeStickButtonMappingInfo(
                    this,
                    core::mem::transmute_copy(&buttonelement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetFlightStickAxisMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            axiselement: GameInputFlightStickAxes,
            mapping: *mut GameInputAxisMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetFlightStickAxisMappingInfo(
                    this,
                    core::mem::transmute_copy(&axiselement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetFlightStickButtonMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buttonelement: GameInputFlightStickButtons,
            mapping: *mut GameInputButtonMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetFlightStickButtonMappingInfo(
                    this,
                    core::mem::transmute_copy(&buttonelement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetGamepadAxisMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            axiselement: GameInputGamepadAxes,
            mapping: *mut GameInputAxisMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetGamepadAxisMappingInfo(
                    this,
                    core::mem::transmute_copy(&axiselement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetGamepadButtonMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buttonelement: GameInputGamepadButtons,
            mapping: *mut GameInputButtonMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetGamepadButtonMappingInfo(
                    this,
                    core::mem::transmute_copy(&buttonelement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetRacingWheelAxisMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            axiselement: GameInputRacingWheelAxes,
            mapping: *mut GameInputAxisMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetRacingWheelAxisMappingInfo(
                    this,
                    core::mem::transmute_copy(&axiselement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        unsafe extern "system" fn GetRacingWheelButtonMappingInfo<
            Identity: IGameInputMapper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buttonelement: GameInputRacingWheelButtons,
            mapping: *mut GameInputButtonMapping,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputMapper_Impl::GetRacingWheelButtonMappingInfo(
                    this,
                    core::mem::transmute_copy(&buttonelement),
                    core::mem::transmute_copy(&mapping),
                )
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetArcadeStickButtonMappingInfo: GetArcadeStickButtonMappingInfo::<Identity, OFFSET>,
            GetFlightStickAxisMappingInfo: GetFlightStickAxisMappingInfo::<Identity, OFFSET>,
            GetFlightStickButtonMappingInfo: GetFlightStickButtonMappingInfo::<Identity, OFFSET>,
            GetGamepadAxisMappingInfo: GetGamepadAxisMappingInfo::<Identity, OFFSET>,
            GetGamepadButtonMappingInfo: GetGamepadButtonMappingInfo::<Identity, OFFSET>,
            GetRacingWheelAxisMappingInfo: GetRacingWheelAxisMappingInfo::<Identity, OFFSET>,
            GetRacingWheelButtonMappingInfo: GetRacingWheelButtonMappingInfo::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputMapper as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputMapper {}
windows_core::imp::define_interface!(
    IGameInputRawDeviceReport,
    IGameInputRawDeviceReport_Vtbl,
    0xe35550a9_8507_404b_823c_2035958a6936
);
windows_core::imp::interface_hierarchy!(IGameInputRawDeviceReport, windows_core::IUnknown);
impl IGameInputRawDeviceReport {
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
    pub unsafe fn GetReportInfo(&self) -> GameInputRawDeviceReportInfo {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetReportInfo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub unsafe fn GetRawDataSize(&self) -> usize {
        unsafe {
            (windows_core::Interface::vtable(self).GetRawDataSize)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub unsafe fn GetRawData(&self, buffersize: usize, buffer: *mut u8) -> usize {
        unsafe {
            (windows_core::Interface::vtable(self).GetRawData)(
                windows_core::Interface::as_raw(self),
                buffersize,
                buffer as _,
            )
        }
    }
    pub unsafe fn SetRawData(&self, buffer: &[u8]) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).SetRawData)(
                windows_core::Interface::as_raw(self),
                buffer.len().try_into().unwrap(),
                buffer.as_ptr(),
            )
        }
    }
}
#[repr(C)]
pub struct IGameInputRawDeviceReport_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetDevice: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub GetReportInfo:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputRawDeviceReportInfo),
    pub GetRawDataSize: unsafe extern "system" fn(*mut core::ffi::c_void) -> usize,
    pub GetRawData: unsafe extern "system" fn(*mut core::ffi::c_void, usize, *mut u8) -> usize,
    pub SetRawData: unsafe extern "system" fn(*mut core::ffi::c_void, usize, *const u8) -> bool,
}
unsafe impl Send for IGameInputRawDeviceReport {}
unsafe impl Sync for IGameInputRawDeviceReport {}
pub trait IGameInputRawDeviceReport_Impl: windows_core::IUnknownImpl {
    fn GetDevice(&self, device: windows_core::OutRef<IGameInputDevice>);
    fn GetReportInfo(&self, reportinfo: *mut GameInputRawDeviceReportInfo);
    fn GetRawDataSize(&self) -> usize;
    fn GetRawData(&self, buffersize: usize, buffer: *mut u8) -> usize;
    fn SetRawData(&self, buffersize: usize, buffer: *const u8) -> bool;
}
impl IGameInputRawDeviceReport_Vtbl {
    pub const fn new<Identity: IGameInputRawDeviceReport_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetDevice<
            Identity: IGameInputRawDeviceReport_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            device: *mut *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputRawDeviceReport_Impl::GetDevice(this, core::mem::transmute_copy(&device));
            }
        }
        unsafe extern "system" fn GetReportInfo<
            Identity: IGameInputRawDeviceReport_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            reportinfo: *mut GameInputRawDeviceReportInfo,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputRawDeviceReport_Impl::GetReportInfo(
                    this,
                    core::mem::transmute_copy(&reportinfo),
                );
            }
        }
        unsafe extern "system" fn GetRawDataSize<
            Identity: IGameInputRawDeviceReport_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> usize {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputRawDeviceReport_Impl::GetRawDataSize(this)
            }
        }
        unsafe extern "system" fn GetRawData<
            Identity: IGameInputRawDeviceReport_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buffersize: usize,
            buffer: *mut u8,
        ) -> usize {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputRawDeviceReport_Impl::GetRawData(
                    this,
                    core::mem::transmute_copy(&buffersize),
                    core::mem::transmute_copy(&buffer),
                )
            }
        }
        unsafe extern "system" fn SetRawData<
            Identity: IGameInputRawDeviceReport_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buffersize: usize,
            buffer: *const u8,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputRawDeviceReport_Impl::SetRawData(
                    this,
                    core::mem::transmute_copy(&buffersize),
                    core::mem::transmute_copy(&buffer),
                )
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetDevice: GetDevice::<Identity, OFFSET>,
            GetReportInfo: GetReportInfo::<Identity, OFFSET>,
            GetRawDataSize: GetRawDataSize::<Identity, OFFSET>,
            GetRawData: GetRawData::<Identity, OFFSET>,
            SetRawData: SetRawData::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputRawDeviceReport as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputRawDeviceReport {}
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
    pub unsafe fn GetControllerAxisCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerAxisCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerAxisState(&self, statearraycount: u32, statearray: *mut f32) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerAxisState)(
                windows_core::Interface::as_raw(self),
                statearraycount,
                statearray as _,
            )
        }
    }
    pub unsafe fn GetControllerButtonCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerButtonCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerButtonState(
        &self,
        statearraycount: u32,
        statearray: *mut bool,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerButtonState)(
                windows_core::Interface::as_raw(self),
                statearraycount,
                statearray as _,
            )
        }
    }
    pub unsafe fn GetControllerSwitchCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerSwitchCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub unsafe fn GetControllerSwitchState(
        &self,
        statearraycount: u32,
        statearray: *mut GameInputSwitchPosition,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetControllerSwitchState)(
                windows_core::Interface::as_raw(self),
                statearraycount,
                statearray as _,
            )
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
    pub unsafe fn GetSensorsState(&self, state: *mut GameInputSensorsState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetSensorsState)(
                windows_core::Interface::as_raw(self),
                state as _,
            )
        }
    }
    pub unsafe fn GetArcadeStickState(&self, state: *mut GameInputArcadeStickState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetArcadeStickState)(
                windows_core::Interface::as_raw(self),
                state as _,
            )
        }
    }
    pub unsafe fn GetFlightStickState(&self, state: *mut GameInputFlightStickState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetFlightStickState)(
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
    pub unsafe fn GetRacingWheelState(&self, state: *mut GameInputRacingWheelState) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetRacingWheelState)(
                windows_core::Interface::as_raw(self),
                state as _,
            )
        }
    }
    pub unsafe fn GetRawReport(&self, report: *mut Option<IGameInputRawDeviceReport>) -> bool {
        unsafe {
            (windows_core::Interface::vtable(self).GetRawReport)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(report),
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
    pub unsafe fn GetTouchContactCount(&self, surfaceindex: u32) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetTouchContactCount)(
                windows_core::Interface::as_raw(self),
                surfaceindex,
            )
        }
    }
    pub unsafe fn GetTouchSurfaceState(
        &self,
        surfaceindex: u32,
        contactarraycount: u32,
        contactarray: Option<*mut GameInputTouchContact>,
    ) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetTouchSurfaceState)(
                windows_core::Interface::as_raw(self),
                surfaceindex,
                contactarraycount,
                contactarray.unwrap_or(core::mem::zeroed()) as _,
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
    pub GetControllerAxisCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerAxisState:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut f32) -> u32,
    pub GetControllerButtonCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerButtonState:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut bool) -> u32,
    pub GetControllerSwitchCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetControllerSwitchState:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut GameInputSwitchPosition) -> u32,
    pub GetKeyCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetKeyState:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut GameInputKeyState) -> u32,
    pub GetMouseState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputMouseState) -> bool,
    pub GetSensorsState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputSensorsState) -> bool,
    pub GetArcadeStickState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputArcadeStickState) -> bool,
    pub GetFlightStickState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputFlightStickState) -> bool,
    pub GetGamepadState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputGamepadState) -> bool,
    pub GetRacingWheelState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputRacingWheelState) -> bool,
    pub GetRawReport:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> bool,
    pub GetSystemButtonState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut GameInputSystemButtons) -> bool,
    pub GetTouchContactCount: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> u32,
    pub GetTouchSurfaceState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        *mut GameInputTouchContact,
    ) -> u32,
}
unsafe impl Send for IGameInputReading {}
unsafe impl Sync for IGameInputReading {}
pub trait IGameInputReading_Impl: windows_core::IUnknownImpl {
    fn GetInputKind(&self) -> GameInputKind;
    fn GetTimestamp(&self) -> u64;
    fn GetDevice(&self, device: windows_core::OutRef<IGameInputDevice>);
    fn GetControllerAxisCount(&self) -> u32;
    fn GetControllerAxisState(&self, statearraycount: u32, statearray: *mut f32) -> u32;
    fn GetControllerButtonCount(&self) -> u32;
    fn GetControllerButtonState(&self, statearraycount: u32, statearray: *mut bool) -> u32;
    fn GetControllerSwitchCount(&self) -> u32;
    fn GetControllerSwitchState(
        &self,
        statearraycount: u32,
        statearray: *mut GameInputSwitchPosition,
    ) -> u32;
    fn GetKeyCount(&self) -> u32;
    fn GetKeyState(&self, statearraycount: u32, statearray: *mut GameInputKeyState) -> u32;
    fn GetMouseState(&self, state: *mut GameInputMouseState) -> bool;
    fn GetSensorsState(&self, state: *mut GameInputSensorsState) -> bool;
    fn GetArcadeStickState(&self, state: *mut GameInputArcadeStickState) -> bool;
    fn GetFlightStickState(&self, state: *mut GameInputFlightStickState) -> bool;
    fn GetGamepadState(&self, state: *mut GameInputGamepadState) -> bool;
    fn GetRacingWheelState(&self, state: *mut GameInputRacingWheelState) -> bool;
    fn GetRawReport(&self, report: windows_core::OutRef<IGameInputRawDeviceReport>) -> bool;
    fn GetSystemButtonState(&self, buttons: *mut GameInputSystemButtons) -> bool;
    fn GetTouchContactCount(&self, surfaceindex: u32) -> u32;
    fn GetTouchSurfaceState(
        &self,
        surfaceindex: u32,
        contactarraycount: u32,
        contactarray: *mut GameInputTouchContact,
    ) -> u32;
}
impl IGameInputReading_Vtbl {
    pub const fn new<Identity: IGameInputReading_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetInputKind<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> GameInputKind {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetInputKind(this)
            }
        }
        unsafe extern "system" fn GetTimestamp<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u64 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetTimestamp(this)
            }
        }
        unsafe extern "system" fn GetDevice<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            device: *mut *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetDevice(this, core::mem::transmute_copy(&device));
            }
        }
        unsafe extern "system" fn GetControllerAxisCount<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerAxisCount(this)
            }
        }
        unsafe extern "system" fn GetControllerAxisState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            statearraycount: u32,
            statearray: *mut f32,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerAxisState(
                    this,
                    core::mem::transmute_copy(&statearraycount),
                    core::mem::transmute_copy(&statearray),
                )
            }
        }
        unsafe extern "system" fn GetControllerButtonCount<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerButtonCount(this)
            }
        }
        unsafe extern "system" fn GetControllerButtonState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            statearraycount: u32,
            statearray: *mut bool,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerButtonState(
                    this,
                    core::mem::transmute_copy(&statearraycount),
                    core::mem::transmute_copy(&statearray),
                )
            }
        }
        unsafe extern "system" fn GetControllerSwitchCount<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerSwitchCount(this)
            }
        }
        unsafe extern "system" fn GetControllerSwitchState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            statearraycount: u32,
            statearray: *mut GameInputSwitchPosition,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetControllerSwitchState(
                    this,
                    core::mem::transmute_copy(&statearraycount),
                    core::mem::transmute_copy(&statearray),
                )
            }
        }
        unsafe extern "system" fn GetKeyCount<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetKeyCount(this)
            }
        }
        unsafe extern "system" fn GetKeyState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            statearraycount: u32,
            statearray: *mut GameInputKeyState,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetKeyState(
                    this,
                    core::mem::transmute_copy(&statearraycount),
                    core::mem::transmute_copy(&statearray),
                )
            }
        }
        unsafe extern "system" fn GetMouseState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputMouseState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetMouseState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetSensorsState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputSensorsState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetSensorsState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetArcadeStickState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputArcadeStickState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetArcadeStickState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetFlightStickState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputFlightStickState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetFlightStickState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetGamepadState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputGamepadState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetGamepadState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetRacingWheelState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut GameInputRacingWheelState,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetRacingWheelState(this, core::mem::transmute_copy(&state))
            }
        }
        unsafe extern "system" fn GetRawReport<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            report: *mut *mut core::ffi::c_void,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetRawReport(this, core::mem::transmute_copy(&report))
            }
        }
        unsafe extern "system" fn GetSystemButtonState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            buttons: *mut GameInputSystemButtons,
        ) -> bool {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetSystemButtonState(
                    this,
                    core::mem::transmute_copy(&buttons),
                )
            }
        }
        unsafe extern "system" fn GetTouchContactCount<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            surfaceindex: u32,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetTouchContactCount(
                    this,
                    core::mem::transmute_copy(&surfaceindex),
                )
            }
        }
        unsafe extern "system" fn GetTouchSurfaceState<
            Identity: IGameInputReading_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            surfaceindex: u32,
            contactarraycount: u32,
            contactarray: *mut GameInputTouchContact,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGameInputReading_Impl::GetTouchSurfaceState(
                    this,
                    core::mem::transmute_copy(&surfaceindex),
                    core::mem::transmute_copy(&contactarraycount),
                    core::mem::transmute_copy(&contactarray),
                )
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetInputKind: GetInputKind::<Identity, OFFSET>,
            GetTimestamp: GetTimestamp::<Identity, OFFSET>,
            GetDevice: GetDevice::<Identity, OFFSET>,
            GetControllerAxisCount: GetControllerAxisCount::<Identity, OFFSET>,
            GetControllerAxisState: GetControllerAxisState::<Identity, OFFSET>,
            GetControllerButtonCount: GetControllerButtonCount::<Identity, OFFSET>,
            GetControllerButtonState: GetControllerButtonState::<Identity, OFFSET>,
            GetControllerSwitchCount: GetControllerSwitchCount::<Identity, OFFSET>,
            GetControllerSwitchState: GetControllerSwitchState::<Identity, OFFSET>,
            GetKeyCount: GetKeyCount::<Identity, OFFSET>,
            GetKeyState: GetKeyState::<Identity, OFFSET>,
            GetMouseState: GetMouseState::<Identity, OFFSET>,
            GetSensorsState: GetSensorsState::<Identity, OFFSET>,
            GetArcadeStickState: GetArcadeStickState::<Identity, OFFSET>,
            GetFlightStickState: GetFlightStickState::<Identity, OFFSET>,
            GetGamepadState: GetGamepadState::<Identity, OFFSET>,
            GetRacingWheelState: GetRacingWheelState::<Identity, OFFSET>,
            GetRawReport: GetRawReport::<Identity, OFFSET>,
            GetSystemButtonState: GetSystemButtonState::<Identity, OFFSET>,
            GetTouchContactCount: GetTouchContactCount::<Identity, OFFSET>,
            GetTouchSurfaceState: GetTouchSurfaceState::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGameInputReading as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGameInputReading {}
