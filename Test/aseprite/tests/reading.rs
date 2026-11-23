// .aseprite files that the tests write, a chunk at a time, read into the
// canvas that Aseprite shows, and Glyphs.aseprite at the root.

use std::path::Path;

use aseprite::{Canvas, adler32, blend_normal, load, read};

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const CLEAR: [u8; 4] = [0; 4];

// The header's flag that says layers' opacity is valid.
const LAYER_OPACITY_VALID: u32 = 1;
// Layers' flags and types.
const VISIBLE: u16 = 1;
const REFERENCE: u16 = 64;
const NORMAL: u16 = 0;
const GROUP: u16 = 1;
const TILEMAP: u16 = 2;

struct Chunk {
    kind: u16,
    data: Vec<u8>,
}

// A file of one frame with the chunks, in the format's layout.
fn file(width: u16, height: u16, depth: u16, flags: u32, chunks: &[Chunk]) -> Vec<u8> {
    let mut frame = Vec::new();
    for chunk in chunks {
        frame.extend((chunk.data.len() as u32 + 6).to_le_bytes());
        frame.extend(chunk.kind.to_le_bytes());
        frame.extend(&chunk.data);
    }
    let mut bytes = Vec::new();
    let size = 128 + 16 + frame.len() as u32;
    bytes.extend(size.to_le_bytes());
    bytes.extend(0xA5E0u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(width.to_le_bytes());
    bytes.extend(height.to_le_bytes());
    bytes.extend(depth.to_le_bytes());
    bytes.extend(flags.to_le_bytes());
    bytes.resize(128, 0);
    bytes.extend((16 + frame.len() as u32).to_le_bytes());
    bytes.extend(0xF1FAu16.to_le_bytes());
    bytes.extend((chunks.len() as u16).to_le_bytes());
    bytes.extend(100u16.to_le_bytes());
    bytes.extend([0, 0]);
    bytes.extend((chunks.len() as u32).to_le_bytes());
    bytes.extend(frame);
    assert_eq!(bytes.len() as u32, size);
    bytes
}

fn layer(flags: u16, kind: u16, child_level: u16, blend_mode: u16, opacity: u8, name: &str) -> Chunk {
    let mut data = Vec::new();
    for value in [flags, kind, child_level, 0, 0, blend_mode] {
        data.extend(value.to_le_bytes());
    }
    data.push(opacity);
    data.extend([0, 0, 0]);
    data.extend((name.len() as u16).to_le_bytes());
    data.extend(name.as_bytes());
    Chunk { kind: 0x2004, data }
}

fn visible_layer(name: &str) -> Chunk {
    layer(VISIBLE, NORMAL, 0, 0, 255, name)
}

fn cel_header(layer: u16, x: i16, y: i16, opacity: u8, kind: u16, z_index: i16) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend(layer.to_le_bytes());
    data.extend(x.to_le_bytes());
    data.extend(y.to_le_bytes());
    data.push(opacity);
    data.extend(kind.to_le_bytes());
    data.extend(z_index.to_le_bytes());
    data.extend([0; 5]);
    data
}

// A raw cel, of pixels `width` wide.
fn raw_cel(layer: u16, x: i16, y: i16, opacity: u8, z_index: i16, width: u16, pixels: &[[u8; 4]]) -> Chunk {
    let mut data = cel_header(layer, x, y, opacity, 0, z_index);
    data.extend(width.to_le_bytes());
    data.extend((pixels.len() as u16 / width).to_le_bytes());
    data.extend(pixels.iter().flatten());
    Chunk { kind: 0x2005, data }
}

// A compressed cel, whose pixels are in a zlib stream of stored blocks.
fn compressed_cel(layer: u16, x: i16, y: i16, width: u16, pixels: &[[u8; 4]]) -> Chunk {
    let mut data = cel_header(layer, x, y, 255, 2, 0);
    data.extend(width.to_le_bytes());
    data.extend((pixels.len() as u16 / width).to_le_bytes());
    data.extend(zlib_stored(&pixels.concat()));
    Chunk { kind: 0x2005, data }
}

