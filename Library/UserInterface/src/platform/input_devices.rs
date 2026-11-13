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
// its place in the stream, and starts over from each device's newest reading
// (see `Walker`), so that no device keeps what it held before, and only the
// readings that left the history in between are lost. Readings' times are GameInput's microseconds, which `qpc` turns
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
    // Where the walk of the stream is.
    walker: Walker<IGameInput>,
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
            walker: Walker::default(),
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
        let known: Vec<IGameInputDevice> = self.known.iter().map(|(object, _)| object.clone()).collect();
        let walked = self.walker.walk(&self.game_input, &known);
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

// GameInput's stream as a walk sees it: GetCurrentReading, GetNextReading
// and GetPreviousReading, of every kind that frames read, with a device's
// filter or without, each with the HRESULT it fails with, and a reading's
// device. A test makes one up.
pub trait Stream {
    type Reading: Clone + PartialEq;
    type Device: Clone + PartialEq;
    fn current(&self, device: Option<&Self::Device>) -> Result<Self::Reading, HRESULT>;
    fn next(&self, after: &Self::Reading, device: Option<&Self::Device>) -> Result<Self::Reading, HRESULT>;
    fn previous(&self, before: &Self::Reading) -> Result<Self::Reading, HRESULT>;
    fn device(&self, reading: &Self::Reading) -> Option<Self::Device>;
}

// What a frame's walk of the stream found.
#[derive(Clone, Debug, PartialEq)]
pub struct Walked<R> {
    // The readings to hand out, in the stream's order.
    pub readings: Vec<R>,
    // Whether the walk lost its place and started over, losing readings.
    pub restarted: bool,
}

// Walks GameInput's stream, a frame at a time, as Microsoft's documentation
// and its GameInputSequential sample walk it:
//
// - Each walk hands out every reading after the last one handed out, in the
//   stream's order, until GAMEINPUT_E_READING_NOT_FOUND says there's nothing
//   newer.
// - The first walk, and one that loses its place, on any other failure, like
//   the last reading having left GameInput's history, start from the newest
//   reading. Microsoft's sample walks one device, and starts that device over
//   from its newest reading; this walks every device, so it starts each over
//   from its newest reading, which it hands out unless it already has: those
//   in the history after the last reading handed out in the stream's order,
//   after those older than the history, whose order the history no longer
//   has. So no device keeps what it held before the readings that were lost,
//   like a key that was let go of in them. Only the readings in between are
//   lost, which a frame that took longer than the history lasts can't help.
// - If there's no newest reading, the next walk starts as the first did.
//
// Readings are known by their objects, which GameInput's readings compare
// by, never by their times, which can't tell readings apart, so a reading is
// never handed out twice, and nothing is reordered.
pub struct Walker<S: Stream> {
    // The last reading handed out, which the next walk starts after.
    last: Option<S::Reading>,
    // The last reading handed out of each device.
    latest: Vec<(Option<S::Device>, S::Reading)>,
}

impl<S: Stream> Default for Walker<S> {
    fn default() -> Self {
        Walker {
            last: None,
            latest: Vec::new(),
        }
    }
}

impl<S: Stream> Walker<S> {
    // A frame's walk of the stream, with the devices known, which a walk that
    // starts over brings up to date.
    pub fn walk(&mut self, stream: &S, devices: &[S::Device]) -> Walked<S::Reading> {
        let mut walked = Walked {
            readings: Vec::new(),
            restarted: false,
        };
        let Some(mut from) = self.last.clone() else {
            for reading in self.start(stream, devices, None) {
                self.hand_out(stream, reading, &mut walked.readings);
            }
            return walked;
        };
        loop {
            match stream.next(&from, None) {
                Ok(reading) => {
                    self.hand_out(stream, reading.clone(), &mut walked.readings);
                    from = reading;
                }
                Err(code) if code == GAMEINPUT_E_READING_NOT_FOUND => {
                    self.last = Some(from);
                    break;
                }
                Err(_) => {
                    walked.restarted = true;
                    for reading in self.start(stream, devices, Some(&from)) {
                        self.hand_out(stream, reading, &mut walked.readings);
                    }
                    break;
                }
            }
        }
        walked
    }

