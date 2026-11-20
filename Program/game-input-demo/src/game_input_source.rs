// Devices and readings from GameInput, as Library/user-interface reads them
// for every program: once a frame, right before the frame, with every reading
// since the last one, in the order of GameInput's stream. The demo shows the
// same readings that the interface makes its events from, and runs gamepads'
// motors through the library.
//
// input.rs keeps each device's last reading itself, to say what changed.

use std::cell::RefCell;
use std::rc::Rc;

use game_input::Runtime;
use game_input_raw::*;
use user_interface::Devices;

use crate::device_tree::Tree;
use crate::devices::{Device, InputSource, Reading};

pub struct GameInputSource {
    devices: Rc<RefCell<Devices>>,
    runtime: Runtime,
    // The last frame whose device events, readings and focus changes were
    // handed out, so that each is handed out once.
    events_frame: u64,
    readings_frame: u64,
    focus_frame: u64,
    // Every device seen, with its node in Windows' device tree, which is
    // looked up once.
    described: Vec<Device>,
    // The devices connected, as the device events handed out say.
    handed: Vec<usize>,
}

// What a source hands out instead of the device events of frames that it
// didn't see, from `handed`, the devices that it said were connected, and
// the devices connected now: the ones gone, which disconnected, and the
// ones that connected, in the order they did. A device that went and came
// back meanwhile is neither.
pub fn missed_events(handed: &[usize], connected: &[user_interface::Device]) -> (Vec<usize>, Vec<user_interface::Device>) {
    let gone = handed.iter().copied().filter(|id| !connected.iter().any(|device| device.id == *id)).collect();
    let came = connected.iter().filter(|device| !handed.contains(&device.id)).cloned().collect();
    (gone, came)
}

impl GameInputSource {
    pub fn new(devices: Rc<RefCell<Devices>>) -> Self {
        let runtime = {
            let devices = devices.borrow();
            Runtime {
                path: devices.runtime().path.clone(),
                version: devices.runtime().version,
            }
        };
        Self {
            devices,
            runtime,
            events_frame: 0,
            readings_frame: 0,
            focus_frame: 0,
            described: Vec::new(),
            handed: Vec::new(),
        }
    }

    // The demo's data for a device, with its node in the device tree.
    fn describe(&mut self, device: &user_interface::Device, changed: bool) -> Device {
        if changed {
            self.described.retain(|known| known.id != device.id);
        }
        if let Some(known) = self.described.iter().find(|known| known.id == device.id) {
            return known.clone();
        }
        let described = Device::from_game_input(device, Tree::look_up(device.pnp_path.clone()));
        self.described.push(described.clone());
        described
    }
}

impl InputSource for GameInputSource {
    fn runtime(&self) -> Option<&Runtime> {
        Some(&self.runtime)
    }

    fn now(&mut self) -> u64 {
        self.devices.borrow().now()
    }

    // The library's devices hand out each frame's device events in that
    // frame only, while the demo runs only in the frames that show it, not
    // in those that show the error page of a wrong window setup. So after
    // frames that it didn't run, it hands out what the devices connected now
    // say happened instead, which would otherwise be lost for good, like the
    // devices that were connected as it started.
    #[allow(non_upper_case_globals)]
    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)> {
        let (events, gone) = {
            let devices = self.devices.borrow();
            if devices.frame() == self.events_frame {
                return Vec::new();
            }
            let missed = devices.frame() > self.events_frame + 1;
            self.events_frame = devices.frame();
            if missed {
                let (gone, came) = missed_events(&self.handed, devices.connected());
                (came.into_iter().map(|device| (GameInputDeviceEventConnected, device)).collect(), gone)
            } else {
                (devices.device_events().to_vec(), Vec::new())
            }
        };
        let gone = gone.into_iter().filter_map(|id| self.described.iter().find(|known| known.id == id).cloned());
        let mut handed: Vec<_> = gone.map(|device| (GameInputDeviceEventDisconnected, device)).collect();
        for (event, device) in events {
            // A device that connects, or whose keyboard layout or XInput
            // index changes, is described again.
            let changed = matches!(
                event,
                GameInputDeviceEventConnected | GameInputDeviceEventKeyboardLayoutUpdated | GameInputDeviceEventXInputIndexUpdated
            );
            handed.push((event, self.describe(&device, changed)));
        }
        for (event, device) in &handed {
            match *event {
                GameInputDeviceEventConnected if !self.handed.contains(&device.id) => self.handed.push(device.id),
                GameInputDeviceEventDisconnected => self.handed.retain(|id| *id != device.id),
                _ => {}
            }
        }
        handed
    }

    // The library's devices say, with the times of their focus changes.
    fn focus(&mut self) -> (bool, bool) {
        (self.devices.borrow().focused(), false)
    }

    fn focus_changes(&mut self) -> Vec<(u64, bool)> {
        let devices = self.devices.borrow();
        if devices.frame() == self.focus_frame {
            return Vec::new();
        }
        self.focus_frame = devices.frame();
        devices.focus_changes().to_vec()
    }

    fn readings(&mut self) -> Vec<Reading> {
        let devices = self.devices.borrow();
        if devices.frame() == self.readings_frame {
            return Vec::new();
        }
        self.readings_frame = devices.frame();
        devices.readings().to_vec()
    }

    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>) {
        self.devices.borrow_mut().set_rumble(device, params);
    }
}
