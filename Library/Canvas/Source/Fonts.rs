// The fonts that interfaces draw text and icons with, rasterized with
// DirectWrite into one texture atlas at startup, so that the renderer draws
// each glyph as a textured quad, and text costs no font work per frame.
// Programs only support 100% scaling (see Library/Application), so the sizes
// never change.
//
// The type ramp is Fluent 2's, Windows 11's, in Segoe UI Variable, at the
// optical size that each style's size is drawn for: Small for captions, Text
// for body text, and Display for subtitles and titles. Mono is Cascadia Mono,
// for numbers in columns, code and logs. Icons are Segoe Fluent Icons, the
// icons of Windows 11. Each is the Windows font of its kind, and falls back to
// Segoe UI or Consolas if it isn't installed.
//
// Text is proportional, so each glyph is rasterized at SUBPIXELS horizontal
// offsets, and a glyph is drawn with the offset nearest where it falls, so
// that the gaps between letters are even. Baselines are on whole pixels.

use std::collections::HashMap;

use windows_api::*;
use windows_core::{Error, HSTRING, PCWSTR, Result};

// A style of text, from Fluent 2's type ramp.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextStyle {
    // 12 pixels, for small print: captions, hints, keys.
    Caption,
    // 14 pixels, for most text.
    Body,
    // Body in semibold, for titles of cards and what stands out.
    BodyStrong,
    // 20 pixels in semibold, for the app's title.
    Subtitle,
    // 28 pixels in semibold, for big numbers and headings.
    Title,
    // 13 pixels of Cascadia Mono, on Body's lines, for numbers in columns
    // and code.
    Mono,
}

impl TextStyle {
    pub const ALL: [TextStyle; 6] = [
        TextStyle::Caption,
        TextStyle::Body,
        TextStyle::BodyStrong,
        TextStyle::Subtitle,
        TextStyle::Title,
        TextStyle::Mono,
    ];

    fn index(self) -> usize {
        self as usize
    }

    // The font family, with what to use if it isn't installed, the weight,
    // the size in pixels, and the line's height.
    fn face(self) -> ([&'static str; 2], i32, f32, f32) {
        match self {
            TextStyle::Caption => (["Segoe UI Variable Small", "Segoe UI"], 400, 12.0, 16.0),
            TextStyle::Body => (["Segoe UI Variable Text", "Segoe UI"], 400, 14.0, 20.0),
            TextStyle::BodyStrong => (["Segoe UI Variable Text", "Segoe UI"], 600, 14.0, 20.0),
            TextStyle::Subtitle => (["Segoe UI Variable Display", "Segoe UI"], 600, 20.0, 28.0),
            TextStyle::Title => (["Segoe UI Variable Display", "Segoe UI"], 600, 28.0, 36.0),
            TextStyle::Mono => (["Cascadia Mono", "Consolas"], 400, 13.0, 20.0),
        }
    }
}

// An icon from Segoe Fluent Icons, by what it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    CheckMark,
    Dismiss,
    Search,
    Info,
    Warning,
    Error,
    Success,
    Add,
    Remove,
    Delete,
    Save,
    Refresh,
    Settings,
    Keyboard,
    Mouse,
    Gamepad,
    Display,
    Play,
    Pause,
    Edit,
    Copy,
    Color,
    Brightness,
    Volume,
}

impl Icon {
    pub const ALL: [Icon; 28] = [
        Icon::ChevronDown,
        Icon::ChevronUp,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::CheckMark,
        Icon::Dismiss,
        Icon::Search,
        Icon::Info,
        Icon::Warning,
        Icon::Error,
        Icon::Success,
        Icon::Add,
        Icon::Remove,
        Icon::Delete,
        Icon::Save,
        Icon::Refresh,
        Icon::Settings,
        Icon::Keyboard,
        Icon::Mouse,
        Icon::Gamepad,
        Icon::Display,
        Icon::Play,
        Icon::Pause,
        Icon::Edit,
        Icon::Copy,
        Icon::Color,
        Icon::Brightness,
        Icon::Volume,
    ];

