// How far a mouse's motion moves the pointer: by Windows' pointer speed and,
// with "Enhance pointer precision" on, its acceleration, as Windows moves its
// own cursor, so that the pointer that the interface keeps and draws moves
// like the cursor the user is used to. The reader applies it to GameInput's
// motion, which has neither (see input/reader.rs), and
// platform/mouse_settings.rs reads the settings from Windows.
//
// Ported to Rust from SDL 3's WIN_ApplySystemScale, ReadMouseCurve and
// WIN_UpdateMouseSystemScale, in src/video/windows/SDL_windowsmouse.c, which
// SDL applies to raw motion with SDL_HINT_MOUSE_RELATIVE_SYSTEM_SCALE. This is
// an altered version of that source: SDL's integer arithmetic, with signed
// division throughout, where SDL divides a negative intercept, which only a
// curve that isn't convex has, as unsigned, and for a window at 96 DPI, the
// one supported setup, where SDL multiplies by the display's DPI. Its notice:
//
//   Simple DirectMedia Layer
//   Copyright (C) 1997-2026 Sam Lantinga <slouken@libsdl.org>
//
//   This software is provided 'as-is', without any express or implied
//   warranty.  In no event will the authors be held liable for any damages
//   arising from the use of this software.
//
//   Permission is granted to anyone to use this software for any purpose,
//   including commercial applications, and to alter it and redistribute it
//   freely, subject to the following restrictions:
//
//   1. The origin of this software must not be misrepresented; you must not
//      claim that you wrote the original software. If you use this software
//      in a product, an acknowledgment in the product documentation would be
//      appreciated but is not required.
//   2. Altered source versions must be plainly marked as such, and must not be
//      misrepresented as being the original software.
//   3. This notice may not be removed or altered from any source distribution.

use crate::drawing::geometry::Point;

// An acceleration curve: five points, each a mouse speed and how far the
// pointer goes at it, in 16.16 fixed point, as Windows keeps them in
// HKEY_CURRENT_USER\Control Panel\Mouse's SmoothMouseXCurve and
// SmoothMouseYCurve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Curve {
    pub x: [u32; 5],
    pub y: [u32; 5],
}

// Windows' curve since Windows 8, for when the registry has none.
pub const WINDOWS_CURVE: Curve = Curve {
    x: [0x0000_0000, 0x0000_6e15, 0x0001_4000, 0x0003_dc29, 0x0028_0000],
    y: [0x0000_0000, 0x0001_11fd, 0x0004_2400, 0x0012_fc00, 0x01bb_c000],
};

// Windows' pointer speed and acceleration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerSpeed {
    // The pointer speed, 1 to 20, which Windows' slider sets, and which is
    // 10 by default: a count a pixel without acceleration.
    pub speed: i32,
    // The acceleration curve, while "Enhance pointer precision" is on.
    pub acceleration: Option<Curve>,
}

impl PointerSpeed {
    // A count a pixel, which tests and scripts move by.
    pub const ONE_TO_ONE: PointerSpeed = PointerSpeed { speed: 10, acceleration: None };
}

impl Default for PointerSpeed {
    fn default() -> Self {
        PointerSpeed::ONE_TO_ONE
    }
}

// The DPI that SDL's arithmetic scales by: the window's, at 100% scaling.
const DPI: i128 = 96;
// What the acceleration's scale is divided by, on Windows 8 and later.
const DENOMINATOR: i128 = (10 * 120) << 16;

// A pointer speed, ready to apply, with the curve segment that the last
// motion was on, as the acceleration eases into a faster segment by
// averaging it with the last one.
#[derive(Clone, Debug)]
pub struct Ballistics {
    // The scale without acceleration, in 32nds.
    scale: i128,
    // The curve's points, scaled as SDL scales them.
    nodes: Option<([i128; 5], [i128; 5])>,
    last_node: usize,
}

impl Default for Ballistics {
    fn default() -> Self {
        Ballistics::new(PointerSpeed::ONE_TO_ONE)
    }
}

impl Ballistics {
    pub fn new(speed: PointerSpeed) -> Self {
        let v = speed.speed.clamp(1, 20) as i128;
        let nodes = speed.acceleration.map(|curve| {
            let mut xs = [0; 5];
            let mut ys = [0; 5];
            // The first point is always the origin.
            for i in 1..5 {
                xs[i] = 7 * curve.x[i] as i128;
                ys[i] = (v * curve.y[i] as i128) << 17;
            }
            (xs, ys)
        });
        Ballistics {
            scale: v.max((v - 2) * 4).max((v - 6) * 8),
            nodes,
            last_node: 0,
        }
    }

    // How far the pointer moves for a mouse's motion, in counts, in pixels.
    pub fn apply(&mut self, x: i64, y: i64) -> Point {
        let (ix, iy) = (x as i128 * 65536, y as i128 * 65536);
        let Some((xs, ys)) = &self.nodes else {
            let scale = self.scale * DPI;
            let move_by = |i: i128| (i * scale / DPI / 32) as f32 / 65536.0;
            return Point::new(move_by(ix), move_by(iy));
        };
        let (absx, absy) = (ix.abs(), iy.abs());
        // Windows' approximation of how fast the mouse moves.
        let speed = absx.min(absy) + (absx.max(absy) << 1);
        if speed == 0 {
            return Point::ZERO;
        }
        let (mut i, mut j) = (1, 1);
        while i < 5 {
            j = i;
            if speed < xs[j] {
                break;
            }
            i += 1;
        }
        i -= 1;
        j -= 1;
        let k = std::mem::replace(&mut self.last_node, j);

        // The scale on segment j, through point i.
        let segment = |j: usize, i: usize| {
            let xdiff = xs[j + 1] - xs[j];
            if xdiff == 0 {
                return 0;
            }
            let slope = (ys[j + 1] - ys[j]) / xdiff;
            let intercept = slope * xs[i] - ys[i];
            slope - intercept / speed
        };
        let mut denominator = DENOMINATOR;
        let mut scale = segment(j, i);
        // Into a faster segment: half of each.
        if j > k {
            denominator <<= 1;
            scale += segment(k, k);
        }
        scale *= DPI;
        Point::new((ix * scale / denominator) as f32 / 65536.0, (iy * scale / denominator) as f32 / 65536.0)
    }
}
