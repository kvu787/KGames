// Lays out and draws the demo's panels with Library/UserInterface, and
// handles what the input does to the editor. Programs support two client
// areas, 2560 x 1369 maximized and 2560 x 1440 in borderless fullscreen (see
// Library/Application), so the panels have fixed places in pixels. In
// fullscreen, the GPU constants panel at the bottom right is taller, and the
// footer stays at the bottom.
//
// The preview panel is drawn with a white middle, which the sphere's image
// is copied over after the canvas is drawn.
//
// The tone curve panel's curves and color ramp are most of the panels'
// vertices and math, so they're drawn only when the base color or Brightness
// changes, and their vertices are kept for the frames in between. See Cache.

use std::path::Path;

use simple_paint::Parameters;
use user_interface::theme::{ACCENT, BORDER, DIM, ERROR, FAINT, INSET, SUCCESS, TEXT, TITLE};
use user_interface::{
    Cached, Canvas, Color, Frame, HANDLE_RADIUS, Lines, MARGIN, PADDING, PANELS_TOP, Rect, panel,
};

use crate::editor::{Editor, SLIDERS};
use crate::preview::IMAGE_SIZE;

const PREVIEW_X: f32 = MARGIN;
const PREVIEW_WIDTH: f32 = IMAGE_SIZE as f32 + PADDING * 2.0;
const EDITOR_X: f32 = PREVIEW_X + PREVIEW_WIDTH + MARGIN;
// Each slider's row, and the parts of it.
const ROW_HEIGHT: f32 = 44.0;
const LABEL_WIDTH: f32 = 120.0;
const VALUE_WIDTH: f32 = 260.0;
const SETTINGS_HEIGHT: f32 = 530.0;
const CURVE_HEIGHT: f32 = 464.0;
const PLOT_HEIGHT: f32 = 280.0;
const RAMP_HEIGHT: f32 = 28.0;
// The curves' colors, one for each channel.
const CHANNELS: [Color; 3] = [Color::rgb(0xCF222E), Color::rgb(0x1A7F37), Color::rgb(0x0969DA)];

// Where things are in what was drawn, besides the widgets, which the
// interface knows.
pub struct Layout {
    // The top left of the sphere's image.
    pub image: Option<[u32; 2]>,
}

// What draw keeps from frame to frame, for one font: the tone curve panel's
// curves and color ramp, each by what it shows (see CurveKey).
#[derive(Default)]
pub struct Cache {
    curves: Cached<CurveKey>,
    ramp: Cached<CurveKey>,
}

// The base color and Brightness, which the color curve depends on, and the
// area it's drawn in.
type CurveKey = ([f64; 3], f64, [f32; 4]);

// Handles the frame's input, and draws the panels. The editor's sliders
// don't take keyboard focus, as the arrow keys move the selected one, which
// the program's keys select.
pub fn draw(frame: &mut Frame, cache: &mut Cache, editor: &mut Editor, paint_path: &Path) -> Layout {
    let keys: Vec<_> = frame.program_events().copied().collect();
    for event in &keys {
        editor.key(event, paint_path);
    }
    frame.set_focusable(false);

    let adapter = frame.status.adapter;
    frame.header(
        "SimplePaint Demo",
        "SimplePaint on a UV sphere, with an editor for its settings",
        &[("GPU: ", DIM), (adapter, TEXT)],
    );
    let bottom = frame.footer(
        &[
            ("F11", "borderless fullscreen"),
            ("Up/Down", "select a setting"),
            ("Left/Right", "change it, 10 steps with Shift"),
            ("Ctrl+S", "save to Paint.json"),
        ],
        "",
    );

    // The preview, which takes the left side.
    let preview = Rect::new(PREVIEW_X, PANELS_TOP, PREVIEW_WIDTH, bottom - PANELS_TOP);
    let inner = frame.panel(
        preview,
        "Preview",
        "A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.",
    );
    let layout = Layout {
        image: Some([inner.x as u32, inner.y.ceil() as u32]),
    };

    // The editor, in three panels down the right side.
    let editor_width = frame.width - MARGIN - EDITOR_X;
    let settings = Rect::new(EDITOR_X, PANELS_TOP, editor_width, SETTINGS_HEIGHT);
    let curve = Rect::new(EDITOR_X, settings.bottom() + MARGIN, editor_width, CURVE_HEIGHT);
    let constants_y = curve.bottom() + MARGIN;
    let constants = Rect::new(EDITOR_X, constants_y, editor_width, bottom - constants_y);
    draw_settings(frame, editor, settings, paint_path);
    draw_curve(&mut frame.canvas, cache, &editor.parameters, curve);
    draw_constants(&mut frame.canvas, editor, constants);
    layout
}

