// Library/Canvas without a GPU: the fonts of the type ramp and the icons,
// measuring, fitting and wrapping text, the instances that shapes and text
// make, clipping, kept instances, colors, and that each theme's text can be
// read on its surfaces.

use std::sync::OnceLock;

use canvas::theme::{DARK, LIGHT};
use canvas::{Cached, Canvas, Color, ELLIPSIS, Fonts, ICON_SIZES, Icon, Instance, Rect, TextStyle, Theme};

fn fonts() -> &'static Fonts {
    static FONTS: OnceLock<Fonts> = OnceLock::new();
    FONTS.get_or_init(|| Fonts::new().expect("the fonts are installed"))
}

#[test]
fn the_type_ramp_is_windows_11s_fonts_at_their_sizes() {
    let fonts = fonts();
    assert_eq!(
        fonts.families,
        [
            "Segoe UI Variable Small",
            "Segoe UI Variable Text",
            "Segoe UI Variable Text",
            "Segoe UI Variable Display",
            "Segoe UI Variable Display",
            "Cascadia Mono",
        ]
    );
    let lines: Vec<f32> = TextStyle::ALL.iter().map(|style| fonts.line_height(*style)).collect();
    assert_eq!(lines, [16.0, 20.0, 20.0, 28.0, 36.0, 20.0]);
    // Semibold is wider than regular, and bigger is wider.
    let width = |style| fonts.measure("Hello, World", style);
    assert!(width(TextStyle::BodyStrong) > width(TextStyle::Body));
    assert!(width(TextStyle::Body) > width(TextStyle::Caption));
    assert!(width(TextStyle::Title) > width(TextStyle::Subtitle));
    // Proportional, but for Mono.
    assert!(fonts.measure("iiii", TextStyle::Body) < fonts.measure("WWWW", TextStyle::Body));
    assert_eq!(fonts.measure("iiii", TextStyle::Mono), fonts.measure("WWWW", TextStyle::Mono));
}

#[test]
fn every_icon_has_a_glyph_at_every_size() {
    for icon in Icon::ALL {
        for size in ICON_SIZES {
            let glyph = fonts().icon(icon, size).unwrap_or_else(|| panic!("{icon:?} at {size}"));
            let [left, top, right, bottom] = glyph.texels;
            assert!(right > left && bottom > top, "{icon:?} at {size} is empty");
            assert!(right - left <= size + 2.0, "{icon:?} at {size} is too wide");
        }
    }
}

#[test]
fn the_fonts_have_latin_1_and_a_few_marks_but_not_everything() {
    for character in ['a', 'Z', '~', '\u{E9}', '\u{FF}', '\u{2026}', '\u{2212}', '\u{2192}'] {
        assert!(Fonts::has_glyph(character), "{character}");
    }
    for character in ['\u{4E2D}', '\u{1F600}', '\u{7F}'] {
        assert!(!Fonts::has_glyph(character), "{character}");
    }
    // What they lack is measured as a question mark.
    assert_eq!(fonts().measure("\u{4E2D}", TextStyle::Body), fonts().measure("?", TextStyle::Body));
}

#[test]
fn text_is_cut_short_with_an_ellipsis_and_wraps_at_spaces() {
    let mut instances = Vec::new();
    let canvas = Canvas::new(fonts(), &mut instances);
    let text = "A sentence that is too long for the room";
    let width = canvas.measure("A sentence that", TextStyle::Body);
    let fitted = canvas.fit(text, TextStyle::Body, width);
    assert!(fitted.ends_with(ELLIPSIS));
    assert!(canvas.measure(&fitted, TextStyle::Body) <= width);
    assert_eq!(canvas.fit("Short", TextStyle::Body, width), "Short");
    let lines = fonts().wrap(text, TextStyle::Body, width, 0.0);
    assert!(lines.len() > 1);
    assert!(lines.iter().all(|line| fonts().measure(line, TextStyle::Body) <= width));
    assert_eq!(lines.join(" "), text);
    // A newline starts a line, and a word wider than the room stays whole.
    assert_eq!(fonts().wrap("one\ntwo", TextStyle::Body, 1000.0, 0.0), ["one", "two"]);
    assert_eq!(fonts().wrap("unbreakable", TextStyle::Body, 10.0, 0.0), ["unbreakable"]);
}

