// Every program's input devices, read with GameInput: gamepads, keyboards and
// mice, which are all the input there is. The main loop reads them once a
// frame, right before the frame, and turns what they read into the
// interface's events (input/reader.rs). Programs that show the devices
// themselves, like GameInputDemo, read the same frame's readings and device
// events from here, through Startup::devices, and run gamepads' motors.
//
// GameInput's input is readings, not presses and releases: each reading is a
// snapshot of one device's whole state, with the time that it came. GameInput
// keeps a history of them, which held about 512 of a gamepad's on the
// development machine, with GameInput 4.0.306: under a tenth of a second at
// thousands a second. Each frame walks the history from
// the last reading that it handed out with GetNextReading, right before the
// frame, so that the newest readings are as fresh as they can be. A frame
// that took longer than the history lasts would miss the readings that left
// it, so GameInput's reading callback also queues every reading as it comes,
// on one of GameInput's threads, and each frame hands out the readings of
// both, each once, oldest first (see `frame_readings`). Readings' times are
// GameInput's microseconds, which `qpc` turns into the performance counter's
// ticks, on the clock that frames and the display's flips are timed on.

use std::collections::VecDeque;
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

type ReadingQueue = Mutex<VecDeque<IGameInputReading>>;

// The most readings that the reading queue keeps between frames, the oldest
// going first: a second of two gamepads at 8,000 reports a second, for a
// frame that's that late, like one that waited on a window being moved.
const READING_QUEUE_CAPACITY: usize = 16_384;

// GameInput's reading callback runs on one of its threads, for each reading
// in the order that they came, one at a time. It only queues the reading,
// and the main thread takes it in `read`.
unsafe extern "system" fn on_reading(_token: GameInputCallbackToken, context: *mut c_void, reading: *mut c_void) {
    let queue = unsafe { &*(context as *const ReadingQueue) };
    if let Some(reading) = unsafe { IGameInputReading::from_raw_borrowed(&reading) } {
        let mut queue = queue.lock().unwrap();
        if queue.len() >= READING_QUEUE_CAPACITY {
            queue.pop_front();
        }
        queue.push_back(reading.clone());
    }
}

pub struct Devices {
    game_input: IGameInput,
    runtime: Runtime,
    callback_token: GameInputCallbackToken,
    // The queue that the callback fills, which it holds a reference to.
    queue: *const DeviceQueue,
    reading_callback_token: GameInputCallbackToken,
    // The queue that the reading callback fills, which it holds a reference
    // to.
    reading_queue: *const ReadingQueue,
    // The last reading that the walk of the history got to, for each kind of
    // input.
    last_readings: Vec<(GameInputKind, IGameInputReading)>,
    // The newest reading handed out, of each kind, by its time and device.
    handed_out: [Option<(u64, usize)>; 3],
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
    // Starts GameInput, with GameInput.lib's loader, which loads the installed
    // runtime (see Library/GameInput), and starts listening for gamepads,
    // keyboards and mice. The callback also reports the devices that are already
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
        let reading_queue = Arc::into_raw(Arc::new(ReadingQueue::default()));
        let reading_callback_token = unsafe {
            game_input.RegisterReadingCallback(None, KINDS[0] | KINDS[1] | KINDS[2], Some(reading_queue as *const c_void), Some(on_reading))
        };
        let reading_callback_token = match reading_callback_token {
            Ok(token) => token,
            Err(error) => {
                unsafe {
                    game_input.UnregisterCallback(callback_token);
                    drop(Arc::from_raw(queue));
                    drop(Arc::from_raw(reading_queue));
                }
                return Err(format!("GameInput couldn't listen for readings: {} ({})", error.message(), error.code()));
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
            reading_callback_token,
            reading_queue,
            last_readings: Vec::new(),
            handed_out: [None; 3],
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
        let queued: Vec<IGameInputReading> = unsafe { (*self.reading_queue).lock().unwrap().drain(..).collect() };
        let queued: Vec<(u64, usize, IGameInputReading)> = queued.into_iter().map(|reading| unsafe { keyed(reading) }).collect();
        for (index, kind) in KINDS.into_iter().enumerate() {
            let walked = unsafe { self.new_readings(kind) };
            let walked = walked.into_iter().map(|reading| unsafe { keyed(reading) });
            let of_kind = queued.iter().filter(|(_, _, reading)| unsafe { reading.GetInputKind() } & kind != 0).cloned();
            let objects = frame_readings(of_kind, walked, self.handed_out[index]);
            if let Some((timestamp, device, _)) = objects.last() {
                self.handed_out[index] = Some((*timestamp, *device));
            }
            let readings = objects.iter().map(|(_, _, reading)| unsafe { self.reading(reading) }).collect();
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
// input, or a frame that took longer than the history lasts, when the last
// reading handled has left GameInput's history, starts over from the newest
// reading, as Microsoft's sample for GetNextReading does. Otherwise a failure
// that kept coming would stop that kind of input for good.
pub fn after_next_reading_failed(code: HRESULT) -> NextReadingFailure {
    if code == GAMEINPUT_E_READING_NOT_FOUND {
        NextReadingFailure::NothingNewer
    } else {
        NextReadingFailure::FindFromNewest
    }
}

// The readings of one kind for a frame, oldest first: those that the reading
// callback queued, and those that the walk of the history found, each once,
// and only those newer than `handed_out`, the newest that an earlier frame
// handed out. Each is known by its time, in GameInput's microseconds, and its
// device: no two of a device's readings have the same time.
pub fn frame_readings<T>(
    queued: impl IntoIterator<Item = (u64, usize, T)>,
    walked: impl IntoIterator<Item = (u64, usize, T)>,
    handed_out: Option<(u64, usize)>,
) -> Vec<(u64, usize, T)> {
    let mut readings: Vec<(u64, usize, T)> = queued
        .into_iter()
        .chain(walked)
        .filter(|(timestamp, device, _)| handed_out.is_none_or(|newest| (*timestamp, *device) > newest))
        .collect();
    readings.sort_by_key(|(timestamp, device, _)| (*timestamp, *device));
    readings.dedup_by_key(|(timestamp, device, _)| (*timestamp, *device));
    readings
}

// A reading with its time and its device's address, which tells devices
// apart, or 0 when GameInput doesn't say.
unsafe fn keyed(reading: IGameInputReading) -> (u64, usize, IGameInputReading) {
    unsafe {
        let device = reading.GetDevice().map_or(0, |device| device.as_raw() as usize);
        (reading.GetTimestamp(), device, reading)
    }
}

impl Drop for Devices {
    fn drop(&mut self) {
        unsafe {
            // Once they return, the callbacks don't run again, so their
            // queues can go.
            self.game_input.UnregisterCallback(self.callback_token);
            drop(Arc::from_raw(self.queue));
            self.game_input.UnregisterCallback(self.reading_callback_token);
            drop(Arc::from_raw(self.reading_queue));
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
