// Rasterizes a monospaced font into a texture atlas with GDI, so the renderer
// can draw text as textured quads without a font library. It runs at startup
// and again when the window moves to a monitor with a different DPI.

use crate::bindings::*;
use windows_core::{Error, Result, w};

const COLUMNS: u32 = 16;
// Cell 0 is solid white for shapes, so shapes and text share one texture and
// one draw call. Then printable ASCII, then the printable half of Latin-1,
// which covers most characters that keyboard layouts type.
const CELL_COUNT: u32 = 1 + 95 + 96;

pub struct Font {
    pub cell_width: u32,
    pub cell_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    // One byte of coverage per texel, row by row.
    pub pixels: Vec<u8>,
}

impl Font {
    // pixel_height is the em height in physical pixels.
    pub fn new(pixel_height: i32) -> Result<Self> {
        unsafe {
            let dc = CreateCompatibleDC(None);
            if dc.0.is_null() {
                return Err(Error::from_thread());
            }
            let font = CreateFontW(
                -pixel_height,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                // Grayscale antialiasing. ClearType's colored fringes would
                // be wrong once the coverage is tinted and blended.
                ANTIALIASED_QUALITY as u32,
                (FIXED_PITCH | FF_MODERN) as u32,
                w!("Consolas"),
            );
            let old_font = SelectObject(dc, HGDIOBJ(font.0));

            let mut metrics = TEXTMETRICW::default();
            GetTextMetricsW(dc, &mut metrics).ok()?;
            let cell_width = metrics.tmAveCharWidth as u32;
            let cell_height = metrics.tmHeight as u32;
            let atlas_width = COLUMNS * cell_width;
            let atlas_height = CELL_COUNT.div_ceil(COLUMNS) * cell_height;

            // A top-down 32-bit DIB that GDI draws into and the CPU can read.
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: atlas_width as i32,
                    biHeight: -(atlas_height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits = core::ptr::null_mut();
            let bitmap =
                CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS as u32, &mut bits, None, 0);
            if bitmap.0.is_null() {
                return Err(Error::from_thread());
            }
            let old_bitmap = SelectObject(dc, HGDIOBJ(bitmap.0));

            // White on black, so any color channel is the glyph's coverage.
            SetTextColor(dc, COLORREF(0x00FF_FFFF));
            SetBkColor(dc, COLORREF(0));
            for character in (' '..='~').chain('\u{A0}'..='\u{FF}') {
                let index = glyph_index(character).unwrap();
                let x = (index % COLUMNS * cell_width) as i32;
                let y = (index / COLUMNS * cell_height) as i32;
                let mut utf16 = [0u16; 2];
                let _ = TextOutW(dc, x, y, character.encode_utf16(&mut utf16));
            }
            GdiFlush().ok()?;

            let texel_count = (atlas_width * atlas_height) as usize;
            let bgra = core::slice::from_raw_parts(bits as *const u8, texel_count * 4);
            let mut pixels: Vec<u8> = bgra
                .as_chunks::<4>()
                .0
                .iter()
                .map(|texel| texel[1])
                .collect();
            for y in 0..cell_height {
                let row = (y * atlas_width) as usize;
                pixels[row..row + cell_width as usize].fill(255);
            }

            SelectObject(dc, old_bitmap);
            SelectObject(dc, old_font);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteObject(HGDIOBJ(font.0));
            let _ = DeleteDC(dc);

            Ok(Self {
                cell_width,
                cell_height,
                atlas_width,
                atlas_height,
                pixels,
            })
        }
    }

    // Texture coordinates of a character's cell: left, top, right, bottom.
    // Characters the atlas lacks are drawn as '?'.
    pub fn glyph_uv(&self, character: char) -> [f32; 4] {
        let index = glyph_index(character).or(glyph_index('?')).unwrap();
        let x = (index % COLUMNS * self.cell_width) as f32;
        let y = (index / COLUMNS * self.cell_height) as f32;
        [
            x / self.atlas_width as f32,
            y / self.atlas_height as f32,
            (x + self.cell_width as f32) / self.atlas_width as f32,
            (y + self.cell_height as f32) / self.atlas_height as f32,
        ]
    }

    // A texture coordinate in the solid white cell.
    pub fn solid_uv(&self) -> [f32; 2] {
        [
            self.cell_width as f32 * 0.5 / self.atlas_width as f32,
            self.cell_height as f32 * 0.5 / self.atlas_height as f32,
        ]
    }

    pub fn has_glyph(character: char) -> bool {
        glyph_index(character).is_some()
    }
}

fn glyph_index(character: char) -> Option<u32> {
    match character as u32 {
        code @ 0x20..=0x7E => Some(code - 0x20 + 1),
        code @ 0xA0..=0xFF => Some(code - 0xA0 + 96),
        _ => None,
    }
}
