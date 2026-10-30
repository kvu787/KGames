// Reads gamepads, keyboards and mice with GameInput, and keeps what the
// panels show: the latest state of each, the connected devices, and a log of
// input events.
//
// GameInput doesn't report events. It keeps a short history of readings,
// where each reading is a snapshot of one device's whole state with a
// timestamp. This file walks that history every frame with GetNextReading,
// so no reading is missed between frames, and compares each reading with the
// same device's previous one (GetPreviousReading) to say what changed.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::bindings::*;
use crate::game_input::*;
use windows_core::{Interface, Result};

const LOG_CAPACITY: usize = 200;
const TRAIL_LENGTH: usize = 96;
// Virtual-key codes of the demo's own shortcuts.
const KEY_TOGGLE_RUMBLE: u8 = 0x70; // F1
const KEY_CLEAR_LOG: u8 = 0x71; // F2
const KEY_TOGGLE_FULLSCREEN: u8 = 0x7A; // F11

// The device callback runs on one of GameInput's threads. It only queues the
// event, and the main thread handles it in Input::update.
static DEVICE_EVENTS: Mutex<Vec<(GameInputDeviceEvent, IGameInputDevice)>> = Mutex::new(Vec::new());

unsafe extern "system" fn on_device_event(
    _token: GameInputCallbackToken,
    _context: *mut core::ffi::c_void,
    event: GameInputDeviceEvent,
    device: *mut core::ffi::c_void,
) {
    if let Some(device) = unsafe { IGameInputDevice::from_raw_borrowed(&device) } {
        DEVICE_EVENTS.lock().unwrap().push((event, device.clone()));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Gamepad,
    Keyboard,
    Mouse,
    Device,
    Demo,
}

impl Source {
    pub fn name(self) -> &'static str {
        match self {
            Source::Gamepad => "Gamepad",
            Source::Keyboard => "Keyboard",
            Source::Mouse => "Mouse",
            Source::Device => "Device",
            Source::Demo => "Demo",
        }
    }
}

const READING_KINDS: [(Source, GameInputKind); 3] = [
    (Source::Gamepad, GameInputKindGamepad),
    (Source::Keyboard, GameInputKindKeyboard),
    (Source::Mouse, GameInputKindMouse),
];

pub struct DeviceInfo {
    pub device: IGameInputDevice,
    pub label: String,
    pub family: GameInputDeviceFamily,
    pub supported_input: GameInputKind,
    pub rumble_motors: GameInputRumbleMotors,
    pub xinput_index: Option<u32>,
    pub mouse_sample_rate: Option<u32>,
    pub gamepad_layout: Option<GameInputGamepadButtons>,
}

impl DeviceInfo {
    unsafe fn new(device: &IGameInputDevice) -> Self {
        unsafe {
            let size = device.GetDisplayName(&mut []);
            let mut name = vec![0u8; size as usize];
            device.GetDisplayName(&mut name);
            let name = String::from_utf8_lossy(&name)
                .trim_end_matches('\0')
                .to_string();
            let name = if name.is_empty() {
                "Unnamed device".to_string()
            } else {
                name
            };

            let mut xinput_index = 0;
            let mut mouse_info = GameInputMouseInfo::default();
            let mut gamepad_info = GameInputGamepadInfo::default();
            Self {
                device: device.clone(),
                label: format!(
                    "{name} ({:04X}:{:04X})",
                    device.GetVendorId(),
                    device.GetProductId()
                ),
                family: device.GetDeviceFamily(),
                supported_input: device.GetSupportedInput(),
                rumble_motors: device.GetSupportedRumbleMotors(),
                xinput_index: device
                    .GetXInputDeviceIndex(&mut xinput_index)
                    .then_some(xinput_index),
                mouse_sample_rate: device
                    .GetMouseInfo(&mut mouse_info)
                    .then_some(mouse_info.sampleRate)
                    .filter(|&rate| rate > 0),
                gamepad_layout: device
                    .GetGamepadInfo(&mut gamepad_info)
                    .then_some(gamepad_info.supportedLayout),
            }
        }
    }

    // Matching on GameInput's constants needs their native names allowed.
    #[allow(non_upper_case_globals)]
    pub fn family_name(&self) -> &'static str {
        match self.family {
            GameInputFamilyVirtual => "virtual",
            GameInputFamilyXboxOne => "Xbox One",
            GameInputFamilyXbox360 => "Xbox 360",
            GameInputFamilyHid => "HID",
            GameInputFamilyI8042 => "i8042 (PS/2)",
            GameInputFamilyAggregate => "aggregate",
            _ => "unknown",
        }
    }

    pub fn kinds(&self) -> String {
        kind_names(self.supported_input)
    }
}

pub struct LogEntry {
    pub time: f64,
    pub source: Source,
    pub text: String,
    // How many readings were merged into this entry. See Input::log.
    pub count: u32,
    merge_key: Option<(Source, usize)>,
}

