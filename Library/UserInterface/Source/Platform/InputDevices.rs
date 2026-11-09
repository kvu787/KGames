// Every program's input devices, read with GameInput: gamepads, keyboards and
// mice, which are all the input there is. The main loop reads them once a
// frame, right before the frame, and turns what they read into the
// interface's events (Input/Reader.rs). Programs that show the devices
// themselves, like GameInputDemo, read the same frame's readings and device
// events from here, through Startup::devices, and run gamepads' motors.
//
// GameInput's input is readings, not presses and releases: each reading is a
// snapshot of one device's whole state, with the time that it came, and
// GameInput keeps half a second of them. This polls that history once a
// frame and walks it with GetNextReading, so that no reading is missed
// between frames. Readings' times are GameInput's
// microseconds, which `qpc` turns into the performance counter's ticks, on
// the clock that frames and the display's flips are timed on.

use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use game_input::*;
use windows_core::{GUID, HRESULT, Interface};

use crate::platform::clock;

// A gamepad, keyboard or mouse, with everything that GameInput says about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    // Tells devices apart, and stays the same while a device is connected.
    pub id: usize,
    // What it calls itself, like "Xbox Controller".
    pub name: String,
    // Its name, then its vendor and product IDs, like "Xbox Controller
    // (045E:0B13)".
    pub label: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub revision: u16,
    // Its HID usage page and usage, like 0x01 and 0x06 for a keyboard.
    pub usage: GameInputUsage,
    pub hardware_version: GameInputVersion,
    pub firmware_version: GameInputVersion,
    pub family: GameInputDeviceFamily,
    pub supported_input: GameInputKind,
    pub rumble_motors: GameInputRumbleMotors,
    pub system_buttons: GameInputSystemButtons,
    // Windows' container of the physical device, which the devices it makes
    // up share, like a mouse's keyboard for its macros.
    pub container_id: GUID,
    // The path of its device interface, or "" if it has none.
    pub pnp_path: String,
    pub xinput_index: Option<u32>,
    pub keyboard: Option<GameInputKeyboardInfo>,
    pub mouse: Option<GameInputMouseInfo>,
    pub gamepad: Option<GameInputGamepadInfo>,
}

// A snapshot of one device's whole state, at a time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    // The ID of the device it's from (see Device), or None when GameInput
    // doesn't say.
    pub device: Option<usize>,
    // On GameInput's clock, in microseconds.
    pub timestamp: u64,
    // Which kinds of input the reading has, like a gamepad's, or both a
    // keyboard's and a mouse's.
    pub kinds: GameInputKind,
    pub gamepad: GameInputGamepadState,
    // The keys held down.
    pub keys: Vec<GameInputKeyState>,
    // The buttons held, and running totals of the motion and the wheels.
    pub mouse: GameInputMouseState,
}

// The kinds of input that every frame reads, each with its own history.
pub const KINDS: [GameInputKind; 3] = [GameInputKindGamepad, GameInputKindKeyboard, GameInputKindMouse];

type DeviceQueue = Mutex<Vec<(GameInputDeviceEvent, IGameInputDevice)>>;

// GameInput's device callback runs on one of its threads. It only queues the
// event, and the main thread takes it in `read`.
unsafe extern "system" fn on_device_event(
    _token: GameInputCallbackToken,
    context: *mut c_void,
    event: GameInputDeviceEvent,
    device: *mut c_void,
) {
    let queue = unsafe { &*(context as *const DeviceQueue) };
    if let Some(device) = unsafe { IGameInputDevice::from_raw_borrowed(&device) } {
        queue.lock().unwrap().push((event, device.clone()));
    }
}

pub struct Devices {
    game_input: IGameInput,
    runtime: Runtime,
    callback_token: GameInputCallbackToken,
    // The queue that the callback fills, which it holds a reference to.
    queue: *const DeviceQueue,
    // The last reading handed out, for each kind of input.
    last_readings: Vec<(GameInputKind, IGameInputReading)>,
    // Every device seen, as GameInput's object and as data. GameInput keeps
    // one object per device, so the object's address tells devices apart.
    known: Vec<(IGameInputDevice, Device)>,
    // GameInput's microseconds at the counter's tick 0, which GameInput's
    // clock is a fixed distance from. Checked again every few seconds.
    epoch: i64,
    calibrated: i64,
    // This frame's readings by kind, oldest first, and device events.
    readings: [Vec<Reading>; 3],
    device_events: Vec<(GameInputDeviceEvent, Device)>,
    frame: u64,
}

