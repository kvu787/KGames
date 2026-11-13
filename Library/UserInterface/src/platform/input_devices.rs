// Every program's input devices, read with GameInput: gamepads, keyboards and
// mice, which are all the input there is. The main loop reads them once a
// frame, right before the frame, and turns what they read into the
// interface's events (input/reader.rs). Programs that show the devices
// themselves, like GameInputDemo, read the same frame's readings and device
// events from here, through Startup::devices, and run gamepads' motors.
//
// GameInput's input is readings, not presses and releases: each reading is a
// snapshot of one device's whole state, with the time that it came. GameInput
// puts every device's readings in one stream, "in the order that the input
// occurred", and keeps a short history of it. Each frame walks the stream, as
// Microsoft's documentation and its GameInputSequential sample do: from the
// last reading handed out, with GetNextReading, until there's nothing newer,
// filtered to gamepad, keyboard and mouse input together, so that the frame
// hands out every reading since the last frame once, in the stream's order.
// The reading objects are where the walk is: GameInput's readings are
// singletons, which compare by pointer, while two readings can have the same
// timestamp, so nothing here orders or tells readings apart by their times.
//
// The history is short: about 513 readings, which is about 70 ms of a gamepad
// that reports thousands of times a second, measured on the development
// machine with GameInput 4.0.306. A frame that takes longer than that loses
// its place in the stream, and starts over from the newest reading, as
// Microsoft's sample does, so the readings that left the history in between
// are lost. Readings' times are GameInput's microseconds, which `qpc` turns
// into the performance counter's ticks, on the clock that frames, the window's
// focus changes and the display's flips are timed on.

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

// The kinds of input that every frame reads, in one walk of the stream: a
// reading with any of them.
pub const KINDS: GameInputKind = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;

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
    // The last reading handed out, which the next frame's walk starts after,
    // or None before the first.
    last: Option<IGameInputReading>,
    // How many times a frame's walk lost its place in the stream, and started
    // over from the newest reading.
    restarts: u64,
    // Every device seen, as GameInput's object and as data. GameInput keeps
    // one object per device, so the object's address tells devices apart.
    known: Vec<(IGameInputDevice, Device)>,
    // GameInput's microseconds at the counter's tick 0, which GameInput's
    // clock is a fixed distance from. Checked again every few seconds.
    epoch: i64,
    calibrated: i64,
    // This frame's readings, in the stream's order, and device events.
    readings: Vec<Reading>,
    device_events: Vec<(GameInputDeviceEvent, Device)>,
    frame: u64,
}

// How often GameInput's clock is checked against the counter's, in seconds.
const CALIBRATION_SECONDS: f64 = 10.0;