// The most recent reading that changed something, described in full.
pub struct LatestEvent {
    pub number: u64,
    pub time: f64,
    pub source: Source,
    pub device: String,
    pub kinds: String,
    pub changes: Vec<String>,
    pub state: Vec<String>,
    // GameInput's own timestamp, in microseconds.
    pub timestamp: u64,
    // Since the same device's previous reading. None when there was no
    // earlier reading to compare with, so the changes are from an idle state.
    pub interval: Option<f64>,
}

// Counts readings per second.
#[derive(Default)]
pub struct RateCounter {
    window_start: f64,
    count: u32,
    pub per_second: u32,
}

impl RateCounter {
    fn add(&mut self, now: f64) {
        self.tick(now);
        self.count += 1;
    }

    fn tick(&mut self, now: f64) {
        if now - self.window_start >= 1.0 {
            // A second with no readings at all reads as 0, not the last rate.
            self.per_second = if now - self.window_start >= 2.0 {
                0
            } else {
                self.count
            };
            self.count = 0;
            self.window_start = now;
        }
    }
}

#[derive(Default)]
pub struct GamepadView {
    pub device: Option<IGameInputDevice>,
    pub layout: Option<GameInputGamepadButtons>,
    pub rumble_motors: GameInputRumbleMotors,
    pub state: GameInputGamepadState,
    pub left_trail: VecDeque<[f32; 2]>,
    pub right_trail: VecDeque<[f32; 2]>,
    pub rate: RateCounter,
}

// How far a gamepad's left and right sticks last were from their centers,
// and the rumble last sent to it.
struct Rumble {
    device: IGameInputDevice,
    sticks: [f32; 2],
    sent: GameInputRumbleParams,
}

// Which gamepad the demo listens to, if any, chosen in the gamepad panel's
// select box. The others are ignored, along with any keyboard or mouse input
// they have: a GameSir in Xbox 360 mode is one device with a gamepad, a
// keyboard and a mouse.
#[derive(Clone, PartialEq)]
pub enum Listen {
    // Kept by its device, so that it's listened to again if it reconnects.
    Gamepad(IGameInputDevice),
    NoGamepad,
}

impl Listen {
    unsafe fn includes(&self, device: &IGameInputDevice) -> bool {
        match self {
            Listen::Gamepad(chosen) => chosen == device || !unsafe { is_gamepad(device) },
            Listen::NoGamepad => !unsafe { is_gamepad(device) },
        }
    }
}

unsafe fn is_gamepad(device: &IGameInputDevice) -> bool {
    unsafe { device.GetSupportedInput() & GameInputKindGamepad != 0 }
}

#[derive(Default)]
pub struct KeyboardView {
    pub label: String,
    pub keys: Vec<GameInputKeyState>,
    pub rate: RateCounter,
}

#[derive(Default)]
pub struct MouseView {
    pub label: String,
    pub state: GameInputMouseState,
    pub delta: [i64; 2],
    // Relative motion added up and kept inside -1..1, for the motion pad.
    pub pad: [f32; 2],
    pub pad_trail: VecDeque<[f32; 2]>,
    pub sample_rate: Option<u32>,
    pub rate: RateCounter,
}

pub struct Input {
    game_input: IGameInput,
    pub runtime: Runtime,
    callback_token: GameInputCallbackToken,
    // GameInput timestamps are in microseconds. Shown relative to this.
    start_timestamp: u64,
    // The last reading handled for each of READING_KINDS.
    last_readings: [Option<IGameInputReading>; 3],
    pub now: f64,
    pub devices: Vec<DeviceInfo>,
    pub gamepad: GamepadView,
    pub keyboard: KeyboardView,
    pub mouse: MouseView,
    pub latest: Option<LatestEvent>,
    pub log: VecDeque<LogEntry>,
    pub reading_count: u64,
    pub rumble_enabled: bool,
    // The rumble test's inner dead zone for both sticks, as a fraction of
    // full deflection, from the settings. GameInput reports raw stick values,
    // so applying a dead zone is up to the game. The demo applies this one
    // only to the rumble test. Everything else, from the panels to the log,
    // shows the raw values, and the stick graphs draw this as a ring.
    pub rumble_dead_zone: f32,
    // One for each gamepad listened to that has sent a reading.
    rumbles: Vec<Rumble>,
    // Changed with pick_listen, or by listen_by_default until then.
    pub listen: Listen,
    // Whether listen was picked in the select box.
    pub listen_picked: bool,
    // Set when the fullscreen shortcut is pressed. The main loop owns the
    // window, so it does the switch and clears this.
    pub fullscreen_toggled: bool,
    // Virtual keys of the demo's shortcuts, pressed in the reading being handled.
    shortcuts: Vec<u8>,
    // Whether the window had focus at the last update, so keyboard and mouse
    // input was handled.
    focused: bool,
}

