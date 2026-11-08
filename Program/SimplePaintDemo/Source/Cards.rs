// Describes the demo's cards from the editor's state, with
// Library/UserInterface's widgets, and what the input on each one means.
// Programs support two client areas, 2560 x 1369 maximized and 2560 x 1440
// in borderless fullscreen, so the cards have fixed places in pixels. Of the
// 71 pixels that borderless fullscreen adds to the height, the preview and
// the GPU constants card get all of them, as each is at the bottom of its
// column.
//
// The preview card has a view in it, which the demo's render fills with the
// sphere before the interface draws over the rest of the frame.

use user_interface::controls::{LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_HEIGHT, LIST_ITEM_RADIUS, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS};
use user_interface::tokens::{
    BODY, CARD_GAP, CARD_PADDING, CONTROL_HEIGHT, ICON, MONO, RADIUS_CONTROL, SPACE_L, SPACE_M, SPACE_S, SPACE_XXL,
};
use user_interface::{
    AppBar, Border, Button, Canvas, Card, Color, Dispatch, Draw, Icon, Interaction, Point, Rect, Run, Sense, Severity, Size, Slider, Spans,
    Status, StatusIcon, Text, TextColor, Tree, Ui, Widget,
};

use crate::demo::{Demo, Message, TITLE};
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

pub fn view(demo: &Demo, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    let subtitle = "A SimplePaint material on a UV sphere, with an editor for its settings";
    let status = status_bar(ui);
    let shell = tree.shell(ui, AppBar::new(Icon::Color, TITLE).subtitle(subtitle), false, Some(status));
    let mut page = shell.page;
    let mut overflow = demo.overflow.borrow_mut();
    overflow.clear();
    let preview_rect = page.cut_left(PREVIEW_WIDTH, CARD_GAP);
    preview(demo, ui, &mut tree, preview_rect);
    let mut column = page;
    let fits = [
        ("Settings", settings(demo, ui, &mut tree, column.cut_top(SETTINGS_HEIGHT, CARD_GAP))),
        ("GPU constants", constants(&demo.editor, ui, &mut tree, column)),
    ];
    overflow.extend(fits.iter().filter(|(_, fits)| !fits).map(|(name, _)| *name));
    tree
}

fn status_bar(ui: &mut Ui) -> Vec<Status> {
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    vec![
        Status::shortcut("Up/Down", "Select a setting"),
        Status::shortcut("Left/Right", "Change it"),
        Status::shortcut("Shift+Left/Right", "By 10 steps"),
        Status::shortcut("Ctrl+S", "Save to Paint.json"),
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::text(environment.gpu.clone()),
        Status::text(display.trim().to_string()),
    ]
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
    fn paragraph(&mut self, ui: &mut Ui, tree: &mut Tree<Message>, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, BODY);
        let rect = self.take(height, 0.0);
        tree.draw(rect, Text::paragraph(text).secondary());
    }
}

// The preview: a card whose view the demo's render fills with the sphere,
// around the card's color. The demo notes where the view is and the card's
// color here, for render.
fn preview(demo: &Demo, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let card = Card::new("Preview")
        .subtitle("A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.")
        .with_view();
    let area = card.area(ui, rect);
    demo.view.set(Some(area.content));
    demo.background.set(ui.theme().card_opaque());
    tree.draw(rect, card);
}

