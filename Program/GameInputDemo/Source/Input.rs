// Keeps what the panels show from gamepads, keyboards and mice: the latest
// state of each, the connected devices, and a log of input events. The
// readings come from an InputSource (see Devices.rs): GameInput, or a script
// or a test.
//
// Each reading is a snapshot of one device's whole state, so this compares
// each one with the same device's reading before it to say what changed. It
// keeps every device's last reading itself for that, because GameInput's
// GetPreviousReading fails once that reading has left its short history,
// like after a pause in input. Then a held key would show as pressed again.

use std::collections::VecDeque;

use game_input::*;
use json::Value;
use windows_api::GetKeyNameTextW;

use crate::devices::{Device, InputSource, Reading};
use crate::scripted_source::ScriptedSource;

const LOG_CAPACITY: usize = 200;
const TRAIL_LENGTH: usize = 96;
// Virtual-key codes of the demo's own shortcuts.
const KEY_TOGGLE_RUMBLE: u8 = 0x70; // F1
const KEY_CLEAR_LOG: u8 = 0x71; // F2
const KEY_TOGGLE_FULLSCREEN: u8 = 0x7A; // F11

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

pub struct LogEntry {
    pub time: f64,
    pub source: Source,
    pub text: String,
    // How many readings were merged into this entry. See Input::log.
    pub count: u32,
    // The entry as the log panel shows it, made when the entry changes
    // instead of every frame: the time, the source from `source_at`, and the
    // text and count from `text_at`.
    pub line: String,
    pub source_at: usize,
    pub text_at: usize,
    merge_key: Option<(Source, usize)>,
}

impl LogEntry {
    fn format_line(&mut self) {
        let time = format!("{:>8.3}  ", self.time);
        let source = format!("{:<9}", self.source.name());
        self.source_at = time.len();
        self.text_at = time.len() + source.len();
        self.line = time + &source + &self.text;
        if self.count > 1 {
            self.line += &format!("  (x{})", self.count);
        }
    }
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
    // Since the same device's previous reading. None for the first reading
    // of the device that the demo has seen, so the changes are from an idle
    // state.
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
    // The device's ID. See Device.
    pub device: Option<usize>,
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
    device: usize,
    sticks: [f32; 2],
    sent: GameInputRumbleParams,
}

// A device's newest reading with one kind of input, handled or not, which
// its next one is compared with.
struct LastReading {
    kind: GameInputKind,
    reading: Reading,
    masked: Masked,
}

// One device's keys or buttons of one kind of input that were held when the
// window got focus, or while it didn't have it, and haven't been let go of
// since. The demo ignores them until they are: it doesn't show them as held,
// and doesn't say they were pressed or let go of. So the click that gives the
// window focus, or the Alt of the Alt+Tab that switches to it, does only
// that, and a shortcut held down in another window doesn't run here.
#[derive(Default)]
struct Masked {
    // Gamepad or mouse buttons.
    buttons: i32,
    // Keyboard keys, by scan code.
    keys: Vec<u32>,
}

impl Masked {
    // Everything held in the reading, of this kind of input.
    fn held(kind: GameInputKind, reading: &Reading) -> Self {
        Self {
            buttons: buttons(kind, reading),
            keys: if kind == GameInputKindKeyboard {
                reading.keys.iter().map(|key| key.scanCode).collect()
            } else {
                Vec::new()
            },
        }
    }

    // Of these, what's still held in the reading.
    fn still_held(&self, kind: GameInputKind, reading: &Reading) -> Self {
        Self {
            buttons: self.buttons & buttons(kind, reading),
            keys: self
                .keys
                .iter()
                .copied()
                .filter(|&code| reading.keys.iter().any(|key| key.scanCode == code))
                .collect(),
        }
    }

    // The state without what's masked.
    fn gamepad(&self, mut state: GameInputGamepadState) -> GameInputGamepadState {
        state.buttons &= !self.buttons;
        state
    }

    fn mouse(&self, mut state: GameInputMouseState) -> GameInputMouseState {
        state.buttons &= !self.buttons;
        state
    }

    fn keys(&self, keys: &[GameInputKeyState]) -> Vec<GameInputKeyState> {
        keys.iter()
            .filter(|key| !self.keys.contains(&key.scanCode))
            .copied()
            .collect()
    }
}