impl Input {
    pub fn new(rumble_dead_zone: f32) -> Result<Self> {
        let (game_input, runtime) = crate::game_input::create()?;
        unsafe {
            let callback_token = game_input.RegisterDeviceCallback(
                None,
                GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse,
                None,
                Some(on_device_event),
            )?;
            let start_timestamp = game_input.GetCurrentTimestamp();
            Ok(Self {
                game_input,
                runtime,
                callback_token,
                start_timestamp,
                last_readings: [None, None, None],
                now: 0.0,
                devices: Vec::new(),
                gamepad: GamepadView::default(),
                keyboard: KeyboardView::default(),
                mouse: MouseView::default(),
                latest: None,
                log: VecDeque::new(),
                reading_count: 0,
                rumble_enabled: false,
                rumble_dead_zone,
                rumbles: Vec::new(),
                listen: Listen::NoGamepad,
                listen_picked: false,
                fullscreen_toggled: false,
                shortcuts: Vec::new(),
                focused: false,
            })
        }
    }

    fn seconds(&self, timestamp: u64) -> f64 {
        timestamp.saturating_sub(self.start_timestamp) as f64 / 1_000_000.0
    }

    // `focused` is whether the window has focus. Keyboard and mouse input is
    // only handled while it does, see handle_reading.
    pub fn update(&mut self, focused: bool) {
        unsafe {
            if self.focused && !focused {
                // Keys and buttons let go of after the window lost focus are
                // never seen, so stop showing them as held.
                self.keyboard.keys.clear();
                self.mouse.state.buttons = 0;
            }
            self.focused = focused;
            self.now = self.seconds(self.game_input.GetCurrentTimestamp());
            let events: Vec<_> = DEVICE_EVENTS.lock().unwrap().drain(..).collect();
            for (event, device) in events {
                self.handle_device_event(event, &device);
            }
            self.listen_by_default();

            // Gather every new reading of each kind, then handle them all in
            // timestamp order, so the log interleaves devices correctly.
            let mut readings = Vec::new();
            for (index, &(source, kind)) in READING_KINDS.iter().enumerate() {
                for reading in self.new_readings(index, kind) {
                    readings.push((reading.GetTimestamp(), source, kind, reading));
                }
            }
            readings.sort_by_key(|(timestamp, ..)| *timestamp);
            for (_, source, kind, reading) in readings {
                self.handle_reading(source, kind, &reading);
            }

            self.gamepad.rate.tick(self.now);
            self.keyboard.rate.tick(self.now);
            self.mouse.rate.tick(self.now);
            self.update_rumble();
        }
    }

    // Readings of one kind since the last call, oldest first.
    unsafe fn new_readings(&mut self, index: usize, kind: GameInputKind) -> Vec<IGameInputReading> {
        let mut readings = Vec::new();
        unsafe {
            let Some(mut last) = self.last_readings[index].clone() else {
                // The first time, start from the newest reading.
                if let Ok(current) = self.game_input.GetCurrentReading(kind, None) {
                    self.last_readings[index] = Some(current.clone());
                    readings.push(current);
                }
                return readings;
            };
            loop {
                match self.game_input.GetNextReading(&last, kind, None) {
                    Ok(next) => {
                        readings.push(next.clone());
                        last = next;
                    }
                    // GameInput keeps readings for only a short time, so after
                    // a pause in input the last reading handled is no longer in
                    // its history. Find the ones after it from the newest end.
                    Err(error) if error.code() == GAMEINPUT_E_REFERENCE_READING_TOO_OLD => {
                        readings.extend(self.readings_after(last.GetTimestamp(), kind));
                        break;
                    }
                    // GAMEINPUT_E_READING_NOT_FOUND: nothing newer yet.
                    Err(_) => break,
                }
            }
            if let Some(newest) = readings.last() {
                self.last_readings[index] = Some(newest.clone());
            }
        }
        readings
    }

    // Readings of one kind newer than a timestamp, oldest first. Walks back
    // from the newest reading with GetPreviousReading.
    unsafe fn readings_after(&self, timestamp: u64, kind: GameInputKind) -> Vec<IGameInputReading> {
        let mut readings = Vec::new();
        unsafe {
            let mut reading = self.game_input.GetCurrentReading(kind, None).ok();
            while let Some(current) = reading {
                if current.GetTimestamp() <= timestamp {
                    break;
                }
                reading = self
                    .game_input
                    .GetPreviousReading(&current, kind, None)
                    .ok();
                readings.push(current);
            }
        }
        readings.reverse();
        readings
    }

