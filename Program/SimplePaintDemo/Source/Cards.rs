// Lays out and draws the demo's cards with Library/UserInterface, and does
// what the input does to the editor. Programs support two client areas,
// 2560 x 1369 maximized and 2560 x 1440 in borderless fullscreen, so the
// cards have fixed places in pixels. Of the 71 pixels that borderless
// fullscreen adds to the height, the preview and the GPU constants card get
// all of them, as each is at the bottom of its column.
//
// The preview card has a view in it, which the demo's render fills with the
// sphere before the interface draws over the rest of the frame.

use user_interface::tokens::{
    BODY, CAPTION, CARD_GAP, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, CURVE_DECELERATE,
    DURATION_FAST, DURATION_FASTER, ICON, MONO, RADIUS_CONTROL, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS,
    SPACE_L, SPACE_M, SPACE_S, SPACE_XS, SPACE_XXL,
};
use user_interface::{Align, Button, Color, Corners, Icon, Painter, Point, Rect, Sense, Size, Status, Stroke, Theme, Ui, id};

use crate::demo::{Demo, TITLE};
use crate::editor::{self, Editor, SETTINGS};
use crate::preview::IMAGE_WIDTH;

// The preview's card, whose view is as wide as the sphere's image.
pub const PREVIEW_WIDTH: f32 = IMAGE_WIDTH as f32 + CARD_PADDING * 2.0;
// The heights of the cards that aren't at the bottom of their column.
pub const SETTINGS_HEIGHT: f32 = 468.0;
pub const CURVE_HEIGHT: f32 = 508.0;
// Each setting's row, and the parts of it.
const ROW_HEIGHT: f32 = 40.0;
const LABEL_WIDTH: f32 = 120.0;
const VALUE_WIDTH: f32 = 72.0;
const RANGE_WIDTH: f32 = 128.0;
const SWATCH_SIZE: [f32; 2] = [64.0, 24.0];
// The tone curve's plot and its color ramp.
const PLOT_HEIGHT: f32 = 320.0;
const RAMP_HEIGHT: f32 = 24.0;
// The ramp's colors, each this wide, which shade into the next.
const RAMP_SEGMENT: f32 = 4.0;
// The GPU constants' names, and their values right of them.
const NAME_WIDTH: f32 = 152.0;

pub fn draw(demo: &mut Demo, ui: &mut Ui) {
    keys(demo, ui);
    app_bar(ui);
    status_bar(ui);
    let mut page = ui.page();
    demo.overflow.clear();
    let preview_rect = page.cut_left(PREVIEW_WIDTH, CARD_GAP);
    preview(demo, ui, preview_rect);
    let mut column = page;
    let fits = [
        ("Settings", settings(demo, ui, column.cut_top(SETTINGS_HEIGHT, CARD_GAP))),
        ("Tone curve", tone_curve(&demo.editor, ui, column.cut_top(CURVE_HEIGHT, CARD_GAP))),
        ("GPU constants", constants(&demo.editor, ui, column)),
    ];
    demo.overflow.extend(fits.iter().filter(|(_, fits)| !fits).map(|(name, _)| *name));
}

// Does what each key pressed this frame says, in order, unless a widget with
// focus takes it, like a button takes Space.
fn keys(demo: &mut Demo, ui: &Ui) {
    let modifiers = ui.input().modifiers();
    for &(key, repeat) in ui.input().keys_pressed() {
        if ui.key_pressed(key)
            && let Some(command) = editor::command(key, repeat, modifiers)
        {
            demo.editor.apply(command, &demo.paint_path);
        }
    }
}

fn app_bar(ui: &mut Ui) {
    let bar = ui.app_bar(Icon::Color, TITLE);
    let color = ui.theme().text_secondary;
    ui.text(bar, "A SimplePaint material on a UV sphere, with an editor for its settings", CAPTION, color);
}

fn status_bar(ui: &mut Ui) {
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    ui.status_bar(&[
        Status::Shortcut("Up/Down", "Select a setting"),
        Status::Shortcut("Left/Right", "Change it"),
        Status::Shortcut("Shift+Left/Right", "By 10 steps"),
        Status::Shortcut("Ctrl+S", "Save to Paint.json"),
        Status::Shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::Text(&environment.gpu),
        Status::Text(display.trim()),
    ]);
}

// A card's content, cut from the top down, which notes whether everything
// fit in it.
struct Content {
    area: Rect,
    fits: bool,
}

