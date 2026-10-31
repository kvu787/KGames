// Rasterizes a monospaced font into a texture atlas with DirectWrite, so the
// renderer can draw text as textured quads without a font library. It runs
// once, at startup: programs only support 100% scaling (see
// Library/Application), so the font never changes size.

use windows_api::*;
use windows_core::{Error, PCWSTR, Result, w};

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
            let factory: IDWriteFactory2 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_ISOLATED)?;
            let face = find_font_face(&factory, w!("Consolas"))?;
            let em_size = pixel_height as f32;

            let characters: Vec<char> = (' '..='~').chain('\u{A0}'..='\u{FF}').collect();
            let code_points: Vec<u32> = characters.iter().map(|&c| c as u32).collect();
            let mut glyphs = vec![0u16; characters.len()];
            face.GetGlyphIndices(
                code_points.as_ptr(),
                code_points.len() as u32,
                glyphs.as_mut_ptr(),
            )
            .ok()?;

            // The cell is as wide as a glyph's advance, which is the same for
            // every glyph of a monospaced font, rounded to whole pixels. It's
            // as tall as the font's ascent and descent, each rounded up.
            let mut metrics = DWRITE_FONT_METRICS::default();
            face.GetMetrics(&mut metrics);
            let scale = em_size / metrics.designUnitsPerEm as f32;
            let mut glyph_metrics = DWRITE_GLYPH_METRICS::default();
            face.GetDesignGlyphMetrics(&glyphs[0], 1, &mut glyph_metrics, false)
                .ok()?;
            let advance = glyph_metrics.advanceWidth as f32 * scale;
            let ascent = (metrics.ascent as f32 * scale).ceil();
            let descent = (metrics.descent as f32 * scale).ceil();
            let cell_width = advance.round() as u32;
            let cell_height = (ascent + descent) as u32;
            let atlas_width = COLUMNS * cell_width;
            let atlas_height = CELL_COUNT.div_ceil(COLUMNS) * cell_height;

            let mut pixels = vec![0u8; (atlas_width * atlas_height) as usize];
            for y in 0..cell_height {
                let row = (y * atlas_width) as usize;
                pixels[row..row + cell_width as usize].fill(255);
            }

            for (&character, glyph) in characters.iter().zip(&glyphs) {
                let run = DWRITE_GLYPH_RUN {
                    // Borrow the font face without touching its reference count.
                    fontFace: std::mem::transmute_copy(&face),
                    fontEmSize: em_size,
                    glyphCount: 1,
                    glyphIndices: glyph,
                    ..Default::default()
                };
                // Grayscale antialiasing. ClearType's colored fringes would
                // be wrong once the coverage is tinted and blended. The
                // glyph is centered across the cell, as the cell's width is
                // its advance rounded to whole pixels.
                let analysis = factory.CreateGlyphRunAnalysis(
                    &run,
                    None,
                    DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
                    DWRITE_MEASURING_MODE_NATURAL,
                    DWRITE_GRID_FIT_MODE_ENABLED,
                    DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
                    (cell_width as f32 - advance) * 0.5,
                    ascent,
                )?;
                // Despite its name, ALIASED_1x1 is the texture type for
                // grayscale antialiasing too: one byte of coverage per pixel.
                let bounds = analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1)?;
                let width = (bounds.right - bounds.left) as usize;
                let height = (bounds.bottom - bounds.top) as usize;
                if width == 0 || height == 0 {
                    continue;
                }
                let mut coverage = vec![0u8; width * height];
                analysis
                    .CreateAlphaTexture(
                        DWRITE_TEXTURE_ALIASED_1x1,
                        &bounds,
                        coverage.as_mut_ptr(),
                        coverage.len() as u32,
                    )
                    .ok()?;

                // The bounds are relative to the cell's top left. Anything
                // outside the cell is cut off.
                let index = glyph_index(character).unwrap();
                let cell_x = (index % COLUMNS * cell_width) as i32;
                let cell_y = (index / COLUMNS * cell_height) as i32;
                for y in bounds.top.max(0)..bounds.bottom.min(cell_height as i32) {
                    for x in bounds.left.max(0)..bounds.right.min(cell_width as i32) {
                        let source = (y - bounds.top) as usize * width + (x - bounds.left) as usize;
                        let destination =
                            (cell_y + y) as usize * atlas_width as usize + (cell_x + x) as usize;
                        pixels[destination] = coverage[source];
                    }
                }
            }

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

// The regular face of an installed font family.
unsafe fn find_font_face(factory: &IDWriteFactory2, family: PCWSTR) -> Result<IDWriteFontFace> {
    unsafe {
        let mut collection = None;
        factory
            .GetSystemFontCollection(&mut collection, false)
            .ok()?;
        let collection = collection.unwrap();
        let mut index = 0;
        let mut exists = windows_core::BOOL(0);
        collection
            .FindFamilyName(family, &mut index, &mut exists)
            .ok()?;
        if !exists.as_bool() {
            return Err(Error::new(
                E_FAIL,
                format!("the font {} isn't installed", family.display()),
            ));
        }
        collection
            .GetFontFamily(index)?
            .GetFirstMatchingFont(
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
            )?
            .CreateFontFace()
    }
}

fn glyph_index(character: char) -> Option<u32> {
    match character as u32 {
        code @ 0x20..=0x7E => Some(code - 0x20 + 1),
        code @ 0xA0..=0xFF => Some(code - 0xA0 + 96),
        _ => None,
    }
}