    #[allow(non_upper_case_globals)]
    unsafe fn handle_device_event(
        &mut self,
        event: GameInputDeviceEvent,
        device: &IGameInputDevice,
    ) {
        unsafe {
            let existing = self.devices.iter().position(|info| info.device == *device);
            let label = match existing {
                Some(index) => self.devices[index].label.clone(),
                None => DeviceInfo::new(device).label,
            };
            let text = match event {
                GameInputDeviceEventConnected => {
                    let info = DeviceInfo::new(device);
                    let text = format!("Connected: {} [{}]", info.label, info.kinds());
                    match existing {
                        Some(index) => self.devices[index] = info,
                        None => self.devices.push(info),
                    }
                    // Gamepads first, then keyboards, then mice.
                    self.devices.sort_by_key(|info| {
                        let kinds = info.supported_input;
                        (
                            kinds & GameInputKindGamepad == 0,
                            kinds & GameInputKindKeyboard == 0,
                        )
                    });
                    text
                }
                GameInputDeviceEventDisconnected => {
                    if let Some(index) = existing {
                        self.devices.remove(index);
                    }
                    if self.gamepad.device.as_ref() == Some(device) {
                        self.gamepad = GamepadView::default();
                    }
                    self.rumbles.retain(|rumble| rumble.device != *device);
                    format!("Disconnected: {label}")
                }
                GameInputDeviceEventKeyboardLayoutUpdated => {
                    format!("Keyboard layout changed: {label}")
                }
                GameInputDeviceEventHapticInfoUpdated => format!("Haptic info changed: {label}"),
                GameInputDeviceEventXInputIndexUpdated => {
                    if let Some(index) = existing {
                        self.devices[index] = DeviceInfo::new(device);
                    }
                    format!("XInput index changed: {label}")
                }
                other => format!("Device event {other}: {label}"),
            };
            self.log(self.now, Source::Device, text, None);
        }
    }

    // Shows another gamepad in the panel, starting from an idle state.
    fn view_gamepad(&mut self, device: Option<&IGameInputDevice>) {
        let info = self
            .devices
            .iter()
            .find(|info| Some(&info.device) == device);
        self.gamepad = GamepadView {
            device: device.cloned(),
            layout: info.and_then(|info| info.gamepad_layout),
            rumble_motors: info.map_or(0, |info| info.rumble_motors),
            ..Default::default()
        };
    }

    // The connected gamepads, which the select box offers besides none.
    pub fn gamepads(&self) -> impl Iterator<Item = &DeviceInfo> {
        self.devices
            .iter()
            .filter(|info| info.supported_input & GameInputKindGamepad != 0)
    }

    // Whether the demo handles the device's readings.
    pub fn listens_to(&self, info: &DeviceInfo) -> bool {
        unsafe { self.listen.includes(&info.device) }
    }

    // For a choice clicked in the select box. The demo keeps to it from then on.
    pub fn pick_listen(&mut self, listen: Listen) {
        self.listen_picked = true;
        self.set_listen(listen);
    }

    // Until a choice is picked, the demo listens to a connected gamepad if
    // there is one: the one it has while that's connected, or else the first.
    fn listen_by_default(&mut self) {
        if self.listen_picked {
            return;
        }
        if let Listen::Gamepad(device) = &self.listen
            && self.gamepads().any(|info| info.device == *device)
        {
            return;
        }
        let first = self.gamepads().next().map(|info| info.device.clone());
        if let Some(device) = first {
            self.set_listen(Listen::Gamepad(device));
        }
    }

    fn set_listen(&mut self, listen: Listen) {
        if listen == self.listen {
            return;
        }
        self.listen = listen;
        unsafe {
            // Gamepads no longer listened to stop rumbling.
            let listen = &self.listen;
            self.rumbles.retain(|rumble| {
                let keep = listen.includes(&rumble.device);
                if !keep {
                    rumble.device.SetRumbleState(None);
                }
                keep
            });
            let text = match self.listen.clone() {
                Listen::Gamepad(device) => {
                    let label = self.label_of(Some(&device));
                    if self.gamepad.device.as_ref() != Some(&device) {
                        self.view_gamepad(Some(&device));
                        // Its current state, if GameInput still has a reading of it.
                        if let Ok(reading) = self
                            .game_input
                            .GetCurrentReading(GameInputKindGamepad, Some(&device))
                        {
                            reading.GetGamepadState(&mut self.gamepad.state);
                        }
                    }
                    format!("Listening to {label} only")
                }
                Listen::NoGamepad => {
                    self.gamepad = GamepadView::default();
                    "Ignoring all gamepads".to_string()
                }
            };
            self.log(self.now, Source::Demo, text, None);
        }
    }

    pub fn label_of(&self, device: Option<&IGameInputDevice>) -> String {
        let Some(device) = device else {
            return "Unknown device".to_string();
        };
        match self.devices.iter().find(|info| info.device == *device) {
            Some(info) => info.label.clone(),
            None => unsafe { DeviceInfo::new(device).label },
        }
    }

