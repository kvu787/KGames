// Text with DirectWrite: fonts, layout, measuring, hit testing, and the glyph
// atlas that the renderer draws text from.
//
// DirectWrite lays text out, with kerning, ligatures, font fallback and line
// breaking, in an IDWriteTextLayout. A text renderer of our own, written by
// hand as a COM object below, collects the glyphs that the layout places.
// Layouts are cached by their text and style, so a frame that shows the same
// text as the last one asks DirectWrite for nothing. Glyphs are rasterized
// once each, with grayscale antialiasing, at a quarter-pixel horizontal
// offset, into an 8-bit atlas that the GPU samples.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use windows_api::*;
use windows_core::{BOOL, GUID, HRESULT, IUnknown_Vtbl, Interface, PCWSTR, Result};

use crate::geometry::{Point, Size};

// Segoe UI Variable's optical sizes, Cascadia Mono, and Segoe Fluent Icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Font {
    Small,
    Text,
    Display,
    Mono,
    Icons,
}

impl Font {
    // The families to try, in order.
    fn families(self) -> &'static [&'static str] {
        match self {
            Font::Small => &["Segoe UI Variable Small", "Segoe UI"],
            Font::Text => &["Segoe UI Variable Text", "Segoe UI"],
            Font::Display => &["Segoe UI Variable Display", "Segoe UI"],
            Font::Mono => &["Cascadia Mono", "Consolas"],
            Font::Icons => &["Segoe Fluent Icons", "Segoe MDL2 Assets"],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weight {
    Regular,
    Semibold,
    Bold,
}

impl Weight {
    fn value(self) -> DWRITE_FONT_WEIGHT {
        match self {
            Weight::Regular => DWRITE_FONT_WEIGHT_NORMAL,
            Weight::Semibold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
            Weight::Bold => DWRITE_FONT_WEIGHT_BOLD,
        }
    }
}

// A font at a size, with the height of its lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub font: Font,
    pub weight: Weight,
    pub size: f32,
    pub line_height: f32,
}

impl TextStyle {
    pub const fn new(font: Font, weight: Weight, size: f32, line_height: f32) -> Self {
        Self {
            font,
            weight,
            size,
            line_height,
        }
    }

    pub const fn with_weight(self, weight: Weight) -> Self {
        Self { weight, ..self }
    }

    // Segoe Fluent Icons at a size, in lines that are as tall as the icons.
    pub const fn icons(size: f32) -> Self {
        Self::new(Font::Icons, Weight::Regular, size, size)
    }

    fn key(&self) -> StyleKey {
        StyleKey {
            font: self.font,
            weight: self.weight,
            size: self.size.to_bits(),
            line_height: self.line_height.to_bits(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StyleKey {
    font: Font,
    weight: Weight,
    size: u32,
    line_height: u32,
}

// A glyph that a layout placed: its font face, by index into the text
// system's faces, its index in the face, and its pen position from the
// layout's top left, on its line's baseline.
#[derive(Clone, Copy, Debug)]
pub struct PlacedGlyph {
    pub face: u16,
    pub glyph: u16,
    pub size: f32,
    pub x: f32,
    pub y: f32,
}

// Text laid out in one style, maybe wrapped at a width.
pub struct TextLayout {
    // The widest line, with its trailing spaces.
    pub width: f32,
    pub height: f32,
    pub line_count: u32,
    pub glyphs: Vec<PlacedGlyph>,
    layout: IDWriteTextLayout,
    text: String,
    last_used: u64,
}

impl TextLayout {
    // Where a caret before the character at a byte index of the text goes,
    // from the layout's left.
    pub fn caret_x(&self, byte_index: usize) -> f32 {
        let position = utf16_index(&self.text, byte_index);
        let (mut x, mut y) = (0.0, 0.0);
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        unsafe {
            let _ = self.layout.HitTestTextPosition(position as u32, false, &mut x, &mut y, &mut metrics);
        }
        x
    }

    // The byte index of the text nearest a point in the layout, between
    // characters, for putting a caret where a click lands.
    pub fn hit(&self, point: Point) -> usize {
        let mut trailing = BOOL(0);
        let mut inside = BOOL(0);
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        unsafe {
            let _ = self.layout.HitTestPoint(point.x, point.y, &mut trailing, &mut inside, &mut metrics);
        }
        let mut position = metrics.textPosition as usize;
        if trailing.as_bool() {
            position += metrics.length as usize;
        }
        byte_index(&self.text, position)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

// The index in UTF-16 code units of a byte index in UTF-8.
pub fn utf16_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index.min(text.len())].encode_utf16().count()
}

// The byte index of a UTF-16 index, at a character boundary.
pub fn byte_index(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units >= utf16 {
            return index;
        }
        units += character.len_utf16();
    }
    text.len()
}

// Where a glyph is in the atlas, and where its pixels go from its pen
// position.
#[derive(Clone, Copy, Debug)]
struct AtlasGlyph {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    left: i16,
    top: i16,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    face: u16,
    glyph: u16,
    size: u32,
    // Quarter pixels right of the pixel grid, 0 to 3.
    offset: u8,
}

// A glyph's pixels to draw: where on screen, and where in the atlas, in
// pixels of each.
#[derive(Clone, Copy, Debug)]
pub struct GlyphQuad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub atlas_x: f32,
    pub atlas_y: f32,
}

// A glyph's pixels that the renderer must copy into the atlas texture.
pub struct GlyphUpload {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub const ATLAS_SIZE: u32 = 2048;

struct Atlas {
    glyphs: HashMap<GlyphKey, Option<AtlasGlyph>>,
    // Shelf packing: glyphs fill rows left to right, and a row is as tall as
    // its tallest glyph.
    shelf_x: u32,
    shelf_y: u32,
    shelf_height: u32,
    full: bool,
    uploads: Vec<GlyphUpload>,
    // Whether the renderer must clear its texture before the uploads, after
    // the atlas filled up and started over.
    reset: bool,
}

impl Atlas {
    fn new() -> Self {
        Self {
            glyphs: HashMap::new(),
            shelf_x: 1,
            shelf_y: 1,
            shelf_height: 0,
            full: false,
            uploads: Vec::new(),
            reset: true,
        }
    }

    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // A pixel between glyphs, so that sampling never bleeds.
        if self.shelf_x + width + 1 > ATLAS_SIZE {
            self.shelf_x = 1;
            self.shelf_y += self.shelf_height + 1;
            self.shelf_height = 0;
        }
        if self.shelf_y + height + 1 > ATLAS_SIZE {
            self.full = true;
            return None;
        }
        let place = (self.shelf_x, self.shelf_y);
        self.shelf_x += width + 1;
        self.shelf_height = self.shelf_height.max(height);
        Some(place)
    }
}

// Every face that layouts used, so that glyphs can name theirs by index.
struct Faces {
    faces: Vec<IDWriteFontFace>,
    by_pointer: HashMap<usize, u16>,
}

impl Faces {
    fn index(&mut self, face: &IDWriteFontFace) -> u16 {
        let pointer = face.as_raw() as usize;
        if let Some(&index) = self.by_pointer.get(&pointer) {
            return index;
        }
        let index = self.faces.len() as u16;
        // The clone holds a reference, so the pointer stays this face's.
        self.faces.push(face.clone());
        self.by_pointer.insert(pointer, index);
        index
    }
}

pub struct Fonts {
    factory: IDWriteFactory2,
    formats: HashMap<StyleKey, IDWriteTextFormat>,
    layouts: HashMap<u64, Vec<TextLayout>>,
    faces: Faces,
    atlas: Atlas,
    frame: u64,
    // The family each font resolved to, like "Segoe UI Variable Text".
    resolved: HashMap<Font, String>,
}

impl Fonts {
    pub fn new() -> Result<Self> {
        let factory: IDWriteFactory2 = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let mut fonts = Self {
            factory,
            formats: HashMap::new(),
            layouts: HashMap::new(),
            faces: Faces {
                faces: Vec::new(),
                by_pointer: HashMap::new(),
            },
            atlas: Atlas::new(),
            frame: 0,
            resolved: HashMap::new(),
        };
        for font in [Font::Small, Font::Text, Font::Display, Font::Mono, Font::Icons] {
            let family = fonts.find_family(font)?;
            fonts.resolved.insert(font, family);
        }
        Ok(fonts)
    }

    // The family that a font resolved to.
    pub fn family(&self, font: Font) -> &str {
        &self.resolved[&font]
    }

    fn find_family(&self, font: Font) -> Result<String> {
        unsafe {
            let mut collection = None;
            self.factory.GetSystemFontCollection(&mut collection, false).ok()?;
            let collection = collection.unwrap();
            for family in font.families() {
                let mut index = 0;
                let mut exists = BOOL(0);
                let name = wide(family);
                collection
                    .FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists)
                    .ok()?;
                if exists.as_bool() {
                    return Ok(family.to_string());
                }
            }
            // DirectWrite falls back to another family for a missing one.
            Ok(font.families()[0].to_string())
        }
    }

    // Starts a frame: layouts unused for a while are let go, and an atlas
    // that filled up starts over.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
        if self.frame.is_multiple_of(600) {
            let frame = self.frame;
            self.layouts.retain(|_, layouts| {
                layouts.retain(|layout| frame - layout.last_used < 600);
                !layouts.is_empty()
            });
        }
        if self.atlas.full {
            self.atlas = Atlas::new();
        }
    }

    fn format(&mut self, style: TextStyle) -> Result<IDWriteTextFormat> {
        if let Some(format) = self.formats.get(&style.key()) {
            return Ok(format.clone());
        }
        unsafe {
            let family = wide(&self.resolved[&style.font]);
            let format = self.factory.CreateTextFormat(
                PCWSTR(family.as_ptr()),
                None,
                style.weight.value(),
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size,
                PCWSTR(wide("en-us").as_ptr()),
            )?;
            // Every line is line_height tall, with the font's ascent and
            // descent centered in it, as Fluent's type ramp sets text.
            let (ascent, descent) = self.font_extent(style)?;
            let baseline = (style.line_height - (ascent + descent)) / 2.0 + ascent;
            format
                .SetLineSpacing(DWRITE_LINE_SPACING_METHOD_UNIFORM, style.line_height, baseline)
                .ok()?;
            self.formats.insert(style.key(), format.clone());
            Ok(format)
        }
    }

    // The font's ascent and descent at the style's size, in pixels.
    fn font_extent(&self, style: TextStyle) -> Result<(f32, f32)> {
        unsafe {
            let mut collection = None;
            self.factory.GetSystemFontCollection(&mut collection, false).ok()?;
            let collection = collection.unwrap();
            let mut index = 0;
            let mut exists = BOOL(0);
            let name = wide(&self.resolved[&style.font]);
            collection.FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists).ok()?;
            if !exists.as_bool() {
                return Ok((style.size * 0.9, style.size * 0.25));
            }
            let family = collection.GetFontFamily(index)?;
            let font = family.GetFirstMatchingFont(
                style.weight.value(),
                DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
            )?;
            let face = font.CreateFontFace()?;
            let mut metrics = DWRITE_FONT_METRICS::default();
            face.GetMetrics(&mut metrics);
            let scale = style.size / metrics.designUnitsPerEm as f32;
            Ok((metrics.ascent as f32 * scale, metrics.descent as f32 * scale))
        }
    }

    // The text laid out in the style, wrapped at `wrap` pixels or not at all.
    pub fn layout(&mut self, text: &str, style: TextStyle, wrap: Option<f32>) -> &TextLayout {
        let key = layout_key(text, style, wrap);
        let position = self
            .layouts
            .get(&key)
            .and_then(|layouts| layouts.iter().position(|layout| layout.text == text));
        let index = match position {
            Some(index) => index,
            None => {
                let layout = self
                    .create_layout(text, style, wrap)
                    .unwrap_or_else(|error| panic!("DirectWrite couldn't lay out {text:?}: {error}"));
                let layouts = self.layouts.entry(key).or_default();
                layouts.push(layout);
                layouts.len() - 1
            }
        };
        let frame = self.frame;
        let layout = &mut self.layouts.get_mut(&key).unwrap()[index];
        layout.last_used = frame;
        layout
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.layout(text, style, None).size()
    }

    fn create_layout(&mut self, text: &str, style: TextStyle, wrap: Option<f32>) -> Result<TextLayout> {
        let format = self.format(style)?;
        unsafe {
            let utf16: Vec<u16> = text.encode_utf16().collect();
            let layout = self.factory.CreateTextLayout(&utf16, &format, wrap.unwrap_or(100_000.0), 100_000.0)?;
            if wrap.is_none() {
                layout.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP).ok()?;
            }
            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics).ok()?;
            let mut collector = Collector {
                vtable: &COLLECTOR_VTABLE,
                glyphs: Vec::new(),
                faces: &mut self.faces,
            };
            let renderer_pointer = &mut collector as *mut Collector as *mut core::ffi::c_void;
            let renderer = IDWriteTextRenderer::from_raw_borrowed(&renderer_pointer).unwrap();
            layout.Draw(None, renderer, 0.0, 0.0).ok()?;
            let glyphs = std::mem::take(&mut collector.glyphs);
            let line_count = metrics.lineCount.max(1);
            Ok(TextLayout {
                width: metrics.widthIncludingTrailingWhitespace,
                height: line_count as f32 * style.line_height,
                line_count,
                glyphs,
                layout,
                text: text.to_string(),
                last_used: self.frame,
            })
        }
    }

    // The quads of the text's glyphs, with its layout's top left at
    // `origin`, rasterizing the glyphs that aren't in the atlas yet.
    pub fn quads(&mut self, text: &str, style: TextStyle, wrap: Option<f32>, origin: Point, emit: &mut impl FnMut(GlyphQuad)) {
        // The layout is created first, then the fields are borrowed apart,
        // so that the atlas can change while the layout is read.
        self.layout(text, style, wrap);
        let key = layout_key(text, style, wrap);
        let Some(layout) = self.layouts.get(&key).and_then(|layouts| layouts.iter().find(|layout| layout.text == text))
        else {
            return;
        };
        let atlas = &mut self.atlas;
        let faces = &self.faces.faces;
        let factory = &self.factory;
        for placed in &layout.glyphs {
            let x = origin.x + placed.x;
            let baseline = (origin.y + placed.y).round();
            let mut pixel = x.floor();
            let mut offset = ((x - pixel) * 4.0).round() as u8;
            if offset == 4 {
                pixel += 1.0;
                offset = 0;
            }
            let key = GlyphKey {
                face: placed.face,
                glyph: placed.glyph,
                size: placed.size.to_bits(),
                offset,
            };
            let entry = match atlas.glyphs.get(&key) {
                Some(entry) => *entry,
                None => {
                    let entry = rasterize(factory, &faces[placed.face as usize], placed, offset, atlas);
                    if entry.is_some() || !atlas.full {
                        atlas.glyphs.insert(key, entry);
                    }
                    entry
                }
            };
            if let Some(glyph) = entry {
                emit(GlyphQuad {
                    x: pixel + glyph.left as f32,
                    y: baseline + glyph.top as f32,
                    width: glyph.width as f32,
                    height: glyph.height as f32,
                    atlas_x: glyph.x as f32,
                    atlas_y: glyph.y as f32,
                });
            }
        }
    }

    // The glyphs rasterized since the last call, for the renderer to copy
    // into its texture, and whether it must clear the texture first.
    pub fn take_uploads(&mut self) -> (bool, Vec<GlyphUpload>) {
        let reset = std::mem::replace(&mut self.atlas.reset, false);
        (reset, std::mem::take(&mut self.atlas.uploads))
    }
}

