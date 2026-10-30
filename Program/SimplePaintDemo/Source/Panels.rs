// Lays out and draws the demo's panels onto a Canvas. Programs support two
// client areas, 2560 x 1369 maximized and 2560 x 1440 in borderless
// fullscreen (see Library/Application), so the panels have fixed places in
// pixels. In fullscreen, the GPU constants panel at the bottom right is
// taller, and the footer stays at the bottom.
//
// The preview panel is drawn with a white middle, which the sphere's image
// is copied over after the canvas is drawn.

use application::FrameCounter;
use canvas::theme::{ACCENT, BORDER, DIM, ERROR, FAINT, INSET, PANEL, SUCCESS, TEXT, TITLE, WARNING};
use canvas::{Canvas, Color, EDGE, Lines, MARGIN, PADDING, Rect, draw_window_problems, panel, wrap};
use simple_paint::Parameters;

use crate::editor::{Click, Editor, SLIDERS, Target};
use crate::preview::IMAGE_SIZE;

// The header has a title, a status line, and room for this many lines of
// notices, so the panels start at the same place whether there are any.
const NOTICE_LINES: usize = 2;
const PANELS_TOP: f32 = 96.0;
const PREVIEW_X: f32 = MARGIN;
const PREVIEW_WIDTH: f32 = IMAGE_SIZE as f32 + PADDING * 2.0;
const EDITOR_X: f32 = PREVIEW_X + PREVIEW_WIDTH + MARGIN;
// Each slider's row, and the parts of it.
const ROW_HEIGHT: f32 = 44.0;
const LABEL_WIDTH: f32 = 120.0;
const VALUE_WIDTH: f32 = 260.0;
const HANDLE_RADIUS: f32 = 8.0;
const SETTINGS_HEIGHT: f32 = 530.0;
const CURVE_HEIGHT: f32 = 420.0;
const PLOT_HEIGHT: f32 = 280.0;
const RAMP_HEIGHT: f32 = 28.0;
// The curves' colors, one for each channel.
const CHANNELS: [Color; 3] = [Color::rgb(0xCF222E), Color::rgb(0x1A7F37), Color::rgb(0x0969DA)];

// What the header and footer show, besides the editor.
pub struct Status<'a> {
    pub focused: bool,
    pub frames: &'a FrameCounter,
    pub display: &'a str,
    pub adapter: &'a str,
    pub notices: &'a [String],
    pub supported: &'a str,
    pub window_problems: &'a [String],
    pub cursor: Option<[f32; 2]>,
}

// Where things are in what was drawn.
pub struct Layout {
    // Where clicks do something.
    pub targets: Vec<Target>,
    // The top left of the sphere's image, if it's shown.
    pub image: Option<[u32; 2]>,
}