    unsafe fn handle_reading(
        &mut self,
        source: Source,
        kind: GameInputKind,
        reading: &IGameInputReading,
    ) {
        unsafe {
            // Keyboard and mouse input is meant for the window in focus.
            // GameInput 4.0.306, loaded from the exe's folder, gives it to
            // the demo without focus too, even with EnableBackgroundInput(false),
            // so the demo drops it itself. Otherwise it would show and log what
            // is typed into other windows, and run its shortcuts for them.
            if !self.focused && matches!(source, Source::Keyboard | Source::Mouse) {
                return;
            }
            let device = reading.GetDevice();
            if let Some(device) = &device
                && !self.listen.includes(device)
            {
                return;
            }
            self.reading_count += 1;
            let time = self.seconds(reading.GetTimestamp());
            // The same device's reading before this one. There's none for the
            // first reading of a device, or when it's no longer in the history.
            let previous = device.as_ref().and_then(|device| {
                self.game_input
                    .GetPreviousReading(reading, kind, Some(device))
                    .ok()
            });
            let label = self.label_of(device.as_ref());

            let (changes, state, motion_only) = match source {
                Source::Gamepad => {
                    self.handle_gamepad(time, reading, previous.as_ref(), device.as_ref())
                }
                Source::Keyboard => self.handle_keyboard(time, reading, previous.as_ref(), &label),
                _ => self.handle_mouse(time, reading, previous.as_ref(), device.as_ref(), &label),
            };
            // A reading can repeat the previous state, or change only
            // another kind of input on a device with several kinds. Those are
            // counted but not reported.
            if changes.is_empty() {
                return;
            }

            // Analog motion arrives up to thousands of times a second, so a run
            // of motion-only readings from one device shares one log entry.
            let merge_key = motion_only.then(|| {
                (
                    source,
                    device.as_ref().map_or(0, |device| device.as_raw() as usize),
                )
            });
            self.log(
                time,
                source,
                format!("{label}: {}", changes.join(", ")),
                merge_key,
            );

            self.latest = Some(LatestEvent {
                number: self.reading_count,
                time,
                source,
                device: label,
                kinds: kind_names(reading.GetInputKind()),
                changes,
                state,
                timestamp: reading.GetTimestamp(),
                interval: previous.map(|previous| {
                    reading
                        .GetTimestamp()
                        .saturating_sub(previous.GetTimestamp()) as f64
                        / 1000.0
                }),
            });

            // After logging the key press, so the log shows cause before effect.
            for key in std::mem::take(&mut self.shortcuts) {
                match key {
                    KEY_TOGGLE_RUMBLE => self.toggle_rumble(),
                    KEY_CLEAR_LOG => self.log.clear(),
                    KEY_TOGGLE_FULLSCREEN => self.fullscreen_toggled = true,
                    _ => {}
                }
            }
        }
    }

    unsafe fn handle_gamepad(
        &mut self,
        time: f64,
        reading: &IGameInputReading,
        previous: Option<&IGameInputReading>,
        device: Option<&IGameInputDevice>,
    ) -> (Vec<String>, Vec<String>, bool) {
        unsafe {
            let mut state = GameInputGamepadState::default();
            reading.GetGamepadState(&mut state);
            let mut old = GameInputGamepadState::default();
            if let Some(previous) = previous {
                previous.GetGamepadState(&mut old);
            }
            self.gamepad.rate.add(time);
            if let Some(device) = device {
                let sticks = [
                    deflection(state.leftThumbstickX, state.leftThumbstickY),
                    deflection(state.rightThumbstickX, state.rightThumbstickY),
                ];
                match self
                    .rumbles
                    .iter_mut()
                    .find(|rumble| rumble.device == *device)
                {
                    Some(rumble) => rumble.sticks = sticks,
                    None => self.rumbles.push(Rumble {
                        device: device.clone(),
                        sticks,
                        sent: GameInputRumbleParams::default(),
                    }),
                }
            }

            // The panel shows the gamepad listened to. Its view is reset when
            // it disconnects, so this sets it up again after it reconnects.
            if self.gamepad.device.as_ref() != device {
                self.view_gamepad(device);
            }
            let view = &mut self.gamepad;
            view.state = state;
            push_trail(
                &mut view.left_trail,
                [state.leftThumbstickX, state.leftThumbstickY],
            );
            push_trail(
                &mut view.right_trail,
                [state.rightThumbstickX, state.rightThumbstickY],
            );

            let mut changes = Vec::new();
            for &(button, name) in GAMEPAD_BUTTONS {
                let (now, before) = (state.buttons & button != 0, old.buttons & button != 0);
                if now && !before {
                    changes.push(format!("{name} pressed"));
                } else if before && !now {
                    changes.push(format!("{name} released"));
                }
            }
            let motion_only = changes.is_empty();
            if (state.leftThumbstickX, state.leftThumbstickY)
                != (old.leftThumbstickX, old.leftThumbstickY)
            {
                changes.push(format!(
                    "left stick {}",
                    pair(state.leftThumbstickX, state.leftThumbstickY)
                ));
            }
            if (state.rightThumbstickX, state.rightThumbstickY)
                != (old.rightThumbstickX, old.rightThumbstickY)
            {
                changes.push(format!(
                    "right stick {}",
                    pair(state.rightThumbstickX, state.rightThumbstickY)
                ));
            }
            if state.leftTrigger != old.leftTrigger {
                changes.push(format!("left trigger {:.3}", state.leftTrigger));
            }
            if state.rightTrigger != old.rightTrigger {
                changes.push(format!("right trigger {:.3}", state.rightTrigger));
            }

            let held: Vec<&str> = GAMEPAD_BUTTONS
                .iter()
                .filter(|(button, _)| state.buttons & button != 0)
                .map(|(_, name)| *name)
                .collect();
            let summary = vec![
                format!(
                    "Left stick {}   Right stick {}",
                    pair(state.leftThumbstickX, state.leftThumbstickY),
                    pair(state.rightThumbstickX, state.rightThumbstickY)
                ),
                format!(
                    "Left trigger {:.3}   Right trigger {:.3}",
                    state.leftTrigger, state.rightTrigger
                ),
                format!(
                    "Buttons held: {}",
                    if held.is_empty() {
                        "none".to_string()
                    } else {
                        held.join(" ")
                    }
                ),
                format!("buttons = 0x{:08X}", state.buttons),
            ];
            (changes, summary, motion_only)
        }
    }

