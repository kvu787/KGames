// Devices and readings from GameInput itself.
//
// GameInput doesn't report events. It keeps a short history of readings,
// where each reading is a snapshot of one device's whole state with a
// timestamp. This walks that history with GetNextReading, so no reading is
// missed between frames. Input.rs keeps each device's last reading itself,
// to say what changed, as GetPreviousReading fails once the reading before
// has left the history, like after a pause in input.

use std::sync::Mutex;

use game_input::*;
use windows_core::{Interface, Result};

use crate::devices::{Device, InputSource, Reading};

// The device callback runs on one of GameInput's threads. It only queues the
// event, and the main thread takes it in device_events.
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

pub struct GameInputSource {
    game_input: IGameInput,
    runtime: Runtime,
    callback_token: GameInputCallbackToken,
    // The last reading handed out, for each kind of input.
    last_readings: Vec<(GameInputKind, IGameInputReading)>,
    // Every device seen, as GameInput's object and as data. GameInput keeps
    // one object per device, so the object's address tells devices apart.
    devices: Vec<(IGameInputDevice, Device)>,
}

impl GameInputSource {
    // Loads the newest GameInput runtime, and starts listening for gamepads,
    // keyboards and mice. The callback also reports the devices that are
    // already connected.
    pub fn new() -> Result<Self> {
        let (game_input, runtime) = game_input::create()?;
        unsafe {
            let callback_token = game_input.RegisterDeviceCallback(
                None,
                GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse,
                None,
                Some(on_device_event),
            )?;
            Ok(Self {
                game_input,
                runtime,
                callback_token,
                last_readings: Vec::new(),
                devices: Vec::new(),
            })
        }
    }

    // The device's data, from what's known about it, or from GameInput.
    unsafe fn device(&mut self, device: &IGameInputDevice) -> Device {
        match self.known(device) {
            Some(index) => self.devices[index].1.clone(),
            None => {
                let described = unsafe { describe(device) };
                self.devices.push((device.clone(), described.clone()));
                described
            }
        }
    }

    // The device's ID, for a reading of it. Its object is kept, even if
    // its device event hasn't come yet, so that no other device can get its
    // address, and with it its ID.
    unsafe fn device_id(&mut self, device: &IGameInputDevice) -> usize {
        if self.known(device).is_none() {
            let described = unsafe { describe(device) };
            self.devices.push((device.clone(), described));
        }
        device.as_raw() as usize
    }

    fn known(&self, device: &IGameInputDevice) -> Option<usize> {
        self.devices.iter().position(|(object, _)| object == device)
    }

    unsafe fn object(&self, id: usize) -> Option<&IGameInputDevice> {
        self.devices
            .iter()
            .find(|(_, device)| device.id == id)
            .map(|(object, _)| object)
    }

    unsafe fn reading(&mut self, reading: &IGameInputReading) -> Reading {
        unsafe {
            let kinds = reading.GetInputKind();
            let mut data = Reading {
                device: reading.GetDevice().map(|device| self.device_id(&device)),
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
                if let Ok(current) = self.game_input.GetCurrentReading(kind, None) {
                    self.last_readings.push((kind, current.clone()));
                    readings.push(current);
                }
                return readings;
            };
            let mut last = self.last_readings[index].1.clone();
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
                self.last_readings[index].1 = newest.clone();
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
}

impl InputSource for GameInputSource {
    fn runtime(&self) -> Option<&Runtime> {
        Some(&self.runtime)
    }

    fn now(&mut self) -> u64 {
        unsafe { self.game_input.GetCurrentTimestamp() }
    }

    #[allow(non_upper_case_globals)]
    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)> {
        let events: Vec<_> = DEVICE_EVENTS.lock().unwrap().drain(..).collect();
        let mut result = Vec::new();
        unsafe {
            for (event, object) in events {
                // A device that connects, or whose XInput index changes, is
                // described again.
                if matches!(
                    event,
                    GameInputDeviceEventConnected | GameInputDeviceEventXInputIndexUpdated
                ) {
                    self.devices.retain(|(known, _)| *known != object);
                }
                result.push((event, self.device(&object)));
            }
        }
        result
    }

    fn readings(&mut self, kind: GameInputKind) -> Vec<Reading> {
        unsafe {
            let readings = self.new_readings(kind);
            readings.iter().map(|reading| self.reading(reading)).collect()
        }
    }

    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>) {
        unsafe {
            if let Some(object) = self.object(device) {
                object.SetRumbleState(params);
            }
        }
    }
}

impl Drop for GameInputSource {
    fn drop(&mut self) {
        unsafe { self.game_input.UnregisterCallback(self.callback_token) };
    }
}

// A device's data, from GameInput.
unsafe fn describe(device: &IGameInputDevice) -> Device {
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
        Device {
            id: device.as_raw() as usize,
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