pub fn draw(canvas: &mut Canvas, editor: &Editor, status: &Status, width: f32, height: f32) -> Layout {
    let mut layout = Layout {
        targets: Vec::new(),
        image: None,
    };
    if !status.window_problems.is_empty() {
        draw_window_problems(canvas, status.supported, status.window_problems, width);
        return layout;
    }
    let line_height = canvas.line_height();
    let margin = EDGE;

    // The header: the title, and the GPU under it, with whether the window
    // has focus and the frame rate on the right.
    let x = canvas.text(margin, margin, "SimplePaint Demo", TITLE);
    canvas.text(
        x,
        margin,
        "  SimplePaint on a UV sphere, with an editor for its settings",
        DIM,
    );
    let status_y = margin + line_height;
    let (rate, frame_time) = match status.frames.per_second {
        Some(rate) => (
            format!("{rate:.0}"),
            format!("  ({:.2} ms per frame)", 1000.0 / rate),
        ),
        None => ("measuring".to_string(), String::new()),
    };
    let rate_x = width - margin - canvas.text_width(&format!("FPS: {rate}{frame_time}"));
    let mut end = canvas.text(rate_x, status_y, "FPS: ", DIM);
    end = canvas.text(end, status_y, &rate, TEXT);
    canvas.text(end, status_y, &frame_time, DIM);
    end = canvas.text(margin, status_y, "GPU: ", DIM);
    canvas.text_clipped(end, status_y, status.adapter, rate_x - end - margin, TEXT);
    let focus_text = if status.focused {
        "Window has focus"
    } else {
        "Window doesn't have focus: input is ignored"
    };
    canvas.text(
        width - margin - canvas.text_width(focus_text),
        margin,
        focus_text,
        if status.focused { SUCCESS } else { WARNING },
    );
    // Notices go under the status, wrapped to the window's width, in the
    // lines the header has for them.
    let columns = ((width - margin * 2.0) / canvas.char_width()).floor() as usize;
    let mut lines: Vec<String> = Vec::new();
    for notice in status.notices {
        for (index, line) in wrap(notice, columns, 2).into_iter().enumerate() {
            lines.push(if index == 0 { line } else { format!("  {line}") });
        }
    }
    if lines.len() > NOTICE_LINES {
        lines.truncate(NOTICE_LINES);
        lines[NOTICE_LINES - 1] += " ...";
    }
    for (index, line) in lines.iter().enumerate() {
        let y = status_y + line_height * (index + 1) as f32;
        canvas.text_clipped(margin, y, line, width - margin * 2.0, WARNING);
    }

    // The footer: the shortcuts, and the display on the right.
    let footer_y = height - margin - line_height;
    let display_x = width - margin - canvas.text_width(&format!("Display: {}", status.display));
    end = canvas.text(display_x, footer_y, "Display: ", DIM);
    canvas.text(end, footer_y, status.display, TEXT);
    end = margin;
    for (key, action) in [
        ("F11", " borderless fullscreen   "),
        ("Up/Down", " select a setting   "),
        ("Left/Right", " change it, 10 steps with Shift   "),
        ("Ctrl+S", " save to Paint.json"),
    ] {
        end = canvas.text(end, footer_y, key, TEXT);
        end = canvas.text(end, footer_y, action, DIM);
    }
    let bottom = footer_y - margin;

    // The preview, which takes the left side.
    let preview = Rect::new(PREVIEW_X, PANELS_TOP, PREVIEW_WIDTH, bottom - PANELS_TOP);
    let inner = panel(
        canvas,
        preview,
        "Preview",
        "A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.",
    );
    layout.image = Some([inner.x as u32, inner.y.ceil() as u32]);

    // The editor, in three panels down the right side.
    let editor_width = width - MARGIN - EDITOR_X;
    let settings = Rect::new(EDITOR_X, PANELS_TOP, editor_width, SETTINGS_HEIGHT);
    let curve = Rect::new(EDITOR_X, settings.bottom() + MARGIN, editor_width, CURVE_HEIGHT);
    let constants_y = curve.bottom() + MARGIN;
    let constants = Rect::new(EDITOR_X, constants_y, editor_width, bottom - constants_y);
    draw_settings(canvas, editor, status.cursor, settings, &mut layout.targets);
    draw_curve(canvas, &editor.parameters, curve);
    draw_constants(canvas, editor, constants);
    layout
}

