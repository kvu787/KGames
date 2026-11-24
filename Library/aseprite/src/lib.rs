// Reads Aseprite's .aseprite files: the first frame of an RGBA sprite, as
// its canvas shows it, with its visible layers flattened, or one of its
// layers on its own. See README.md for
// what it reads, and the file format's specification:
// https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md

mod inflate;

use std::path::Path;

pub use inflate::{adler32, zlib_decompress};

const FILE_MAGIC: u16 = 0xA5E0;
const FRAME_MAGIC: u16 = 0xF1FA;
const HEADER_SIZE: usize = 128;
const FRAME_HEADER_SIZE: usize = 16;
const CHUNK_HEADER_SIZE: usize = 6;

const LAYER_CHUNK: u16 = 0x2004;
const CEL_CHUNK: u16 = 0x2005;

// The header's flag that says its layers' opacity is valid.
const LAYER_OPACITY_VALID: u32 = 1;
// Layers' flags.
const LAYER_VISIBLE: u16 = 1;
const LAYER_REFERENCE: u16 = 64;
// Layers' types.
const NORMAL_LAYER: u16 = 0;
const TILEMAP_LAYER: u16 = 2;
// Cels' types.
const RAW_CEL: u16 = 0;
const LINKED_CEL: u16 = 1;
const COMPRESSED_CEL: u16 = 2;

// A sprite's first frame, as its canvas shows it: red, green, blue and
// alpha, not premultiplied, a row at a time from the top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

impl Canvas {
    // A canvas of transparent pixels.
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            width,
            height,
            pixels: vec![[0; 4]; width as usize * height as usize],
        }
    }

    // Red, green, blue and alpha of a pixel.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[y as usize * self.width as usize + x as usize]
    }
}

// The first frame of the .aseprite file at the path.
pub fn load(path: &Path) -> Result<Canvas, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
    read(&bytes).map_err(|error| format!("Couldn't read {}: {error}", path.display()))
}

// The first frame of an .aseprite file's bytes: its visible layers' cels,
// flattened onto a transparent canvas with Aseprite's Normal blend mode.
// Only RGBA sprites are read. Reference layers, which Aseprite doesn't
// export, are left out.
pub fn read(bytes: &[u8]) -> Result<Canvas, String> {
    let frame = Frame::read(bytes)?;
    let mut canvas = Canvas::new(frame.width, frame.height);
    for cel in &frame.cels {
        let layer = frame.layers.get(cel.layer).ok_or_else(|| format!("A cel is on layer {}, which doesn't exist.", cel.layer))?;
        if !layer.shown {
            continue;
        }
        if layer.kind == TILEMAP_LAYER {
            return Err(format!("The layer \"{}\" is a tilemap, which can't be read.", layer.name));
        }
        if layer.blend_mode != 0 {
            return Err(format!("The layer \"{}\" has a blend mode other than Normal, which can't be read.", layer.name));
        }
        cel.draw(&mut canvas, layer.opacity_of(cel), &layer.name)?;
    }
    Ok(canvas)
}

// One layer of the first frame of the .aseprite file at the path, on its
// own.
pub fn load_layer(path: &Path, name: &str) -> Result<Canvas, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
    read_layer(&bytes, name).map_err(|error| format!("Couldn't read {}: {error}", path.display()))
}

// One layer of the first frame of an .aseprite file's bytes, on its own: the
// layer named `name`, which has to be the only one with that name, and a
// layer of pixels, not a group or a tilemap. Its cel is on a transparent
// canvas, at its opacity, as the canvas would show it if it were the only
// visible layer, whether it's visible or not.
pub fn read_layer(bytes: &[u8], name: &str) -> Result<Canvas, String> {
    let frame = Frame::read(bytes)?;
    let mut named = frame.layers.iter().enumerate().filter(|(_, layer)| layer.name == name);
    let (index, layer) = named.next().ok_or_else(|| format!("It has no layer named \"{name}\"."))?;
    if named.next().is_some() {
        return Err(format!("It has more than one layer named \"{name}\"."));
    }
    if layer.kind != NORMAL_LAYER {
        return Err(format!("The layer \"{name}\" is a group or a tilemap, not a layer of pixels."));
    }
    let mut canvas = Canvas::new(frame.width, frame.height);
    for cel in frame.cels.iter().filter(|cel| cel.layer == index) {
        cel.draw(&mut canvas, layer.opacity_of(cel), name)?;
    }
    Ok(canvas)
}

