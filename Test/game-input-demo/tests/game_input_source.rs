// The demo's devices from GameInput, as the library reads them: the library
// hands out each frame's device events in that frame only, while the demo
// runs only in the frames that show it, not in those that show the error
// page of a wrong window setup, so it catches up on what it missed from the
// devices connected now.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use game_input_demo::game_input_source::{GameInputSource, missed_events};
use game_input_demo::input::Input;

fn device(id: usize, label: &str) -> user_interface::Device {
    user_interface::Device {
        id,
        label: label.to_string(),
        ..Default::default()
    }
}

// Devices that went meanwhile disconnected, and those that came connected,
// in the order they did. One that went and came back is neither.
#[test]
fn what_frames_that_the_demo_missed_did_to_the_devices() {
    let connected = [device(5, "Pad"), device(2, "Keys"), device(7, "Rodent")];
    let (gone, came) = missed_events(&[1, 2, 3], &connected);
    assert_eq!(gone, [1, 3]);
    assert_eq!(came.iter().map(|device| device.id).collect::<Vec<_>>(), [5, 7]);
    let (gone, came) = missed_events(&[5, 2, 7], &connected);
    assert!(gone.is_empty() && came.is_empty());
}

// GameInput reports the devices that are connected as it starts as
// connecting, in the first frames, which a demo that starts in a wrong
// window setup doesn't run in. It shows them once it runs, rather than
// none, and drops no reading of them as one of a device that isn't
// connected. Needs GameInput 4.0.306, as every program does, and a device
// connected, like this computer's keyboard. A test doesn't have focus, so
// it reads no readings.
#[test]
fn the_demo_shows_the_devices_that_connected_in_frames_that_it_did_not_run() {
    let devices = Rc::new(RefCell::new(user_interface::Devices::new().unwrap()));
    // The frames of the error page, until GameInput has reported a device.
    let start = Instant::now();
    loop {
        devices.borrow_mut().read(&[]);
        if devices.borrow().frame() >= 2 && !devices.borrow().connected().is_empty() {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(10), "GameInput reported no device: this test needs one connected");
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut input = Input::new(Box::new(GameInputSource::new(devices.clone())), 0.1);
    // The first frame that the demo runs in.
    devices.borrow_mut().read(&[]);
    input.update(true);
    let mut connected: Vec<usize> = devices.borrow().connected().iter().map(|device| device.id).collect();
    let mut shown: Vec<usize> = input.devices.iter().map(|device| device.id).collect();
    connected.sort();
    shown.sort();
    assert_eq!(shown, connected);
}