fn draw_settings(canvas: &mut Canvas, editor: &Editor, cursor: Option<[f32; 2]>, area: Rect, targets: &mut Vec<Target>) {
    let inner = panel(canvas, area, "Settings", "Material::compile checks them; sliders stay in range");
    let line_height = canvas.line_height();
    let parameters = &editor.parameters;
    let track_left = inner.x + LABEL_WIDTH;
    let track_right = inner.right() - VALUE_WIDTH;
    let mut y = inner.y;
    for (index, slider) in SLIDERS.iter().enumerate() {
        let row = Rect::new(inner.x, y, inner.width, ROW_HEIGHT);
        let middle = y + ROW_HEIGHT * 0.5;
        let text_y = middle - line_height * 0.5;
        let selected = index == editor.selected;
        let active = editor.dragging == Some(index);
        let hovered = cursor.is_some_and(|point| row.contains(point));
        if selected {
            canvas.rect(row.x - PADDING, row.y, 3.0, row.height, ACCENT);
        }
        canvas.text(row.x, text_y, slider.name, if selected { ACCENT } else { TEXT });

        // The track, filled up to the handle.
        let handle_x = track_left + (track_right - track_left) * slider.fraction(parameters) as f32;
        canvas.rect(track_left, middle - 2.0, track_right - track_left, 4.0, BORDER);
        canvas.rect(track_left, middle - 2.0, handle_x - track_left, 4.0, ACCENT);
        let radius = if active || hovered { HANDLE_RADIUS + 2.0 } else { HANDLE_RADIUS };
        canvas.circle([handle_x, middle], radius, ACCENT);
        canvas.circle([handle_x, middle], radius - 3.0, PANEL);

        let value = slider.value(parameters);
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
        targets.push(Target {
            area: Rect::new(track_left - HANDLE_RADIUS * 2.0, row.y, track_right - track_left + HANDLE_RADIUS * 4.0, row.height),
            click: Click::Slider(index),
            track: [track_left, track_right],
        });
        y += ROW_HEIGHT;
    }

    // The base color, as the sliders make it, in sRGB and in linear.
    y += line_height * 0.5;
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
    for (label, click) in [("Reset", Click::Reset), ("Save", Click::Save)] {
        let button = Rect::new(x, y, canvas.text_width(label) + PADDING * 2.0, line_height + PADDING);
        let hovered = cursor.is_some_and(|point| button.contains(point));
        canvas.rect(button.x, button.y, button.width, button.height, if hovered { INSET } else { PANEL });
        canvas.rect_outline(button.x, button.y, button.width, button.height, 1.0, BORDER);
        canvas.text(button.x + PADDING, button.y + PADDING * 0.5, label, TEXT);
        targets.push(Target {
            area: button,
            click,
            track: [0.0; 2],
        });
        x = button.right() + PADDING;
    }
    let (message, color) = match &editor.status {
        Some((message, failed)) => (message.as_str(), if *failed { ERROR } else { SUCCESS }),
        None => ("Reset goes back to SimplePaint's defaults.", DIM),
    };
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
fn draw_curve(canvas: &mut Canvas, parameters: &Parameters, area: Rect) {
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
    for (channel, color) in CHANNELS.iter().enumerate() {
        let base = parameters.base_color_srgb[channel];
        let point = |segment: usize| {
            let tone = segment as f64 / SEGMENTS as f64;
            [x_of(tone), y_of(srgb(curve(base, parameters.brightness, tone)))]
        };
        for segment in 0..SEGMENTS {
            canvas.line(point(segment), point(segment + 1), 2.0, *color);
        }
    }
    canvas.rect_outline(plot.x, plot.y, plot.width, plot.height, 1.0, BORDER);

    // Labels under the plot, at the marks.
    let mut y = plot.bottom() + 4.0;
    let labels = [
        (dark, format!("Dark Point {}", fixed(dark)), DIM),
        (light, format!("Light Point {}", fixed(light)), DIM),
        (anchor, format!("Base color {}", fixed(anchor)), FAINT),
    ];
    // A label that would overlap one on the first line goes on the second.
    let mut first_line: Vec<[f32; 2]> = Vec::new();
    let edge = canvas.char_width() * 2.0;
    for (tone, label, color) in labels {
        let label_width = canvas.text_width(&label);
        let x = (x_of(tone) - label_width * 0.5).clamp(plot.x, plot.right() - label_width);
        let overlaps = first_line
            .iter()
            .any(|&[left, right]| x < right + edge && x + label_width > left - edge);
        if overlaps {
            canvas.text(x, y + line_height, &label, color);
        } else {
            canvas.text(x, y, &label, color);
            first_line.push([x, x + label_width]);
        }
    }
    y += line_height * 2.0 + 4.0;

    // The colors along the curve, with the sphere's tones marked.
    let ramp = Rect::new(plot.x, y, plot.width, RAMP_HEIGHT);
    let columns = ramp.width as usize;
    for column in 0..columns {
        let tone = (column as f64 + 0.5) / columns as f64;
        let byte = |channel: usize| {
            let value = srgb(curve(parameters.base_color_srgb[channel], parameters.brightness, tone));
            (value.clamp(0.0, 1.0) * 255.0).round() as u32
        };
        let color = Color::rgb((byte(0) << 16) | (byte(1) << 8) | byte(2));
        canvas.rect(ramp.x + column as f32, ramp.y, 1.0, ramp.height, color);
    }
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

// SimplePaint's color curve for one channel, from Specification.md.
fn curve(color_srgb: f64, brightness: f64, tone: f64) -> f64 {
    let c = linear(color_srgb);
    let a = c * brightness;
    let b = (1.0 - c) * (1.0 - brightness);
    a * tone / (a * tone + b * (1.0 - tone))
}

fn fixed(value: f64) -> String {
    format!("{value:.3}")
}