// The first frame of a file: the canvas's size, the layers, and their cels,
// bottom to top.
struct Frame {
    width: u32,
    height: u32,
    layers: Vec<Layer>,
    cels: Vec<Cel>,
}

impl Frame {
    fn read(bytes: &[u8]) -> Result<Frame, String> {
        let mut reader = Reader { bytes, at: 0 };
        let _file_size = reader.u32()?;
        if reader.u16()? != FILE_MAGIC {
            return Err("It isn't an Aseprite file.".to_string());
        }
        let frames = reader.u16()?;
        let width = reader.u16()?;
        let height = reader.u16()?;
        let depth = reader.u16()?;
        let flags = reader.u32()?;
        if depth != 32 {
            let mode = match depth {
                16 => "Grayscale",
                8 => "Indexed",
                _ => "an unknown",
            };
            return Err(format!("It's in {mode} color mode, with {depth} bits a pixel. Only RGBA sprites can be read."));
        }
        if frames == 0 {
            return Err("It has no frames.".to_string());
        }

        let mut layers: Vec<Layer> = Vec::new();
        let mut cels: Vec<Cel> = Vec::new();
        let frame_start = HEADER_SIZE;
        let mut frame = Reader { bytes, at: frame_start };
        let frame_size = frame.u32()? as usize;
        if frame.u16()? != FRAME_MAGIC {
            return Err("Its first frame's header is wrong.".to_string());
        }
        let old_chunk_count = frame.u16()?;
        let _duration = frame.u16()?;
        frame.skip(2)?;
        let new_chunk_count = frame.u32()?;
        let chunk_count = if new_chunk_count == 0 { u32::from(old_chunk_count) } else { new_chunk_count };
        let frame_end = frame_start.checked_add(frame_size).filter(|&end| end <= bytes.len()).ok_or("Its first frame runs past its end.")?;

        let mut at = frame_start + FRAME_HEADER_SIZE;
        for _ in 0..chunk_count {
            let mut chunk = Reader { bytes: &bytes[..frame_end], at };
            let size = chunk.u32()? as usize;
            let kind = chunk.u16()?;
            if size < CHUNK_HEADER_SIZE || at + size > frame_end {
                return Err(format!("A chunk of type {kind:#06X} runs past its frame's end."));
            }
            let data = &bytes[at + CHUNK_HEADER_SIZE..at + size];
            match kind {
                LAYER_CHUNK => layers.push(Layer::read(data, &layers, flags)?),
                CEL_CHUNK => cels.push(Cel::read(data)?),
                // Palettes, color profiles, tags, slices, user data and the
                // rest change nothing on the canvas.
                _ => {}
            }
            at += size;
        }

        // Bottom to top: a cel's order is its layer's index plus its z-index,
        // and the cel with the lower z-index goes first of two in the same
        // place.
        cels.sort_by_key(|cel| (cel.layer as i32 + i32::from(cel.z_index), cel.z_index));
        Ok(Frame {
            width: u32::from(width),
            height: u32::from(height),
            layers,
            cels,
        })
    }
}

struct Layer {
    name: String,
    kind: u16,
    child_level: u16,
    blend_mode: u16,
    opacity: u8,
    // Whether it shows: it's visible, and so is each group it's in, and it
    // isn't a reference layer.
    shown: bool,
    // Whether it's visible, and so is each group it's in, which a group's
    // layers need.
    visible: bool,
}

impl Layer {
    // The layer of a layer chunk. The layers before it say which group it's
    // in: the last one before it a level up.
    fn read(data: &[u8], before: &[Layer], header_flags: u32) -> Result<Layer, String> {
        let mut reader = Reader { bytes: data, at: 0 };
        let flags = reader.u16()?;
        let kind = reader.u16()?;
        let child_level = reader.u16()?;
        let _default_width = reader.u16()?;
        let _default_height = reader.u16()?;
        let blend_mode = reader.u16()?;
        let opacity = reader.u8()?;
        reader.skip(3)?;
        let name = reader.string()?;
        let group_visible = match child_level {
            0 => true,
            level => before.iter().rev().find(|layer| layer.child_level < level).is_none_or(|group| group.visible),
        };
        let visible = flags & LAYER_VISIBLE != 0 && group_visible;
        Ok(Layer {
            name,
            kind,
            child_level,
            blend_mode,
            opacity: if header_flags & LAYER_OPACITY_VALID != 0 { opacity } else { 255 },
            shown: visible && flags & LAYER_REFERENCE == 0 && (kind == NORMAL_LAYER || kind == TILEMAP_LAYER),
            visible,
        })
    }

