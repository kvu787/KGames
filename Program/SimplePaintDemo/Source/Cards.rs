// Lays the demo's cards out with Library/UserInterface, and keeps them true
// to the editor. Programs support two client areas, 2560 x 1369 maximized and
// 2560 x 1440 in borderless fullscreen, so the cards have fixed places in
// pixels. Of the 71 pixels that borderless fullscreen adds to the height, the
// preview and the GPU constants card get all of them, as each is at the
// bottom of its column.
//
// The preview card has a view in it, which the demo's render fills with the
// sphere before the interface draws over the rest of the frame.

use simple_paint::GpuMaterial;
use user_interface::controls::{LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_HEIGHT, LIST_ITEM_RADIUS, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS};
use user_interface::tokens::{
    BODY, CARD_GAP, CARD_PADDING, CONTROL_HEIGHT, CURVE_FAST_OUT_SLOW_IN, DURATION_FAST, DURATION_FASTER, ICON, MONO, RADIUS_CONTROL, SPACE_L,
    SPACE_M, SPACE_S, SPACE_XXL,
};
use user_interface::{
    Align, AppBar, Border, Button, Canvas, Card, Color, Icon, Painter, Point, Rect, Sense, Severity, Size, Slider, Span, Status, Text, Ui, id,
};

use crate::demo::{Demo, TITLE};
use crate::editor::{Editor, SETTINGS};
use crate::preview::IMAGE_WIDTH;

// The preview's card, whose view is as wide as the sphere's image.
pub const PREVIEW_WIDTH: f32 = IMAGE_WIDTH as f32 + CARD_PADDING * 2.0;
// The height of the settings card, above the GPU constants.
pub const SETTINGS_HEIGHT: f32 = 468.0;
// Reset and Save, which are as wide as each other.
const BUTTON_WIDTH: f32 = 96.0;
// Each setting's row, like a list's, and the parts of it.
const ROW_HEIGHT: f32 = LIST_ITEM_HEIGHT;
const LABEL_WIDTH: f32 = 120.0;
const VALUE_WIDTH: f32 = 72.0;
const RANGE_WIDTH: f32 = 128.0;
const SWATCH_SIZE: Size = Size::new(64.0, 24.0);
// The GPU constants' names, and their values right of them.
const NAME_WIDTH: f32 = 152.0;

pub fn build(demo: &mut Demo, ui: &mut Ui) {
    ui.app_bar(AppBar::new(Icon::Color, TITLE).subtitle("A SimplePaint material on a UV sphere, with an editor for its settings"));
    status_bar(ui);
    let mut page = ui.page();
    demo.overflow.clear();
    let preview_rect = page.cut_left(PREVIEW_WIDTH, CARD_GAP);
    preview(demo, ui, preview_rect);
    let mut column = page;
    let fits = [
        ("Settings", settings(demo, ui, column.cut_top(SETTINGS_HEIGHT, CARD_GAP))),
        ("GPU constants", constants(&demo.editor, ui, column)),
    ];
    demo.overflow.extend(fits.iter().filter(|(_, fits)| !fits).map(|(name, _)| *name));
}

