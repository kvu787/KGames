// What the Devices panel's tooltip says about a device, from what GameInput
// and Windows' device tree say about it. A test gives the device's data, so
// it doesn't need the devices themselves.

use game_input::*;
use game_input_demo::device_tree::{Node, Tree};
use game_input_demo::devices::{Device, Section};
use game_input_demo::input::Input;
use game_input_demo::scripted_source::ScriptedSource;
use windows_core::GUID;

// A keyboard from a script, which has only the data that scripts give.
fn keyboard() -> Device {
    let mut input = Input::new(Box::new(ScriptedSource::default()), 0.1);
    let step = r#"{ "Connect": { "Id": 1, "Name": "Keys", "Input": ["Keyboard"] } }"#;
    input.scripted().unwrap().apply(&json::parse(step).unwrap()).unwrap();
    input.update(true);
    input.devices[0].clone()
}

fn node(instance_id: &str, name: &str, class: &str) -> Node {
    Node {
        instance_id: instance_id.to_string(),
        name: name.to_string(),
        class: class.to_string(),
        ..Default::default()
    }
}

fn section<'a>(sections: &'a [Section], title: &str) -> &'a Section {
    sections
        .iter()
        .find(|section| section.title == title)
        .unwrap_or_else(|| panic!("no {title} section"))
}

fn value<'a>(section: &'a Section, name: &str) -> &'a str {
    section
        .rows
        .iter()
        .find(|(row, _)| row == name)
        .map(|(_, value)| value.as_str())
        .unwrap_or_else(|| panic!("{} has no {name}", section.title))
}

fn titles(sections: &[Section]) -> Vec<&str> {
    sections.iter().map(|section| section.title.as_str()).collect()
}

#[test]
fn a_scripted_device_says_why_windows_has_nothing_on_it() {
    let sections = keyboard().details();
    assert_eq!(titles(&sections), ["GameInput", "Windows"]);
    let game_input = section(&sections, "GameInput");
    assert_eq!(value(game_input, "Family"), "virtual");
    assert_eq!(value(game_input, "Input"), "keyboard");
    assert_eq!(value(game_input, "IDs"), "vendor 0x0000, product 0x0000, revision 0x0000");
    assert_eq!(value(game_input, "Container"), "none");
    assert_eq!(value(game_input, "PnP path"), "none");
    let windows = section(&sections, "Windows");
    assert_eq!(windows.rows, [("Not found".to_string(), "the device comes from a script".to_string())]);
}

#[test]
fn names_what_game_input_says() {
    let mut device = keyboard();
    device.vendor_id = 0x29EA;
    device.product_id = 0x0360;
    device.revision = 0x64;
    device.usage = GameInputUsage { page: 0x01, id: 0x06 };
    device.firmware_version = GameInputVersion { major: 1, minor: 2, build: 3, revision: 4 };
    device.container_id = GUID::from_u128(0x00000000_0000_0000_ffff_ffffffffffff);
    device.keyboard = Some(GameInputKeyboardInfo {
        kind: GameInputIsoKeyboard,
        layout: 0x409,
        keyCount: 104,
        functionKeyCount: 12,
        maxSimultaneousKeys: 6,
        platformType: 4,
        platformSubtype: 0,
    });
    device.mouse = Some(GameInputMouseInfo {
        supportedButtons: GameInputMouseLeftButton | GameInputMouseRightButton,
        sampleRate: 1000,
        hasWheelX: false,
        hasWheelY: true,
    });
    let sections = device.details();
    let game_input = section(&sections, "GameInput");
    assert_eq!(value(game_input, "IDs"), "vendor 0x29EA, product 0x0360, revision 0x0064");
    assert_eq!(value(game_input, "HID usage"), "Generic Desktop: Keyboard (page 0x01, usage 0x06)");
    assert_eq!(value(game_input, "Versions"), "hardware 0.0.0.0, firmware 1.2.3.4");
    assert_eq!(
        value(game_input, "Container"),
        "{00000000-0000-0000-FFFF-FFFFFFFFFFFF}, built into this computer"
    );
    assert_eq!(
        value(game_input, "Keyboard"),
        "ISO, layout 0x00000409, 104 keys, 12 function keys, 6 at once, type 4, subtype 0"
    );
    assert_eq!(value(game_input, "Mouse"), "buttons Left, Right, wheel, 1000 Hz");
    // Only what the device has.
    for missing in ["Gamepad", "Rumble", "System buttons", "XInput index"] {
        assert!(game_input.rows.iter().all(|(name, _)| name != missing), "{missing} is shown");
    }
}