// How often GameInput's clock is checked against the counter's, in seconds.
const CALIBRATION_SECONDS: f64 = 10.0;

impl Devices {
    // Loads the installed GameInput runtime, the only one that KGames uses
    // (see Library/GameInput), and starts listening for gamepads, keyboards
    // and mice. The callback also reports the devices that are already
    // connected.
    pub fn new() -> Result<Devices, String> {
        let (game_input, runtime) = game_input::create()?;
        let queue = Arc::into_raw(Arc::new(DeviceQueue::default()));
        let callback_token = unsafe {
            game_input.RegisterDeviceCallback(
                None,
                GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse,
                Some(queue as *const c_void),
                Some(on_device_event),
            )
        };
        let callback_token = match callback_token {
            Ok(token) => token,
            Err(error) => {
                drop(unsafe { Arc::from_raw(queue) });
                return Err(format!("GameInput couldn't listen for devices: {} ({})", error.message(), error.code()));
            }
        };
        // By default, GameInput gives a process input only while it has
        // focus, on GameInput's own timing. The reader follows the devices'
        // state without focus too, so that keys and buttons already held when
        // the window gets focus, like the click that gives it focus, count
        // only once they're let go of, and it drops what came without focus
        // itself, by the window's own focus changes (Input/Reader.rs).
        unsafe { game_input.EnableBackgroundInput(true) };
        let mut devices = Devices {
            game_input,
            runtime,
            callback_token,
            queue,
            last_readings: Vec::new(),
            known: Vec::new(),
            epoch: 0,
            calibrated: 0,
            readings: Default::default(),
            device_events: Vec::new(),
            frame: 0,
        };
        devices.calibrate();
        Ok(devices)
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    // The time now on GameInput's clock, in microseconds.
    pub fn now(&self) -> u64 {
        unsafe { self.game_input.GetCurrentTimestamp() }
    }

    // A time on GameInput's clock as the counter's ticks.
    pub fn qpc(&self, timestamp: u64) -> i64 {
        clock::ticks_from_microseconds(timestamp as i64 - self.epoch)
    }

    // Finds where GameInput's clock is from the counter's, from the closest
    // of a few readings of both.
    fn calibrate(&mut self) {
        let mut best = (i64::MAX, 0, 0);
        for _ in 0..8 {
            let before = clock::now();
            let game_input = self.now() as i64;
            let after = clock::now();
            if after - before < best.0 {
                best = (after - before, (before + after) / 2, game_input);
            }
        }
        let (_, middle, game_input) = best;
        let middle_microseconds = (middle as i128 * 1_000_000 / clock::frequency() as i128) as i64;
        self.epoch = game_input - middle_microseconds;
        self.calibrated = middle;
    }

    // Reads the frame's input: every reading since the last frame, of each
    // kind, and what happened to devices.
    #[allow(non_upper_case_globals)]
    pub fn read(&mut self) {
        self.frame += 1;
        if clock::seconds(clock::now() - self.calibrated) >= CALIBRATION_SECONDS {
            self.calibrate();
        }
        self.device_events.clear();
        let events: Vec<_> = unsafe { (*self.queue).lock().unwrap().drain(..).collect() };
        for (event, object) in events {
            // A device that connects, or whose keyboard layout or XInput
            // index changes, is described again.
            if matches!(
                event,
                GameInputDeviceEventConnected | GameInputDeviceEventKeyboardLayoutUpdated | GameInputDeviceEventXInputIndexUpdated
            ) {
                self.known.retain(|(known, _)| *known != object);
            }
            let device = unsafe { self.device(&object) };
            self.device_events.push((event, device));
        }
        for (index, kind) in KINDS.into_iter().enumerate() {
            let objects = unsafe { self.new_readings(kind) };
            let readings = objects.iter().map(|reading| unsafe { self.reading(reading) }).collect();
            self.readings[index] = readings;
        }
    }

    // How many frames have read input, which tells a program whether it has
    // already seen this frame's readings.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    // This frame's readings with the kind of input, oldest first.
    pub fn readings(&self, kind: GameInputKind) -> &[Reading] {
        match KINDS.iter().position(|known| *known == kind) {
            Some(index) => &self.readings[index],
            None => &[],
        }
    }

    // What happened to devices this frame, oldest first: they connected,
    // disconnected, or changed.
    pub fn device_events(&self) -> &[(GameInputDeviceEvent, Device)] {
        &self.device_events
    }

    // Runs a gamepad's motors, or stops them with None.
    pub fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>) {
        if let Some((object, _)) = self.known.iter().find(|(_, known)| known.id == device) {
            unsafe { object.SetRumbleState(params) };
        }
    }

