// What the demo reads, as plain data: the devices, and readings of their
// state. Input.rs works on this data only, from an InputSource: GameInput
// itself (GameInputSource.rs), or a script or a test (ScriptedSource.rs), so
// that tests can give the demo any readings, like a gamepad's, without one.

use game_input::*;
use windows_core::GUID;

use crate::device_tree::Tree;
use crate::input::{GAMEPAD_BUTTONS, MOUSE_BUTTONS};
use crate::scripted_source::ScriptedSource;

// A gamepad, keyboard or mouse that GameInput reports, with everything that
// GameInput and Windows say about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    // Tells devices apart, and stays the same while a device is connected.
    pub id: usize,
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
    // Its node in Windows' device tree, then the nodes above it, or why
    // there are none, once they're looked up.
    pub tree: Tree,
}

// Part of what the Devices panel's tooltip says about a device: a title,
// and values with their names. A value can have more than one line.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

// Windows gives devices built into the computer this container.
const BUILT_IN_CONTAINER: GUID = GUID::from_u128(0x00000000_0000_0000_ffff_ffffffffffff);

impl Device {
    // Matching on GameInput's constants needs their native names allowed.
    #[allow(non_upper_case_globals)]
    pub fn family_name(&self) -> &'static str {
        match self.family {
            GameInputFamilyVirtual => "virtual",
            GameInputFamilyXboxOne => "Xbox One",
            GameInputFamilyXbox360 => "Xbox 360",
            GameInputFamilyHid => "HID",
            GameInputFamilyI8042 => "i8042 (PS/2)",
            GameInputFamilyAggregate => "aggregate",
            _ => "unknown",
        }
    }

    pub fn kinds(&self) -> String {
        crate::input::kind_names(self.supported_input)
    }

    pub fn is_gamepad(&self) -> bool {
        self.supported_input & GameInputKindGamepad != 0
    }

    // How often it says it sends mouse input, if it does.
    pub fn mouse_sample_rate(&self) -> Option<u32> {
        self.mouse
            .map(|mouse| mouse.sampleRate)
            .filter(|&rate| rate > 0)
    }

    // What the Devices panel's tooltip says about it: what GameInput says,
    // then Windows' node for it, and what that's connected through.
    pub fn details(&self) -> Vec<Section> {
        let mut rows = vec![
            ("Family".to_string(), self.family_name().to_string()),
            ("Input".to_string(), self.kinds()),
            (
                "IDs".to_string(),
                format!(
                    "vendor 0x{:04X}, product 0x{:04X}, revision 0x{:04X}",
                    self.vendor_id, self.product_id, self.revision
                ),
            ),
            ("HID usage".to_string(), usage_name(self.usage)),
            (
                "Versions".to_string(),
                format!(
                    "hardware {}, firmware {}",
                    version(self.hardware_version),
                    version(self.firmware_version)
                ),
            ),
        ];
        if let Some(keyboard) = self.keyboard {
            rows.push(("Keyboard".to_string(), keyboard_info(&keyboard)));
        }
        if let Some(mouse) = self.mouse {
            rows.push(("Mouse".to_string(), mouse_info(&mouse)));
        }
        if let Some(gamepad) = self.gamepad {
            rows.push(("Gamepad".to_string(), gamepad_info(&gamepad)));
        }
        if let Some(index) = self.xinput_index {
            rows.push(("XInput index".to_string(), index.to_string()));
        }
        if self.rumble_motors != 0 {
            rows.push(("Rumble".to_string(), rumble_motor_names(self.rumble_motors)));
        }
        if self.system_buttons != 0 {
            rows.push((
                "System buttons".to_string(),
                names(self.system_buttons, SYSTEM_BUTTONS),
            ));
        }
        let mut container = format!("{{{:?}}}", self.container_id);
        if self.container_id == BUILT_IN_CONTAINER {
            container += ", built into this computer";
        } else if self.container_id == GUID::zeroed() {
            container = "none".to_string();
        }
        rows.push(("Container".to_string(), container));
        rows.push(("PnP path".to_string(), or_none(&self.pnp_path)));
        let mut sections = vec![Section {
            title: "GameInput".to_string(),
            rows,
        }];

        let nodes = match self.tree.get() {
            Some(Ok(nodes)) => nodes.as_slice(),
            Some(Err(problem)) => {
                sections.push(Section {
                    title: "Windows".to_string(),
                    rows: vec![("Not found".to_string(), problem.clone())],
                });
                return sections;
            }
            None => {
                sections.push(Section {
                    title: "Windows".to_string(),
                    rows: vec![("Device tree".to_string(), "looking it up".to_string())],
                });
                return sections;
            }
        };
        if let Some(node) = nodes.first() {
            let mut rows = vec![
                ("Instance ID".to_string(), node.instance_id.clone()),
                ("Name".to_string(), or_none(&node.name)),
            ];
            if !node.bus_name.is_empty() {
                rows.push(("Bus name".to_string(), node.bus_name.clone()));
            }
            rows.extend([
                ("Manufacturer".to_string(), or_none(&node.manufacturer)),
                ("Class".to_string(), or_none(&node.class)),
                ("Driver".to_string(), or_none(&node.service)),
            ]);
            if !node.inf.is_empty() {
                rows.push(("INF file".to_string(), node.inf.clone()));
            }
            if !node.location.is_empty() {
                rows.push(("Location".to_string(), node.location.clone()));
            }
            rows.push(("Hardware IDs".to_string(), or_none(&node.hardware_ids.join("\n"))));
            sections.push(Section {
                title: "Windows".to_string(),
                rows,
            });
        }
        // Each node above it by its class, then its names and where it is,
        // and its instance ID under them.
        if nodes.len() > 1 {
            let rows = nodes[1..]
                .iter()
                .map(|node| {
                    let mut text = node.name.clone();
                    if !node.bus_name.is_empty() && node.bus_name != node.name {
                        text += &format!(", \"{}\"", node.bus_name);
                    }
                    if !node.location.is_empty() {
                        text += &format!(", at {}", node.location);
                    }
                    (node.class.clone(), format!("{text}\n{}", node.instance_id))
                })
                .collect();
            sections.push(Section {
                title: "Connected through, nearest first".to_string(),
                rows,
            });
        }
        sections
    }
}

