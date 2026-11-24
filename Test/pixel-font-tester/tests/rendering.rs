// The tester's frames, drawn offscreen with the GPU, as the window draws
// them, against the same frames drawn on the CPU from the scene's quads: each
// glyph's pixel is exactly `scale` × `scale` of the frame's, in the glyph's
// coverage and the text's color, on the display's pixel grid, with nothing
// filtered or blended across pixels' edges. Glyphs.aseprite's pixels are
// opaque or transparent, so the frames match exactly.

mod common;

use common::{real_font, the_box};
use graphics::{AdapterChoice, Gpu, Image, Offscreen};
use pixel_font_tester::font::Font;
use pixel_font_tester::layout::{WINDOW_HEIGHT, WINDOW_WIDTH};
use pixel_font_tester::renderer::Renderer;
use pixel_font_tester::scene::{SOLID, Scene, scene};
use pixel_font_tester::tester::{Input, Key, Modifiers, Tester, Theme};
use pixel_font_tester::text::Text;

// The scene's pixels, red, green and blue, drawn on the CPU.
fn reference(scene: &Scene, font: &Font) -> Vec<[u8; 3]> {
    let (width, height) = (WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize);
    let mut pixels = vec![scene.background; width * height];
    for quad in &scene.quads {
        let color = [quad.color as u8, (quad.color >> 8) as u8, (quad.color >> 16) as u8];
        for y in quad.area.top..quad.area.bottom {
            for x in quad.area.left..quad.area.right {
                let pixel = &mut pixels[y as usize * width + x as usize];
                let coverage = if quad.glyph == SOLID {
                    255
                } else {
                    let texel_x = ((x - quad.origin[0]) / quad.scale as i32) as u32;
                    let texel_y = ((y - quad.origin[1]) / quad.scale as i32) as u32;
                    u32::from(font.coverage(quad.glyph as u8, texel_x, texel_y))
                };
                for channel in 0..3 {
                    let blended = (u32::from(color[channel]) * coverage + u32::from(pixel[channel]) * (255 - coverage) + 127) / 255;
                    pixel[channel] = blended as u8;
                }
            }
        }
    }
    pixels
}

fn draw(gpu: &mut Gpu, renderer: &mut Renderer, scene: &Scene) -> Image {
    // Made to be cleared to the scene's background, as the renderer does.
    let [r, g, b] = scene.background.map(|channel| f32::from(channel) / 255.0);
    let mut offscreen = Offscreen::new(gpu, WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, [r, g, b, 1.0]).unwrap();
    let frame = offscreen.begin().unwrap();
    renderer.draw(gpu, &frame, scene).unwrap();
    offscreen.finish(gpu, frame).unwrap()
}

fn assert_matches(image: &Image, expected: &[[u8; 3]], what: &str) {
    let mut wrong = 0;
    let mut first = None;
    for y in 0..image.height {
        for x in 0..image.width {
            let [r, g, b, _] = image.pixel(x, y);
            let want = expected[(y * image.width + x) as usize];
            if [r, g, b] != want {
                wrong += 1;
                first.get_or_insert((x, y, [r, g, b], want));
            }
        }
    }
    assert_eq!(wrong, 0, "{what}: {wrong} pixels differ, the first at {first:?} (x, y, drawn, expected)");
}

#[test]
fn frames_on_the_gpu_match_the_scene_pixel_for_pixel_at_every_size() {
    let font = real_font();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut renderer = Renderer::new(&mut gpu, &font).unwrap();
    // Every character, with a line long enough at the bigger sizes to be cut
    // at the right, and enough lines to be cut at the bottom.
    let printable: String = (0x20u8..0x7F).map(char::from).collect();
    let text = format!("{printable}\tend\nThe quick brown fox jumps over the lazy dog\n{}", "gjpqy Wm\n".repeat(30));
    let cases = [(1, Theme::Light, 2), (2, Theme::Dark, 0), (3, Theme::Light, 1), (7, Theme::Dark, 9), (10, Theme::Light, 2)];
    for (scale, theme, spacing) in cases {
        let mut tester = Tester::new(Text::typed(&text), scale, theme, true, &font);
        tester.set_spacing(spacing, &font);
        // From the top left, so that the long line is cut at the right.
        tester.handle(Input::Key(Key::Home, Modifiers::CONTROL), &font);
        let scene = scene(&tester, &font, &[]);
        let image = draw(&mut gpu, &mut renderer, &scene);
        assert_matches(&image, &reference(&scene, &font), &format!("scale {scale}, spacing {spacing}"));
    }
}

