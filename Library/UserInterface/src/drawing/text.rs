// Text with DirectWrite: the type ramp's fonts, layout, measuring, hit
// testing for carets, and the glyph atlas that the renderer draws text from.
//
// DirectWrite lays text out, with kerning, ligatures, font fallback and line
// breaking, in an IDWriteTextLayout. A text renderer of our own, a COM object
// written by hand below, collects the glyphs that the layout places. Layouts
// are cached by their text and style, so a frame that shows the same text as
// the last asks DirectWrite for nothing. Each glyph is rasterized once, with
// grayscale antialiasing as WinUI's text is, at a quarter-pixel horizontal
// offset, into an 8-bit atlas that the GPU reads.

use windows_api::*;
use windows_core::{BOOL, GUID, HRESULT, IUnknown_Vtbl, Interface, PCWSTR, Result};

use crate::drawing::geometry::{Point, Size};
use crate::drawing::hashing::QuickMap;
use crate::platform::window::wide;

// Segoe UI Variable's optical sizes, as Windows names them as families,
// Cascadia Mono, and Segoe Fluent Icons. The type ramp says which of Small,
// Text and Display each style is set in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Font {
    Small,
    Text,
    Display,
    Mono,
    Icons,
}

impl Font {
    pub const ALL: [Font; 5] = [Font::Small, Font::Text, Font::Display, Font::Mono, Font::Icons];

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

// A font at a size, and the height of its lines, like the type ramp's
// Body: Segoe UI Variable Text, regular, 14 on 20.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub font: Font,
    pub weight: Weight,
    pub size: f32,
    pub line_height: f32,
}

impl TextStyle {
    pub const fn new(font: Font, weight: Weight, size: f32, line_height: f32) -> Self {
        Self { font, weight, size, line_height }
    }

    pub const fn weight(self, weight: Weight) -> Self {
        Self { weight, ..self }
    }

    // Segoe Fluent Icons at a size, in lines as tall as the icons.
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

// A glyph that a layout placed: its face, by index into the faces that
// layouts used, its index in the face, and its pen position on its line's
// baseline, from the layout's top left.
#[derive(Clone, Copy, Debug)]
struct PlacedGlyph {
    face: u16,
    glyph: u16,
    size: f32,
    x: f32,
    y: f32,
}

// Text laid out in one style, on one line or wrapped at a width.
pub struct TextLayout {
    // The widest line, with its trailing spaces.
    pub width: f32,
    pub height: f32,
    pub line_count: u32,
    glyphs: Vec<PlacedGlyph>,
    layout: IDWriteTextLayout,
    text: String,
    last_used: u64,
}

impl TextLayout {
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    // Where a caret before the character at the byte index goes, from the
    // layout's left.
    pub fn caret_x(&self, byte_index: usize) -> f32 {
        let position = utf16_index(&self.text, byte_index);
        let (mut x, mut y) = (0.0, 0.0);
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        unsafe {
            let _ = self.layout.HitTestTextPosition(position as u32, false, &mut x, &mut y, &mut metrics);
        }
        x
    }

    // The byte index between characters nearest a point in the layout, for
    // putting a caret where a click lands.
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
}

// The UTF-16 index of a byte index of UTF-8 text.
pub fn utf16_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index.min(text.len())].encode_utf16().count()
}

// The byte index of a UTF-16 index, at a character's start.
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

// A glyph's pixels to draw: where on screen, and where in the atlas.
#[derive(Clone, Copy, Debug)]
pub struct GlyphQuad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub atlas_x: f32,
    pub atlas_y: f32,
}