const SYSTEM_BUTTONS: &[(GameInputSystemButtons, &str)] = &[
    (GameInputSystemButtonGuide, "Guide"),
    (GameInputSystemButtonShare, "Share"),
];

// The names of the flags that are set, from `choices`, or "none".
fn names(flags: i32, choices: &[(i32, &str)]) -> String {
    let names: Vec<&str> = choices
        .iter()
        .filter(|(flag, _)| flags & flag != 0)
        .map(|(_, name)| *name)
        .collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

pub fn rumble_motor_names(motors: GameInputRumbleMotors) -> String {
    names(
        motors,
        &[
            (GameInputRumbleLowFrequency, "low frequency"),
            (GameInputRumbleHighFrequency, "high frequency"),
            (GameInputRumbleLeftTrigger, "left trigger"),
            (GameInputRumbleRightTrigger, "right trigger"),
        ],
    )
}

fn or_none(text: &str) -> String {
    if text.is_empty() {
        "none".to_string()
    } else {
        text.to_string()
    }
}

fn version(version: GameInputVersion) -> String {
    format!(
        "{}.{}.{}.{}",
        version.major, version.minor, version.build, version.revision
    )
}

// The usage page and usage, with their names from the HID Usage Tables for
// the ones that input devices have.
fn usage_name(usage: GameInputUsage) -> String {
    let name = match (usage.page, usage.id) {
        (0, 0) => return "none".to_string(),
        (0x01, 0x01) => "Generic Desktop: Pointer",
        (0x01, 0x02) => "Generic Desktop: Mouse",
        (0x01, 0x04) => "Generic Desktop: Joystick",
        (0x01, 0x05) => "Generic Desktop: Gamepad",
        (0x01, 0x06) => "Generic Desktop: Keyboard",
        (0x01, 0x07) => "Generic Desktop: Keypad",
        (0x01, 0x08) => "Generic Desktop: Multi-axis Controller",
        (0x01, 0x80) => "Generic Desktop: System Control",
        (0x0C, 0x01) => "Consumer: Consumer Control",
        (0x0D, 0x05) => "Digitizers: Touch Pad",
        _ => "",
    };
    let numbers = format!("page 0x{:02X}, usage 0x{:02X}", usage.page, usage.id);
    if name.is_empty() {
        numbers
    } else {
        format!("{name} ({numbers})")
    }
}

#[allow(non_upper_case_globals)]
fn keyboard_info(info: &GameInputKeyboardInfo) -> String {
    let kind = match info.kind {
        GameInputAnsiKeyboard => "ANSI",
        GameInputIsoKeyboard => "ISO",
        GameInputKsKeyboard => "KS (Korean)",
        GameInputAbntKeyboard => "ABNT (Brazilian)",
        GameInputJisKeyboard => "JIS (Japanese)",
        _ => "unknown kind",
    };
    format!(
        "{kind}, layout 0x{:08X}, {} keys, {} function keys, {} at once, type {}, subtype {}",
        info.layout,
        info.keyCount,
        info.functionKeyCount,
        info.maxSimultaneousKeys,
        info.platformType,
        info.platformSubtype
    )
}

fn mouse_info(info: &GameInputMouseInfo) -> String {
    let mut text = format!("buttons {}", names(info.supportedButtons, MOUSE_BUTTONS));
    if info.hasWheelY {
        text += ", wheel";
    }
    if info.hasWheelX {
        text += ", horizontal wheel";
    }
    if info.sampleRate > 0 {
        text += &format!(", {} Hz", info.sampleRate);
    }
    text
}

fn gamepad_info(info: &GameInputGamepadInfo) -> String {
    format!(
        "buttons {}, {} extra buttons, {} extra axes",
        names(info.supportedLayout, GAMEPAD_BUTTONS),
        info.extraButtonCount,
        info.extraAxisCount
    )
}

// A snapshot of one device's whole state, at a time. GameInput keeps a
// short history of these, and doesn't send events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    // The ID of the device it's from (see Device), or None when GameInput
    // doesn't say. The device's data comes with its device events, so a
    // reading doesn't copy it.
    pub device: Option<usize>,
    // On GameInput's clock, in microseconds.
    pub timestamp: u64,
    // Which kinds of input the reading has, like a gamepad's, or both a
    // keyboard's and a mouse's.
    pub kinds: GameInputKind,
    pub gamepad: GameInputGamepadState,
    pub keys: Vec<GameInputKeyState>,
    pub mouse: GameInputMouseState,
}

// Where devices and readings come from.
pub trait InputSource {
    // The GameInput runtime that was loaded, for the header, if any.
    fn runtime(&self) -> Option<&Runtime> {
        None
    }

    // The time now, on the readings' clock, in microseconds.
    fn now(&mut self) -> u64;

    // What happened to devices since the last call, oldest first: they
    // connected, disconnected, or changed.
    fn device_events(&mut self) -> Vec<(GameInputDeviceEvent, Device)>;

    // The readings with this kind of input since the last call, oldest
    // first. Input.rs compares each with the same device's reading before
    // it, which it keeps itself.
    fn readings(&mut self, kind: GameInputKind) -> Vec<Reading>;

    // Runs a gamepad's motors, or stops them with None.
    fn set_rumble(&mut self, device: usize, params: Option<&GameInputRumbleParams>);

    // The source, if it's a script's or a test's.
    fn as_scripted(&mut self) -> Option<&mut ScriptedSource> {
        None
    }
}