    unsafe fn handle_keyboard(
        &mut self,
        time: f64,
        reading: &IGameInputReading,
        previous: Option<&IGameInputReading>,
        label: &str,
    ) -> (Vec<String>, Vec<String>, bool) {
        unsafe {
            let keys = key_states(reading);
            let old = previous
                .map(|previous| key_states(previous))
                .unwrap_or_default();
            self.keyboard.rate.add(time);

            let mut changes = Vec::new();
            for key in &keys {
                if !old.iter().any(|old| old.scanCode == key.scanCode) {
                    changes.push(format!("{} down", key_name(key)));
                    if matches!(
                        key.virtualKey,
                        KEY_TOGGLE_RUMBLE | KEY_CLEAR_LOG | KEY_TOGGLE_FULLSCREEN
                    ) {
                        self.shortcuts.push(key.virtualKey);
                    }
                }
            }
            for key in &old {
                if !keys.iter().any(|new| new.scanCode == key.scanCode) {
                    changes.push(format!("{} up", key_name(key)));
                }
            }

            let mut summary: Vec<String> = keys.iter().map(describe_key).collect();
            if summary.is_empty() {
                summary.push("No keys down".to_string());
            }
            self.keyboard.label = label.to_string();
            self.keyboard.keys = keys;
            (changes, summary, false)
        }
    }

    unsafe fn handle_mouse(
        &mut self,
        time: f64,
        reading: &IGameInputReading,
        previous: Option<&IGameInputReading>,
        device: Option<&IGameInputDevice>,
        label: &str,
    ) -> (Vec<String>, Vec<String>, bool) {
        unsafe {
            let mut state = GameInputMouseState::default();
            reading.GetMouseState(&mut state);
            let old = previous.map(|previous| {
                let mut old = GameInputMouseState::default();
                previous.GetMouseState(&mut old);
                old
            });
            self.mouse.rate.add(time);

            let mut changes = Vec::new();
            let old_buttons = old.map_or(0, |old| old.buttons);
            for &(button, name) in MOUSE_BUTTONS {
                let (now, before) = (state.buttons & button != 0, old_buttons & button != 0);
                if now && !before {
                    changes.push(format!("{name} pressed"));
                } else if before && !now {
                    changes.push(format!("{name} released"));
                }
            }
            let motion_only = changes.is_empty();

            // Position and wheel are running totals, so the change since the
            // previous reading is the motion. The positions flags say which
            // of relative and absolute position the device reports.
            let mut delta = [0, 0];
            if let Some(old) = old {
                let relative = GameInputMouseRelativePosition;
                if state.positions & relative != 0 && old.positions & relative != 0 {
                    delta = [
                        state.positionX - old.positionX,
                        state.positionY - old.positionY,
                    ];
                }
                if delta != [0, 0] {
                    changes.push(format!("moved ({:+}, {:+})", delta[0], delta[1]));
                }
                let wheel = [state.wheelX - old.wheelX, state.wheelY - old.wheelY];
                if wheel[1] != 0 {
                    changes.push(format!("wheel {:+}", wheel[1]));
                }
                if wheel[0] != 0 {
                    changes.push(format!("horizontal wheel {:+}", wheel[0]));
                }
                if state.positions & GameInputMouseAbsolutePosition != 0
                    && (state.absolutePositionX, state.absolutePositionY)
                        != (old.absolutePositionX, old.absolutePositionY)
                {
                    changes.push(format!(
                        "absolute position ({}, {})",
                        state.absolutePositionX, state.absolutePositionY
                    ));
                }
            }

            let view = &mut self.mouse;
            view.label = label.to_string();
            view.state = state;
            view.delta = delta;
            view.sample_rate = self
                .devices
                .iter()
                .find(|info| Some(&info.device) == device)
                .and_then(|info| info.mouse_sample_rate);
            // 400 counts cross half the pad. Raw counts have no pointer
            // acceleration, unlike the cursor.
            for (pad, delta) in view.pad.iter_mut().zip(delta) {
                *pad = (*pad + delta as f32 / 400.0).clamp(-1.0, 1.0);
            }
            if delta != [0, 0] {
                push_trail(&mut view.pad_trail, [view.pad[0], -view.pad[1]]);
            }

            let held: Vec<&str> = MOUSE_BUTTONS
                .iter()
                .filter(|(button, _)| state.buttons & button != 0)
                .map(|(_, name)| *name)
                .collect();
            let mut summary = vec![
                format!(
                    "Total motion ({}, {})   wheel ({}, {})",
                    state.positionX, state.positionY, state.wheelX, state.wheelY
                ),
                format!(
                    "Buttons held: {}",
                    if held.is_empty() {
                        "none".to_string()
                    } else {
                        held.join(", ")
                    }
                ),
            ];
            if state.positions & GameInputMouseAbsolutePosition != 0 {
                summary.push(format!(
                    "Absolute position ({}, {})",
                    state.absolutePositionX, state.absolutePositionY
                ));
            }
            (changes, summary, motion_only)
        }
    }

