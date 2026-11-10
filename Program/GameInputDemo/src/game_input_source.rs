// Devices and readings from GameInput, as Library/UserInterface reads them
// for every program: once a frame, right before the frame, with every reading
// since the last one. The demo shows the same readings that the interface
// makes its events from, and runs gamepads' motors through the library.
//
// input.rs keeps each device's last reading itself, to say what changed.

use std::cell::RefCell;
use std::rc::Rc;

use game_input::*;
use user_interface::Devices;

use crate::device_tree::Tree;
use crate::devices::{Device, InputSource, Reading};

pub struct GameInputSource {
    devices: Rc<RefCell<Devices>>,
    runtime: Runtime,
    // The last frame whose device events, and whose readings of each kind,
    // were handed out, so that each is handed out once.
    events_frame: u64,
    readings_frames: Vec<(GameInputKind, u64)>,
    // Every device seen, with its node in Windows' device tree, which is
    // looked up once.
    described: Vec<Device>,
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
            readings_frames: Vec::new(),
            described: Vec::new(),
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

    #[allow(non_upper_case_globals)]
    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)> {
        let events = {
            let devices = self.devices.borrow();
            if devices.frame() == self.events_frame {
                return Vec::new();
            }
            self.events_frame = devices.frame();
            devices.device_events().to_vec()
        };
        events
            .into_iter()
            .map(|(event, device)| {
                // A device that connects, or whose keyboard layout or XInput
                // index changes, is described again.
                let changed = matches!(
                    event,
                    GameInputDeviceEventConnected | GameInputDeviceEventKeyboardLayoutUpdated | GameInputDeviceEventXInputIndexUpdated
                );
                (event, self.describe(&device, changed))
            })
            .collect()
    }

    fn readings(&mut self, kind: GameInputKind) -> Vec<Reading> {
        let devices = self.devices.borrow();
        let frame = devices.frame();
        match self.readings_frames.iter_mut().find(|(known, _)| *known == kind) {
            Some((_, taken)) if *taken == frame => return Vec::new(),
            Some((_, taken)) => *taken = frame,
            None => self.readings_frames.push((kind, frame)),
        }
        devices.readings(kind).to_vec()
    }

    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>) {
        self.devices.borrow_mut().set_rumble(device, params);
    }
}