    // Its code point in Segoe Fluent Icons.
    fn code_point(self) -> u32 {
        match self {
            Icon::ChevronDown => 0xE70D,
            Icon::ChevronUp => 0xE70E,
            Icon::ChevronLeft => 0xE76B,
            Icon::ChevronRight => 0xE76C,
            Icon::CheckMark => 0xE73E,
            Icon::Dismiss => 0xE711,
            Icon::Search => 0xE721,
            Icon::Info => 0xE946,
            Icon::Warning => 0xE7BA,
            Icon::Error => 0xE783,
            Icon::Success => 0xE930,
            Icon::Add => 0xE710,
            Icon::Remove => 0xE738,
            Icon::Delete => 0xE74D,
            Icon::Save => 0xE74E,
            Icon::Refresh => 0xE72C,
            Icon::Settings => 0xE713,
            Icon::Keyboard => 0xE765,
            Icon::Mouse => 0xE962,
            Icon::Gamepad => 0xE7FC,
            Icon::Display => 0xE7F4,
            Icon::Play => 0xE768,
            Icon::Pause => 0xE769,
            Icon::Edit => 0xE70F,
            Icon::Copy => 0xE8C8,
            Icon::Color => 0xE790,
            Icon::Brightness => 0xE706,
            Icon::Volume => 0xE767,
        }
    }
}

// The sizes that icons come in, in pixels: 12 inside small controls, like a
// combo box's chevron or a checkbox's check, and 16 elsewhere.
pub const ICON_SIZES: [f32; 2] = [12.0, 16.0];

// Each glyph is rasterized at this many offsets across a pixel.
const SUBPIXELS: usize = 3;
// The atlas is this wide, and as tall as its glyphs need.
const ATLAS_WIDTH: u32 = 2048;
// Between glyphs in the atlas, so that none samples another.
const GUTTER: u32 = 1;

// Printable ASCII, the printable half of Latin-1, which covers most of what
// keyboard layouts type, and a few marks of typography and math.
const EXTRA_CHARACTERS: [char; 24] = [
    '\u{2013}', // en dash
    '\u{2014}', // em dash
    '\u{2018}', // quotation marks
    '\u{2019}',
    '\u{201C}',
    '\u{201D}',
    '\u{2022}', // bullet
    '\u{2026}', // ellipsis
    '\u{2190}', // arrows
    '\u{2191}',
    '\u{2192}',
    '\u{2193}',
    '\u{2212}', // minus
    '\u{2248}', // almost equal
    '\u{2260}', // not equal
    '\u{2264}', // less or equal
    '\u{2265}', // greater or equal
    '\u{221E}', // infinity
    '\u{0394}', // delta
    '\u{03BC}', // mu
    '\u{03C0}', // pi
    '\u{03C3}', // sigma
    '\u{2032}', // prime
    '\u{2033}', // double prime
];

// The ellipsis that cuts text short.
pub const ELLIPSIS: &str = "\u{2026}";

// A rasterized glyph: where its coverage is in the atlas, in texels, and
// where its top left goes from the pen, which is on the baseline.
#[derive(Clone, Copy, Debug, Default)]
pub struct Glyph {
    pub texels: [f32; 4],
    pub offset: [f32; 2],
}

// What a style's text needs: its line's height, where the baseline is down
// it, each character's advance, and its glyphs.
pub struct StyleMetrics {
    pub size: f32,
    pub line_height: f32,
    // From the top of a line to its baseline, in whole pixels.
    pub baseline: f32,
    pub cap_height: f32,
    advances: Vec<f32>,
    // SUBPIXELS for each character.
    glyphs: Vec<Glyph>,
}

pub struct Fonts {
    styles: Vec<StyleMetrics>,
    // Each icon at each of ICON_SIZES, from its center.
    icons: HashMap<(Icon, u32), Glyph>,
    pub atlas_width: u32,
    pub atlas_height: u32,
    // One byte of coverage per texel, row by row.
    pub pixels: Vec<u8>,
    // The family that each style ended up with, for tests and the showcase.
    pub families: Vec<String>,
}

