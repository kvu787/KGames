// Windows' pointer speed and acceleration, as the user set them, which the
// pointer moves by (see input/ballistics.rs): the speed from
// SPI_GETMOUSESPEED, whether "Enhance pointer precision" is on from
// SPI_GETMOUSE, and its curve from the registry, or Windows' own if the
// registry has none, as SDL 3's WIN_UpdateMouseSystemScale and ReadMouseCurve
// read them. The window says when they change (see platform/window.rs).

use windows_api::*;
use windows_core::w;

use crate::input::ballistics::{Curve, PointerSpeed, WINDOWS_CURVE};

pub fn read() -> PointerSpeed {
    unsafe {
        let mut speed = 10i32;
        let speed = if SystemParametersInfoW(SPI_GETMOUSESPEED as u32, 0, (&raw mut speed).cast(), 0).as_bool() {
            speed.clamp(1, 20)
        } else {
            10
        };
        // The two thresholds of Windows before XP, and whether it
        // accelerates.
        let mut mouse = [0i32; 3];
        let enhanced = SystemParametersInfoW(SPI_GETMOUSE as u32, 0, mouse.as_mut_ptr().cast(), 0).as_bool() && mouse[2] != 0;
        PointerSpeed {
            speed,
            acceleration: enhanced.then(registry_curve),
        }
    }
}

// The curve in HKEY_CURRENT_USER\Control Panel\Mouse: two values of five
// 64-bit points each, of which the low 32 bits count, as SDL reads them, each
// Windows' own where the registry doesn't have it.
fn registry_curve() -> Curve {
    let read = |value| unsafe {
        let mut bytes = [0u8; 40];
        let mut size = bytes.len() as u32;
        let status = RegGetValueW(HKEY_CURRENT_USER, w!("Control Panel\\Mouse"), value, RRF_RT_REG_BINARY as u32, None, Some(bytes.as_mut_ptr().cast()), Some(&mut size));
        (status.0 == 0 && size == 40).then(|| std::array::from_fn(|i| u32::from_le_bytes(bytes[i * 8..i * 8 + 4].try_into().unwrap())))
    };
    Curve {
        x: read(w!("SmoothMouseXCurve")).unwrap_or(WINDOWS_CURVE.x),
        y: read(w!("SmoothMouseYCurve")).unwrap_or(WINDOWS_CURVE.y),
    }
}