    fn log(&mut self, time: f64, source: Source, text: String, merge_key: Option<(Source, usize)>) {
        if let (Some(key), Some(last)) = (merge_key, self.log.front_mut())
            && last.merge_key == Some(key)
        {
            last.time = time;
            last.text = text;
            last.count += 1;
            return;
        }
        self.log.push_front(LogEntry {
            time,
            source,
            text,
            count: 1,
            merge_key,
        });
        self.log.truncate(LOG_CAPACITY);
    }

    fn toggle_rumble(&mut self) {
        self.rumble_enabled = !self.rumble_enabled;
        let text = if self.rumble_enabled {
            "Rumble test on: the gamepad's sticks drive its motors"
        } else {
            "Rumble test off"
        };
        self.log(self.now, Source::Demo, text.to_string(), None);
    }

    // With the rumble test on, a gamepad's left stick drives its
    // low-frequency (left grip) motor and its right stick the high-frequency
    // (right grip) one. Each motor is off while its stick is inside its dead
    // zone, which keeps a stick at rest from running it with its noise. Past
    // the dead zone, the motor's strength grows linearly with how far the
    // stick is pushed, up to full at the edge. Only those two motors, the
    // standard rumble that Xbox 360, PlayStation 4 and 5 and most other
    // gamepads with rumble have. Xbox One and Series gamepads' trigger motors
    // stay off.
    unsafe fn update_rumble(&mut self) {
        for rumble in &mut self.rumbles {
            let [left, right] = rumble.sticks;
            let params = if self.rumble_enabled {
                GameInputRumbleParams {
                    lowFrequency: past_dead_zone(left, self.rumble_dead_zone),
                    highFrequency: past_dead_zone(right, self.rumble_dead_zone),
                    ..Default::default()
                }
            } else {
                GameInputRumbleParams::default()
            };
            if params != rumble.sent {
                unsafe { rumble.device.SetRumbleState(Some(&params)) };
                rumble.sent = params;
            }
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            for rumble in &self.rumbles {
                rumble.device.SetRumbleState(None);
            }
            self.game_input.UnregisterCallback(self.callback_token);
        }
    }
}

pub const GAMEPAD_BUTTONS: &[(GameInputGamepadButtons, &str)] = &[
    (GameInputGamepadA, "A"),
    (GameInputGamepadB, "B"),
    (GameInputGamepadX, "X"),
    (GameInputGamepadY, "Y"),
    (GameInputGamepadC, "C"),
    (GameInputGamepadZ, "Z"),
    (GameInputGamepadView, "View"),
    (GameInputGamepadMenu, "Menu"),
    (GameInputGamepadLeftShoulder, "LB"),
    (GameInputGamepadRightShoulder, "RB"),
    (GameInputGamepadLeftTriggerButton, "LT"),
    (GameInputGamepadRightTriggerButton, "RT"),
    (GameInputGamepadLeftThumbstick, "LS"),
    (GameInputGamepadRightThumbstick, "RS"),
    (GameInputGamepadDPadUp, "Up"),
    (GameInputGamepadDPadDown, "Down"),
    (GameInputGamepadDPadLeft, "Left"),
    (GameInputGamepadDPadRight, "Right"),
    (GameInputGamepadLeftThumbstickUp, "LS Up"),
    (GameInputGamepadLeftThumbstickDown, "LS Down"),
    (GameInputGamepadLeftThumbstickLeft, "LS Left"),
    (GameInputGamepadLeftThumbstickRight, "LS Right"),
    (GameInputGamepadRightThumbstickUp, "RS Up"),
    (GameInputGamepadRightThumbstickDown, "RS Down"),
    (GameInputGamepadRightThumbstickLeft, "RS Left"),
    (GameInputGamepadRightThumbstickRight, "RS Right"),
    (GameInputGamepadPaddleLeft1, "P1"),
    (GameInputGamepadPaddleRight1, "P2"),
    (GameInputGamepadPaddleLeft2, "P3"),
    (GameInputGamepadPaddleRight2, "P4"),
];