impl Content {
    fn new(area: Rect) -> Self {
        Self { area, fits: true }
    }

    fn take(&mut self, height: f32, gap: f32) -> Rect {
        self.fits &= height <= self.area.height;
        self.area.cut_top(height, gap)
    }

    // A paragraph of secondary text, wrapped to the content's width.
    // Cut off where it doesn't fit.
    fn paragraph(&mut self, ui: &mut Ui, text: &str) {
        let color = ui.theme().text_secondary;
        let mut painter = ui.painter();
        let height = painter.measure_wrapped(text, BODY, self.area.width).height;
        let rect = self.take(height, 0.0);
        painter.push_clip(rect);
        painter.text_wrapped(rect, text, BODY, color);
        painter.pop_clip();
    }
}

// A caption right of a card's title, in its header.
fn header_caption(ui: &mut Ui, header: Rect, text: &str) {
    let color = ui.theme().text_secondary;
    ui.text(header, text, BODY, color);
}

// The preview: a card whose view the demo's render fills with the sphere.
fn preview(demo: &mut Demo, ui: &mut Ui, rect: Rect) {
    let view = rect.inset_by(
        CARD_PADDING,
        CARD_PADDING + CARD_HEADER_HEIGHT + CARD_HEADER_GAP,
        CARD_PADDING,
        CARD_PADDING,
    );
    let card = ui.card_with_view(rect, "Preview", view);
    header_caption(
        ui,
        card.header,
        "A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.",
    );
    let theme = ui.theme();
    demo.background = theme.card_background.over(theme.background);
    demo.view = Some(view);
}