#[test]
fn a_scrolled_frame_and_the_setup_s_problems_match_too() {
    let font = real_font();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut renderer = Renderer::new(&mut gpu, &font).unwrap();
    let text = format!("{}END\n{}", "abcdefghij".repeat(40), "line\n".repeat(100));
    let mut tester = Tester::new(Text::typed(&text), 4, Theme::Dark, true, &font);
    tester.handle(Input::Key(Key::Up, Modifiers::NONE), &font);
    assert!(tester.scroll.line > 0);
    let scrolled = scene(&tester, &font, &[]);
    assert_matches(&draw(&mut gpu, &mut renderer, &scrolled), &reference(&scrolled, &font), "scrolled");

    let problems = ["The display is 1920 x 1080 pixels. It must be 2560 x 1440.".to_string()];
    let error = scene(&tester, &font, &problems);
    assert_matches(&draw(&mut gpu, &mut renderer, &error), &reference(&error, &font), "problems");
}

// What the GPU draws on the frame that Tab is pressed: the outline of a whole
// cell, in the text's color, where the tab is.
#[test]
fn pressing_tab_draws_the_box() {
    let font = real_font();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut renderer = Renderer::new(&mut gpu, &font).unwrap();
    let scale = 4;
    let mut tester = Tester::new(Text::new(), scale, Theme::Dark, true, &font);
    tester.handle(Input::Character(0x09), &font);
    let scene = scene(&tester, &font, &[]);
    let image = draw(&mut gpu, &mut renderer, &scene);
    let quad = scene.quads.iter().find(|quad| quad.glyph == u32::from(b'\t')).unwrap();
    let colors = Theme::Dark.colors();
    let the_box = the_box(&font);
    for y in 0..font.glyph_height {
        for x in 0..font.glyph_width {
            let expected = if the_box[(y * font.glyph_width + x) as usize] > 0 { colors.text } else { colors.field };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = (quad.origin[0] + (x * scale + dx) as i32) as u32;
                    let py = (quad.origin[1] + (y * scale + dy) as i32) as u32;
                    let [r, g, b, _] = image.pixel(px, py);
                    assert_eq!([r, g, b], expected, "the box's pixel {x}, {y} at {px}, {py}");
                }
            }
        }
    }
}

#[test]
fn a_glyph_s_pixels_are_scale_by_scale_blocks_in_the_text_s_color() {
    let font = real_font();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut renderer = Renderer::new(&mut gpu, &font).unwrap();
    let tester = Tester::new(Text::typed("A"), 5, Theme::Light, true, &font);
    let scene = scene(&tester, &font, &[]);
    let image = draw(&mut gpu, &mut renderer, &scene);
    let quad = scene.quads.iter().find(|quad| quad.glyph == u32::from(b'A')).unwrap();
    let colors = Theme::Light.colors();
    for y in 0..font.glyph_height {
        for x in 0..font.glyph_width {
            let expected = if font.coverage(b'A', x, y) > 0 { colors.text } else { colors.field };
            for dy in 0..5 {
                for dx in 0..5 {
                    let px = (quad.origin[0] + (x * 5 + dx) as i32) as u32;
                    let py = (quad.origin[1] + (y * 5 + dy) as i32) as u32;
                    let [r, g, b, _] = image.pixel(px, py);
                    assert_eq!([r, g, b], expected, "the glyph's pixel {x}, {y} at {px}, {py}");
                }
            }
        }
    }
}