fn draw_settings(frame: &mut Frame, editor: &mut Editor, area: Rect, paint_path: &Path) {
    let inner = frame.panel(area, "Settings", "Material::compile checks them; sliders stay in range");
    let line_height = frame.line_height();
    let track_left = inner.x + LABEL_WIDTH;
    let track_right = inner.right() - VALUE_WIDTH;
    let mut y = inner.y;
    editor.dragging = None;
    for (index, slider) in SLIDERS.iter().enumerate() {
        let row = Rect::new(inner.x, y, inner.width, ROW_HEIGHT);
        let middle = y + ROW_HEIGHT * 0.5;
        let text_y = middle - line_height * 0.5;
        // Where a press or the wheel moves the slider: its track, and a
        // little past each end, for the handle.
        let area = Rect::new(
            track_left - HANDLE_RADIUS * 2.0,
            row.y,
            track_right - track_left + HANDLE_RADIUS * 4.0,
            row.height,
        );
        let mut value = slider.value(&editor.parameters);
        let response = frame.slider(slider.name, area, &mut value, &slider.range());
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
        let selected = index == editor.selected;
        let canvas = &mut frame.canvas;
        if selected {
            canvas.rect(row.x - PADDING, row.y, 3.0, row.height, ACCENT);
        }
        canvas.text(row.x, text_y, slider.name, if selected { ACCENT } else { TEXT });
        let value_x = track_right + PADDING * 2.0;
        let end = canvas.text(value_x, text_y, &slider.format(value), TITLE);
        canvas.text(
            end.max(value_x + canvas.char_width() * 10.0),
            text_y,
            &format!(
                "{:.2$} to {}",
                slider.minimum,
                slider.format(slider.maximum),
                slider.decimals
            ),
            DIM,
        );
        y += ROW_HEIGHT;
    }

    // The base color, as the sliders make it, in sRGB and in linear.
    y += line_height * 0.5;
    let parameters = editor.parameters;
    let canvas = &mut frame.canvas;
    let [red, green, blue] = parameters.base_color_srgb;
    let byte = |channel: f64| (channel * 255.0).round() as u32;
    let swatch = Color::rgb((byte(red) << 16) | (byte(green) << 8) | byte(blue));
    canvas.text(inner.x, y + 6.0, "Base color", TEXT);
    canvas.rect(track_left, y, 120.0, 28.0, swatch);
    canvas.rect_outline(track_left, y, 120.0, 28.0, 1.0, BORDER);
    let linear = parameters.base_color_srgb.map(linear);
    canvas.text(
        track_left + 120.0 + PADDING,
        y + 6.0,
        &format!(
            "#{:02X}{:02X}{:02X}   linear {:.4}, {:.4}, {:.4}   appears at tone 1 - Brightness = {:.3}",
            byte(red),
            byte(green),
            byte(blue),
            linear[0],
            linear[1],
            linear[2],
            1.0 - parameters.brightness
        ),
        DIM,
    );
    y += 28.0 + line_height;

    // Buttons, and what the last save did.
    let mut x = inner.x;
    for label in ["Reset", "Save"] {
        let [width, height] = frame.button_size(label);
        let button = Rect::new(x, y, width, height);
        if frame.button(label, button, label).clicked {
            match label {
                "Reset" => editor.reset(),
                _ => editor.save(paint_path),
            }
        }
        x = button.right() + PADDING;
    }
    let (message, color) = match &editor.status {
        Some((message, failed)) => (message.as_str(), if *failed { ERROR } else { SUCCESS }),
        None => ("Reset goes back to SimplePaint's defaults.", DIM),
    };
    let canvas = &mut frame.canvas;
    canvas.text_clipped(x + PADDING, y + PADDING * 0.5, message, inner.right() - x - PADDING, color);
    y += line_height + PADDING * 2.0;

    let mut lines = Lines::new(Rect::new(inner.x, y, inner.width, inner.bottom() - y), line_height);
    lines.wrapped(
        canvas,
        "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and \
         Rotation by 0.1 degrees, and stop at the last step that SimplePaint accepts.",
        "",
        DIM,
    );
}