impl Devices {
    // Starts GameInput, with GameInput.lib's loader, which loads the installed
    // runtime (see Library/GameInput), and starts listening for gamepads,
    // keyboards and mice. The callback also reports the devices that are already
    // connected.
    pub fn new() -> Result<Devices, String> {
        let (game_input, runtime) = game_input::create()?;
        let queue = Arc::into_raw(Arc::new(DeviceQueue::default()));
        let callback_token = unsafe { game_input.RegisterDeviceCallback(None, KINDS, Some(queue as *const c_void), Some(on_device_event)) };
        let callback_token = match callback_token {
            Ok(token) => token,
            Err(error) => {
                drop(unsafe { Arc::from_raw(queue) });
                return Err(format!("GameInput couldn't listen for devices: {} ({})", error.message(), error.code()));
            }
        };
        // GameInput's background input stays off, as it is by default, so
        // GameInput gives the process input only while it has focus. On, it
        // would give input without focus too, but GameInput then reads a
        // gamepad in Xbox 360 mode on a timer instead of as it reports. On the
        // development machine, with GameInput 4.0.306, two GameSir controllers
        // in Xbox 360 mode that report 8,000 times a second gave a focused
        // window 64 readings a second with background input on, 15.6 ms
        // apart, and about 7,500 with it off.
        let mut devices = Devices {
            game_input,
            runtime,
            callback_token,
            queue,
            last: None,
            restarts: 0,
            known: Vec::new(),
            epoch: 0,
            calibrated: 0,
            readings: Vec::new(),
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

    // Reads the frame's input: every reading since the last frame, in the
    // stream's order, and what happened to devices.
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
        let game_input = self.game_input.clone();
        let walked = walk_stream(
            &mut self.last,
            || unsafe { current_reading(&game_input) },
            |last| unsafe { next_reading(&game_input, last) },
        );
        if walked.restarted {
            self.restarts += 1;
        }
        self.readings = walked.readings.iter().map(|reading| unsafe { self.reading(reading) }).collect();
    }

    // How many frames have read input, which tells a program whether it has
    // already seen this frame's readings.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    // This frame's readings, in the stream's order: the order the input
    // happened in.
    pub fn readings(&self) -> &[Reading] {
        &self.readings
    }

    // How many times a frame lost its place in the stream, like after a frame
    // that took longer than GameInput's history lasts, and lost the readings
    // that left the history.
    pub fn restarts(&self) -> u64 {
        self.restarts
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
}

// What a frame's walk of the stream found.
#[derive(Clone, Debug, PartialEq)]
pub struct Walked<R> {
    // The readings to hand out, in the stream's order.
    pub readings: Vec<R>,
    // Whether the walk lost its place and started over from the newest
    // reading, losing what came in between.
    pub restarted: bool,
}

// One frame's walk of GameInput's stream, as Microsoft's documentation and
// its GameInputSequential sample walk it, with `current` for GetCurrentReading
// and `next` for GetNextReading, each with the HRESULT it fails with:
//
// - The first walk, with no last reading, hands out the newest reading, and
//   starts from it.
// - Then each reading after the last one handed out, in the stream's order,
//   until GAMEINPUT_E_READING_NOT_FOUND says there's nothing newer.
// - Any other failure, like the last reading having left GameInput's history,
//   starts over from the newest reading, which is handed out unless it's the
//   last one handed out. The readings in between are lost.
// - If there's no newest reading to start over from, the next walk starts as
//   the first did.
//
// Readings are known by their objects, which GameInput's readings compare
// by, so a reading is never handed out twice, and nothing is reordered.
pub fn walk_stream<R: Clone + PartialEq>(
    last: &mut Option<R>,
    current: impl Fn() -> Result<R, HRESULT>,
    next: impl Fn(&R) -> Result<R, HRESULT>,
) -> Walked<R> {
    let mut walked = Walked {
        readings: Vec::new(),
        restarted: false,
    };
    let Some(mut from) = last.clone() else {
        if let Ok(newest) = current() {
            walked.readings.push(newest.clone());
            *last = Some(newest);
        }
        return walked;
    };
    loop {
        match next(&from) {
            Ok(reading) => {
                walked.readings.push(reading.clone());
                from = reading;
            }
            Err(code) if code == GAMEINPUT_E_READING_NOT_FOUND => break,
            Err(_) => {
                walked.restarted = true;
                match current() {
                    Ok(newest) => {
                        if newest != from {
                            walked.readings.push(newest.clone());
                        }
                        from = newest;
                    }
                    Err(_) => {
                        *last = None;
                        return walked;
                    }
                }
                break;
            }
        }
    }
    *last = Some(from);
    walked
}

// GetCurrentReading and GetNextReading of every kind that frames read, with
// the HRESULT that GameInput returns. GetNextReading fails every frame that
// has nothing new, and the bindings' Result makes each failure a
// windows_core::Error, which asks COM for the thread's error info.
unsafe fn current_reading(game_input: &IGameInput) -> Result<IGameInputReading, HRESULT> {
    let mut reading = std::ptr::null_mut();
    unsafe {
        let code = (Interface::vtable(game_input).GetCurrentReading)(game_input.as_raw(), KINDS, std::ptr::null_mut(), &mut reading);
        returned(code, reading)
    }
}

unsafe fn next_reading(game_input: &IGameInput, last: &IGameInputReading) -> Result<IGameInputReading, HRESULT> {
    let mut reading = std::ptr::null_mut();
    unsafe {
        let code = (Interface::vtable(game_input).GetNextReading)(game_input.as_raw(), last.as_raw(), KINDS, std::ptr::null_mut(), &mut reading);
        returned(code, reading)
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