fn zlib_stored(bytes: &[u8]) -> Vec<u8> {
    let mut stream = vec![0x78, 0x01];
    let mut blocks = bytes.chunks(65535).peekable();
    if blocks.peek().is_none() {
        stream.extend([1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(block) = blocks.next() {
        stream.push(u8::from(blocks.peek().is_none()));
        let length = block.len() as u16;
        stream.extend(length.to_le_bytes());
        stream.extend((!length).to_le_bytes());
        stream.extend(block);
    }
    stream.extend(adler32(bytes).to_be_bytes());
    stream
}

fn rows(canvas: &Canvas) -> Vec<Vec<[u8; 4]>> {
    canvas.pixels.chunks(canvas.width as usize).map(|row| row.to_vec()).collect()
}

#[test]
fn a_raw_cel_lands_where_it_is_and_the_rest_is_transparent() {
    let bytes = file(4, 3, 32, 0, &[visible_layer("Ink"), raw_cel(0, 1, 1, 255, 0, 2, &[RED, BLUE, GREEN, CLEAR])]);
    let canvas = read(&bytes).unwrap();
    assert_eq!((canvas.width, canvas.height), (4, 3));
    assert_eq!(
        rows(&canvas),
        [vec![CLEAR, CLEAR, CLEAR, CLEAR], vec![CLEAR, RED, BLUE, CLEAR], vec![CLEAR, GREEN, CLEAR, CLEAR]]
    );
}

#[test]
fn a_compressed_cel_reads_as_its_pixels() {
    let pixels = [RED, BLUE, GREEN, RED, BLUE, GREEN];
    let bytes = file(3, 2, 32, 0, &[visible_layer("Ink"), compressed_cel(0, 0, 0, 3, &pixels)]);
    assert_eq!(read(&bytes).unwrap().pixels, pixels);
}

#[test]
fn the_parts_of_a_cel_off_the_canvas_are_left_out() {
    let pixels = [RED, RED, RED, RED, BLUE, RED, RED, RED, GREEN];
    let bytes = file(2, 2, 32, 0, &[visible_layer("Ink"), raw_cel(0, -1, -1, 255, 0, 3, &pixels)]);
    assert_eq!(rows(&read(&bytes).unwrap()), [vec![BLUE, RED], vec![RED, GREEN]]);
}

#[test]
fn hidden_layers_layers_in_hidden_groups_and_reference_layers_don_t_show() {
    let chunks = [
        // 0: a hidden group, with a visible layer in it.
        layer(0, GROUP, 0, 0, 255, "Hidden group"),
        layer(VISIBLE, NORMAL, 1, 0, 255, "In the hidden group"),
        // 2: a visible group, with a visible layer in it, and a hidden one.
        layer(VISIBLE, GROUP, 0, 0, 255, "Visible group"),
        layer(VISIBLE, NORMAL, 1, 0, 255, "In the visible group"),
        layer(0, NORMAL, 1, 0, 255, "Hidden"),
        // 5: a reference layer.
        layer(VISIBLE | REFERENCE, NORMAL, 0, 0, 255, "Reference"),
        raw_cel(1, 0, 0, 255, 0, 1, &[RED]),
        raw_cel(3, 1, 0, 255, 0, 1, &[BLUE]),
        raw_cel(4, 2, 0, 255, 0, 1, &[GREEN]),
        raw_cel(5, 3, 0, 255, 0, 1, &[RED]),
    ];
    let canvas = read(&file(4, 1, 32, 0, &chunks)).unwrap();
    assert_eq!(canvas.pixels, [CLEAR, BLUE, CLEAR, CLEAR]);
}

#[test]
fn a_group_after_a_hidden_group_s_layers_shows_its_own() {
    let chunks = [
        layer(0, GROUP, 0, 0, 255, "Hidden group"),
        layer(VISIBLE, NORMAL, 1, 0, 255, "In the hidden group"),
        layer(VISIBLE, GROUP, 0, 0, 255, "Visible group"),
        layer(VISIBLE, GROUP, 1, 0, 255, "Visible group in it"),
        layer(VISIBLE, NORMAL, 2, 0, 255, "Two levels down"),
        raw_cel(1, 0, 0, 255, 0, 1, &[RED]),
        raw_cel(4, 1, 0, 255, 0, 1, &[BLUE]),
    ];
    assert_eq!(read(&file(2, 1, 32, 0, &chunks)).unwrap().pixels, [CLEAR, BLUE]);
}

#[test]
fn layers_go_bottom_up_and_blend_at_their_cel_s_and_layer_s_opacity() {
    let chunks = |layer_opacity: u8| {
        [
            visible_layer("Bottom"),
            layer(VISIBLE, NORMAL, 0, 0, layer_opacity, "Top"),
            raw_cel(0, 0, 0, 255, 0, 2, &[RED, RED]),
            // Half opaque on its own, and a quarter at the layer's half.
            raw_cel(1, 0, 0, 128, 0, 2, &[BLUE, [0, 0, 255, 0]]),
        ]
    };
    let canvas = read(&file(2, 1, 32, LAYER_OPACITY_VALID, &chunks(128))).unwrap();
    assert_eq!(canvas.pixels[0], blend_normal(RED, BLUE, 64));
    assert_eq!(canvas.pixels[1], RED);
    // Without the header's flag, layers' opacity counts for nothing.
    let canvas = read(&file(2, 1, 32, 0, &chunks(128))).unwrap();
    assert_eq!(canvas.pixels[0], blend_normal(RED, BLUE, 128));
}

#[test]
fn normal_blending_mixes_by_the_source_s_alpha() {
    assert_eq!(blend_normal(RED, BLUE, 255), BLUE);
    assert_eq!(blend_normal(RED, BLUE, 0), RED);
    assert_eq!(blend_normal(CLEAR, BLUE, 128), [0, 0, 255, 128]);
    let half = blend_normal(RED, BLUE, 128);
    assert_eq!(half, [127, 0, 128, 255]);
    // Over half-transparent red, half-transparent blue weighs more.
    let over = blend_normal([255, 0, 0, 128], [0, 0, 255, 128], 255);
    assert_eq!(over[3], 192);
    assert!(over[2] > over[0], "{over:?}");
}

#[test]
fn a_cel_s_z_index_moves_it_up_through_the_layers() {
    let chunks = [visible_layer("Bottom"), visible_layer("Top"), raw_cel(0, 0, 0, 255, 2, 1, &[RED]), raw_cel(1, 0, 0, 255, 0, 1, &[BLUE])];
    assert_eq!(read(&file(1, 1, 32, 0, &chunks)).unwrap().pixels, [RED]);
}

#[test]
fn what_it_can_t_read_fails_and_says_why() {
    let error = |bytes: &[u8]| read(bytes).unwrap_err();
    let cel = || raw_cel(0, 0, 0, 255, 0, 1, &[RED]);
    assert!(error(&file(1, 1, 8, 0, &[])).contains("Indexed"));
    assert!(error(&file(1, 1, 16, 0, &[])).contains("Grayscale"));
    assert!(error(b"Not a sprite").contains("Aseprite"));
    let whole = file(1, 1, 32, 0, &[visible_layer("Ink"), cel()]);
    for length in [0, 10, 130, whole.len() - 1] {
        assert!(read(&whole[..length]).is_err(), "{length} bytes read");
    }
    let tilemap = file(1, 1, 32, 0, &[layer(VISIBLE, TILEMAP, 0, 0, 255, "Tiles"), cel()]);
    assert!(error(&tilemap).contains("tilemap"));
    let multiply = file(1, 1, 32, 0, &[layer(VISIBLE, NORMAL, 0, 1, 255, "Multiply"), cel()]);
    assert!(error(&multiply).contains("blend mode"));
    // Hidden, they change nothing on the canvas, so they're fine.
    let hidden = file(1, 1, 32, 0, &[layer(0, TILEMAP, 0, 0, 255, "Tiles"), layer(0, NORMAL, 0, 1, 255, "Multiply"), cel()]);
    assert_eq!(read(&hidden).unwrap().pixels, [CLEAR]);
}

#[test]
fn the_glyphs_at_the_root_read() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Glyphs.aseprite");
    let canvas = load(&path).unwrap();
    assert!(canvas.width > 0 && canvas.height > 0);
    assert!(canvas.pixels.iter().any(|pixel| pixel[3] == 255), "nothing is drawn");
    let error = load(&path.with_extension("missing")).unwrap_err();
    assert!(error.contains("Glyphs.missing"), "{error}");
}