// The color curve of each channel over the tone, as it's displayed, with the
// tones that the sphere shows marked, and a ramp of the colors.
fn draw_curve(canvas: &mut Canvas, cache: &mut Cache, parameters: &Parameters, area: Rect) {
    let inner = panel(
        canvas,
        area,
        "Tone curve",
        "Each channel's displayed value over the tone, from 0 at the left to 1 at the right",
    );
    let line_height = canvas.line_height();
    let plot = Rect::new(inner.x, inner.y, inner.width, PLOT_HEIGHT);
    canvas.rect(plot.x, plot.y, plot.width, plot.height, INSET);
    for quarter in 1..4 {
        let fraction = quarter as f32 * 0.25;
        canvas.rect(plot.x + plot.width * fraction, plot.y, 1.0, plot.height, BORDER);
        canvas.rect(plot.x, plot.y + plot.height * fraction, plot.width, 1.0, BORDER);
    }
    let x_of = |tone: f64| plot.x + plot.width * tone as f32;
    let y_of = |value: f64| plot.bottom() - plot.height * value as f32;

    // The tones from Dark Point at the sphere's rim to Light Point at its peak.
    let (dark, light) = (parameters.dark_point, parameters.light_point);
    let (low, high) = (dark.min(light), dark.max(light));
    canvas.rect(x_of(low), plot.y, (x_of(high) - x_of(low)).max(1.0), plot.height, ACCENT.alpha(0.08));
    let anchor = 1.0 - parameters.brightness;
    for (tone, color) in [(dark, DIM), (light, DIM), (anchor, FAINT)] {
        canvas.rect(x_of(tone) - 0.5, plot.y, 1.0, plot.height, color);
    }

    // Each channel's curve, as displayed: the shader's linear output, encoded to sRGB.
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
        for (curve, color) in curves.iter().zip(CHANNELS) {
            let mut from = [x_of(0.0), y_of(srgb(curve(0.0)))];
            for segment in 1..=SEGMENTS {
                let tone = segment as f64 / SEGMENTS as f64;
                let to = [x_of(tone), y_of(srgb(curve(tone)))];
                canvas.line(from, to, 2.0, color);
                from = to;
            }
        }
    });
    canvas.rect_outline(plot.x, plot.y, plot.width, plot.height, 1.0, BORDER);

    // Labels under the plot, at the marks, on as many lines as there are
    // labels, so that they never overlap, even when they're all at one tone.
    let mut y = plot.bottom() + 4.0;
    let labels = [
        (dark, format!("Dark Point {}", fixed(dark)), DIM),
        (light, format!("Light Point {}", fixed(light)), DIM),
        (anchor, format!("Base color {}", fixed(anchor)), FAINT),
    ];
    let spans: Vec<[f32; 2]> = labels
        .iter()
        .map(|(tone, label, _)| {
            let label_width = canvas.text_width(label);
            let x = (x_of(*tone) - label_width * 0.5).clamp(plot.x, plot.right() - label_width);
            [x, x + label_width]
        })
        .collect();
    let lines = label_lines(&spans, canvas.char_width() * 2.0);
    for ((_, label, color), (span, line)) in labels.iter().zip(spans.iter().zip(lines)) {
        canvas.text(span[0], y + line_height * line as f32, label, *color);
    }
    y += line_height * labels.len() as f32 + 4.0;

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
            canvas.rect(ramp.x + column as f32, ramp.y, 1.0, ramp.height, color);
        }
    });
    canvas.rect_outline(ramp.x, ramp.y, ramp.width, ramp.height, 1.0, BORDER);
    for tone in [low, high] {
        canvas.rect(x_of(tone) - 1.0, ramp.y - 4.0, 2.0, ramp.height + 8.0, TITLE);
    }
    let mut lines = Lines::new(
        Rect::new(inner.x, ramp.bottom() + 6.0, inner.width, inner.bottom() - ramp.bottom() - 6.0),
        line_height,
    );
    lines.wrapped(
        canvas,
        "The sphere shows the tones between the marks: Dark Point at its rim and on its back, and \
         Light Point where it faces the camera most, which Shift moves toward Rotation.",
        "",
        DIM,
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

// What Material::compile made for the GPU.
fn draw_constants(canvas: &mut Canvas, editor: &Editor, area: Rect) {
    let inner = panel(canvas, area, "GPU constants", "GpuMaterial: 80 bytes at b1, five float4 registers");
    let mut lines = Lines::new(inner, canvas.line_height());
    let m = &editor.material;
    // Adding 0 turns -0, like sin(-0), into 0.
    let [cos, sin, k, flag] = m.warp.map(|value| value + 0.0);
    lines.pair(
        canvas,
        "warp              ",
        &format!("cos {cos:+.6}   sin {sin:+.6}   k {k:.6}   shifted {flag}"),
        TEXT,
    );
    for (name, values) in [
        ("numerator_dark    ", m.numerator_dark),
        ("numerator_light   ", m.numerator_light),
        ("complement_dark   ", m.complement_dark),
        ("complement_light  ", m.complement_light),
    ] {
        lines.pair(
            canvas,
            name,
            &format!("R {:.6e}   G {:.6e}   B {:.6e}", values[0], values[1], values[2]),
            TEXT,
        );
    }
    lines.gap(0.5);
    lines.wrapped(
        canvas,
        "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), \
         where numerator = numerator_dark * Q + numerator_light * P, and likewise for complement.",
        "",
        DIM,
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