// A glyph's coverage before it's packed into the atlas.
struct Raster {
    width: u32,
    height: u32,
    coverage: Vec<u8>,
    // The top left from the pen.
    offset: [f32; 2],
}

impl Fonts {
    pub fn new() -> Result<Self> {
        unsafe {
            let factory: IDWriteFactory2 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_ISOLATED)?;
            let mut collection = None;
            factory.GetSystemFontCollection(&mut collection, false).ok()?;
            let collection = collection.unwrap();

            let characters = characters();
            let mut rasters: Vec<Raster> = Vec::new();
            let mut styles = Vec::new();
            let mut families = Vec::new();
            for style in TextStyle::ALL {
                let (names, weight, size, line_height) = style.face();
                let (face, family) = find_face(&collection, &names, weight)?;
                families.push(family);
                let mut metrics = DWRITE_FONT_METRICS::default();
                face.GetMetrics(&mut metrics);
                let scale = size / metrics.designUnitsPerEm as f32;
                let cap_height = metrics.capHeight as f32 * scale;
                let glyph_indices = glyph_indices(&face, &characters)?;
                let mut design = vec![DWRITE_GLYPH_METRICS::default(); glyph_indices.len()];
                face.GetDesignGlyphMetrics(glyph_indices.as_ptr(), glyph_indices.len() as u32, design.as_mut_ptr(), false)
                    .ok()?;
                let advances: Vec<f32> = design.iter().map(|glyph| glyph.advanceWidth as f32 * scale).collect();
                // The cap height is centered down the line, which centers
                // text in controls by what most of its letters fill.
                let baseline = (line_height * 0.5 + cap_height * 0.5).round();
                let mut glyphs = Vec::with_capacity(glyph_indices.len() * SUBPIXELS);
                for &index in &glyph_indices {
                    for subpixel in 0..SUBPIXELS {
                        let x = subpixel as f32 / SUBPIXELS as f32;
                        glyphs.push(rasters.len());
                        rasters.push(rasterize(&factory, &face, index, size, [x, 0.0])?);
                    }
                }
                styles.push((
                    StyleMetrics {
                        size,
                        line_height,
                        baseline,
                        cap_height,
                        advances,
                        glyphs: Vec::new(),
                    },
                    glyphs,
                ));
            }

            // Icons, centered on the middle of their em square, which is
            // what Segoe Fluent Icons draws them in.
            let (icon_face, _) = find_face(&collection, &["Segoe Fluent Icons", "Segoe MDL2 Assets"], 400)?;
            let mut metrics = DWRITE_FONT_METRICS::default();
            icon_face.GetMetrics(&mut metrics);
            let code_points: Vec<u32> = Icon::ALL.iter().map(|icon| icon.code_point()).collect();
            let mut icon_indices = vec![0u16; code_points.len()];
            icon_face
                .GetGlyphIndices(code_points.as_ptr(), code_points.len() as u32, icon_indices.as_mut_ptr())
                .ok()?;
            let mut icon_rasters = Vec::new();
            for size in ICON_SIZES {
                let scale = size / metrics.designUnitsPerEm as f32;
                let middle = (metrics.ascent as f32 - metrics.descent as f32) * 0.5 * scale;
                for (icon, &index) in Icon::ALL.iter().zip(&icon_indices) {
                    // With the em square's middle at (0, 0), the offset is
                    // from the icon's center.
                    let raster = rasterize(&factory, &icon_face, index, size, [-size * 0.5, middle.round()])?;
                    icon_rasters.push(((*icon, size as u32), rasters.len()));
                    rasters.push(raster);
                }
            }

            // Packs the glyphs onto shelves, tallest first, so that the
            // shelves waste little.
            let mut order: Vec<usize> = (0..rasters.len()).collect();
            order.sort_by_key(|&index| std::cmp::Reverse(rasters[index].height));
            let mut places = vec![[0u32; 2]; rasters.len()];
            let (mut x, mut y, mut shelf) = (GUTTER, GUTTER, 0u32);
            for &index in &order {
                let raster = &rasters[index];
                if raster.width == 0 {
                    continue;
                }
                if x + raster.width + GUTTER > ATLAS_WIDTH {
                    x = GUTTER;
                    y += shelf + GUTTER;
                    shelf = 0;
                }
                places[index] = [x, y];
                x += raster.width + GUTTER;
                shelf = shelf.max(raster.height);
            }
            let atlas_height = (y + shelf + GUTTER).next_multiple_of(4);
            let mut pixels = vec![0u8; (ATLAS_WIDTH * atlas_height) as usize];
            let glyph_of = |index: usize| {
                let raster = &rasters[index];
                let [x, y] = places[index];
                Glyph {
                    texels: [
                        x as f32,
                        y as f32,
                        (x + raster.width) as f32,
                        (y + raster.height) as f32,
                    ],
                    offset: raster.offset,
                }
            };
            for (index, raster) in rasters.iter().enumerate() {
                let [x, y] = places[index];
                for row in 0..raster.height {
                    let source = (row * raster.width) as usize;
                    let destination = ((y + row) * ATLAS_WIDTH + x) as usize;
                    pixels[destination..destination + raster.width as usize]
                        .copy_from_slice(&raster.coverage[source..source + raster.width as usize]);
                }
            }
            let styles = styles
                .into_iter()
                .map(|(mut metrics, glyphs)| {
                    metrics.glyphs = glyphs.into_iter().map(glyph_of).collect();
                    metrics
                })
                .collect();
            let icons = icon_rasters
                .into_iter()
                .map(|(key, index)| (key, glyph_of(index)))
                .collect();
            Ok(Self {
                styles,
                icons,
                atlas_width: ATLAS_WIDTH,
                atlas_height,
                pixels,
                families,
            })
        }
    }

    pub fn style(&self, style: TextStyle) -> &StyleMetrics {
        &self.styles[style.index()]
    }

    pub fn line_height(&self, style: TextStyle) -> f32 {
        self.style(style).line_height
    }

    // How wide `text` is in a style, in pixels.
    pub fn measure(&self, text: &str, style: TextStyle) -> f32 {
        let metrics = self.style(style);
        text.chars().map(|character| metrics.advance(character)).sum()
    }

    // The glyph of a character at a pen's x, and where its quad's left goes.
    pub fn glyph(&self, style: TextStyle, character: char, pen: f32) -> (Glyph, f32) {
        let metrics = self.style(style);
        let index = character_index(character).unwrap_or_else(|| character_index('?').unwrap());
        let whole = pen.floor();
        let mut subpixel = ((pen - whole) * SUBPIXELS as f32).round() as usize;
        let mut left = whole;
        if subpixel == SUBPIXELS {
            subpixel = 0;
            left += 1.0;
        }
        let glyph = metrics.glyphs[index * SUBPIXELS + subpixel];
        (glyph, left + glyph.offset[0])
    }

    pub fn icon(&self, icon: Icon, size: f32) -> Option<Glyph> {
        self.icons.get(&(icon, size as u32)).copied()
    }

    // Whether the fonts draw a character, instead of a question mark.
    pub fn has_glyph(character: char) -> bool {
        character_index(character).is_some()
    }

    // How much of `text` fits in `width` in a style, in bytes, from the
    // start.
    pub fn fit(&self, text: &str, style: TextStyle, width: f32) -> usize {
        let metrics = self.style(style);
        let mut x = 0.0;
        for (index, character) in text.char_indices() {
            x += metrics.advance(character);
            if x > width + 0.01 {
                return index;
            }
        }
        text.len()
    }

    // Splits text into lines at spaces, each at most `width` wide, but a
    // word that's wider on its own. Lines after the first are `indent`
    // narrower, for a hanging indent. Newlines start new lines.
    pub fn wrap(&self, text: &str, style: TextStyle, width: f32, indent: f32) -> Vec<String> {
        let space = self.measure(" ", style);
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            let mut line_width = 0.0;
            for word in paragraph.split(' ') {
                let limit = if lines.is_empty() { width } else { (width - indent).max(1.0) };
                let word_width = self.measure(word, style);
                if !line.is_empty() && line_width + space + word_width > limit + 0.01 {
                    lines.push(std::mem::take(&mut line));
                    line_width = 0.0;
                }
                if !line.is_empty() {
                    line.push(' ');
                    line_width += space;
                }
                line.push_str(word);
                line_width += word_width;
            }
            lines.push(line);
        }
        lines
    }
}