// The buttons held in the reading, of this kind of input. Keys aren't
// buttons.
fn buttons(kind: GameInputKind, reading: &Reading) -> i32 {
    if kind == GameInputKindGamepad {
        reading.gamepad.buttons
    } else if kind == GameInputKindMouse {
        reading.mouse.buttons
    } else {
        0
    }
}

// Which gamepad the demo listens to, if any, chosen in the gamepad panel's
// select box. The others are ignored, along with any keyboard or mouse input
// they have: a GameSir in Xbox 360 mode is one device with a gamepad, a
// keyboard and a mouse.
#[derive(Clone, Debug)]
pub enum Listen {
    // Kept with its device's data, so that it's listened to again if it
    // reconnects, and named while it's gone.
    Gamepad(Device),
    NoGamepad,
}

// The same choice, whatever the device's data says now.
impl PartialEq for Listen {
    fn eq(&self, other: &Listen) -> bool {
        match (self, other) {
            (Listen::Gamepad(chosen), Listen::Gamepad(other)) => chosen.id == other.id,
            (Listen::NoGamepad, Listen::NoGamepad) => true,
            _ => false,
        }
    }
}

impl Listen {
    fn includes(&self, device: &Device) -> bool {
        match self {
            Listen::Gamepad(chosen) => chosen.id == device.id || !device.is_gamepad(),
            Listen::NoGamepad => !device.is_gamepad(),
        }
    }

    // Like "Xbox Controller (045E:0B13)", or "None".
    pub fn label(&self) -> &str {
        match self {
            Listen::Gamepad(device) => &device.label,
            Listen::NoGamepad => "None",
        }
    }
}

// The keyboard and mouse panels each show one device: the last one whose
// reading changed their kind of input. A reading can have several kinds, like
// one from a keyboard that is also a mouse, and a reading that doesn't change
// a panel's kind leaves the panel showing the device it was showing. The rate
// counts the readings that the panel took.
#[derive(Default)]
pub struct KeyboardView {
    pub device: Option<usize>,
    pub label: String,
    pub keys: Vec<GameInputKeyState>,
    // Each key's describe_key, in the same order, made when the keys change
    // instead of every frame, as GetKeyNameTextW and the formatting take a
    // third of a microsecond a key.
    pub descriptions: Vec<String>,
    pub rate: RateCounter,
}

#[derive(Default)]
pub struct MouseView {
    pub device: Option<usize>,
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
    source: Box<dyn InputSource>,
    // The source's timestamps are in microseconds. Shown relative to this.
    start_timestamp: u64,
    pub now: f64,
    pub devices: Vec<Device>,
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
    // For each device and kind of input it has sent a reading of. A device's
    // are dropped when it disconnects, so one that reconnects starts from an
    // idle state.
    last_readings: Vec<LastReading>,
    // Changed with pick_listen, or by listen_by_default until then.
    pub listen: Listen,
    // Whether listen was picked in the select box.
    pub listen_picked: bool,
    // Set when the fullscreen shortcut is pressed. The main loop owns the
    // window, so it does the switch and clears this.
    pub fullscreen_toggled: bool,
    // Virtual keys of the demo's shortcuts, pressed in the reading being handled.
    shortcuts: Vec<u8>,
    // Whether the window had focus at the last update, so input was handled.
    focused: bool,
    // When the window last got focus, in the source's clock. Readings from
    // before then are input without focus too, like the click that gave it.
    focus_timestamp: u64,
}

impl Input {
    pub fn new(mut source: Box<dyn InputSource>, rumble_dead_zone: f32) -> Self {
        let start_timestamp = source.now();
        Self {
            source,
            start_timestamp,
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
            last_readings: Vec::new(),
            listen: Listen::NoGamepad,
            listen_picked: false,
            fullscreen_toggled: false,
            shortcuts: Vec::new(),
            focused: false,
            focus_timestamp: u64::MAX,
        }
    }

    // The GameInput runtime that was loaded, for the header, or None when
    // the input comes from a script.
    pub fn runtime(&self) -> Option<&Runtime> {
        self.source.runtime()
    }

    // The source, if it's a script's or a test's.
    pub fn scripted(&mut self) -> Option<&mut ScriptedSource> {
        self.source.as_scripted()
    }

    fn seconds(&self, timestamp: u64) -> f64 {
        timestamp.saturating_sub(self.start_timestamp) as f64 / 1_000_000.0
    }