// Pixels that the renderer copies into its atlas texture.
pub struct GlyphUpload {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub const ATLAS_SIZE: u32 = 2048;

#[derive(Clone, Copy, Debug)]
struct AtlasGlyph {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    // Where its pixels start from its pen position.
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

// Shelf packing: glyphs fill rows left to right, and a row is as tall as its
// tallest glyph. Once it's full, it starts over with the next frame.
struct Atlas {
    glyphs: QuickMap<GlyphKey, Option<AtlasGlyph>>,
    shelf_x: u32,
    shelf_y: u32,
    shelf_height: u32,
    full: bool,
    uploads: Vec<GlyphUpload>,
}

impl Atlas {
    fn new() -> Self {
        Self {
            glyphs: QuickMap::default(),
            shelf_x: 1,
            shelf_y: 1,
            shelf_height: 0,
            full: false,
            uploads: Vec::new(),
        }
    }

    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // A pixel between glyphs keeps them from bleeding into each other.
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

// Every face that layouts used, so that glyphs name theirs by index.
#[derive(Default)]
struct Faces {
    faces: Vec<IDWriteFontFace>,
    by_pointer: QuickMap<usize, u16>,
}

impl Faces {
    fn index(&mut self, face: &IDWriteFontFace) -> u16 {
        let pointer = face.as_raw() as usize;
        if let Some(index) = self.by_pointer.get(&pointer) {
            return *index;
        }
        let index = self.faces.len() as u16;
        // The clone holds a reference, so the pointer stays this face's.
        self.faces.push(face.clone());
        self.by_pointer.insert(pointer, index);
        index
    }
}

// Layouts not used for this many frames are let go of.
const LAYOUT_LIFETIME: u64 = 600;

pub struct Fonts {
    factory: IDWriteFactory2,
    collection: IDWriteFontCollection,
    formats: QuickMap<StyleKey, IDWriteTextFormat>,
    // By layout_key's hash of the text and how it's laid out.
    layouts: QuickMap<u64, Vec<TextLayout>>,
    faces: Faces,
    atlas: Atlas,
    frame: u64,
    // The family that each font resolved to, like "Segoe UI Variable Text".
    resolved: QuickMap<Font, String>,
}

impl Fonts {
    pub fn new() -> Result<Self> {
        unsafe {
            let factory: IDWriteFactory2 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let mut collection = None;
            factory.GetSystemFontCollection(&mut collection, false).ok()?;
            let collection = collection.ok_or_else(windows_core::Error::empty)?;
            let mut fonts = Self {
                factory,
                collection,
                formats: QuickMap::default(),
                layouts: QuickMap::default(),
                faces: Faces::default(),
                atlas: Atlas::new(),
                frame: 0,
                resolved: QuickMap::default(),
            };
            for font in Font::ALL {
                let family = fonts.find_family(font)?;
                fonts.resolved.insert(font, family);
            }
            Ok(fonts)
        }
    }

    // The family that the font resolved to.
    pub fn family(&self, font: Font) -> &str {
        &self.resolved[&font]
    }

    fn find_family(&self, font: Font) -> Result<String> {
        for family in font.families() {
            if self.family_index(family)?.is_some() {
                return Ok(family.to_string());
            }
        }
        // DirectWrite falls back to another family for a missing one.
        Ok(font.families()[0].to_string())
    }

    fn family_index(&self, family: &str) -> Result<Option<u32>> {
        let mut index = 0;
        let mut exists = BOOL(0);
        let name = wide(family);
        unsafe { self.collection.FindFamilyName(PCWSTR(name.as_ptr()), &mut index, &mut exists).ok()? };
        Ok(exists.as_bool().then_some(index))
    }