    // Hands a reading out, as its device's last.
    fn hand_out(&mut self, stream: &S, reading: S::Reading, readings: &mut Vec<S::Reading>) {
        let device = stream.device(&reading);
        match self.latest.iter_mut().find(|(known, _)| *known == device) {
            Some((_, latest)) => *latest = reading.clone(),
            None => self.latest.push((device, reading.clone())),
        }
        readings.push(reading);
    }

    // Starts the walk from the newest reading, after `from`, the last reading
    // handed out, if any: each device's newest reading that isn't handed out
    // yet, those older than the history first, then those in it, in the
    // stream's order, the newest last.
    fn start(&mut self, stream: &S, devices: &[S::Device], from: Option<&S::Reading>) -> Vec<S::Reading> {
        let Ok(newest) = stream.current(None) else {
            self.last = None;
            return Vec::new();
        };
        // Each device's newest reading in the history, met walking back from
        // the newest reading, as far as `from` or the history's start.
        let mut met: Vec<(Option<S::Device>, S::Reading)> = vec![(stream.device(&newest), newest.clone())];
        let mut reading = newest.clone();
        while Some(&reading) != from {
            let Ok(previous) = stream.previous(&reading) else {
                break;
            };
            if Some(&previous) == from {
                break;
            }
            let device = stream.device(&previous);
            if !met.iter().any(|(known, _)| *known == device) {
                met.push((device, previous.clone()));
            }
            reading = previous;
        }
        met.reverse();
        // The newest reading of each device that the walk back didn't meet,
        // which is older than the history, or at or before `from`, unless it's
        // after the newest, which the next walk hands out: one is, if the
        // device has a reading after the newest, which is looked for after
        // the device's newest reading was taken, so that a reading that comes
        // in between can't be handed out twice.
        let mut older = Vec::new();
        for device in devices {
            if met.iter().any(|(known, _)| known.as_ref() == Some(device)) {
                continue;
            }
            let Ok(current) = stream.current(Some(device)) else {
                continue;
            };
            if matches!(stream.next(&newest, Some(device)), Err(code) if code == GAMEINPUT_E_READING_NOT_FOUND) {
                older.push((Some(device.clone()), current));
            }
        }
        self.last = Some(newest);
        older
            .into_iter()
            .chain(met)
            .filter(|(device, reading)| !self.latest.iter().any(|(known, latest)| known == device && latest == reading))
            .map(|(_, reading)| reading)
            .collect()
    }
}

// GameInput's stream, read with GetCurrentReading, GetNextReading and
// GetPreviousReading of every kind that frames read, with the HRESULT that
// GameInput returns. GetNextReading fails every frame that has nothing new,
// and the bindings' Result makes each failure a windows_core::Error, which
// asks COM for the thread's error info.
impl Stream for IGameInput {
    type Reading = IGameInputReading;
    type Device = IGameInputDevice;

    fn current(&self, device: Option<&IGameInputDevice>) -> Result<IGameInputReading, HRESULT> {
        let mut reading = std::ptr::null_mut();
        let device = device.map_or(std::ptr::null_mut(), |device| device.as_raw());
        unsafe {
            let code = (Interface::vtable(self).GetCurrentReading)(self.as_raw(), KINDS, device, &mut reading);
            returned(code, reading)
        }
    }

    fn next(&self, after: &IGameInputReading, device: Option<&IGameInputDevice>) -> Result<IGameInputReading, HRESULT> {
        let mut reading = std::ptr::null_mut();
        let device = device.map_or(std::ptr::null_mut(), |device| device.as_raw());
        unsafe {
            let code = (Interface::vtable(self).GetNextReading)(self.as_raw(), after.as_raw(), KINDS, device, &mut reading);
            returned(code, reading)
        }
    }

    fn previous(&self, before: &IGameInputReading) -> Result<IGameInputReading, HRESULT> {
        let mut reading = std::ptr::null_mut();
        unsafe {
            let code = (Interface::vtable(self).GetPreviousReading)(self.as_raw(), before.as_raw(), KINDS, std::ptr::null_mut(), &mut reading);
            returned(code, reading)
        }
    }

    fn device(&self, reading: &IGameInputReading) -> Option<IGameInputDevice> {
        unsafe { reading.GetDevice() }
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
