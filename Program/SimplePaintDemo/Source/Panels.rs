// Lays out and draws the demo's cards with Library/UserInterface, and
// handles what the input does to the editor. Programs support two client
// areas, 2560 x 1369 maximized and 2560 x 1440 in borderless fullscreen (see
// Library/Application), so the cards have fixed places in pixels. In
// fullscreen, the GPU constants card at the bottom right is taller, and the
// status bar stays at the bottom.
//
// The preview card is drawn with a well in its middle, which the sphere's
// image is copied over after the canvas is drawn.
//
// The tone curve card's curves and color ramp are most of the cards'
// instances and math, so they're drawn only when the base color or
// Brightness changes, and their instances are kept for the frames in
// between. See Cache.

use std::path::Path;

use simple_paint::Parameters;
use user_interface::{
    Align, Button, CARD_PADDING, CONTROL_HEIGHT, Cached, Canvas, Color, Frame, Icon, Lines, MARGIN, PANELS_TOP, SPACE_L,
    PILL_SIZE, RADIUS_CONTROL, Rect, SPACE_M, SPACE_S, SPACE_XS, Stack, TextStyle, Theme, card, well,
};

use crate::editor::{Editor, SLIDERS};
use crate::preview::IMAGE_SIZE;

const PREVIEW_X: f32 = MARGIN;
const PREVIEW_WIDTH: f32 = IMAGE_SIZE as f32 + CARD_PADDING * 2.0;
pub const EDITOR_X: f32 = PREVIEW_X + PREVIEW_WIDTH + MARGIN;
// Each slider's row, and the parts of it.
const ROW_HEIGHT: f32 = 40.0;
const LABEL_WIDTH: f32 = 120.0;
const VALUE_WIDTH: f32 = 96.0;
const RANGE_WIDTH: f32 = 168.0;
const SETTINGS_HEIGHT: f32 = 512.0;
const CURVE_HEIGHT: f32 = 488.0;
const PLOT_HEIGHT: f32 = 280.0;
const RAMP_HEIGHT: f32 = 28.0;
const SWATCH_WIDTH: f32 = 120.0;

// Where things are in what was drawn, besides the widgets, which the
// interface knows.
pub struct Layout {
    // The top left of the sphere's image.
    pub image: Option<[u32; 2]>,
}

// What draw keeps from frame to frame: the tone curve card's curves and
// color ramp, each by what it shows (see CurveKey).
#[derive(Default)]
pub struct Cache {
    curves: Cached<CurveKey>,
    ramp: Cached<CurveKey>,
}

// The base color and Brightness, which the color curve depends on, and the
// area it's drawn in.
type CurveKey = ([f64; 3], f64, [f32; 4]);

// Handles the frame's input, and draws the cards. The editor's sliders don't
// take keyboard focus, as the arrow keys move the selected one, which the
// program's keys select.
pub fn draw(frame: &mut Frame, cache: &mut Cache, editor: &mut Editor, paint_path: &Path) -> Layout {
    let keys: Vec<_> = frame.program_events().copied().collect();
    for event in &keys {
        editor.key(event, paint_path);
    }
    frame.set_focusable(false);

    let theme = frame.theme();
    let adapter = frame.status.adapter;
    frame.header("SimplePaint Demo", "SimplePaint on a UV sphere, with an editor for its settings");
    let bottom = frame.footer(
        &[
            ("F11", "fullscreen"),
            ("Up/Down", "select a setting"),
            ("Left/Right", "change it, 10 steps with Shift"),
            ("Ctrl+S", "save to Paint.json"),
        ],
        &[(adapter, theme.text_secondary)],
    );

    // The preview, which takes the left side, with a well for the image.
    let preview = Rect::from_edges(PREVIEW_X, PANELS_TOP, PREVIEW_X + PREVIEW_WIDTH, bottom);
    let inner = frame.card(
        preview,
        "Preview",
        "A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.",
    );
    let image = Rect::new(inner.x, inner.y, IMAGE_SIZE as f32, IMAGE_SIZE as f32);
    frame.canvas.fill(image, RADIUS_CONTROL, theme.well);
    let layout = Layout {
        image: Some([image.x as u32, image.y.ceil() as u32]),
    };

    // The editor, in three cards down the right side.
    let right = frame.width - MARGIN;
    let settings = Rect::from_edges(EDITOR_X, PANELS_TOP, right, PANELS_TOP + SETTINGS_HEIGHT);
    let curve = Rect::new(EDITOR_X, settings.bottom() + MARGIN, settings.width, CURVE_HEIGHT);
    let constants = Rect::from_edges(EDITOR_X, curve.bottom() + MARGIN, right, bottom);
    draw_settings(frame, editor, settings, paint_path);
    draw_curve(&mut frame.canvas, theme, cache, &editor.parameters, curve);
    draw_constants(&mut frame.canvas, theme, editor, constants);
    layout
}