pub const MOUSE_BUTTONS: &[(GameInputMouseButtons, &str)] = &[
    (GameInputMouseLeftButton, "Left"),
    (GameInputMouseRightButton, "Right"),
    (GameInputMouseMiddleButton, "Middle"),
    (GameInputMouseButton4, "Back"),
    (GameInputMouseButton5, "Forward"),
    (GameInputMouseWheelTiltLeft, "Tilt left"),
    (GameInputMouseWheelTiltRight, "Tilt right"),
];

const KIND_NAMES: &[(GameInputKind, &str)] = &[
    (GameInputKindGamepad, "gamepad"),
    (GameInputKindKeyboard, "keyboard"),
    (GameInputKindMouse, "mouse"),
    (GameInputKindArcadeStick, "arcade stick"),
    (GameInputKindFlightStick, "flight stick"),
    (GameInputKindRacingWheel, "racing wheel"),
    (GameInputKindControllerAxis, "axes"),
    (GameInputKindControllerButton, "buttons"),
    (GameInputKindControllerSwitch, "switches"),
    (GameInputKindSensors, "sensors"),
    (GameInputKindTouch, "touch"),
    (GameInputKindLighting, "lighting"),
    (GameInputKindSystemButton, "system buttons"),
    (GameInputKindRawDeviceReport, "raw reports"),
];

pub fn kind_names(kinds: GameInputKind) -> String {
    let names: Vec<&str> = KIND_NAMES
        .iter()
        .filter(|(kind, _)| kinds & kind != 0)
        .map(|(_, name)| *name)
        .collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

// How far a stick is from its center: 0 at rest, 1 at its edge. Toward the
// corners, many sticks read a little past 1, so that's cut off at 1.
fn deflection(x: f32, y: f32) -> f32 {
    x.hypot(y).min(1.0)
}

// A stick's deflection rescaled to 0 at the dead zone's edge and 1 at the
// stick's, and 0 inside the dead zone, so there's no jump where it starts.
fn past_dead_zone(deflection: f32, dead_zone: f32) -> f32 {
    ((deflection - dead_zone) / (1.0 - dead_zone)).max(0.0)
}

fn pair(x: f32, y: f32) -> String {
    format!("({x:+.3}, {y:+.3})")
}

fn push_trail(trail: &mut VecDeque<[f32; 2]>, point: [f32; 2]) {
    if trail.back() != Some(&point) {
        trail.push_back(point);
        if trail.len() > TRAIL_LENGTH {
            trail.pop_front();
        }
    }
}

unsafe fn key_states(reading: &IGameInputReading) -> Vec<GameInputKeyState> {
    unsafe {
        let mut keys = vec![GameInputKeyState::default(); reading.GetKeyCount() as usize];
        let count = reading.GetKeyState(&mut keys);
        keys.truncate(count as usize);
        keys
    }
}

// The key's name in the current keyboard layout, like "Left Shift" or "Num 7".
pub fn key_name(key: &GameInputKeyState) -> String {
    // GetKeyNameTextW takes the scan code the way WM_KEYDOWN's lParam holds
    // it: in bits 16-23, with bit 24 set for keys that send an 0xE0 prefix.
    let extended = (key.scanCode & 0xFF00) == 0xE000;
    let lparam = ((key.scanCode & 0xFF) << 16) | if extended { 1 << 24 } else { 0 };
    let mut buffer = [0u16; 64];
    let length =
        unsafe { GetKeyNameTextW(lparam as i32, windows_core::PWSTR(buffer.as_mut_ptr()), 64) };
    if length > 0 {
        String::from_utf16_lossy(&buffer[..length as usize])
    } else {
        format!("Key 0x{:02X}", key.virtualKey)
    }
}

pub fn describe_key(key: &GameInputKeyState) -> String {
    let mut text = format!(
        "{}  scan 0x{:04X}  VK 0x{:02X}",
        key_name(key),
        key.scanCode,
        key.virtualKey
    );
    if let Some(character) = char::from_u32(key.codePoint).filter(|c| !c.is_control() && *c != '\0')
    {
        if crate::font::Font::has_glyph(character) {
            text += &format!("  '{character}'");
        }
        text += &format!("  U+{:04X}", key.codePoint);
    }
    if key.isDeadKey {
        text += "  dead key";
    }
    text
}