impl StyleMetrics {
    pub fn advance(&self, character: char) -> f32 {
        let index = character_index(character).unwrap_or_else(|| character_index('?').unwrap());
        self.advances[index]
    }
}

fn characters() -> Vec<char> {
    (' '..='~').chain('\u{A0}'..='\u{FF}').chain(EXTRA_CHARACTERS).collect()
}

fn character_index(character: char) -> Option<usize> {
    match character as u32 {
        code @ 0x20..=0x7E => Some((code - 0x20) as usize),
        code @ 0xA0..=0xFF => Some((code - 0xA0) as usize + 95),
        _ => EXTRA_CHARACTERS
            .iter()
            .position(|extra| *extra == character)
            .map(|index| index + 95 + 96),
    }
}

// A glyph's index for each character, or '?''s for those the font lacks.
unsafe fn glyph_indices(face: &IDWriteFontFace, characters: &[char]) -> Result<Vec<u16>> {
    unsafe {
        let code_points: Vec<u32> = characters.iter().map(|&character| character as u32).collect();
        let mut indices = vec![0u16; code_points.len()];
        face.GetGlyphIndices(code_points.as_ptr(), code_points.len() as u32, indices.as_mut_ptr())
            .ok()?;
        let question = indices[character_index('?').unwrap()];
        Ok(indices
            .into_iter()
            .map(|index| if index == 0 { question } else { index })
            .collect())
    }
}