    // `focused` is whether the window has focus. Input is only handled while
    // it does, see take_reading. A minimized window doesn't have it, so the
    // main loop calls this with false then, and restoring the window counts
    // as getting focus.
    pub fn update(&mut self, focused: bool) {
        if self.focused && !focused {
            // Keys, buttons and sticks let go of after the window lost focus
            // are never seen, so stop showing them as held.
            self.keyboard.keys.clear();
            self.keyboard.descriptions.clear();
            self.mouse.state.buttons = 0;
            let device = self.gamepad.device;
            self.view_gamepad(device);
            for rumble in &mut self.rumbles {
                rumble.sticks = [0.0; 2];
            }
        }
        let now = self.source.now();
        if !self.focused && focused {
            self.focus_timestamp = now;
            // What's held now was pressed without focus.
            for last in &mut self.last_readings {
                last.masked = Masked::held(last.kind, &last.reading);
            }
        }
        self.focused = focused;
        self.now = self.seconds(now);
        for (event, device) in self.source.device_events() {
            self.handle_device_event(event, device);
        }
        self.listen_by_default();

        // Gather every new reading of each kind, then handle them all in
        // timestamp order, so the log interleaves devices correctly.
        let mut readings = Vec::new();
        for &(source, kind) in READING_KINDS.iter() {
            for reading in self.source.readings(kind) {
                readings.push((source, kind, reading));
            }
        }
        readings.sort_by_key(|(_, _, reading)| reading.timestamp);
        for (source, kind, reading) in readings {
            self.take_reading(source, kind, reading);
        }

        self.gamepad.rate.tick(self.now);
        self.keyboard.rate.tick(self.now);
        self.mouse.rate.tick(self.now);
        // Without focus, the sticks don't drive the motors.
        if focused {
            self.update_rumble();
        } else {
            self.stop_rumble();
        }
    }