fn draw_settings(frame: &mut Frame, editor: &mut Editor, area: Rect, paint_path: &Path) {
    let inner = frame.card(area, "Settings", "Material::compile checks them, and the sliders stay in range");
    let theme = frame.theme();
    let mut stack = Stack::down(inner, 0.0);
    editor.dragging = None;
    for (index, slider) in SLIDERS.iter().enumerate() {
        let row = stack.next(ROW_HEIGHT);
        let selected = index == editor.selected;
        // The selected row stands out like a list's selected row, from the
        // card's edge.
        if selected {
            let face = Rect::from_edges(row.x - CARD_PADDING + SPACE_XS, row.y + 2.0, row.right() + CARD_PADDING - SPACE_XS, row.bottom() - 2.0);
            frame.canvas.fill(face, RADIUS_CONTROL, theme.subtle_selected);
            let [pill_width, pill_height] = PILL_SIZE;
            let pill = Rect::new(face.x, face.center()[1] - pill_height * 0.5, pill_width, pill_height);
            frame.canvas.fill(pill, pill_width * 0.5, theme.accent);
        }
        let (label, rest) = row.split_left(LABEL_WIDTH, 0.0);
        let (range_text, rest) = rest.split_right(RANGE_WIDTH, SPACE_M);
        let (value_text, track) = rest.split_right(VALUE_WIDTH, SPACE_M);
        let mut value = slider.value(&editor.parameters);
        let response = frame.slider(slider.name, track, &mut value, &slider.range());
        // A press or the wheel selects it.
        if response.pressed || response.wheel != 0.0 {
            editor.selected = index;
        }
        if response.changed {
            editor.change(index, value);
        }
        if response.held {
            editor.dragging = Some(index);
        }
        let style = if selected { TextStyle::BodyStrong } else { TextStyle::Body };
        frame.label(label, slider.name, style, theme.text, Align::Start);
        frame.label(value_text, &slider.format(value), TextStyle::Mono, theme.text, Align::End);
        let range = format!("{:.2$} to {}", slider.minimum, slider.format(slider.maximum), slider.decimals);
        frame.label(range_text, &range, TextStyle::Body, theme.text_secondary, Align::Start);
    }

    // The base color, as the sliders make it, in sRGB and in linear.
    stack.skip(SPACE_M);
    let row = stack.next(CONTROL_HEIGHT);
    let parameters = editor.parameters;
    let [red, green, blue] = parameters.base_color_srgb;
    let byte = |channel: f64| (channel * 255.0).round() as u32;
    let swatch_color = Color::rgb((byte(red) << 16) | (byte(green) << 8) | byte(blue));
    let (label, rest) = row.split_left(LABEL_WIDTH, 0.0);
    frame.label(label, "Base color", TextStyle::Body, theme.text, Align::Start);
    let swatch = Rect::new(rest.x + 1.0, rest.y, SWATCH_WIDTH, rest.height);
    frame.canvas.fill_stroke(swatch, RADIUS_CONTROL, swatch_color, 1.0, theme.control_stroke);
    let linear = parameters.base_color_srgb.map(linear);
    let text = Rect::from_edges(swatch.right() + SPACE_M, row.y, row.right(), row.bottom());
    let mut x = frame.label(text, &format!("#{:02X}{:02X}{:02X}", byte(red), byte(green), byte(blue)), TextStyle::Mono, theme.text, Align::Start);
    for (label, value) in [
        ("linear", format!("{:.4}, {:.4}, {:.4}", linear[0], linear[1], linear[2])),
        ("appears at tone 1 - Brightness =", format!("{:.3}", 1.0 - parameters.brightness)),
    ] {
        let text = Rect::from_edges(x + SPACE_M * 2.0, row.y, row.right(), row.bottom());
        x = frame.label(text, label, TextStyle::Body, theme.text_secondary, Align::Start);
        let text = Rect::from_edges(x + SPACE_S, row.y, row.right(), row.bottom());
        x = frame.label(text, &value, TextStyle::Mono, theme.text, Align::Start);
    }

    // Buttons, and what the last save did.
    stack.skip(SPACE_L);
    let row = stack.next(CONTROL_HEIGHT);
    let mut buttons = Stack::across(row, SPACE_S);
    for (label, button) in [
        ("Reset", Button::new("Reset").icon(Icon::Refresh)),
        ("Save", Button::new("Save").icon(Icon::Save).accent()),
    ] {
        let place = buttons.next(frame.button_width(&button).max(96.0));
        if frame.button(label, place, button).clicked {
            match label {
                "Reset" => editor.reset(),
                _ => editor.save(paint_path),
            }
        }
    }
    let status = buttons.rest().inset_by(SPACE_S, 0.0);
    let (icon, message, color) = match &editor.status {
        Some((message, true)) => (Some(Icon::Error), message.as_str(), theme.critical),
        Some((message, false)) => (Some(Icon::Success), message.as_str(), theme.success),
        None => (None, "Reset goes back to SimplePaint's defaults.", theme.text_secondary),
    };
    let text = match icon {
        Some(icon) => {
            frame.canvas.icon([status.x + 8.0, status.center()[1]], icon, 16.0, color);
            Rect::from_edges(status.x + 16.0 + SPACE_S, status.y, status.right(), status.bottom())
        }
        None => status,
    };
    let text_color = if icon.is_some() { theme.text } else { color };
    frame.label(text, message, TextStyle::Body, text_color, Align::Start);

    stack.skip(SPACE_M);
    let mut lines = Lines::new(stack.rest());
    lines.wrapped(
        &mut frame.canvas,
        "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and Rotation by 0.1 \
         degrees, and stop at the last step that SimplePaint accepts.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

// The curves' colors, one for each channel.
fn channels(theme: &Theme) -> [Color; 3] {
    [theme.critical, theme.success, theme.accent]
}

// The color curve of each channel over the tone, as it's displayed, with the
// tones that the sphere shows marked, and a ramp of the colors.
fn draw_curve(canvas: &mut Canvas, theme: &Theme, cache: &mut Cache, parameters: &Parameters, area: Rect) {
    let inner = card(
        canvas,
        theme,
        area,
        "Tone curve",
        "Each channel's displayed value over the tone, from 0 at the left to 1 at the right",
    );
    let line_height = canvas.line_height(TextStyle::Body);
    let plot = Rect::new(inner.x, inner.y, inner.width, PLOT_HEIGHT);
    well(canvas, theme, plot);
    for quarter in 1..4 {
        let fraction = quarter as f32 * 0.25;
        canvas.rect(Rect::new((plot.x + plot.width * fraction).round(), plot.y, 1.0, plot.height), theme.divider);
        canvas.rect(Rect::new(plot.x, (plot.y + plot.height * fraction).round(), plot.width, 1.0), theme.divider);
    }
    let x_of = |tone: f64| plot.x + plot.width * tone as f32;
    let y_of = |value: f64| plot.bottom() - plot.height * value as f32;

    // The tones from Dark Point at the sphere's rim to Light Point at its peak.
    let (dark, light) = (parameters.dark_point, parameters.light_point);
    let (low, high) = (dark.min(light), dark.max(light));
    let span = Rect::from_edges(x_of(low), plot.y, x_of(high).max(x_of(low) + 1.0), plot.bottom());
    canvas.rect(span, theme.accent.alpha(0.08));
    let anchor = 1.0 - parameters.brightness;
    for (tone, color) in [(dark, theme.text_secondary), (light, theme.text_secondary), (anchor, theme.text_tertiary)] {
        canvas.rect(Rect::new(x_of(tone) - 0.5, plot.y, 1.0, plot.height), color);
    }

    // Each channel's curve, as displayed: the shader's linear output, encoded
    // to sRGB.
    const SEGMENTS: usize = 256;
    let curves = parameters.base_color_srgb.map(|base| curve(base, parameters.brightness));
    let key = |place: Rect| {
        (
            parameters.base_color_srgb,
            parameters.brightness,
            [place.x, place.y, place.width, place.height],
        )
    };
    cache.curves.draw(canvas, key(plot), |canvas| {
        for (curve, color) in curves.iter().zip(channels(theme)) {
            let points: Vec<[f32; 2]> = (0..=SEGMENTS)
                .map(|segment| {
                    let tone = segment as f64 / SEGMENTS as f64;
                    [x_of(tone), y_of(srgb(curve(tone)))]
                })
                .collect();
            canvas.polyline(&points, 2.0, color);
        }
    });

    // Labels under the plot, at the marks, on as many lines as there are
    // labels, so that they never overlap, even when they're all at one tone.
    let y = plot.bottom() + SPACE_XS;
    let labels = [
        (dark, format!("Dark Point {}", fixed(dark)), theme.text_secondary),
        (light, format!("Light Point {}", fixed(light)), theme.text_secondary),
        (anchor, format!("Base color {}", fixed(anchor)), theme.text_tertiary),
    ];
    let spans: Vec<[f32; 2]> = labels
        .iter()
        .map(|(tone, label, _)| {
            let label_width = canvas.measure(label, TextStyle::Body);
            let x = (x_of(*tone) - label_width * 0.5).clamp(plot.x, plot.right() - label_width);
            [x, x + label_width]
        })
        .collect();
    let lines = label_lines(&spans, 16.0);
    for ((_, label, color), (span, line)) in labels.iter().zip(spans.iter().zip(lines)) {
        canvas.text([span[0], y + line_height * line as f32], label, TextStyle::Body, *color);
    }
    let y = y + line_height * labels.len() as f32 + SPACE_XS;

    // The colors along the curve, with the sphere's tones marked.
    let ramp = Rect::new(plot.x, y, plot.width, RAMP_HEIGHT);
    cache.ramp.draw(canvas, key(ramp), |canvas| {
        let columns = ramp.width as usize;
        for column in 0..columns {
            let tone = (column as f64 + 0.5) / columns as f64;
            let [red, green, blue] = curves.each_ref().map(|curve| {
                let value = srgb(curve(tone));
                (value.clamp(0.0, 1.0) * 255.0).round() as u32
            });
            let color = Color::rgb((red << 16) | (green << 8) | blue);
            canvas.rect(Rect::new(ramp.x + column as f32, ramp.y, 1.0, ramp.height), color);
        }
    });
    canvas.stroke(ramp, 0.0, 1.0, theme.control_stroke);
    for tone in [low, high] {
        let mark = Rect::new(x_of(tone) - 1.5, ramp.y - 4.0, 3.0, ramp.height + 8.0);
        canvas.fill_stroke(mark, 1.5, theme.text, 1.0, theme.card);
    }
    let mut lines = Lines::new(Rect::from_edges(inner.x, ramp.bottom() + SPACE_S, inner.right(), inner.bottom()));
    lines.wrapped(
        canvas,
        "The sphere shows the tones between the marks: Dark Point at its rim and on its back, and Light Point \
         where it faces the camera most, which Shift moves toward Rotation.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

// Which line each label goes on, from 0, given where each one goes across,
// from its left to its right: the first line where it's at least `gap` from
// the labels already there.
pub fn label_lines(spans: &[[f32; 2]], gap: f32) -> Vec<usize> {
    let mut lines: Vec<Vec<[f32; 2]>> = Vec::new();
    spans
        .iter()
        .map(|&[left, right]| {
            let clear = |placed: &Vec<[f32; 2]>| {
                placed
                    .iter()
                    .all(|&[other_left, other_right]| right + gap <= other_left || left >= other_right + gap)
            };
            let line = lines.iter().position(clear).unwrap_or(lines.len());
            if line == lines.len() {
                lines.push(Vec::new());
            }
            lines[line].push([left, right]);
            line
        })
        .collect()
}

// What Material::compile made for the GPU, in columns of Mono.
fn draw_constants(canvas: &mut Canvas, theme: &Theme, editor: &Editor, area: Rect) {
    let inner = card(canvas, theme, area, "GPU constants", "GpuMaterial: 80 bytes at b1, five float4 registers");
    let mut lines = Lines::new(inner);
    let m = &editor.material;
    // Adding 0 turns -0, like sin(-0), into 0.
    let [cos, sin, k, flag] = m.warp.map(|value| value + 0.0);
    let name_x = 152.0;
    let row = |lines: &mut Lines, canvas: &mut Canvas, name: &str, value: String| {
        if let Some(area) = lines.next(canvas, TextStyle::Mono) {
            canvas.text([area.x, area.y], name, TextStyle::Mono, theme.text_secondary);
            canvas.text([area.x + name_x, area.y], &value, TextStyle::Mono, theme.text);
        }
    };
    row(&mut lines, canvas, "warp", format!("cos {cos:+.6}   sin {sin:+.6}   k {k:.6}   shifted {flag}"));
    for (name, values) in [
        ("numerator_dark", m.numerator_dark),
        ("numerator_light", m.numerator_light),
        ("complement_dark", m.complement_dark),
        ("complement_light", m.complement_light),
    ] {
        row(&mut lines, canvas, name, format!("R {:.6e}   G {:.6e}   B {:.6e}", values[0], values[1], values[2]));
    }
    lines.gap(SPACE_S);
    lines.wrapped(
        canvas,
        "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), where numerator \
         = numerator_dark * Q + numerator_light * P, and likewise for complement.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

fn linear(srgb: f64) -> f64 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

// SimplePaint's color curve for one channel, from Specification.md, as a
// function of the tone, with what doesn't depend on the tone worked out once.
fn curve(color_srgb: f64, brightness: f64) -> impl Fn(f64) -> f64 {
    let c = linear(color_srgb);
    let a = c * brightness;
    let b = (1.0 - c) * (1.0 - brightness);
    move |tone| a * tone / (a * tone + b * (1.0 - tone))
}

fn fixed(value: f64) -> String {
    format!("{value:.3}")
}