// A slider for each setting, in rows like a list's, of which the arrow keys
// move the selected one, the base color, and the buttons that save and reset
// the material.
fn settings(demo: &mut Demo, ui: &mut Ui, rect: Rect) -> bool {
    let card = ui.card(rect, "Settings");
    let theme = ui.theme().clone();

    // The buttons at the header's right, and what the last save did, left of
    // them.
    let mut header = card.header;
    let save = Button::new("Save").icon(Icon::Save).accent();
    let reset = Button::new("Reset").icon(Icon::Refresh);
    let save_width = ui.button_width(&save).max(96.0);
    let reset_width = ui.button_width(&reset).max(96.0);
    let save_rect = header.cut_right(save_width, SPACE_S);
    let reset_rect = header.cut_right(reset_width, SPACE_L);
    if ui.button("Reset", reset_rect, reset).clicked {
        demo.editor.reset();
    }
    ui.tooltip("Reset", "Goes back to SimplePaint's default material");
    if ui.button("Save", save_rect, save).clicked {
        demo.editor.save(&demo.paint_path);
    }
    ui.tooltip("Save", "Saves the material to Paint.json (Ctrl+S)");
    save_status(ui, header, &demo.editor.status, &theme);

    // The sliders, which don't take keyboard focus, as the arrow keys move
    // the selected one.
    let mut content = Content::new(card.content);
    let editor = &mut demo.editor;
    editor.dragging = None;
    ui.set_focusable(false);
    let mut selected_row = None;
    for index in 0..SETTINGS.len() {
        let row = content.take(ROW_HEIGHT, 0.0);
        setting(editor, ui, index, row, card.rect, &theme);
        if index == editor.selected {
            selected_row = Some(row);
        }
    }
    ui.set_focusable(true);
    // The pill that marks the selected row, which slides to it.
    if let Some(row) = selected_row {
        let target = (row.center().y - SELECTION_PILL_LENGTH / 2.0).round();
        let y = ui.animate(id("Settings/Selection"), 0, target, DURATION_FAST, CURVE_DECELERATE);
        let pill = Rect::new(card.rect.x + SPACE_XS, y, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
        ui.painter().rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
    }

    content.take(0.0, SPACE_M);
    base_color(ui, content.take(CONTROL_HEIGHT, SPACE_M), editor, &theme);
    content.paragraph(
        ui,
        "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and Rotation by 0.1 degrees, \
         and stop at the last step that SimplePaint accepts, so Material::compile always takes them.",
    );
    content.fits
}

// A setting's row: its name, its slider, its value, and its range. A press
// on the row or its slider, or the wheel over the slider, selects it.
fn setting(editor: &mut Editor, ui: &mut Ui, index: usize, row: Rect, card: Rect, theme: &Theme) {
    let setting = &SETTINGS[index];
    // The row's face, from near the card's edges, like a list's row.
    let face = Rect::new(card.x + SPACE_XS, row.y + 2.0, card.width - SPACE_XS * 2.0, row.height - 4.0);
    let row_name = format!("Setting/{}", setting.name);
    let row_response = ui.interact(&row_name, face, Sense::CLICK, "ListViewItem");
    ui.set_label(&row_name, setting.name);
    let hovered = row_response.hovered || ui.hovered_id() == Some(id(setting.name));
    let pressed = row_response.held || ui.pressed_name() == Some(setting.name);
    let selected = index == editor.selected;
    let fill = match (pressed, hovered, selected) {
        (true, _, _) | (false, true, true) => theme.subtle_fill_tertiary,
        (false, true, false) | (false, false, true) => theme.subtle_fill_secondary,
        _ => Color::TRANSPARENT,
    };
    let fill = ui.animate_color(row_response.id, 0, fill, DURATION_FASTER);
    ui.painter().shape(face, Corners::all(RADIUS_CONTROL), fill, None);

    let mut parts = row;
    let label = parts.cut_left(LABEL_WIDTH, SPACE_M);
    let range = parts.cut_right(RANGE_WIDTH, SPACE_XXL);
    let value_rect = parts.cut_right(VALUE_WIDTH, SPACE_L);
    let track = parts.inset_by(0.0, (row.height - CONTROL_HEIGHT) / 2.0, 0.0, (row.height - CONTROL_HEIGHT) / 2.0);

    let mut value = setting.value(&editor.parameters) as f32;
    let response = ui.slider(setting.name, track, &mut value, setting.slider());
    // The editor turns the wheel itself, so that parts of a notch add up, and
    // a value between steps never moves the other way.
    let notches = ui.input().wheel.y as f64;
    if response.hovered && !response.held && notches != 0.0 {
        editor.selected = index;
        editor.wheel(index, notches);
    } else if response.changed {
        editor.set(index, setting.nearest(value as f64));
    }
    if response.pressed || row_response.pressed {
        editor.selected = index;
    }
    if response.held {
        editor.dragging = Some(index);
    }

    // The value, which scripts see as the slider's label.
    let value = setting.format(setting.value(&editor.parameters));
    ui.set_label(setting.name, &value);
    let mut painter = ui.painter();
    painter.text_in(label, setting.name, BODY, theme.text_primary, Align::Start);
    painter.text_in(value_rect, &value, MONO, theme.text_primary, Align::End);
    painter.text_in(range, &setting.range(), BODY, theme.text_secondary, Align::Start);
}

// What the last save did, at the right of the area: a success or an error,
// in a status icon like an info bar's.
fn save_status(ui: &mut Ui, area: Rect, status: &Option<(String, bool)>, theme: &Theme) {
    let Some((message, failed)) = status else {
        return;
    };
    let (color, mark) = if *failed { (theme.critical, Icon::StatusError) } else { (theme.success, Icon::StatusSuccess) };
    let room = (area.width - ICON - SPACE_S).max(0.0);
    let mut painter = ui.painter();
    let width = painter.measure(message, BODY).width.min(room);
    let text = area.right_strip(width);
    painter.text_fit(text, message, BODY, theme.text_primary, Align::Start);
    let icon = Point::new(text.x - SPACE_S - ICON / 2.0, area.center().y);
    painter.icon(icon, Icon::StatusCircle, ICON, color);
    painter.icon(icon, mark, ICON, theme.text_on_accent);
    // A message cut short shows whole while the pointer rests on it.
    if width < painter.measure(message, BODY).width {
        ui.register("SaveStatus", text, Sense::NONE, "Text");
        ui.tooltip("SaveStatus", message);
    }
}

// The base color, as the sliders make it: a swatch, in hex, in linear, and
// the tone that it appears at.
fn base_color(ui: &mut Ui, row: Rect, editor: &Editor, theme: &Theme) {
    let parameters = editor.parameters;
    let srgb = parameters.base_color_srgb;
    let [red, green, blue] = srgb.map(|channel| (channel * 255.0).round() as u8);
    let swatch_color = Color::rgba(red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, 1.0);
    let linear = srgb.map(linear);
    let mut parts = row;
    let label = parts.cut_left(LABEL_WIDTH, SPACE_M);
    let mut painter = ui.painter();
    painter.text_in(label, "Base color", BODY, theme.text_primary, Align::Start);
    let swatch = Rect::from_center(
        Point::new(parts.x + SWATCH_SIZE[0] / 2.0, row.center().y),
        Size::new(SWATCH_SIZE[0], SWATCH_SIZE[1]),
    );
    painter.shape(swatch, Corners::all(RADIUS_CONTROL), swatch_color, Some(Stroke::new(theme.control_stroke, 1.0)));
    parts.cut_left(SWATCH_SIZE[0], SPACE_M);
    let pieces = [
        (format!("#{red:02X}{green:02X}{blue:02X}"), MONO, theme.text_primary, SPACE_XXL),
        ("Linear".to_string(), BODY, theme.text_secondary, SPACE_S),
        (format!("{:.4}, {:.4}, {:.4}", linear[0], linear[1], linear[2]), MONO, theme.text_primary, SPACE_XXL),
        ("Appears at tone 1 - Brightness =".to_string(), BODY, theme.text_secondary, SPACE_S),
        (format!("{:.3}", 1.0 - parameters.brightness), MONO, theme.text_primary, 0.0),
    ];
    for (text, style, color, gap) in pieces {
        let placed = painter.text_in(parts, &text, style, color, Align::Start);
        parts.cut_left(placed.width, gap);
    }
}

// The curves' colors, one for each channel.
fn channels(theme: &Theme) -> [Color; 3] {
    [theme.data.red, theme.data.green, theme.data.blue]
}

// The color curve of each channel over the tone, as it's displayed, with the
// tones that the sphere shows marked, and a ramp of the colors.
fn tone_curve(editor: &Editor, ui: &mut Ui, rect: Rect) -> bool {
    let card = ui.card(rect, "Tone curve");
    header_caption(ui, card.header, "Each channel's displayed value over the tone, from 0 at the left to 1 at the right");
    let theme = ui.theme().clone();
    let card_color = theme.card_background.over(theme.background);
    let parameters = editor.parameters;
    let mut content = Content::new(card.content);
    let plot = content.take(PLOT_HEIGHT, SPACE_XS);
    let mut painter = ui.painter();
    painter.shape(
        plot,
        Corners::all(RADIUS_CONTROL),
        theme.control_alt_fill_secondary,
        Some(Stroke::new(theme.control_stroke, 1.0)),
    );
    // Inside the plot's stroke.
    let area = plot.inset(1.0);
    for quarter in 1..4 {
        let fraction = quarter as f32 * 0.25;
        painter.fill(Rect::new((area.x + area.width * fraction).round(), area.y, 1.0, area.height), theme.divider);
        painter.fill(Rect::new(area.x, (area.y + area.height * fraction).round(), area.width, 1.0), theme.divider);
    }
    let x_of = |tone: f64| area.x + area.width * tone as f32;
    let y_of = |value: f64| area.bottom() - area.height * value as f32;

    // The tones from Dark Point at the sphere's rim to Light Point at its peak.
    let (dark, light) = (parameters.dark_point, parameters.light_point);
    let (low, high) = (dark.min(light), dark.max(light));
    let span = Rect::new(x_of(low), area.y, (x_of(high) - x_of(low)).max(1.0), area.height);
    painter.fill(span, theme.accent.fade(0.08));
    let anchor = 1.0 - parameters.brightness;
    for (tone, color) in [(dark, theme.text_secondary), (light, theme.text_secondary), (anchor, theme.text_tertiary)] {
        painter.fill(Rect::new(x_of(tone) - 0.5, area.y, 1.0, area.height), color);
    }

    // Each channel's curve, as displayed: the shader's linear output, encoded
    // to sRGB.
    const SEGMENTS: usize = 256;
    let curves = parameters.base_color_srgb.map(|base| curve(base, parameters.brightness));
    painter.push_clip(area);
    for (curve, color) in curves.iter().zip(channels(&theme)) {
        let points: Vec<Point> = (0..=SEGMENTS)
            .map(|segment| {
                let tone = segment as f64 / SEGMENTS as f64;
                Point::new(x_of(tone), y_of(srgb(curve(tone))))
            })
            .collect();
        painter.polyline(&points, 2.0, color);
    }
    painter.pop_clip();

    // Labels under the plot, at the marks, on as many lines as there are
    // labels, so that they never overlap, even when they're all at one tone.
    let labels = [
        (dark, format!("Dark Point {dark:.3}"), theme.text_secondary),
        (light, format!("Light Point {light:.3}"), theme.text_secondary),
        (anchor, format!("Base color {anchor:.3}"), theme.text_tertiary),
    ];
    let label_area = content.take(CAPTION.line_height * labels.len() as f32, SPACE_S);
    let spans: Vec<[f32; 2]> = labels
        .iter()
        .map(|(tone, label, _)| {
            let width = painter.measure(label, CAPTION).width;
            let x = (x_of(*tone) - width / 2.0).clamp(area.x, area.right() - width);
            [x, x + width]
        })
        .collect();
    let lines = label_lines(&spans, SPACE_L);
    for ((_, label, color), (span, line)) in labels.iter().zip(spans.iter().zip(lines)) {
        let origin = Point::new(span[0], label_area.y + CAPTION.line_height * line as f32);
        painter.text(origin, label, CAPTION, *color);
    }

    // The colors along the curve, with the sphere's tones marked.
    let ramp = content.take(RAMP_HEIGHT, SPACE_M);
    let ramp_color = |tone: f64| {
        let [r, g, b] = curves.each_ref().map(|curve| srgb(curve(tone)).clamp(0.0, 1.0) as f32);
        Color::rgba(r, g, b, 1.0)
    };
    let count = (ramp.width / RAMP_SEGMENT).ceil() as usize;
    for segment in 0..count {
        let left = ramp.x + segment as f32 * RAMP_SEGMENT;
        let right = (left + RAMP_SEGMENT).min(ramp.right());
        let from = ramp_color(((left - ramp.x) / ramp.width) as f64);
        let to = ramp_color(((right - ramp.x) / ramp.width) as f64);
        painter.gradient(Rect::new(left, ramp.y, right - left, ramp.height), 0.0, from, to, false);
    }
    painter.shape(ramp, Corners::default(), Color::TRANSPARENT, Some(Stroke::new(theme.control_stroke, 1.0)));
    for tone in [low, high] {
        let mark = Rect::new(x_of(tone) - 1.5, ramp.y - 4.0, 3.0, ramp.height + 8.0);
        painter.shape(mark, Corners::all(1.5), theme.text_primary, Some(Stroke::new(card_color, 1.0)));
    }
    content.paragraph(
        ui,
        "The sphere shows the tones between the marks: Dark Point at its rim and on its back, and Light Point where \
         it faces the camera most, which Shift moves toward Rotation.",
    );
    content.fits
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

// What Material::compile made for the GPU, in columns of Cascadia Mono.
fn constants(editor: &Editor, ui: &mut Ui, rect: Rect) -> bool {
    let card = ui.card(rect, "GPU constants");
    header_caption(ui, card.header, "GpuMaterial: 80 bytes at b1, five float4 registers");
    let theme = ui.theme().clone();
    let mut content = Content::new(card.content);
    let m = &editor.material;
    // To the 6 decimals shown, where adding 0 turns -0, like the sine of
    // 180 degrees in floats, -0.0000000874, into 0.
    let [cos, sin, k, flag] = m.warp.map(|value| (value as f64 * 1e6).round() / 1e6 + 0.0);
    let mut rows = vec![("warp", format!("cos {cos:+.6}   sin {sin:+.6}   k {k:.6}   shifted {flag}"))];
    for (name, values) in [
        ("numerator_dark", m.numerator_dark),
        ("numerator_light", m.numerator_light),
        ("complement_dark", m.complement_dark),
        ("complement_light", m.complement_light),
    ] {
        rows.push((name, format!("R {:.6e}   G {:.6e}   B {:.6e}", values[0], values[1], values[2])));
    }
    let mut painter = ui.painter();
    painter.push_clip(card.content);
    for (name, value) in &rows {
        let row = content.take(MONO.line_height, 0.0);
        draw_constant(&mut painter, row, name, value, &theme);
    }
    painter.pop_clip();
    content.take(0.0, SPACE_S);
    content.paragraph(
        ui,
        "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), where numerator = \
         numerator_dark * Q + numerator_light * P, and likewise for complement.",
    );
    content.fits
}

fn draw_constant(painter: &mut Painter, row: Rect, name: &str, value: &str, theme: &Theme) {
    painter.text(row.origin(), name, MONO, theme.text_secondary);
    painter.text(Point::new(row.x + NAME_WIDTH, row.y), value, MONO, theme.text_primary);
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
