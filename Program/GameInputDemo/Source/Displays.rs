// Finds the displays that show part of the desktop, and gives each one an
// identifier that stays the same from one run to the next, so a setting can
// name the display to open on. IdentifyDisplays shows each display's
// identifier on that display.
//
// The identifier is a hash of the monitor's device path, like
// \\?\DISPLAY#DEL40F2#5&2d3b6e3c&0&UID4352#{e6f07b5f-...}. Windows makes the
// path from the monitor's model, from its EDID, and from the GPU output it's
// plugged into. So two monitors of the same model get different identifiers,
// and a monitor keeps its identifier across restarts, but gets a new one if
// it's plugged into another output.

use crate::bindings::*;
use windows_core::{BOOL, Result};

// Crockford's Base32 alphabet. It leaves out I, L, O and U, so an identifier
// read off the screen has no characters that look alike.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
// 30 bits: the chance that two of five displays share an identifier is about
// one in a hundred million.
const IDENTIFIER_LENGTH: usize = 6;

pub struct Display {
    pub identifier: String,
    // The monitor's model from its EDID, like "DELL U2720Q".
    pub name: String,
    // What the monitor is plugged into, like "HDMI" or "Built in".
    pub connection: &'static str,
    // What the identifier is a hash of: the monitor's device path, or the
    // GDI device name, like \\.\DISPLAY1, if Windows doesn't report a monitor,
    // like over Remote Desktop. GDI device names can change between runs.
    pub device_path: String,
    pub monitor: HMONITOR,
    // The display's part of the desktop, and that part without the taskbar,
    // in physical pixels.
    pub area: RECT,
    pub work_area: RECT,
    pub primary: bool,
    pub dpi: u32,
    // In hertz.
    pub refresh_rate: Option<f64>,
}

impl Display {
    pub fn width(&self) -> i32 {
        self.area.right - self.area.left
    }

    pub fn height(&self) -> i32 {
        self.area.bottom - self.area.top
    }

    // Like "3840 x 2160, 144 Hz, 150% scale, DisplayPort".
    pub fn details(&self) -> String {
        let mut details = format!("{} x {}", self.width(), self.height());
        if let Some(rate) = self.refresh_rate {
            let rate = format!("{rate:.2}");
            details += &format!(", {} Hz", rate.trim_end_matches('0').trim_end_matches('.'));
        }
        let scale = (self.dpi * 100).div_ceil(USER_DEFAULT_SCREEN_DPI as u32);
        details += &format!(", {scale}% scale, {}", self.connection);
        details
    }
}

// The displays that show part of the desktop, from left to right.
pub fn enumerate() -> Result<Vec<Display>> {
    // Without QueryDisplayConfig's monitors, identifiers come from GDI device names.
    let targets = unsafe { targets() }.unwrap_or_default();
    let mut displays = Vec::new();
    for monitor in monitors()? {
        let mut info = MONITORINFOEXW::default();
        info.Base.cbSize = size_of::<MONITORINFOEXW>() as u32;
        unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast()) }.ok()?;
        let gdi_name = from_wide(&info.szDevice);
        let mut dpi = [USER_DEFAULT_SCREEN_DPI as u32; 2];
        unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi[0], &mut dpi[1]) }.ok()?;

        let display = |name: String, connection, device_path: String, refresh_rate| Display {
            identifier: identifier(&device_path),
            name,
            connection,
            device_path,
            monitor,
            area: info.Base.rcMonitor,
            work_area: info.Base.rcWork,
            primary: info.Base.dwFlags & MONITORINFOF_PRIMARY as u32 != 0,
            dpi: dpi[0],
            refresh_rate,
        };
        // A monitor that clones another shows the same part of the desktop,
        // so both are on one HMONITOR.
        let count = displays.len();
        for target in targets.iter().filter(|target| target.gdi_name == gdi_name) {
            let connection = connection(target.output_technology);
            let name = if !target.name.is_empty() {
                target.name.clone()
            } else if connection == "Built in" {
                "Built-in display".to_string()
            } else {
                "Unnamed display".to_string()
            };
            let device_path = if target.device_path.is_empty() {
                gdi_name.clone()
            } else {
                target.device_path.clone()
            };
            displays.push(display(name, connection, device_path, target.refresh_rate));
        }
        if displays.len() == count {
            displays.push(display(gdi_name.clone(), "Other", gdi_name.clone(), None));
        }
    }
    displays.sort_by(|a, b| {
        (a.area.left, a.area.top, &a.identifier).cmp(&(b.area.left, b.area.top, &b.identifier))
    });
    Ok(displays)
}