    // Starts a frame: layouts unused for a while are let go of, and a full
    // atlas starts over.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
        if self.frame.is_multiple_of(LAYOUT_LIFETIME) {
            let frame = self.frame;
            self.layouts.retain(|_, layouts| {
                layouts.retain(|layout| frame - layout.last_used < LAYOUT_LIFETIME);
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
            let locale = wide("en-us");
            let format = self.factory.CreateTextFormat(
                PCWSTR(family.as_ptr()),
                None,
                style.weight.value(),
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size,
                PCWSTR(locale.as_ptr()),
            )?;
            // Every line is the style's line height, with the font's ascent
            // and descent centered in it, as XAML sets the type ramp.
            let (ascent, descent) = self.extent(style)?;
            let baseline = (style.line_height - (ascent + descent)) / 2.0 + ascent;
            format.SetLineSpacing(DWRITE_LINE_SPACING_METHOD_UNIFORM, style.line_height, baseline).ok()?;
            self.formats.insert(style.key(), format.clone());
            Ok(format)
        }
    }

    // The font's ascent and descent at the style's size, in pixels.
    fn extent(&self, style: TextStyle) -> Result<(f32, f32)> {
        let Some(index) = self.family_index(&self.resolved[&style.font])? else {
            return Ok((style.size * 0.9, style.size * 0.25));
        };
        unsafe {
            let family = self.collection.GetFontFamily(index)?;
            let font = family.GetFirstMatchingFont(style.weight.value(), DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL)?;
            let face = font.CreateFontFace()?;
            let mut metrics = DWRITE_FONT_METRICS::default();
            face.GetMetrics(&mut metrics);
            let scale = style.size / metrics.designUnitsPerEm as f32;
            Ok((metrics.ascent as f32 * scale, metrics.descent as f32 * scale))
        }
    }

    // The text laid out in the style, wrapped at `wrap` pixels or not at all.
    pub fn layout(&mut self, text: &str, style: TextStyle, wrap: Option<f32>) -> &TextLayout {
        let (key, index) = self.find(text, style, wrap);
        &self.layouts[&key][index]
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.layout(text, style, None).size()
    }

    pub fn measure_wrapped(&mut self, text: &str, style: TextStyle, width: f32) -> Size {
        self.layout(text, style, Some(width)).size()
    }

    // Where the layout is in the cache, laid out now if it isn't there.
    fn find(&mut self, text: &str, style: TextStyle, wrap: Option<f32>) -> (u64, usize) {
        let key = layout_key(text, style, wrap);
        let frame = self.frame;
        if let Some(layouts) = self.layouts.get_mut(&key)
            && let Some(index) = layouts.iter().position(|layout| layout.text == text)
        {
            layouts[index].last_used = frame;
            return (key, index);
        }
        let layout = self
            .create_layout(text, style, wrap)
            .unwrap_or_else(|error| panic!("DirectWrite couldn't lay out {text:?}: {error}"));
        let layouts = self.layouts.entry(key).or_default();
        layouts.push(layout);
        (key, layouts.len() - 1)
    }

    fn create_layout(&mut self, text: &str, style: TextStyle, wrap: Option<f32>) -> Result<TextLayout> {
        let format = self.format(style)?;
        unsafe {
            let units: Vec<u16> = text.encode_utf16().collect();
            let layout = self.factory.CreateTextLayout(&units, &format, wrap.unwrap_or(1.0e6), 1.0e6)?;
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
            let pointer = &mut collector as *mut Collector as *mut core::ffi::c_void;
            let renderer = IDWriteTextRenderer::from_raw_borrowed(&pointer).ok_or_else(windows_core::Error::empty)?;
            layout.Draw(None, renderer, 0.0, 0.0).ok()?;
            let line_count = metrics.lineCount.max(1);
            Ok(TextLayout {
                width: metrics.widthIncludingTrailingWhitespace,
                height: line_count as f32 * style.line_height,
                line_count,
                glyphs: std::mem::take(&mut collector.glyphs),
                layout,
                text: text.to_string(),
                last_used: self.frame,
            })
        }
    }

    // The quads of the text's glyphs, with the layout's top left at
    // `origin`, rasterizing glyphs that aren't in the atlas yet. Returns the
    // layout's size.
    pub fn quads(&mut self, text: &str, style: TextStyle, wrap: Option<f32>, origin: Point, emit: &mut impl FnMut(GlyphQuad)) -> Size {
        // The layout is found first, then the fields are borrowed apart, so
        // that the atlas can grow while the layout is read.
        let (key, index) = self.find(text, style, wrap);
        let layout = &self.layouts[&key][index];
        let atlas = &mut self.atlas;
        let faces = &self.faces.faces;
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
                    let entry = rasterize(&self.factory, &faces[placed.face as usize], placed, offset, atlas);
                    if entry.is_some() || !atlas.full {
                        atlas.glyphs.insert(key, entry);
                    }
                    entry
                }
            };
            if let Some(glyph) = entry
                && glyph.width > 0
            {
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
        layout.size()
    }