fn layout_key(text: &str, style: TextStyle, wrap: Option<f32>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    style.key().hash(&mut hasher);
    wrap.map(f32::to_bits).hash(&mut hasher);
    hasher.finish()
}

fn rasterize(
    factory: &IDWriteFactory2,
    face: &IDWriteFontFace,
    placed: &PlacedGlyph,
    offset: u8,
    atlas: &mut Atlas,
) -> Option<AtlasGlyph> {
    unsafe {
        let advance = 0.0f32;
        let run = DWRITE_GLYPH_RUN {
            fontFace: std::mem::transmute_copy(face),
            fontEmSize: placed.size,
            glyphCount: 1,
            glyphIndices: &placed.glyph,
            glyphAdvances: &advance,
            glyphOffsets: std::ptr::null(),
            isSideways: BOOL(0),
            bidiLevel: 0,
        };
        let analysis = factory
            .CreateGlyphRunAnalysis(
                &run,
                None,
                DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
                DWRITE_MEASURING_MODE_NATURAL,
                DWRITE_GRID_FIT_MODE_DEFAULT,
                DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
                offset as f32 / 4.0,
                0.0,
            )
            .ok()?;
        let bounds = analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1).ok()?;
        let width = (bounds.right - bounds.left).max(0) as u32;
        let height = (bounds.bottom - bounds.top).max(0) as u32;
        if width == 0 || height == 0 {
            // A space: it has an advance but no pixels.
            return Some(AtlasGlyph {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                left: 0,
                top: 0,
            });
        }
        let mut pixels = vec![0u8; (width * height) as usize];
        analysis
            .CreateAlphaTexture(DWRITE_TEXTURE_ALIASED_1x1, &bounds, pixels.as_mut_ptr(), pixels.len() as u32)
            .ok()
            .ok()?;
        let (x, y) = atlas.place(width, height)?;
        atlas.uploads.push(GlyphUpload {
            x,
            y,
            width,
            height,
            pixels,
        });
        Some(AtlasGlyph {
            x: x as u16,
            y: y as u16,
            width: width as u16,
            height: height as u16,
            left: bounds.left as i16,
            top: bounds.top as i16,
        })
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

// The text renderer that IDWriteTextLayout::Draw calls back, which only
// collects the glyphs. It lives on the stack for the one call, so its
// reference count is never used to free it.
#[repr(C)]
struct Collector {
    vtable: *const IDWriteTextRenderer_Vtbl,
    glyphs: Vec<PlacedGlyph>,
    faces: *mut Faces,
}

static COLLECTOR_VTABLE: IDWriteTextRenderer_Vtbl = IDWriteTextRenderer_Vtbl {
    base__: IDWritePixelSnapping_Vtbl {
        base__: IUnknown_Vtbl {
            QueryInterface: query_interface,
            AddRef: add_ref,
            Release: release,
        },
        IsPixelSnappingDisabled: is_pixel_snapping_disabled,
        GetCurrentTransform: get_current_transform,
        GetPixelsPerDip: get_pixels_per_dip,
    },
    DrawGlyphRun: draw_glyph_run,
    DrawUnderline: draw_underline,
    DrawStrikethrough: draw_strikethrough,
    DrawInlineObject: draw_inline_object,
};

unsafe extern "system" fn query_interface(
    this: *mut core::ffi::c_void,
    iid: *const GUID,
    interface: *mut *mut core::ffi::c_void,
) -> HRESULT {
    unsafe {
        let iid = *iid;
        if iid == windows_core::IUnknown::IID || iid == IDWritePixelSnapping::IID || iid == IDWriteTextRenderer::IID {
            *interface = this;
            HRESULT(0)
        } else {
            *interface = std::ptr::null_mut();
            HRESULT(0x8000_4002_u32 as i32)
        }
    }
}

unsafe extern "system" fn add_ref(_this: *mut core::ffi::c_void) -> u32 {
    1
}

unsafe extern "system" fn release(_this: *mut core::ffi::c_void) -> u32 {
    1
}

unsafe extern "system" fn is_pixel_snapping_disabled(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    disabled: *mut BOOL,
) -> HRESULT {
    // Snapped to whole pixels vertically; the glyphs keep their quarter
    // pixels horizontally.
    unsafe { *disabled = BOOL(0) };
    HRESULT(0)
}

unsafe extern "system" fn get_current_transform(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    transform: *mut DWRITE_MATRIX,
) -> HRESULT {
    unsafe {
        *transform = DWRITE_MATRIX {
            m11: 1.0,
            m12: 0.0,
            m21: 0.0,
            m22: 1.0,
            dx: 0.0,
            dy: 0.0,
        };
    }
    HRESULT(0)
}

unsafe extern "system" fn get_pixels_per_dip(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    pixels: *mut f32,
) -> HRESULT {
    unsafe { *pixels = 1.0 };
    HRESULT(0)
}

unsafe extern "system" fn draw_glyph_run(
    this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    baseline_x: f32,
    baseline_y: f32,
    _measuring_mode: DWRITE_MEASURING_MODE,
    run: *const DWRITE_GLYPH_RUN,
    _description: *const DWRITE_GLYPH_RUN_DESCRIPTION,
    _effect: *mut core::ffi::c_void,
) -> HRESULT {
    unsafe {
        let collector = &mut *(this as *mut Collector);
        let run = &*run;
        let Some(face) = run.fontFace.as_ref() else {
            return HRESULT(0);
        };
        let face = (*collector.faces).index(face);
        let right_to_left = run.bidiLevel & 1 == 1;
        let mut pen = baseline_x;
        for index in 0..run.glyphCount as usize {
            let advance = if run.glyphAdvances.is_null() { 0.0 } else { *run.glyphAdvances.add(index) };
            let (advance_offset, ascender_offset) = if run.glyphOffsets.is_null() {
                (0.0, 0.0)
            } else {
                let offset = *run.glyphOffsets.add(index);
                (offset.advanceOffset, offset.ascenderOffset)
            };
            let x = if right_to_left {
                pen - advance - advance_offset
            } else {
                pen + advance_offset
            };
            collector.glyphs.push(PlacedGlyph {
                face,
                glyph: *run.glyphIndices.add(index),
                size: run.fontEmSize,
                x,
                y: baseline_y - ascender_offset,
            });
            pen += if right_to_left { -advance } else { advance };
        }
        HRESULT(0)
    }
}

unsafe extern "system" fn draw_underline(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    _x: f32,
    _y: f32,
    _underline: *const DWRITE_UNDERLINE,
    _effect: *mut core::ffi::c_void,
) -> HRESULT {
    HRESULT(0)
}

unsafe extern "system" fn draw_strikethrough(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    _x: f32,
    _y: f32,
    _strikethrough: *const DWRITE_STRIKETHROUGH,
    _effect: *mut core::ffi::c_void,
) -> HRESULT {
    HRESULT(0)
}

unsafe extern "system" fn draw_inline_object(
    _this: *mut core::ffi::c_void,
    _context: *const core::ffi::c_void,
    _x: f32,
    _y: f32,
    _object: *mut core::ffi::c_void,
    _sideways: BOOL,
    _right_to_left: BOOL,
    _effect: *mut core::ffi::c_void,
) -> HRESULT {
    HRESULT(0)
}