// A slider for each setting, in rows like a list's, of which the arrow keys
// move the selected one, the base color, and the buttons that save and reset
// the material.
fn settings(demo: &Demo, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> bool {
    let editor = &demo.editor;
    let mut fits = true;
    tree.card(ui, None, rect, Card::new("Settings"), |ui, card, area| {
        // The buttons at the header's right, and what the last save did,
        // left of them.
        let mut header = area.header;
        let save = Button::new("Save").icon(Icon::Save).accent().on_press(Message::Save);
        let reset = Button::new("Reset").icon(Icon::Refresh).on_press(Message::Reset);
        let save_rect = header.cut_right(save.width(ui).max(BUTTON_WIDTH), SPACE_S);
        let reset_rect = header.cut_right(reset.width(ui).max(BUTTON_WIDTH), SPACE_L);
        card.add("Reset", reset_rect, reset).set_tooltip("Goes back to SimplePaint's default material");
        card.add("Save", save_rect, save).set_tooltip("Saves the material to Paint.json (Ctrl+S)");
        save_status(ui, card, header, &editor.status);

        // The sliders don't take keyboard focus, as the arrow keys move the
        // selected one.
        card.set_focusable(false);
        let mut content = Content::new(area.content);
        for index in 0..SETTINGS.len() {
            let row = content.take(ROW_HEIGHT, 0.0);
            setting_row(editor, card, index, row, area.rect);
        }
        card.set_focusable(true);

        content.take(0.0, SPACE_M);
        base_color(ui, card, content.take(CONTROL_HEIGHT, SPACE_M), editor);
        content.paragraph(
            ui,
            card,
            "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and Rotation by 0.1 degrees, \
             and stop at the last step that SimplePaint accepts, so Material::compile always takes them.",
        );
        fits = content.fits;
    });
    fits
}

// A setting's row face, like a list's item: a press on it selects the
// setting, its fill follows the pointer on it or on its slider, and the
// selected one has the accent's pill at its left edge.
struct SettingRow {
    index: usize,
    selected: bool,
    // The setting's name, which its slider's target has.
    name: &'static str,
}

impl Widget<Message> for SettingRow {
    fn kind(&self) -> &'static str {
        "ListViewItem"
    }

    fn label(&self) -> Option<String> {
        Some(self.name.to_string())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<Message>) {
        if interaction.whole().pressed {
            ctx.send(Message::Select(self.index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let face = ctx.rect();
        let look = ctx.look().clone();
        // The pointer on the slider, or holding it, is on the row too.
        let slider_hovered = ctx.hovered_target().is_some_and(|target| target.name == self.name);
        let slider_held = ctx.held_target().is_some_and(|target| target.name == self.name);
        let hovered = look.hit.hovered || slider_hovered;
        let pressed = look.hit.pressing() || slider_held;
        let theme = ctx.theme();
        // ListViewItemBackground in each of the row's states.
        let fill = match (self.selected, pressed, hovered) {
            (false, true, _) | (true, false, true) => theme.subtle_fill_tertiary,
            (false, false, true) | (true, _, _) => theme.subtle_fill_secondary,
            (false, false, false) => theme.subtle_fill_transparent,
        };
        let accent = theme.accent;
        let mut painter = ctx.painter();
        painter.rounded(face, LIST_ITEM_RADIUS, fill);
        // The pill that marks the selected row, at its face's left edge, as
        // a list's.
        if self.selected {
            let y = (face.center().y - SELECTION_PILL_LENGTH / 2.0).round();
            let pill = Rect::new(face.x, y, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
            painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, accent);
        }
    }
}

// A setting's row: its face, its name, its slider, its value, and its range.
fn setting_row(editor: &Editor, tree: &mut Tree<Message>, index: usize, row: Rect, card: Rect) {
    let setting = &SETTINGS[index];
    // The row's face, from near the card's edges, like a list's row.
    let face = Rect::new(card.x, row.y, card.width, row.height).pad(LIST_ITEM_BACKPLATE_MARGIN);
    tree.add(
        &format!("Setting/{}", setting.name),
        face,
        SettingRow {
            index,
            selected: index == editor.selected,
            name: setting.name,
        },
    );

    let mut parts = row;
    let label = parts.cut_left(LABEL_WIDTH, SPACE_M);
    let range = parts.cut_right(RANGE_WIDTH, SPACE_XXL);
    let value_rect = parts.cut_right(VALUE_WIDTH, SPACE_L);
    let track = parts.centered(Size::new(parts.width, Slider::<Message>::height(false)));

    let value = setting.value(&editor.parameters);
    let slider = setting
        .slider(value)
        .on_change(move |value| Message::Set(index, value))
        .on_hold(move |held| Message::Drag(index, held))
        .on_wheel(move |_| Message::Select(index));
    tree.add(setting.name, track, slider);

    tree.draw(label, Text::label(setting.name));
    tree.draw(value_rect, Text::label(setting.format(value)).style(MONO).end());
    tree.draw(range, Text::label(setting.range()).secondary());
}

// What the last save did, at the right of the area: a success or an error,
// with InfoBar's status icon, and the whole message as a tooltip if it's cut
// short.
fn save_status(ui: &mut Ui, tree: &mut Tree<Message>, area: Rect, status: &Option<(String, bool)>) {
    let Some((message, failed)) = status else {
        return;
    };
    let severity = if *failed { Severity::Error } else { Severity::Success };
    let room = (area.width - ICON - SPACE_S).max(0.0);
    let width = ui.measure(message, BODY).width.min(room);
    let text = area.right_strip(width);
    tree.add("SaveStatus", text, Text::label(message).hoverable()).set_tooltip(message);
    let icon = Rect::from_center(Point::new(text.x - SPACE_S - ICON / 2.0, area.center().y), Size::square(ICON));
    tree.draw(icon, StatusIcon::new(severity, ICON));
}

// The base color, as the sliders make it: a swatch, in hex, in linear, and
// the tone that it appears at.
fn base_color(ui: &mut Ui, tree: &mut Tree<Message>, row: Rect, editor: &Editor) {
    let parameters = editor.parameters;
    let srgb = parameters.base_color_srgb;
    let [red, green, blue] = srgb.map(|channel| (channel * 255.0).round() as u8);
    let [r, g, b] = srgb.map(linear);
    let hex = format!("#{red:02X}{green:02X}{blue:02X}");
    let linear = format!("{r:.4}, {g:.4}, {b:.4}");
    let tone = format!("{:.3}", 1.0 - parameters.brightness);

    let mut parts = row;
    tree.draw(parts.cut_left(LABEL_WIDTH, SPACE_M), Text::label("Base color"));
    // Like a palette's swatch, which this one isn't, as it can't be chosen.
    let column = parts.cut_left(SWATCH_SIZE.width, SPACE_M);
    let swatch = Rect::new(column.x, row.center().y - SWATCH_SIZE.height / 2.0, SWATCH_SIZE.width, SWATCH_SIZE.height);
    let color = Color::from_bytes([red, green, blue, 255]);
    tree.draw(
        swatch,
        Canvas::new(move |painter, rect| {
            let border = Border::new(painter.theme().control_stroke_secondary, 1.0);
            painter.shape(rect, RADIUS_CONTROL, color, Some(border));
        }),
    );
    let groups = [
        Spans::new(vec![Run::new(hex, MONO, TextColor::Primary)]),
        Spans::new(vec![Run::new("Linear ", BODY, TextColor::Secondary), Run::new(linear, MONO, TextColor::Primary)]),
        Spans::new(vec![
            Run::new("Appears at tone 1 - Brightness = ", BODY, TextColor::Secondary),
            Run::new(tone, MONO, TextColor::Primary),
        ]),
    ];
    let top = (row.center().y - BODY.line_height / 2.0).round();
    let mut x = parts.x;
    for spans in groups {
        let width = spans.width(ui.fonts());
        tree.draw(Rect::new(x, top, width, BODY.line_height), spans);
        x += width + SPACE_XXL;
    }
}

// What Material::compile made for the GPU, in columns of Cascadia Mono.
fn constants(editor: &Editor, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> bool {
    let card = Card::new("GPU constants").subtitle("GpuMaterial: 80 bytes at b1, five float4 registers");
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
    let mut fits = true;
    tree.card(ui, None, rect, card, |ui, card, area| {
        let mut content = Content::new(area.content);
        for (name, value) in rows {
            let row = content.take(MONO.line_height, 0.0);
            card.draw(row, Text::label(name).style(MONO).secondary());
            card.draw(Rect::new(row.x + NAME_WIDTH, row.y, (row.width - NAME_WIDTH).max(0.0), row.height), Text::label(value).style(MONO));
        }
        content.take(0.0, SPACE_S);
        content.paragraph(
            ui,
            card,
            "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), where numerator = \
             numerator_dark * Q + numerator_light * P, and likewise for complement.",
        );
        fits = content.fits;
    });
    fits
}

fn linear(srgb: f64) -> f64 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}