    // The device's data, from what's known about it, or from GameInput.
    unsafe fn device(&mut self, device: &IGameInputDevice) -> Device {
        match self.known.iter().position(|(object, _)| object == device) {
            Some(index) => self.known[index].1.clone(),
            None => {
                let described = unsafe { describe(device) };
                self.known.push((device.clone(), described.clone()));
                described
            }
        }
    }

    unsafe fn reading(&mut self, reading: &IGameInputReading) -> Reading {
        unsafe {
            let kinds = reading.GetInputKind();
            // The device's object is kept, even if its device event hasn't
            // come yet, so that no other device can get its address, and with
            // it its ID.
            let device = reading.GetDevice().map(|device| self.device(&device).id);
            let mut data = Reading {
                device,
                timestamp: reading.GetTimestamp(),
                kinds,
                ..Default::default()
            };
            if kinds & GameInputKindGamepad != 0 {
                reading.GetGamepadState(&mut data.gamepad);
            }
            if kinds & GameInputKindKeyboard != 0 {
                let mut keys = vec![GameInputKeyState::default(); reading.GetKeyCount() as usize];
                let count = reading.GetKeyState(&mut keys);
                keys.truncate(count as usize);
                data.keys = keys;
            }
            if kinds & GameInputKindMouse != 0 {
                reading.GetMouseState(&mut data.mouse);
            }
            data
        }
    }

    // Readings of one kind since the last call, oldest first.
    unsafe fn new_readings(&mut self, kind: GameInputKind) -> Vec<IGameInputReading> {
        let mut readings = Vec::new();
        unsafe {
            let Some(index) = self.last_readings.iter().position(|(last_kind, _)| *last_kind == kind) else {
                // The first time, start from the newest reading.
                if let Ok(current) = self.current_reading(kind) {
                    self.last_readings.push((kind, current.clone()));
                    readings.push(current);
                }
                return readings;
            };
            let mut last = self.last_readings[index].1.clone();
            loop {
                match self.next_reading(&last, kind) {
                    Ok(next) => {
                        readings.push(next.clone());
                        last = next;
                    }
                    Err(code) => {
                        if after_next_reading_failed(code) == NextReadingFailure::FindFromNewest {
                            readings.extend(self.readings_after(last.GetTimestamp(), kind));
                        }
                        break;
                    }
                }
            }
            if let Some(newest) = readings.last() {
                self.last_readings[index].1 = newest.clone();
            }
        }
        readings
    }

    // GetCurrentReading and GetNextReading, with the HRESULT that GameInput
    // returns. They fail every frame for each kind of input with nothing new
    // to read, and the bindings' Result makes each failure a
    // windows_core::Error, which asks COM for the thread's error info.
    unsafe fn current_reading(&self, kind: GameInputKind) -> Result<IGameInputReading, HRESULT> {
        let game_input = &self.game_input;
        let mut reading = std::ptr::null_mut();
        unsafe {
            let code = (Interface::vtable(game_input).GetCurrentReading)(game_input.as_raw(), kind, std::ptr::null_mut(), &mut reading);
            returned(code, reading)
        }
    }

    unsafe fn next_reading(&self, last: &IGameInputReading, kind: GameInputKind) -> Result<IGameInputReading, HRESULT> {
        let game_input = &self.game_input;
        let mut reading = std::ptr::null_mut();
        unsafe {
            let code = (Interface::vtable(game_input).GetNextReading)(
                game_input.as_raw(),
                last.as_raw(),
                kind,
                std::ptr::null_mut(),
                &mut reading,
            );
            returned(code, reading)
        }
    }