// Grayscale coverage of a glyph with its pen at `origin`, in pixels, from
// which its offset is measured. ClearType's colored fringes would be wrong
// once the coverage is tinted and blended.
unsafe fn rasterize(factory: &IDWriteFactory2, face: &IDWriteFontFace, index: u16, size: f32, origin: [f32; 2]) -> Result<Raster> {
    unsafe {
        let run = DWRITE_GLYPH_RUN {
            // Borrow the font face without touching its reference count.
            fontFace: std::mem::transmute_copy(face),
            fontEmSize: size,
            glyphCount: 1,
            glyphIndices: &index,
            ..Default::default()
        };
        let analysis = factory.CreateGlyphRunAnalysis(
            &run,
            None,
            DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
            DWRITE_MEASURING_MODE_NATURAL,
            DWRITE_GRID_FIT_MODE_ENABLED,
            DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
            origin[0],
            origin[1],
        )?;
        // Despite its name, ALIASED_1x1 is the texture type for grayscale
        // antialiasing too: one byte of coverage per pixel.
        let bounds = analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1)?;
        let width = (bounds.right - bounds.left).max(0) as u32;
        let height = (bounds.bottom - bounds.top).max(0) as u32;
        let mut coverage = vec![0u8; (width * height) as usize];
        if width > 0 && height > 0 {
            analysis
                .CreateAlphaTexture(DWRITE_TEXTURE_ALIASED_1x1, &bounds, coverage.as_mut_ptr(), coverage.len() as u32)
                .ok()?;
        }
        Ok(Raster {
            width,
            height,
            coverage,
            offset: [bounds.left as f32, bounds.top as f32],
        })
    }
}

// The face of the first of `families` that's installed, at a weight, and
// the family's name.
unsafe fn find_face(collection: &IDWriteFontCollection, families: &[&str], weight: i32) -> Result<(IDWriteFontFace, String)> {
    unsafe {
        for family in families {
            let mut index = 0;
            let mut exists = windows_core::BOOL(0);
            let name = HSTRING::from(*family);
            collection
                .FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists)
                .ok()?;
            if !exists.as_bool() {
                continue;
            }
            let face = collection
                .GetFontFamily(index)?
                .GetFirstMatchingFont(weight, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL)?
                .CreateFontFace()?;
            return Ok((face, family.to_string()));
        }
        Err(Error::new(E_FAIL, format!("none of the fonts {families:?} is installed")))
    }
}