#[test]
fn shapes_and_glyphs_are_one_instance_each_and_keep_the_clip() {
    let mut instances: Vec<Instance> = Vec::new();
    let mut canvas = Canvas::new(fonts(), &mut instances);
    canvas.fill(Rect::new(0.0, 0.0, 10.0, 10.0), 4.0, LIGHT.accent);
    // Transparent and empty shapes are left out.
    canvas.fill(Rect::new(0.0, 0.0, 10.0, 10.0), 4.0, Color::TRANSPARENT);
    canvas.rect(Rect::new(0.0, 0.0, 0.0, 10.0), LIGHT.accent);
    canvas.push_clip(Rect::new(10.0, 10.0, 100.0, 100.0));
    canvas.push_clip(Rect::new(50.0, 0.0, 100.0, 50.0));
    // A glyph for each character but the space.
    let end = canvas.text([60.0, 20.0], "Hi you", TextStyle::Body, LIGHT.text);
    canvas.pop_clip();
    canvas.line([0.0, 0.0], [10.0, 10.0], 2.0, LIGHT.text);
    canvas.pop_clip();
    canvas.circle([5.0, 5.0], 5.0, LIGHT.text);
    assert_eq!(end, 60.0 + canvas.measure("Hi you", TextStyle::Body));
    assert_eq!(instances.len(), 1 + 5 + 1 + 1);
    assert_eq!(instances[1].clip, [50.0, 10.0, 110.0, 50.0]);
    assert_eq!(instances[6].clip, [10.0, 10.0, 110.0, 110.0]);
    assert!(instances[7].clip[0] < -1000.0);
    // Baselines are on whole pixels.
    assert!(instances[1..6].iter().all(|glyph| glyph.bounds[1].fract() == 0.0));
}

#[test]
fn kept_instances_are_added_again_while_their_key_is_the_same() {
    let mut cached = Cached::default();
    let mut draws = 0;
    for key in [1, 1, 2] {
        let mut instances = Vec::new();
        let mut canvas = Canvas::new(fonts(), &mut instances);
        cached.draw(&mut canvas, key, |canvas| {
            draws += 1;
            canvas.circle([10.0, 10.0], 5.0, LIGHT.accent);
        });
        assert_eq!(instances.len(), 1);
    }
    assert_eq!(draws, 2);
}

#[test]
fn colors_blend_in_srgb() {
    let red = Color::rgb(0xFF0000);
    assert_eq!(red.0, 0xFF00_00FF);
    assert_eq!(Color::rgba(0x0000FF, 0.5).0 >> 24, 128);
    assert_eq!(red.mix(Color::rgb(0x0000FF), 0.5), Color::rgb(0x800080));
    // Half is 128 of 255, so a little more than half.
    assert_eq!(Color::rgba(0x000000, 0.5).over(Color::rgb(0xFFFFFF)), Color::rgb(0x7F7F7F));
    assert_eq!(red.alpha(0.0), Color(0x0000_00FF));
}

// WCAG's contrast ratio between two opaque sRGB colors.
fn contrast(a: Color, b: Color) -> f32 {
    let luminance = |color: Color| {
        let [red, green, blue, _] = color.to_floats().map(|channel| {
            if channel <= 0.04045 { channel / 12.92 } else { ((channel + 0.055) / 1.055).powf(2.4) }
        });
        0.2126 * red + 0.7152 * green + 0.0722 * blue
    };
    let (light, dark) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
    (light + 0.05) / (dark + 0.05)
}

#[test]
fn each_themes_text_can_be_read_on_its_surfaces() {
    let themes: [&Theme; 2] = [&LIGHT, &DARK];
    for theme in themes {
        for surface in [theme.background, theme.card, theme.flyout, theme.control] {
            assert!(contrast(theme.text, surface) >= 7.0, "{} text", theme.name);
            assert!(contrast(theme.text_secondary, surface) >= 4.5, "{} secondary text", theme.name);
        }
        assert!(contrast(theme.on_accent, theme.accent) >= 4.5, "{} text on the accent", theme.name);
        assert!(contrast(theme.accent_text, theme.card) >= 4.5, "{} accent text", theme.name);
        for (status, background) in [
            (theme.success, theme.success_background),
            (theme.caution, theme.caution_background),
            (theme.critical, theme.critical_background),
        ] {
            assert!(contrast(theme.text, background) >= 7.0, "{} info bar text", theme.name);
            assert!(contrast(status, background) >= 3.0, "{} info bar icon", theme.name);
        }
    }
}