    // Readings of one kind newer than a timestamp, oldest first. Walks back
    // from the newest reading with GetPreviousReading.
    unsafe fn readings_after(&self, timestamp: u64, kind: GameInputKind) -> Vec<IGameInputReading> {
        let mut readings = Vec::new();
        unsafe {
            let mut reading = self.current_reading(kind).ok();
            while let Some(current) = reading {
                if current.GetTimestamp() <= timestamp {
                    break;
                }
                reading = self.game_input.GetPreviousReading(&current, kind, None).ok();
                readings.push(current);
            }
        }
        readings.reverse();
        readings
    }
}

// What GetNextReading's failure says about the readings after the last one
// handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextReadingFailure {
    // There are none yet.
    NothingNewer,
    // The last reading can't be walked on from, so find the ones after it
    // from the newest end, by their times.
    FindFromNewest,
}

// Only GAMEINPUT_E_READING_NOT_FOUND says that there's nothing newer. Any
// other failure, like GAMEINPUT_E_REFERENCE_READING_TOO_OLD after a pause in
// input, when the last reading handled has left GameInput's half second of
// history, starts over from the newest reading, as Microsoft's sample for
// GetNextReading does. Otherwise a failure that kept coming would stop that
// kind of input for good.
pub fn after_next_reading_failed(code: HRESULT) -> NextReadingFailure {
    if code == GAMEINPUT_E_READING_NOT_FOUND {
        NextReadingFailure::NothingNewer
    } else {
        NextReadingFailure::FindFromNewest
    }
}

impl Drop for Devices {
    fn drop(&mut self) {
        unsafe {
            // Once it returns, the callback doesn't run again, so its queue
            // can go.
            self.game_input.UnregisterCallback(self.callback_token);
            drop(Arc::from_raw(self.queue));
        }
    }
}

// The reading that a GameInput call returned, which is now ours to release,
// or the HRESULT that it failed with.
unsafe fn returned(code: HRESULT, reading: *mut c_void) -> Result<IGameInputReading, HRESULT> {
    if code.is_ok() {
        Ok(unsafe { IGameInputReading::from_raw(reading) })
    } else {
        Err(code)
    }
}

// A device's data, from GameInput.
unsafe fn describe(device: &IGameInputDevice) -> Device {
    unsafe {
        let name = utf8(|buffer| device.GetDisplayName(buffer));
        let name = if name.is_empty() { "Unnamed device".to_string() } else { name };
        let (vendor_id, product_id) = (device.GetVendorId(), device.GetProductId());
        let mut usage = GameInputUsage::default();
        device.GetUsage(&mut usage);
        let mut hardware_version = GameInputVersion::default();
        device.GetHardwareVersion(&mut hardware_version);
        let mut firmware_version = GameInputVersion::default();
        device.GetFirmwareVersion(&mut firmware_version);
        let mut container_id = GUID::zeroed();
        device.GetContainerId(&mut container_id);
        let mut xinput_index = 0;
        let mut keyboard = GameInputKeyboardInfo::default();
        let mut mouse = GameInputMouseInfo::default();
        let mut gamepad = GameInputGamepadInfo::default();
        Device {
            id: device.as_raw() as usize,
            label: format!("{name} ({vendor_id:04X}:{product_id:04X})"),
            name,
            vendor_id,
            product_id,
            revision: device.GetRevisionNumber(),
            usage,
            hardware_version,
            firmware_version,
            family: device.GetDeviceFamily(),
            supported_input: device.GetSupportedInput(),
            rumble_motors: device.GetSupportedRumbleMotors(),
            system_buttons: device.GetSupportedSystemButtons(),
            container_id,
            pnp_path: utf8(|buffer| device.GetPnpPath(buffer)),
            xinput_index: device.GetXInputDeviceIndex(&mut xinput_index).then_some(xinput_index),
            keyboard: device.GetKeyboardInfo(&mut keyboard).then_some(keyboard),
            mouse: device.GetMouseInfo(&mut mouse).then_some(mouse),
            gamepad: device.GetGamepadInfo(&mut gamepad).then_some(gamepad),
        }
    }
}

// A string that a GameInput method writes as UTF-8 with a terminating zero,
// after saying how big it is when given an empty buffer.
fn utf8(get: impl Fn(&mut [u8]) -> u32) -> String {
    let mut buffer = vec![0u8; get(&mut []) as usize];
    get(&mut buffer);
    String::from_utf8_lossy(&buffer).trim_end_matches('\0').to_string()
}