    // The opacity that a cel of the layer blends at: its own times the
    // layer's.
    fn opacity_of(&self, cel: &Cel) -> u8 {
        ((u32::from(cel.opacity) * u32::from(self.opacity) + 127) / 255) as u8
    }
}

struct Cel {
    layer: usize,
    x: i16,
    y: i16,
    opacity: u8,
    z_index: i16,
    kind: u16,
    width: u16,
    height: u16,
    // The pixels of a raw cel, or the zlib stream of a compressed one.
    data: Vec<u8>,
}

impl Cel {
    fn read(data: &[u8]) -> Result<Cel, String> {
        let mut reader = Reader { bytes: data, at: 0 };
        let layer = usize::from(reader.u16()?);
        let x = reader.i16()?;
        let y = reader.i16()?;
        let opacity = reader.u8()?;
        let kind = reader.u16()?;
        let z_index = reader.i16()?;
        reader.skip(5)?;
        let (width, height) = match kind {
            RAW_CEL | COMPRESSED_CEL => (reader.u16()?, reader.u16()?),
            _ => (0, 0),
        };
        Ok(Cel {
            layer,
            x,
            y,
            opacity,
            z_index,
            kind,
            width,
            height,
            data: data[reader.at.min(data.len())..].to_vec(),
        })
    }

    // Blends the cel's pixels over the canvas, with the opacity, where they
    // fall on it.
    fn draw(&self, canvas: &mut Canvas, opacity: u8, layer: &str) -> Result<(), String> {
        let pixels = match self.kind {
            RAW_CEL => self.data.clone(),
            COMPRESSED_CEL => zlib_decompress(&self.data).map_err(|error| format!("A cel on the layer \"{layer}\" is broken: {error}"))?,
            // The first frame has no frame before it to link to.
            LINKED_CEL => return Err(format!("A cel on the layer \"{layer}\" in the first frame links to another frame.")),
            kind => return Err(format!("A cel on the layer \"{layer}\" is of type {kind}, which can't be read.")),
        };
        let (width, height) = (usize::from(self.width), usize::from(self.height));
        if pixels.len() < width * height * 4 {
            return Err(format!("A cel on the layer \"{layer}\" has fewer pixels than its size."));
        }
        for row in 0..height {
            let y = i64::from(self.y) + row as i64;
            if y < 0 || y >= i64::from(canvas.height) {
                continue;
            }
            for column in 0..width {
                let x = i64::from(self.x) + column as i64;
                if x < 0 || x >= i64::from(canvas.width) {
                    continue;
                }
                let at = (row * width + column) * 4;
                let source = [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]];
                let index = y as usize * canvas.width as usize + x as usize;
                canvas.pixels[index] = blend_normal(canvas.pixels[index], source, opacity);
            }
        }
        Ok(())
    }
}

// Aseprite's Normal blend mode: the source over the backdrop, with the
// source's alpha times the opacity, in colors that aren't premultiplied.
pub fn blend_normal(backdrop: [u8; 4], source: [u8; 4], opacity: u8) -> [u8; 4] {
    let source_alpha = f32::from(source[3]) * f32::from(opacity) / (255.0 * 255.0);
    if source_alpha <= 0.0 {
        return backdrop;
    }
    let backdrop_alpha = f32::from(backdrop[3]) / 255.0;
    let alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
    let mut result = [0u8; 4];
    for channel in 0..3 {
        let color = (f32::from(source[channel]) * source_alpha + f32::from(backdrop[channel]) * backdrop_alpha * (1.0 - source_alpha)) / alpha;
        result[channel] = color.round().clamp(0.0, 255.0) as u8;
    }
    result[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    result
}

// Reads the file's little-endian values, and fails as it runs out.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let bytes = self.bytes.get(self.at..self.at + N).ok_or("It ends early.")?;
        self.at += N;
        Ok(bytes.try_into().unwrap())
    }

    fn skip(&mut self, count: usize) -> Result<(), String> {
        if self.at + count > self.bytes.len() {
            return Err("It ends early.".to_string());
        }
        self.at += count;
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take()?))
    }

    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.take()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    // A length in a WORD, then that many bytes of UTF-8.
    fn string(&mut self) -> Result<String, String> {
        let length = usize::from(self.u16()?);
        let bytes = self.bytes.get(self.at..self.at + length).ok_or("It ends early.")?;
        self.at += length;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}