    #[allow(non_upper_case_globals)]
    fn handle_device_event(&mut self, event: GameInputDeviceEvent, device: Device) {
        let existing = self.devices.iter().position(|info| info.id == device.id);
        let label = device.label.clone();
        let text = match event {
            GameInputDeviceEventConnected => {
                let text = format!("Connected: {} [{}]", device.label, device.kinds());
                match existing {
                    Some(index) => self.devices[index] = device,
                    None => self.devices.push(device),
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
                if self.gamepad.device == Some(device.id) {
                    self.gamepad = GamepadView::default();
                }
                self.rumbles.retain(|rumble| rumble.device != device.id);
                self.last_readings
                    .retain(|last| last.reading.device != Some(device.id));
                format!("Disconnected: {label}")
            }
            GameInputDeviceEventKeyboardLayoutUpdated => {
                format!("Keyboard layout changed: {label}")
            }
            GameInputDeviceEventHapticInfoUpdated => format!("Haptic info changed: {label}"),
            GameInputDeviceEventXInputIndexUpdated => {
                if let Some(index) = existing {
                    self.devices[index] = device;
                }
                format!("XInput index changed: {label}")
            }
            other => format!("Device event {other}: {label}"),
        };
        self.log(self.now, Source::Device, text, None);
    }

    // Shows another gamepad in the panel, starting from an idle state.
    fn view_gamepad(&mut self, device: Option<usize>) {
        let info = self.devices.iter().find(|info| Some(info.id) == device);
        self.gamepad = GamepadView {
            device,
            layout: info
                .and_then(|info| info.gamepad)
                .map(|gamepad| gamepad.supportedLayout),
            rumble_motors: info.map_or(0, |info| info.rumble_motors),
            ..Default::default()
        };
    }

    // The connected gamepads, which the select box offers besides none.
    pub fn gamepads(&self) -> impl Iterator<Item = &Device> {
        self.devices.iter().filter(|info| info.is_gamepad())
    }

    // Whether the demo handles the device's readings.
    pub fn listens_to(&self, info: &Device) -> bool {
        self.listen.includes(info)
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
            && self.gamepads().any(|info| info.id == device.id)
        {
            return;
        }
        let first = self.gamepads().next().cloned();
        if let Some(device) = first {
            self.set_listen(Listen::Gamepad(device));
        }
    }

    fn set_listen(&mut self, listen: Listen) {
        if listen == self.listen {
            return;
        }
        self.listen = listen;
        // Gamepads no longer listened to stop rumbling. Only gamepads rumble.
        let listened = match &self.listen {
            Listen::Gamepad(device) => Some(device.id),
            Listen::NoGamepad => None,
        };
        let source = &mut self.source;
        self.rumbles.retain(|rumble| {
            let keep = Some(rumble.device) == listened;
            if !keep {
                source.set_rumble(rumble.device, None);
            }
            keep
        });
        let text = match self.listen.clone() {
            Listen::Gamepad(device) => {
                if self.gamepad.device != Some(device.id) {
                    self.view_gamepad(Some(device.id));
                    if let Some(state) = self.gamepad_state(device.id) {
                        self.gamepad.state = state;
                    }
                }
                format!("Listening to {} only", device.label)
            }
            Listen::NoGamepad => {
                self.gamepad = GamepadView::default();
                "Ignoring all gamepads".to_string()
            }
        };
        self.log(self.now, Source::Demo, text, None);
    }

    // A gamepad's state from its last reading, without what's masked, to
    // show it as soon as it's listened to. None without focus, or when that
    // reading came before the window got focus, as that's input without it.
    fn gamepad_state(&self, device: usize) -> Option<GameInputGamepadState> {
        if !self.focused {
            return None;
        }
        let last = self.last_readings.iter().find(|last| {
            last.kind == GameInputKindGamepad && last.reading.device == Some(device)
        })?;
        (last.reading.timestamp >= self.focus_timestamp)
            .then(|| last.masked.gamepad(last.reading.gamepad))
    }

    // Handles a reading if the window had focus for it, and keeps it as its
    // device's last reading of this kind, which the next one is compared with.
    fn take_reading(&mut self, source: Source, kind: GameInputKind, reading: Reading) {
        let index = self.last_readings.iter().position(|last| {
            last.kind == kind && last.reading.device == reading.device
        });
        let (previous, masked_before) = match index {
            Some(index) => {
                let last = self.last_readings.swap_remove(index);
                (Some(last.reading), last.masked)
            }
            None => (None, Masked::default()),
        };
        // KGames programs ignore all input while their window doesn't have
        // focus. GameInput gives gamepad input to windows without focus. It's
        // meant to give keyboard and mouse input only to the window in focus,
        // but GameInput 4.0.306, loaded from the exe's folder, gives that to
        // the demo without focus too, even with EnableBackgroundInput(false).
        // So the demo drops it all itself, along with input from before the
        // window got focus. Otherwise it would show and log what is typed
        // into other windows, and run its shortcuts for them.
        let focused = self.focused && reading.timestamp >= self.focus_timestamp;
        let masked = if focused {
            masked_before.still_held(kind, &reading)
        } else {
            Masked::held(kind, &reading)
        };
        if focused {
            self.handle_reading(source, kind, &reading, previous.as_ref(), &masked_before, &masked);
        }
        self.last_readings.push(LastReading {
            kind,
            reading,
            masked,
        });
    }

    // `masked_before` is what was masked for the previous reading, and
    // `masked` what is for this one.
    fn handle_reading(
        &mut self,
        source: Source,
        kind: GameInputKind,
        reading: &Reading,
        previous: Option<&Reading>,
        masked_before: &Masked,
        masked: &Masked,
    ) {
        let device = match reading.device {
            Some(id) => match self.devices.iter().find(|info| info.id == id) {
                Some(device) => Some(device),
                // It isn't connected: it has disconnected, like when its
                // reading came just before its disconnect was handled, or its
                // connection hasn't been reported yet. Readings of a device
                // that's gone would bring back its view and its rumble.
                None => return,
            },
            None => None,
        };
        if let Some(device) = device
            && !self.listen.includes(device)
        {
            return;
        }
        let label = device.map_or_else(|| "Unknown device".to_string(), |device| device.label.clone());
        // A reading with several kinds of input comes up once for each, so
        // it's counted only for the first.
        let first_kind = READING_KINDS
            .iter()
            .map(|&(_, kind)| kind)
            .find(|&kind| reading.kinds & kind != 0);
        if first_kind == Some(kind) {
            self.reading_count += 1;
        }
        let time = self.seconds(reading.timestamp);
        let id = reading.device;

        // Both readings without what's masked, so it's neither reported nor
        // shown.
        let (changes, state, motion_only) = match source {
            Source::Gamepad => {
                let state = masked.gamepad(reading.gamepad);
                let old = previous.map(|previous| masked_before.gamepad(previous.gamepad));
                self.handle_gamepad(time, state, old.unwrap_or_default(), id)
            }
            Source::Keyboard => {
                let keys = masked.keys(&reading.keys);
                let old = previous.map(|previous| masked_before.keys(&previous.keys));
                self.handle_keyboard(time, keys, old.unwrap_or_default(), id, &label)
            }
            _ => {
                let state = masked.mouse(reading.mouse);
                let old = previous.map(|previous| masked_before.mouse(previous.mouse));
                self.handle_mouse(time, state, old, id, &label)
            }
        };
        // A reading can repeat the previous state, or change only another
        // kind of input on a device with several kinds. Those are counted but
        // not reported.
        if changes.is_empty() {
            return;
        }

        // Analog motion arrives up to thousands of times a second, so a run
        // of motion-only readings from one device shares one log entry.
        let merge_key = motion_only.then(|| (source, id.unwrap_or(0)));
        self.log(time, source, format!("{label}: {}", changes.join(", ")), merge_key);

        self.latest = Some(LatestEvent {
            number: self.reading_count,
            time,
            source,
            device: label,
            kinds: kind_names(reading.kinds),
            changes,
            state,
            timestamp: reading.timestamp,
            interval: previous.map(|previous| {
                reading.timestamp.saturating_sub(previous.timestamp) as f64 / 1000.0
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

    // `old` is the state of the device's previous reading, or idle.
    fn handle_gamepad(
        &mut self,
        time: f64,
        state: GameInputGamepadState,
        old: GameInputGamepadState,
        device: Option<usize>,
    ) -> (Vec<String>, Vec<String>, bool) {
        self.gamepad.rate.add(time);
        if let Some(device) = device {
            let sticks = [
                deflection(state.leftThumbstickX, state.leftThumbstickY),
                deflection(state.rightThumbstickX, state.rightThumbstickY),
            ];
            match self.rumbles.iter_mut().find(|rumble| rumble.device == device) {
                Some(rumble) => rumble.sticks = sticks,
                None => self.rumbles.push(Rumble {
                    device,
                    sticks,
                    sent: GameInputRumbleParams::default(),
                }),
            }
        }

        // The panel shows the gamepad listened to. Its view is reset when it
        // disconnects, so this sets it up again after it reconnects.
        if self.gamepad.device != device {
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

    // `old` is the keys of the device's previous reading, or none.
    fn handle_keyboard(
        &mut self,
        time: f64,
        keys: Vec<GameInputKeyState>,
        old: Vec<GameInputKeyState>,
        device: Option<usize>,
        label: &str,
    ) -> (Vec<String>, Vec<String>, bool) {
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

        let descriptions: Vec<String> = keys.iter().map(describe_key).collect();
        let mut summary = descriptions.clone();
        if summary.is_empty() {
            summary.push("No keys down".to_string());
        }
        let view = &mut self.keyboard;
        if !changes.is_empty() || view.device == device {
            view.rate.add(time);
            view.device = device;
            view.label = label.to_string();
            view.keys = keys;
            view.descriptions = descriptions;
        }
        (changes, summary, false)
    }

    // `old` is the state of the device's previous reading, if there was one.
    fn handle_mouse(
        &mut self,
        time: f64,
        state: GameInputMouseState,
        old: Option<GameInputMouseState>,
        device: Option<usize>,
        label: &str,
    ) -> (Vec<String>, Vec<String>, bool) {
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
        // previous reading is the motion. The positions flags say which of
        // relative and absolute position the device reports.
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

        let sample_rate = self
            .devices
            .iter()
            .find(|info| Some(info.id) == device)
            .and_then(|info| info.mouse_sample_rate());
        let view = &mut self.mouse;
        if !changes.is_empty() || view.device == device {
            view.rate.add(time);
            view.device = device;
            view.label = label.to_string();
            view.state = state;
            view.delta = delta;
            view.sample_rate = sample_rate;
            // 400 counts cross half the pad. Raw counts have no pointer
            // acceleration, unlike the cursor.
            for (pad, delta) in view.pad.iter_mut().zip(delta) {
                *pad = (*pad + delta as f32 / 400.0).clamp(-1.0, 1.0);
            }
            if delta != [0, 0] {
                push_trail(&mut view.pad_trail, [view.pad[0], -view.pad[1]]);
            }
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

    fn log(&mut self, time: f64, source: Source, text: String, merge_key: Option<(Source, usize)>) {
        if let (Some(key), Some(last)) = (merge_key, self.log.front_mut())
            && last.merge_key == Some(key)
        {
            last.time = time;
            last.text = text;
            last.count += 1;
            last.format_line();
            return;
        }
        let mut entry = LogEntry {
            time,
            source,
            text,
            count: 1,
            line: String::new(),
            source_at: 0,
            text_at: 0,
            merge_key,
        };
        entry.format_line();
        self.log.push_front(entry);
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
    fn update_rumble(&mut self) {
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
                self.source.set_rumble(rumble.device, Some(&params));
                rumble.sent = params;
            }
        }
    }

    // Stops every motor, for while the window doesn't have focus, or is
    // minimized. Otherwise they would keep running at their last strength,
    // as the sticks aren't read then. The sticks count as at rest from then
    // on, so the motors start again only once a stick moves with focus.
    fn stop_rumble(&mut self) {
        let off = GameInputRumbleParams::default();
        for rumble in &mut self.rumbles {
            if rumble.sent != off {
                self.source.set_rumble(rumble.device, None);
                rumble.sent = off;
            }
        }
    }

    // What the panels show, for scripted runs to save and check. Not whether
    // F11 was pressed, as the main loop has switched the window by the time
    // it's saved: the window's own state says whether it's in fullscreen.
    pub fn state(&self) -> Value {
        let gamepad = &self.gamepad;
        let label = |device: Option<usize>| {
            device.and_then(|id| self.devices.iter().find(|info| info.id == id)).map(|info| info.label.clone())
        };
        let names = |flags: i32, buttons: &[(i32, &str)]| -> Vec<String> {
            buttons
                .iter()
                .filter(|(button, _)| flags & button != 0)
                .map(|(_, name)| name.to_string())
                .collect()
        };
        let state = gamepad.state;
        json::object([
            ("ReadingCount", self.reading_count.into()),
            ("Listen", self.listen.label().into()),
            ("Devices", self.devices.iter().map(|info| info.label.clone()).collect::<Vec<_>>().into()),
            (
                "Gamepad",
                json::object([
                    ("Device", label(gamepad.device).into()),
                    ("LeftStick", [state.leftThumbstickX, state.leftThumbstickY].into()),
                    ("RightStick", [state.rightThumbstickX, state.rightThumbstickY].into()),
                    ("LeftTrigger", state.leftTrigger.into()),
                    ("RightTrigger", state.rightTrigger.into()),
                    ("Buttons", names(state.buttons, GAMEPAD_BUTTONS).into()),
                ]),
            ),
            (
                "Keyboard",
                json::object([
                    ("Device", label(self.keyboard.device).into()),
                    ("Keys", self.keyboard.keys.iter().map(key_name).collect::<Vec<_>>().into()),
                ]),
            ),
            (
                "Mouse",
                json::object([
                    ("Device", label(self.mouse.device).into()),
                    ("Buttons", names(self.mouse.state.buttons, MOUSE_BUTTONS).into()),
                    ("Delta", self.mouse.delta.into()),
                    ("Wheel", [self.mouse.state.wheelX, self.mouse.state.wheelY].into()),
                ]),
            ),
            (
                "Latest",
                self.latest.as_ref().map(|latest| latest.changes.clone()).into(),
            ),
            ("Log", self.log.iter().take(10).map(|entry| entry.text.clone()).collect::<Vec<_>>().into()),
            ("RumbleTest", self.rumble_enabled.into()),
            (
                "Rumble",
                Value::Array(
                    self.rumbles
                        .iter()
                        .map(|rumble| {
                            json::object([
                                ("Device", label(Some(rumble.device)).into()),
                                ("LowFrequency", rumble.sent.lowFrequency.into()),
                                ("HighFrequency", rumble.sent.highFrequency.into()),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        for rumble in &self.rumbles {
            self.source.set_rumble(rumble.device, None);
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
        if canvas::Font::has_glyph(character) {
            text += &format!("  '{character}'");
        }
        text += &format!("  U+{:04X}", key.codePoint);
    }
    if key.isDeadKey {
        text += "  dead key";
    }
    text
}