fn status_bar(ui: &mut Ui) {
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    ui.status_bar(vec![
        Status::shortcut("Up/Down", "Select a setting"),
        Status::shortcut("Left/Right", "Change it"),
        Status::shortcut("Shift+Left/Right", "By 10 steps"),
        Status::shortcut("Ctrl+S", "Save to Paint.json"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::text(&environment.gpu),
        Status::text(display.trim()),
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
    fn paragraph(&mut self, ui: &mut Ui, name: &str, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, BODY);
        let rect = self.take(height, 0.0);
        ui.node(name, rect, Text::new(text, BODY).secondary().wrapped());
    }
}

// The preview: a card whose view the demo's render fills with the sphere,
// around the card's color.
fn preview(demo: &mut Demo, ui: &mut Ui, rect: Rect) {
    let card = Card::new("Preview")
        .subtitle("A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.")
        .with_view();
    let area = ui.card("Preview", rect, card, |_, _| {});
    demo.view = Some(area.content);
    demo.background = ui.theme().card_opaque();
}

// What a setting's row face shows: which slider it belongs to, and whether
// it's the selected one. Its hover and press come from the frame's responses.
#[derive(Clone)]
struct RowFace {
    row: String,
    slider: &'static str,
    selected: bool,
}

fn draw_row_face(ui: &mut Ui, face: Rect, data: &RowFace) {
    let response = ui.response(&data.row);
    let slider = ui.response(data.slider);
    let hovered = response.hovered || ui.hovered_name() == Some(data.slider);
    let pressed = response.held || slider.held;
    // ListViewItemBackground in each of the row's states.
    let theme = ui.theme();
    let fill = match (data.selected, pressed, hovered) {
        (false, true, _) | (true, false, true) => theme.subtle_fill_tertiary,
        (false, false, true) | (true, _, _) => theme.subtle_fill_secondary,
        (false, false, false) => theme.subtle_fill_transparent,
    };
    let fill = ui.animate_color(response.id, 0, fill, DURATION_FASTER);
    ui.painter().rounded(face, LIST_ITEM_RADIUS, fill);
}

// The pill that marks the selected row, at its face's left edge, as a
// list's does, which slides to it.
fn draw_selection_pill(ui: &mut Ui, _rect: Rect, face: &Rect) {
    let target = (face.center().y - SELECTION_PILL_LENGTH / 2.0).round();
    let y = ui.animate(id("Settings/Selection"), 0, target, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
    let pill = Rect::new(face.x, y, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
    let accent = ui.theme().accent;
    ui.painter().rounded(pill, SELECTION_PILL_THICKNESS / 2.0, accent);
}

// A slider for each setting, in rows like a list's, of which the arrow keys
// move the selected one, the base color, and the buttons that save and reset
// the material.
fn settings(demo: &Demo, ui: &mut Ui, rect: Rect) -> bool {
    let editor = &demo.editor;
    let card_config = Card::new("Settings");
    let card = card_config.area(ui, rect);

    // The buttons at the header's right, and what the last save did, left of
    // them.
    let mut header = card.header;
    let save = Button::new("Save").icon(Icon::Save).accent();
    let reset = Button::new("Reset").icon(Icon::Refresh);
    let save_rect = header.cut_right(ui.button_width(&save).max(BUTTON_WIDTH), SPACE_S);
    let reset_rect = header.cut_right(ui.button_width(&reset).max(BUTTON_WIDTH), SPACE_L);

    let mut fits = true;
    ui.card("SettingsCard", rect, Card::new("Settings"), |ui, card| {
        ui.node("Reset", reset_rect, reset);
        ui.set_tooltip("Reset", "Goes back to SimplePaint's default material");
        ui.node("Save", save_rect, save);
        ui.set_tooltip("Save", "Saves the material to Paint.json (Ctrl+S)");
        ui.node("SaveStatus", header, Canvas::new(demo.editor.status.clone(), draw_save_status));

        // The sliders, which don't take keyboard focus, as the arrow keys
        // move the selected one.
        let mut content = Content::new(card.content);
        for (index, setting) in SETTINGS.iter().enumerate() {
            let row = content.take(ROW_HEIGHT, 0.0);
            // The row's face, from near the card's edges, like a list's row.
            let face = Rect::new(card.rect.x, row.y, card.rect.width, row.height).pad(LIST_ITEM_BACKPLATE_MARGIN);
            let row_name = format!("Setting/{}", setting.name);
            let row_face = RowFace {
                row: row_name.clone(),
                slider: setting.name,
                selected: index == editor.selected,
            };
            ui.node(&row_name, face, Canvas::new(row_face, draw_row_face).sense(Sense::CLICK).kind("ListViewItem"));

            let mut parts = row;
            let label = parts.cut_left(LABEL_WIDTH, SPACE_M);
            let range = parts.cut_right(RANGE_WIDTH, SPACE_XXL);
            let value_rect = parts.cut_right(VALUE_WIDTH, SPACE_L);
            let track = parts.centered(Size::new(parts.width, Slider::height(false)));
            let slider = Slider::new(setting.value(&editor.parameters), setting.start, setting.end)
                .step(1.0 / setting.steps)
                .allowed(setting.minimum, setting.maximum)
                .decimals(setting.decimals)
                .suffix(setting.unit);
            ui.node(setting.name, track, slider);
            ui.set_focusable(setting.name, false);
            ui.node(&format!("SettingLabel/{}", setting.name), label, Text::label(setting.name));
            ui.node(
                &format!("SettingValue/{}", setting.name),
                value_rect,
                Text::new(&setting.format(setting.value(&editor.parameters)), MONO).align(Align::End),
            );
            ui.node(&format!("SettingRange/{}", setting.name), range, Text::new(&setting.range(), BODY).secondary());
        }
        ui.node("Settings/Selection", card.rect, Canvas::new(Rect::default(), draw_selection_pill));

        content.take(0.0, SPACE_M);
        let color_row = content.take(CONTROL_HEIGHT, SPACE_M);
        ui.node("BaseColor", color_row, Canvas::new(BaseColor::default(), draw_base_color));
        content.paragraph(
            ui,
            "SettingsHelp",
            "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and Rotation by 0.1 degrees, \
             and stop at the last step that SimplePaint accepts, so Material::compile always takes them.",
        );
        fits = content.fits;
    });
    fits
}

// Keeps the settings card true to the editor, every frame.
pub fn sync(demo: &mut Demo, ui: &mut Ui) {
    let editor = &mut demo.editor;
    editor.dragging = None;
    let mut selected_face = Rect::default();
    for (index, setting) in SETTINGS.iter().enumerate() {
        let row_name = format!("Setting/{}", setting.name);
        let row_response = ui.response(&row_name);
        let response = ui.response(setting.name);
        if response.wheel.y != 0.0 && !response.held {
            // The editor turns the wheel instead of the slider, by whole
            // steps of the setting, so that a value always lands on the
            // decimal it shows, and one between steps never moves the other
            // way.
            editor.selected = index;
            editor.wheel(index, response.wheel.y as f64);
        } else if response.held {
            // Only a drag sets the value. The slider also moves a value from
            // Paint.json that's past the range into it, which the editor
            // keeps as it is until it's changed.
            let value = ui.get::<Slider>(setting.name).value();
            editor.set(index, setting.nearest(value));
        }
        if response.pressed || row_response.pressed {
            editor.selected = index;
        }
        if response.held {
            editor.dragging = Some(index);
        }
    }
    // Push the editor's values back into the controls.
    let editor = &demo.editor;
    for (index, setting) in SETTINGS.iter().enumerate() {
        let value = setting.value(&editor.parameters);
        if !ui.response(setting.name).held {
            ui.get_mut::<Slider>(setting.name).set_value(value);
        }
        ui.get_mut::<Text>(&format!("SettingValue/{}", setting.name)).set_text(&setting.format(value));
        let row_name = format!("Setting/{}", setting.name);
        let selected = index == editor.selected;
        ui.get_mut::<Canvas<RowFace>>(&row_name).data.selected = selected;
        if selected {
            selected_face = ui.node_rect(&row_name);
        }
    }
    ui.get_mut::<Canvas<Rect>>("Settings/Selection").data = selected_face;
    ui.get_mut::<Canvas<Option<(String, bool)>>>("SaveStatus").data = editor.status.clone();
    let parameters = editor.parameters;
    ui.get_mut::<Canvas<BaseColor>>("BaseColor").data = BaseColor {
        srgb: parameters.base_color_srgb,
        brightness: parameters.brightness,
    };
    ui.get_mut::<Canvas<GpuMaterial>>("Constants").data = editor.material;
}

// What the last save did, at the right of the area: a success or an error,
// with InfoBar's status icon, and the whole message as a tooltip if it's cut
// short.
fn draw_save_status(ui: &mut Ui, area: Rect, status: &Option<(String, bool)>) {
    let Some((message, failed)) = status else {
        return;
    };
    let severity = if *failed { Severity::Error } else { Severity::Success };
    let room = (area.width - ICON - SPACE_S).max(0.0);
    let width = ui.measure(message, BODY).width.min(room);
    let text = area.right_strip(width);
    let color = ui.theme().text_primary;
    ui.text_with_tooltip("SaveStatusText", text, message, BODY, color);
    ui.status_icon(Point::new(text.x - SPACE_S - ICON / 2.0, area.center().y), severity, ICON);
}

// The base color, as the sliders make it: a swatch, in hex, in linear, and
// the tone that it appears at.
#[derive(Clone, Copy, Default)]
struct BaseColor {
    srgb: [f64; 3],
    brightness: f64,
}

fn draw_base_color(ui: &mut Ui, row: Rect, color: &BaseColor) {
    let srgb = color.srgb;
    let [red, green, blue] = srgb.map(|channel| (channel * 255.0).round() as u8);
    let [r, g, b] = srgb.map(linear);
    let hex = format!("#{red:02X}{green:02X}{blue:02X}");
    let linear = format!("{r:.4}, {g:.4}, {b:.4}");
    let tone = format!("{:.3}", 1.0 - color.brightness);

    let mut parts = row;
    ui.label(parts.cut_left(LABEL_WIDTH, SPACE_M), "Base color");
    // Like a palette's swatch, which this one isn't, as it can't be chosen.
    let swatch = parts.cut_left(SWATCH_SIZE.width, SPACE_M);
    let swatch = Rect::new(swatch.x, row.center().y - SWATCH_SIZE.height / 2.0, SWATCH_SIZE.width, SWATCH_SIZE.height);
    let theme = ui.theme().clone();
    let border = Border::new(theme.control_stroke_secondary, 1.0);
    ui.painter().shape(swatch, RADIUS_CONTROL, Color::from_bytes([red, green, blue, 255]), Some(border));
    let (primary, secondary) = (theme.text_primary, theme.text_secondary);
    let groups = [
        vec![Span::new(&hex, MONO, primary)],
        vec![Span::new("Linear ", BODY, secondary), Span::new(&linear, MONO, primary)],
        vec![Span::new("Appears at tone 1 - Brightness = ", BODY, secondary), Span::new(&tone, MONO, primary)],
    ];
    let top = (row.center().y - BODY.line_height / 2.0).round();
    let mut x = parts.x;
    for group in &groups {
        x += ui.spans(Point::new(x, top), group) + SPACE_XXL;
    }
}

// What Material::compile made for the GPU, in columns of Cascadia Mono.
fn constants(editor: &Editor, ui: &mut Ui, rect: Rect) -> bool {
    let mut fits = true;
    ui.card("ConstantsCard", rect, Card::new("GPU constants").subtitle("GpuMaterial: 80 bytes at b1, five float4 registers"), |ui, card| {
        let mut content = Content::new(card.content);
        let rows_area = content.take(MONO.line_height * 5.0, 0.0);
        ui.node("Constants", rows_area, Canvas::new(editor.material, draw_constants));
        content.take(0.0, SPACE_S);
        content.paragraph(
            ui,
            "ConstantsHelp",
            "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), where numerator = \
             numerator_dark * Q + numerator_light * P, and likewise for complement.",
        );
        fits = content.fits;
    });
    fits
}

fn draw_constants(ui: &mut Ui, area: Rect, m: &GpuMaterial) {
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
    let mut area = area;
    let mut painter = ui.painter();
    painter.push_clip(area);
    for (name, value) in &rows {
        let row = area.cut_top(MONO.line_height, 0.0);
        draw_constant(&mut painter, row, name, value);
    }
    painter.pop_clip();
}

fn draw_constant(painter: &mut Painter, row: Rect, name: &str, value: &str) {
    let theme = painter.theme();
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