#[test]
fn an_unnamed_usage_shows_its_numbers() {
    let mut device = keyboard();
    device.usage = GameInputUsage { page: 0xFF00, id: 0x01 };
    let sections = device.details();
    assert_eq!(value(section(&sections, "GameInput"), "HID usage"), "page 0xFF00, usage 0x01");
}

#[test]
fn shows_the_device_node_then_what_it_is_connected_through() {
    let mut device = keyboard();
    let mut keyboard_node = node(
        r"HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000",
        "HID Keyboard Device",
        "Keyboard",
    );
    keyboard_node.manufacturer = "(Standard keyboards)".to_string();
    keyboard_node.service = "kbdhid".to_string();
    keyboard_node.inf = "keyboard.inf".to_string();
    keyboard_node.hardware_ids = vec![r"HID\VID_29EA&PID_0360&MI_01".to_string(), "HID_DEVICE".to_string()];
    let mut interface = node(r"USB\VID_29EA&PID_0360&MI_01\6&1952A169&0&0001", "USB Input Device", "HIDClass");
    interface.bus_name = "Kinesis Adv360".to_string();
    let mut composite = node(r"USB\VID_29EA&PID_0360\360555127546", "USB Composite Device", "USB");
    composite.bus_name = "Kinesis Adv360".to_string();
    composite.location = "Port_#0001.Hub_#0002".to_string();
    device.tree = Tree::known(Ok(vec![keyboard_node, interface, composite]));

    let sections = device.details();
    assert_eq!(titles(&sections), ["GameInput", "Windows", "Connected through, nearest first"]);
    let windows = section(&sections, "Windows");
    assert_eq!(value(windows, "Instance ID"), r"HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000");
    assert_eq!(value(windows, "Name"), "HID Keyboard Device");
    assert_eq!(value(windows, "Manufacturer"), "(Standard keyboards)");
    assert_eq!(value(windows, "Driver"), "kbdhid");
    assert_eq!(value(windows, "INF file"), "keyboard.inf");
    // One line each.
    assert_eq!(value(windows, "Hardware IDs"), "HID\\VID_29EA&PID_0360&MI_01\nHID_DEVICE");
    // It doesn't report a name on its bus, or say where it is.
    assert!(windows.rows.iter().all(|(name, _)| name != "Bus name" && name != "Location"));

    let through = section(&sections, "Connected through, nearest first");
    assert_eq!(
        through.rows,
        [
            (
                "HIDClass".to_string(),
                "USB Input Device, \"Kinesis Adv360\"\nUSB\\VID_29EA&PID_0360&MI_01\\6&1952A169&0&0001".to_string()
            ),
            (
                "USB".to_string(),
                "USB Composite Device, \"Kinesis Adv360\", at Port_#0001.Hub_#0002\nUSB\\VID_29EA&PID_0360\\360555127546"
                    .to_string()
            ),
        ]
    );
}

#[test]
fn a_device_node_without_a_driver_says_so() {
    let mut device = keyboard();
    device.tree = Tree::known(Ok(vec![node(r"HID\INTC816&COL01\3&125F27AF&0&0000", "", "")]));
    let sections = device.details();
    assert_eq!(titles(&sections), ["GameInput", "Windows"]);
    let windows = section(&sections, "Windows");
    for name in ["Name", "Manufacturer", "Class", "Driver", "Hardware IDs"] {
        assert_eq!(value(windows, name), "none", "{name}");
    }
    assert!(windows.rows.iter().all(|(name, _)| name != "INF file"));
}

#[test]
fn says_while_windows_is_looked_up() {
    let mut device = keyboard();
    device.tree = Tree::default();
    let sections = device.details();
    let windows = section(&sections, "Windows");
    assert_eq!(windows.rows, [("Device tree".to_string(), "looking it up".to_string())]);
}

// Why a lookup found nothing, once its thread is done.
fn problem(pnp_path: &str) -> String {
    let tree = Tree::look_up(pnp_path.to_string());
    let start = std::time::Instant::now();
    while tree.get().is_none() {
        assert!(start.elapsed().as_secs() < 10, "the lookup didn't finish");
        std::thread::yield_now();
    }
    tree.get().unwrap().clone().unwrap_err()
}

#[test]
fn a_lookup_says_why_it_found_nothing() {
    assert_eq!(problem(""), "GameInput gives no PnP path");
    let missing = problem(r"\\?\HID#NoSuchDevice#0#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}");
    assert!(missing.starts_with("Windows has no device interface at the PnP path"), "{missing}");
}