// The display with this identifier, ignoring case.
pub fn find<'a>(displays: &'a [Display], identifier: &str) -> Option<&'a Display> {
    let identifier = identifier.trim();
    displays
        .iter()
        .find(|display| display.identifier.eq_ignore_ascii_case(identifier))
}

pub fn primary(displays: &[Display]) -> Option<&Display> {
    displays
        .iter()
        .find(|display| display.primary)
        .or(displays.first())
}

fn monitors() -> Result<Vec<HMONITOR>> {
    unsafe extern "system" fn add(
        monitor: HMONITOR,
        _dc: HDC,
        _area: *mut RECT,
        monitors: LPARAM,
    ) -> BOOL {
        unsafe { (*(monitors.0 as *mut Vec<HMONITOR>)).push(monitor) };
        true.into()
    }
    let mut monitors = Vec::new();
    unsafe { EnumDisplayMonitors(None, None, Some(add), LPARAM(&raw mut monitors as isize)) }
        .ok()?;
    Ok(monitors)
}

// A monitor showing part of the desktop, as QueryDisplayConfig reports it.
struct Target {
    // The GDI device name of the part of the desktop it shows, which
    // GetMonitorInfoW also reports, like \\.\DISPLAY1.
    gdi_name: String,
    name: String,
    device_path: String,
    output_technology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    refresh_rate: Option<f64>,
}

unsafe fn targets() -> Result<Vec<Target>> {
    let flags = QDC_ONLY_ACTIVE_PATHS as u32;
    let paths = loop {
        let (mut path_count, mut mode_count) = (0, 0);
        check(unsafe { GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count) })?;
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        let error = unsafe {
            QueryDisplayConfig(
                flags,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        };
        // A display came on between the two calls, so ask again.
        if error == ERROR_INSUFFICIENT_BUFFER {
            continue;
        }
        check(error)?;
        paths.truncate(path_count as usize);
        break paths;
    };

    let mut targets = Vec::new();
    for path in &paths {
        let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                adapterId: path.sourceInfo.adapterId,
                id: path.sourceInfo.id,
            },
            ..Default::default()
        };
        // A monitor without its names gets an identifier from its GDI device name.
        if unsafe { DisplayConfigGetDeviceInfo(&mut source.header) } != ERROR_SUCCESS {
            continue;
        }
        let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                size: size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                adapterId: path.targetInfo.adapterId,
                id: path.targetInfo.id,
            },
            ..Default::default()
        };
        if unsafe { DisplayConfigGetDeviceInfo(&mut target.header) } != ERROR_SUCCESS {
            continue;
        }

        let rate = path.targetInfo.refreshRate;
        targets.push(Target {
            gdi_name: from_wide(&source.viewGdiDeviceName),
            name: from_wide(&target.monitorFriendlyDeviceName),
            device_path: from_wide(&target.monitorDevicePath),
            output_technology: path.targetInfo.outputTechnology,
            refresh_rate: (rate.Numerator != 0 && rate.Denominator != 0)
                .then(|| rate.Numerator as f64 / rate.Denominator as f64),
        });
    }
    Ok(targets)
}

fn connection(technology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY) -> &'static str {
    match technology {
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15 => "VGA",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI => "DVI",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI => "HDMI",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL => "DisplayPort",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL => "DisplayPort over USB",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS
        | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED
        | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED
        | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL => "Built in",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST => "Miracast",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED => "Indirect display",
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL => "Virtual display",
        _ => "Other",
    }
}

// FNV-1a, whose output never changes, unlike std's hasher, which may change
// between Rust versions. Its top bits depend on every bit of the input, so
// the identifier is taken from those. Device paths are case-insensitive.
fn identifier(device_path: &str) -> String {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in device_path.to_ascii_lowercase().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01B3);
    }
    (1..=IDENTIFIER_LENGTH)
        .map(|index| ALPHABET[(hash >> (64 - 5 * index)) as usize & 31] as char)
        .collect()
}

fn from_wide(text: &[u16]) -> String {
    let length = text
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(text.len());
    String::from_utf16_lossy(&text[..length])
}

// DisplayConfig functions return Win32 error codes.
fn check(error: i32) -> Result<()> {
    if error == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(error).into())
    }
}