    // The glyphs rasterized since the last call, for the renderer to copy
    // into its texture.
    pub fn take_uploads(&mut self) -> Vec<GlyphUpload> {
        std::mem::take(&mut self.atlas.uploads)
    }
}

// FNV-1a over the text and the style, which is fast for short text.
fn layout_key(text: &str, style: TextStyle, wrap: Option<f32>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut add = |byte: u8| {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    };
    text.bytes().for_each(&mut add);
    let key = style.key();
    add(key.font as u8);
    add(key.weight as u8);
    for word in [key.size, key.line_height, wrap.map_or(u32::MAX, f32::to_bits)] {
        word.to_le_bytes().into_iter().for_each(&mut add);
    }
    hash
}

// Whether the font's glyphs at this size are fitted to the pixel grid, as the
// font's own tables say. CreateGlyphRunAnalysis takes the grid-fit mode, which
// must be one or the other, not DWRITE_GRID_FIT_MODE_DEFAULT, which means
// this.
fn grid_fit(face: &IDWriteFontFace, size: f32) -> DWRITE_GRID_FIT_MODE {
    let mut rendering = DWRITE_RENDERING_MODE_DEFAULT;
    let mut grid_fit = DWRITE_GRID_FIT_MODE_ENABLED;
    unsafe {
        if let Ok(face) = face.cast::<IDWriteFontFace2>() {
            let _ = face.GetRecommendedRenderingMode(
                size,
                USER_DEFAULT_SCREEN_DPI as f32,
                USER_DEFAULT_SCREEN_DPI as f32,
                None,
                false,
                DWRITE_OUTLINE_THRESHOLD_ANTIALIASED,
                DWRITE_MEASURING_MODE_NATURAL,
                None,
                &mut rendering,
                &mut grid_fit,
            );
        }
    }
    if grid_fit == DWRITE_GRID_FIT_MODE_DISABLED { DWRITE_GRID_FIT_MODE_DISABLED } else { DWRITE_GRID_FIT_MODE_ENABLED }
}

fn rasterize(factory: &IDWriteFactory2, face: &IDWriteFontFace, placed: &PlacedGlyph, offset: u8, atlas: &mut Atlas) -> Option<AtlasGlyph> {
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
                grid_fit(face, placed.size),
                DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
                offset as f32 / 4.0,
                0.0,
            )
            .ok()?;
        let bounds = analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1).ok()?;
        let width = (bounds.right - bounds.left).max(0) as u32;
        let height = (bounds.bottom - bounds.top).max(0) as u32;
        if width == 0 || height == 0 {
            // Like a space: an advance, but no pixels.
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
        atlas.uploads.push(GlyphUpload { x, y, width, height, pixels });
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

// The text renderer that IDWriteTextLayout::Draw calls back, which only
// collects the glyphs. It lives on the stack for that one call, so its
// reference count never frees it.
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

unsafe extern "system" fn query_interface(this: *mut core::ffi::c_void, iid: *const GUID, interface: *mut *mut core::ffi::c_void) -> HRESULT {
    unsafe {
        let iid = *iid;
        if iid == windows_core::IUnknown::IID || iid == IDWritePixelSnapping::IID || iid == IDWriteTextRenderer::IID {
            *interface = this;
            HRESULT(0)
        } else {
            *interface = std::ptr::null_mut();
            // E_NOINTERFACE.
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

unsafe extern "system" fn is_pixel_snapping_disabled(_this: *mut core::ffi::c_void, _context: *const core::ffi::c_void, disabled: *mut BOOL) -> HRESULT {
    // Baselines snap to whole pixels; glyphs keep their quarter pixels
    // across.
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

unsafe extern "system" fn get_pixels_per_dip(_this: *mut core::ffi::c_void, _context: *const core::ffi::c_void, pixels: *mut f32) -> HRESULT {
    unsafe { *pixels = 1.0 };
    HRESULT(0)
}

#[allow(clippy::too_many_arguments)]
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
            let (along, up) = if run.glyphOffsets.is_null() {
                (0.0, 0.0)
            } else {
                let offset = *run.glyphOffsets.add(index);
                (offset.advanceOffset, offset.ascenderOffset)
            };
            let x = if right_to_left { pen - advance - along } else { pen + along };
            collector.glyphs.push(PlacedGlyph {
                face,
                glyph: *run.glyphIndices.add(index),
                size: run.fontEmSize,
                x,
                y: baseline_y - up,
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

#[allow(clippy::too_many_arguments)]
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
