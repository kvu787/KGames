// Where a device is in Windows' device tree, from the PnP path that GameInput
// gives: its node, and the nodes above it, which say what it's connected
// through, like a USB hub, or a driver that makes it up from another device.
// Windows' configuration manager answers from what Plug and Play already
// knows, without asking the device, but it still takes a few milliseconds
// for a device, so a thread of its own looks each one up.

use std::sync::{Arc, OnceLock};

use windows_api::*;

// A device's nodes, nearest first, or why there are none, once they're
// looked up. Clones share the lookup.
#[derive(Clone, Debug, Default)]
pub struct Tree(Arc<OnceLock<Result<Vec<Node>, String>>>);

impl Tree {
    // Starts looking up the nodes for a PnP path, on another thread.
    pub fn look_up(pnp_path: String) -> Self {
        let tree = Self::default();
        let result = tree.0.clone();
        std::thread::spawn(move || result.set(nodes(&pnp_path)));
        tree
    }

    // Nodes that are already known, or why there are none.
    pub fn known(result: Result<Vec<Node>, String>) -> Self {
        Self(Arc::new(OnceLock::from(result)))
    }

    // The nodes, or why there are none, or None while they're looked up.
    pub fn get(&self) -> Option<&Result<Vec<Node>, String>> {
        self.0.get()
    }
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.get() == other.get()
    }
}

// A node in the device tree, with what Device Manager shows about it. Each
// is empty when Windows doesn't have it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    // Like "HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000".
    pub instance_id: String,
    // Its friendly name, or else its description, like "USB Input Device".
    pub name: String,
    // The name that the device itself reports on its bus, like a USB
    // device's product string.
    pub bus_name: String,
    pub manufacturer: String,
    // Its setup class, like "Keyboard" or "HIDClass".
    pub class: String,
    // The driver's service, like "kbdhid", and the INF file that installed it.
    pub service: String,
    pub inf: String,
    // Where it is on its bus, like "Port_#0001.Hub_#0002".
    pub location: String,
    pub hardware_ids: Vec<String>,
}

// Nodes in these classes are the computer and its buses, which every device
// is connected through, so the walk up the tree stops at the first.
const SYSTEM_CLASSES: [&str; 2] = ["System", "Computer"];

// The device's node, then the nodes above it, nearest first, up to the first
// one in the System or Computer class. Or why Windows doesn't know it.
fn nodes(pnp_path: &str) -> Result<Vec<Node>, String> {
    if pnp_path.is_empty() {
        return Err("GameInput gives no PnP path".to_string());
    }
    let path = wide(pnp_path);
    let mut instance_id = Vec::new();
    unsafe {
        read_property(&mut instance_id, |kind, buffer, size| {
            CM_Get_Device_Interface_PropertyW(
                windows_core::PCWSTR(path.as_ptr()),
                &DEVPKEY_Device_InstanceId,
                kind,
                buffer,
                size,
                0,
            )
        })
        .map_err(|code| format!("Windows has no device interface at the PnP path ({code})"))?;
        let mut node = DEVNODE::default();
        let code = CM_Locate_DevNodeW(
            &mut node,
            Some(instance_id.as_ptr()),
            CM_LOCATE_DEVNODE_NORMAL as u32,
        );
        if code.0 != CR_SUCCESS as u32 {
            return Err(format!(
                "Windows has no device {} ({})",
                text(&instance_id),
                describe(code)
            ));
        }

        let mut nodes = Vec::new();
        let mut current = DEVINST(node.0);
        loop {
            let class = string(current, &DEVPKEY_Device_Class);
            if !nodes.is_empty() && SYSTEM_CLASSES.contains(&class.as_str()) {
                break;
            }
            let friendly_name = string(current, &DEVPKEY_Device_FriendlyName);
            nodes.push(Node {
                instance_id: string(current, &DEVPKEY_Device_InstanceId),
                name: if friendly_name.is_empty() {
                    string(current, &DEVPKEY_Device_DeviceDesc)
                } else {
                    friendly_name
                },
                bus_name: string(current, &DEVPKEY_Device_BusReportedDeviceDesc),
                manufacturer: string(current, &DEVPKEY_Device_Manufacturer),
                class,
                service: string(current, &DEVPKEY_Device_Service),
                inf: string(current, &DEVPKEY_Device_DriverInfPath),
                location: string(current, &DEVPKEY_Device_LocationInfo),
                hardware_ids: strings(current, &DEVPKEY_Device_HardwareIds),
            });
            // The root of the tree has no parent.
            let mut parent = DEVNODE::default();
            if CM_Get_Parent(&mut parent, current, 0).0 != CR_SUCCESS as u32 {
                break;
            }
            current = DEVINST(parent.0);
        }
        Ok(nodes)
    }
}

// A node's property that's a string, or "" if it doesn't have it.
unsafe fn string(node: DEVINST, key: &DEVPROPKEY) -> String {
    unsafe { strings(node, key) }.into_iter().next().unwrap_or_default()
}

// A node's property that's a string or a list of them, or none if it
// doesn't have it.
unsafe fn strings(node: DEVINST, key: &DEVPROPKEY) -> Vec<String> {
    let mut buffer = Vec::new();
    let read = unsafe {
        read_property(&mut buffer, |kind, data, size| {
            CM_Get_DevNode_PropertyW(node, key, kind, data, size, 0)
        })
    };
    match read {
        Ok(()) => buffer
            .split(|&unit| unit == 0)
            .filter(|item| !item.is_empty())
            .map(String::from_utf16_lossy)
            .collect(),
        Err(_) => Vec::new(),
    }
}

// Reads a property of a string type into `buffer`, as UTF-16 that ends in a
// zero, or a list of such strings that ends in another. `get` is the
// configuration manager's function for it, which first says how big it is.
unsafe fn read_property(
    buffer: &mut Vec<u16>,
    get: impl Fn(*mut DEVPROPTYPE, Option<*mut u8>, *mut u32) -> CONFIGRET,
) -> Result<(), String> {
    let mut kind = DEVPROPTYPE::default();
    let mut size = 0u32;
    let code = get(&mut kind, None, &mut size);
    if code.0 != CR_BUFFER_SMALL as u32 {
        return Err(describe(code));
    }
    buffer.resize((size as usize).div_ceil(2), 0);
    let code = get(&mut kind, Some(buffer.as_mut_ptr().cast()), &mut size);
    if code.0 != CR_SUCCESS as u32 {
        return Err(describe(code));
    }
    if !matches!(kind.0 as i32, DEVPROP_TYPE_STRING | DEVPROP_TYPE_STRING_LIST) {
        return Err(format!("property of type {}", kind.0));
    }
    Ok(())
}

fn describe(code: CONFIGRET) -> String {
    format!("CONFIGRET {}", code.0)
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

// UTF-16 that ends in a zero, as a String.
fn text(units: &[u16]) -> String {
    let end = units.iter().position(|&unit| unit == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}
