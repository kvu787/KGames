// Lays out the demo's cards in fixed pixels, and keeps them true to the
// input every frame: the app bar, the status bar, and between them three
// columns of cards. The first has the Gamepad card over the Keyboard and
// Mouse cards, the second the Latest input event card over the Devices card,
// and the third the Event log. The demo supports two window setups, maximized
// and borderless fullscreen (see Library/UserInterface's README), whose
// client areas differ only in height, so the cards have fixed places in
// pixels. The 71 pixels that fullscreen adds go to the bottom row of cards
// and to the log.
//
// The combo box and the Devices card's rows are the library's widgets, which
// read the mouse's buttons from GameInput, and the pointer from where Windows'
// cursor is in the window. Everything else that the cards show is painted
// from snapshots of the devices' readings, in Canvas and ScrollCanvas nodes
// that `sync` swaps each frame.

use game_input::*;
use user_interface::controls::{
    INFO_BADGE_HEIGHT, LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_HEIGHT, LIST_ITEM_PADDING, SCROLL_BAR_SIZE, SELECTION_PILL_LENGTH,
    SELECTION_PILL_THICKNESS,
};
use user_interface::tokens::{
    BODY, BODY_STRONG, CAPTION, CARD_GAP, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, ICON, MONO, RADIUS_CONTROL,
    SPACE_L, SPACE_M, SPACE_S, SPACE_XS, SPACE_XXL,
};
use user_interface::{
    Align, AppBar, Border, Canvas, Card, Color, ComboBox, Icon, Insets, KEYCAP_HEIGHT, ListRow, ListView, Painter, Point, Rect, RowState,
    ScrollCanvas, Severity, Span, Status, Text, Theme, TooltipLine, Ui,
};

use crate::NAME;
use crate::devices::{Device, rumble_motor_names};
use crate::input::{GAMEPAD_BUTTONS, Input, LatestEvent, Listen, MOUSE_BUTTONS, Source};

// The combo box that picks which gamepad the demo listens to. The library
// names the items of its list "Gamepad/0", "Gamepad/1" and so on.
pub const GAMEPAD_COMBO_BOX: &str = "Gamepad";
// The Devices card's list. Its rows are "Devices/0", "Devices/1" and so on,
// in the card's order, each labeled with its device's label. Two devices can
// have the same label, so the rows are named by their place.
pub const DEVICE_ROWS: &str = "Devices";
// The Latest input event card's details, and the Keyboard card's keys, which
// scroll if they don't fit.
const LATEST_DETAILS: &str = "Latest";
const KEY_ROWS: &str = "Keys";

// The widths of the first two columns. The log takes the rest of the page.
pub const GAMEPAD_COLUMN: f32 = 1000.0;
pub const LATEST_COLUMN: f32 = 660.0;
// The stick graphs are square, and the trigger bars as tall.
pub const STICK_GRAPH: f32 = 400.0;
const TRIGGER_BAR: f32 = 48.0;
const GRAPH_GAP: f32 = SPACE_XXL;
const MOUSE_PAD: f32 = 200.0;
// A chip for a gamepad's or a mouse's button.
const CHIP_HEIGHT: f32 = 24.0;
const CHIP_MIN_WIDTH: f32 = 32.0;
// A row of the Keyboard card: a keycap with the key's name, then its codes.
const KEY_ROW: f32 = 28.0;
const KEY_NAME_COLUMN: f32 = 112.0;
// Where the values of the Latest input event card's and the Mouse card's
// fields start.
const LATEST_LABEL: f32 = 200.0;
const MOUSE_LABEL: f32 = 104.0;
// The combo box is as wide as its widest choice, within these.
const GAMEPAD_COMBO_BOX_MIN_WIDTH: f32 = 200.0;
const GAMEPAD_COMBO_BOX_MAX_WIDTH: f32 = 420.0;
// The combo box's choice that listens to no gamepad, as Listen names it.
const NO_GAMEPAD: &str = "None";

// What the colors mean: the left and right sticks and triggers, and each
// source of input, from the theme's colors for telling series apart.
struct Palette {
    left: Color,
    right: Color,
    keyboard: Color,
    mouse: Color,
    device: Color,
    demo: Color,
}

impl Palette {
    fn of(theme: &Theme) -> Palette {
        Palette {
            left: theme.data.blue,
            right: theme.data.orange,
            keyboard: theme.data.purple,
            mouse: theme.data.teal,
            device: theme.data.green,
            demo: theme.text_secondary,
        }
    }

    fn source(&self, source: Source) -> Color {
        match source {
            Source::Gamepad => self.left,
            Source::Keyboard => self.keyboard,
            Source::Mouse => self.mouse,
            Source::Device => self.device,
            Source::Demo => self.demo,
        }
    }
}

// Builds the bars and the cards. Without GameInput, a notice says why, and
// there's nothing to show.
pub fn build(ui: &mut Ui, input: &Result<Input, String>) {
    let bar = ui.app_bar(AppBar::new(Icon::Gamepad, NAME).subtitle("Gamepads, keyboards and mice, as GameInput reads them"));
    ui.node("AppBarStatus", bar, Canvas::new(AppBarStatus::default(), draw_app_bar_status));
    ui.status_bar(Vec::new());
    let page = ui.page();
    if input.is_err() {
        return;
    }
    let mut rest = page;
    let mut first = rest.cut_left(GAMEPAD_COLUMN, CARD_GAP);
    let mut second = rest.cut_left(LATEST_COLUMN, CARD_GAP);
    let height = gamepad_height(ui);
    gamepad(ui, first.cut_top(height, CARD_GAP));
    let [keyboard_area, mouse_area] = first.columns::<2>(CARD_GAP);
    keyboard(ui, keyboard_area);
    mouse(ui, mouse_area);
    latest(ui, second.cut_top(height, CARD_GAP));
    devices(ui, second);
    log(ui, rest);
}

// Keeps everything the cards show true to the input, and does what the
// combo box picked.
pub fn sync(ui: &mut Ui, input: &mut Result<Input, String>) {
    sync_app_bar(ui);
    sync_status_bar(ui, input);
    let Ok(input) = input else {
        return;
    };
    sync_gamepad(ui, input);
    sync_keyboard(ui, input);
    sync_mouse(ui, input);
    sync_latest(ui, input);
    sync_devices(ui, input);
    sync_log(ui, input);
}

// The app bar's right: whether the window has focus, which the demo needs to
// read input, and the frame rate.
#[derive(Clone, Default)]
struct AppBarStatus {
    rate: String,
    focused: bool,
}

fn sync_app_bar(ui: &mut Ui) {
    let stats = ui.stats();
    let status = AppBarStatus {
        rate: format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time),
        focused: ui.window_focused(),
    };
    ui.get_mut::<Canvas<AppBarStatus>>("AppBarStatus").data = status;
}

fn draw_app_bar_status(ui: &mut Ui, bar: Rect, status: &AppBarStatus) {
    let theme = ui.theme().clone();
    let mut bar = bar;
    let width = ui.measure(&status.rate, CAPTION).width.ceil();
    ui.text(bar.cut_right(width, SPACE_XXL), &status.rate, CAPTION, theme.text_secondary);
    let (text, severity) = if status.focused {
        ("Focused: reading input", Severity::Success)
    } else {
        ("Not focused: input is ignored", Severity::Warning)
    };
    let width = ICON + SPACE_S + ui.measure(text, CAPTION).width.ceil();
    let mut area = bar.cut_right(width, SPACE_XXL);
    let icon = area.cut_left(ICON, SPACE_S);
    ui.status_icon(icon.center(), severity, ICON);
    ui.text(area, text, CAPTION, theme.text_primary);
}

// The status bar: the shortcuts, the GameInput runtime, how many readings
// the demo handled, and the display that the window is on.
fn sync_status_bar(ui: &mut Ui, input: &Result<Input, String>) {
    let environment = ui.environment();
    let display = if environment.display.is_empty() {
        String::new()
    } else {
        format!("{} {}", environment.display, environment.display_name)
    };
    let (runtime, readings) = match input {
        Ok(input) => (runtime(input), format!("Readings handled: {}", input.reading_count)),
        Err(_) => ("GameInput isn't available".to_string(), String::new()),
    };
    let mut items = vec![
        Status::shortcut("F1", "Rumble test"),
        Status::shortcut("F2", "Clear the log"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::text(&runtime),
        Status::Spacer,
    ];
    for text in [&readings, &display] {
        if !text.is_empty() {
            items.push(Status::text(text));
        }
    }
    ui.get_mut::<user_interface::StatusBar>("StatusBar").set_items(items);
}

// The GameInput runtime that the demo loaded, and where from.
fn runtime(input: &Input) -> String {
    match input.runtime() {
        Some(runtime) => {
            let version = runtime.version.map(|part| part.to_string()).join(".");
            let name = runtime.path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            let location = runtime.path.parent().map(|path| format!(" in {}", path.display())).unwrap_or_default();
            format!("Runtime: {name} {version}{location}")
        }
        None => "Runtime: none, the input comes from a script".to_string(),
    }
}

// How tall the Gamepad card is, and the Latest input event card beside it:
// as tall as the Gamepad card's contents.
fn gamepad_height(ui: &mut Ui) -> f32 {
    let labels: Vec<&str> = GAMEPAD_BUTTONS.iter().map(|(_, label)| *label).collect();
    let chips = chips_height(ui, GAMEPAD_COLUMN - CARD_PADDING * 2.0, &labels);
    CARD_PADDING
        + CARD_HEADER_HEIGHT
        + CARD_HEADER_GAP
        + BODY.line_height
        + SPACE_S
        + STICK_GRAPH
        + SPACE_S
        + MONO.line_height * 2.0
        + SPACE_L
        + chips
        + SPACE_L
        + CAPTION.line_height
        + CARD_PADDING
}

// What the Gamepad card paints: a graph for each stick, a bar for each
// trigger, the raw values, a chip for each button, and the status line.
#[derive(Clone, Default)]
struct GamepadData {
    sticks: [[f32; 2]; 2],
    triggers: [f32; 2],
    dead_zone: f32,
    chips: Vec<ChipData>,
    status: String,
}

#[derive(Clone)]
struct ChipData {
    label: String,
    held: bool,
    present: bool,
}

// The note after the combo box, like a disconnected gamepad's.
#[derive(Clone, Default)]
struct ComboNote {
    note: String,
    caution: bool,
}

fn gamepad(ui: &mut Ui, rect: Rect) {
    let card = Card::new("Gamepad");
    let header = card.area(ui, rect).header;
    ui.card("GamepadCard", rect, card, |ui, card| {
        let combo = Rect::new(header.x, header.y, GAMEPAD_COMBO_BOX_MIN_WIDTH, CONTROL_HEIGHT);
        ui.node(GAMEPAD_COMBO_BOX, combo, ComboBox::new(&[NO_GAMEPAD], 0));
        ui.set_focusable(GAMEPAD_COMBO_BOX, false);
        ui.node("GamepadNote", header, Canvas::new(ComboNote::default(), draw_combo_note));
        ui.node("GamepadView", card.content, Canvas::new(GamepadData::default(), draw_gamepad));
    });
}

fn sync_gamepad(ui: &mut Ui, input: &mut Input) {
    // The combo box shows which gamepad the demo listens to, and lists each
    // connected gamepad, a picked one while it's disconnected, and none, to
    // pick another.
    let connected = match &input.listen {
        Listen::Gamepad(device) => input.gamepads().any(|info| info.id == device.id),
        Listen::NoGamepad => true,
    };
    let mut choices: Vec<Option<Device>> = input.gamepads().cloned().map(Some).collect();
    if let Listen::Gamepad(device) = &input.listen
        && !connected
    {
        choices.push(Some(device.clone()));
    }
    choices.push(None);
    let labels: Vec<&str> = choices.iter().map(|choice| choice.as_ref().map_or(NO_GAMEPAD, |device| device.label.as_str())).collect();
    if let Some(picked) = ui.selected(GAMEPAD_COMBO_BOX) {
        let listen = match choices.get(picked) {
            Some(Some(device)) => Listen::Gamepad(device.clone()),
            _ => Listen::NoGamepad,
        };
        input.pick_listen(listen);
    }
    let chosen = match &input.listen {
        Listen::Gamepad(device) => Some(device.id),
        Listen::NoGamepad => None,
    };
    let selected = choices.iter().position(|choice| choice.as_ref().map(|device| device.id) == chosen).unwrap_or(0);
    let header = ui.node_rect("GamepadNote");
    let width = ComboBox::width(ui, &labels).clamp(GAMEPAD_COMBO_BOX_MIN_WIDTH, GAMEPAD_COMBO_BOX_MAX_WIDTH).min(header.width);
    ui.set_rect(GAMEPAD_COMBO_BOX, Rect::new(header.x, header.y, width, CONTROL_HEIGHT));
    let combo = ui.get_mut::<ComboBox>(GAMEPAD_COMBO_BOX);
    combo.set_items(&labels);
    combo.set_selected(selected);

    // Until a choice is picked, the demo switches to a gamepad as soon as one
    // connects.
    let (note, caution) = match &input.listen {
        Listen::Gamepad(_) if !connected && input.listen_picked => ("Disconnected: waiting for it to reconnect", true),
        Listen::Gamepad(_) if !connected => ("Disconnected: waiting for a gamepad to connect", true),
        Listen::Gamepad(_) => ("", false),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected", true),
        Listen::NoGamepad => ("Gamepad input is ignored", false),
    };
    ui.get_mut::<Canvas<ComboNote>>("GamepadNote").data = ComboNote {
        note: note.to_string(),
        caution,
    };

    let view = &input.gamepad;
    let state = view.state;
    let rumble = if input.rumble_enabled { "on" } else { "off" };
    let status = format!(
        "Readings: {}/s   ·   Rumble test: {rumble} (F1)   ·   Motors: {}",
        view.rate.per_second,
        rumble_motor_names(view.rumble_motors)
    );
    ui.get_mut::<Canvas<GamepadData>>("GamepadView").data = GamepadData {
        sticks: [
            [state.leftThumbstickX, state.leftThumbstickY],
            [state.rightThumbstickX, state.rightThumbstickY],
        ],
        triggers: [state.leftTrigger, state.rightTrigger],
        dead_zone: input.rumble_dead_zone,
        chips: GAMEPAD_BUTTONS
            .iter()
            .map(|&(button, label)| ChipData {
                label: label.to_string(),
                held: state.buttons & button != 0,
                present: view.layout.is_none_or(|layout| layout & button != 0),
            })
            .collect(),
        status,
    };
}

fn draw_combo_note(ui: &mut Ui, header: Rect, data: &ComboNote) {
    let combo = ui.node_rect(GAMEPAD_COMBO_BOX);
    let theme = ui.theme().clone();
    let mut area = Rect::new(combo.right() + SPACE_M, header.y, (header.right() - combo.right() - SPACE_M).max(0.0), header.height);
    if data.caution {
        let icon = area.cut_left(ICON, SPACE_S);
        ui.status_icon(icon.center(), Severity::Warning, ICON);
    }
    ui.text(area, &data.note, BODY, if data.caution { theme.text_primary } else { theme.text_secondary });
}

fn draw_gamepad(ui: &mut Ui, content: Rect, data: &GamepadData) {
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let mut area = content;
    let captions = area.cut_top(BODY.line_height, SPACE_S);
    let mut plots = area.cut_top(STICK_GRAPH, SPACE_S);
    let values = area.cut_top(MONO.line_height * 2.0, SPACE_L);
    let status = area.cut_bottom(CAPTION.line_height, SPACE_L);
    let left = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
    let right = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
    let left_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);
    let right_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);

    let mut painter = ui.painter();
    for (column, caption, color) in [
        (left, "Left stick", colors.left),
        (right, "Right stick", colors.right),
        (left_trigger, "LT", colors.left),
        (right_trigger, "RT", colors.right),
    ] {
        let area = Rect::new(column.x, captions.y, column.width + GRAPH_GAP, captions.height);
        legend(&mut painter, &theme, area, caption, color);
    }
    stick_graph(&mut painter, &theme, left, data.sticks[0], data.dead_zone, colors.left);
    stick_graph(&mut painter, &theme, right, data.sticks[1], data.dead_zone, colors.right);
    trigger_bar(&mut painter, &theme, left_trigger, data.triggers[0], colors.left);
    trigger_bar(&mut painter, &theme, right_trigger, data.triggers[1], colors.right);

    // The raw values, in Cascadia Mono, so that their digits stay put.
    for (column, [x, y]) in [(left, data.sticks[0]), (right, data.sticks[1])] {
        painter.text(Point::new(column.x, values.y), &format!("x {x:+.3}  y {y:+.3}"), MONO, theme.text_primary);
        let distance = x.hypot(y);
        let degrees = y.atan2(x).to_degrees();
        let polar = format!("r {distance:.3}  {degrees:+.0}°");
        painter.text(Point::new(column.x, values.y + MONO.line_height), &polar, MONO, theme.text_secondary);
    }
    for (column, value) in [(left_trigger, data.triggers[0]), (right_trigger, data.triggers[1])] {
        painter.text(Point::new(column.x, values.y), &format!("{value:.3}"), MONO, theme.text_primary);
    }

    // A chip for each button. Those that the gamepad says it lacks are faint.
    draw_chips(ui, area, &data.chips);
    ui.caption(status, &data.status);
}

// A dot in a series' color, and its name.
fn legend(painter: &mut Painter, theme: &Theme, area: Rect, text: &str, color: Color) {
    painter.circle(Point::new(area.x + 4.0, area.center().y), 4.0, color);
    painter.text_fit(area.pad(Insets::new(SPACE_M, 0.0, 0.0, 0.0)), text, BODY_STRONG, theme.text_primary, Align::Start);
}

// A stick's position, from -1 to 1 across and up, in a square with rings at
// its edge, half way, and the dead zone, if it has one.
fn stick_graph(painter: &mut Painter, theme: &Theme, area: Rect, value: [f32; 2], dead_zone: f32, color: Color) {
    painter.shape(area, RADIUS_CONTROL, theme.control_alt_fill_secondary, Some(Border::new(theme.control_stroke, 1.0)));
    let center = Point::new(area.center().x.floor(), area.center().y.floor());
    let radius = area.width / 2.0 - SPACE_S;
    // The mouse pad has no dead zone, and a ring of radius 0 would be a dot.
    if dead_zone > 0.0 {
        painter.circle(center, radius * dead_zone, theme.caution_background);
        painter.ring(center, radius * dead_zone, 1.0, theme.caution.with_alpha(0.7));
    }
    painter.fill(Rect::new(area.x + 1.0, center.y, area.width - 2.0, 1.0), theme.divider);
    painter.fill(Rect::new(center.x, area.y + 1.0, 1.0, area.height - 2.0), theme.divider);
    painter.ring(center, radius * 0.5, 1.0, theme.control_stroke_secondary);
    painter.ring(center, radius, 1.0, theme.control_strong_stroke);

    // Stick y is up, and screen y down.
    let position = Point::new(center.x + value[0] * radius, center.y - value[1] * radius);
    painter.line(center, position, 2.0, color.with_alpha(0.5));
    painter.circle(position, 8.0, theme.control_solid_fill);
    painter.circle(position, 6.0, color);
}

// A trigger's value, as a bar that fills its track from the bottom, with a
// mark at each quarter.
fn trigger_bar(painter: &mut Painter, theme: &Theme, area: Rect, value: f32, color: Color) {
    painter.shape(area, RADIUS_CONTROL, theme.control_alt_fill_secondary, Some(Border::new(theme.control_stroke, 1.0)));
    for quarter in 1..4 {
        let y = (area.y + area.height * quarter as f32 / 4.0).round();
        painter.fill(Rect::new(area.x + 1.0, y, area.width * 0.25, 1.0), theme.control_stroke_secondary);
    }
    let track = area.inset(2.0);
    let filled = (track.height * value.clamp(0.0, 1.0)).round();
    if filled > 0.0 {
        painter.rounded(Rect::new(track.x, track.bottom() - filled, track.width, filled), RADIUS_CONTROL - 2.0, color);
    }
}

// Where chips go in the area: left to right, in rows that wrap at its width.
fn chip_rects(ui: &mut Ui, area: Rect, labels: &[&str]) -> Vec<Rect> {
    let (mut x, mut y) = (area.x, area.y);
    let mut rects = Vec::with_capacity(labels.len());
    for label in labels {
        let width = (ui.measure(label, CAPTION).width + SPACE_S * 2.0).max(CHIP_MIN_WIDTH).ceil();
        if x > area.x && x + width > area.right() {
            x = area.x;
            y += CHIP_HEIGHT + SPACE_XS;
        }
        rects.push(Rect::new(x, y, width, CHIP_HEIGHT));
        x += width + SPACE_XS;
    }
    rects
}

// How tall chips with these labels are at a width.
fn chips_height(ui: &mut Ui, width: f32, labels: &[&str]) -> f32 {
    let rects = chip_rects(ui, Rect::new(0.0, 0.0, width, 0.0), labels);
    rects.last().map_or(0.0, |rect| rect.bottom())
}

// Draws the chips, and returns how tall they are.
fn draw_chips(ui: &mut Ui, area: Rect, chips: &[ChipData]) -> f32 {
    let labels: Vec<&str> = chips.iter().map(|chip| chip.label.as_str()).collect();
    let rects = chip_rects(ui, area, &labels);
    let theme = ui.theme().clone();
    let mut painter = ui.painter();
    for (chip, rect) in chips.iter().zip(&rects) {
        let (fill, stroke, text) = if chip.held {
            (theme.accent, theme.control_stroke_on_accent, theme.text_on_accent)
        } else if chip.present {
            (theme.control_alt_fill_secondary, theme.control_stroke, theme.text_secondary)
        } else {
            (Color::TRANSPARENT, theme.control_stroke, theme.text_disabled)
        };
        painter.shape(*rect, RADIUS_CONTROL, fill, Some(Border::new(stroke, 1.0)));
        painter.text_in(*rect, &chip.label, CAPTION, text, Align::Center);
    }
    rects.last().map_or(0.0, |rect| rect.bottom() - area.y)
}

// The Latest input event card: the newest reading that changed something,
// described in full. Details that don't fit scroll.
fn latest(ui: &mut Ui, rect: Rect) {
    ui.card("LatestCard", rect, Card::new("Latest input event"), |ui, card| {
        ui.node(LATEST_DETAILS, card.content, ScrollCanvas::new(None, 0.0, draw_latest));
    });
}

fn sync_latest(ui: &mut Ui, input: &Input) {
    let subtitle = input.latest.as_ref().map(|event| format!("at {:.6} s", event.time)).unwrap_or_default();
    ui.get_mut::<Card>("LatestCard").set_subtitle(&subtitle);
    let area = ui.node_rect(LATEST_DETAILS);
    let changed = {
        let shown: &Option<LatestEvent> = &ui.get::<ScrollCanvas<Option<LatestEvent>>>(LATEST_DETAILS).data;
        match (shown, &input.latest) {
            (Some(shown), Some(latest)) => (shown.number, shown.source) != (latest.number, latest.source),
            (None, None) => false,
            _ => true,
        }
    };
    if changed {
        // The details' height, at full width, or less the bar if they scroll.
        let mut height = 0.0;
        if let Some(event) = &input.latest {
            height = latest_details(ui, event, Rect::new(area.x, area.y, area.width, f32::MAX), false);
            if height > area.height {
                let width = area.width - SCROLL_BAR_SIZE;
                height = latest_details(ui, event, Rect::new(area.x, area.y, width, f32::MAX), false);
            }
        }
        let canvas = ui.get_mut::<ScrollCanvas<Option<LatestEvent>>>(LATEST_DETAILS);
        canvas.data = input.latest.clone();
        canvas.set_content_height(height);
    }
}

fn draw_latest(ui: &mut Ui, content: Rect, event: &Option<LatestEvent>) {
    match event {
        Some(event) => {
            latest_details(ui, event, content, true);
        }
        None => {
            let secondary = ui.theme().text_secondary;
            ui.paragraph(content, "Waiting for input. Press a key, move the mouse, or use a gamepad.", BODY, secondary);
        }
    }
}

// Lays the event's details out from the top of the area, and draws them if
// `draw`, so that they can be measured first. Returns how tall they are.
fn latest_details(ui: &mut Ui, event: &LatestEvent, area: Rect, draw: bool) -> f32 {
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let mut rest = area;

    let mut heading = rest.cut_top(BODY.line_height, SPACE_S);
    if draw {
        let icon = heading.cut_left(ICON, SPACE_S);
        ui.icon(icon, source_icon(event.source), ICON, colors.source(event.source));
        ui.heading(heading, &format!("{} reading #{}", event.source.name(), event.number));
    }
    let timestamp = format!("{} µs", event.timestamp);
    let interval = event.interval.map(|interval| format!("{interval:.3} ms"));
    let mut fields = vec![
        ("Device", event.device.as_str()),
        ("Reading has", event.kinds.as_str()),
        ("Timestamp", timestamp.as_str()),
    ];
    if let Some(interval) = &interval {
        fields.push(("Since its previous reading", interval));
    }
    for (label, value) in fields {
        let row = rest.cut_top(BODY.line_height, SPACE_XS);
        if draw {
            ui.field(row, label, LATEST_LABEL, value);
        }
    }
    if interval.is_none() {
        let text = "No earlier reading from this device, so this is compared with an idle state.";
        let height = ui.paragraph_height(rest.width, text, BODY);
        let row = rest.cut_top(height, SPACE_XS);
        if draw {
            ui.paragraph(row, text, BODY, theme.text_secondary);
        }
    }

    let divider = rest.cut_top(SPACE_L, SPACE_XS);
    let title = rest.cut_top(BODY.line_height, SPACE_XS);
    if draw {
        ui.divider(divider);
        ui.heading(title, "Changed");
    }
    for change in &event.changes {
        // A bullet, and the change after it, which may wrap.
        let text = rest.pad(Insets::new(SPACE_XS + SPACE_M, 0.0, 0.0, 0.0));
        let height = ui.paragraph_height(text.width, change, BODY);
        let row = rest.cut_top(height, SPACE_XS);
        if draw {
            ui.text(Rect::new(row.x + SPACE_XS, row.y, SPACE_M, BODY.line_height), "•", BODY, theme.text_secondary);
            ui.paragraph(Rect::new(text.x, row.y, text.width, height), change, BODY, theme.text_primary);
        }
    }

    let divider = rest.cut_top(SPACE_L, SPACE_XS);
    let title = rest.cut_top(BODY.line_height, SPACE_XS);
    if draw {
        ui.divider(divider);
        ui.heading(title, "State now");
    }
    for line in &event.state {
        let row = rest.cut_top(MONO.line_height, 0.0);
        if draw {
            ui.text(row, line, MONO, theme.text_primary);
        }
    }
    rest.y - area.y
}

// What the Keyboard card shows: the keys held on the keyboard that last
// changed them, each with its codes, and the character it types.
#[derive(Clone, Default)]
struct KeyboardData {
    descriptions: Vec<(String, String)>,
}

fn keyboard(ui: &mut Ui, rect: Rect) {
    ui.card("KeyboardCard", rect, Card::new("Keyboard"), |ui, card| {
        let mut area = card.content;
        let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
        ui.node("KeyboardStatus", status, Text::caption(""));
        let heading = area.cut_top(BODY.line_height, SPACE_S);
        ui.node("KeyboardHeading", heading, Canvas::new(0usize, draw_keys_heading));
        ui.node(KEY_ROWS, area, ScrollCanvas::new(KeyboardData::default(), 0.0, draw_keys));
    });
}

fn sync_keyboard(ui: &mut Ui, input: &Input) {
    let view = &input.keyboard;
    ui.get_mut::<Card>("KeyboardCard").set_subtitle(&view.label);
    ui.get_mut::<Text>("KeyboardStatus").set_text(&format!("Readings: {}/s", view.rate.per_second));
    ui.get_mut::<Canvas<usize>>("KeyboardHeading").data = view.keys.len();
    let canvas = ui.get_mut::<ScrollCanvas<KeyboardData>>(KEY_ROWS);
    let changed = canvas.data.descriptions != view.descriptions;
    if changed {
        canvas.data = KeyboardData {
            descriptions: view.descriptions.clone(),
        };
        canvas.set_content_height(view.descriptions.len() as f32 * KEY_ROW);
    }
}

// "Keys down", with a badge of how many, or "None" under it when none are.
fn draw_keys_heading(ui: &mut Ui, heading: Rect, count: &usize) {
    let title = ui.heading(heading, "Keys down");
    if *count > 0 {
        // The badge is centered on its point, and a count of one digit is
        // round.
        ui.badge(Point::new(title.right() + SPACE_S + INFO_BADGE_HEIGHT / 2.0, heading.center().y), &count.to_string());
    }
}

fn draw_keys(ui: &mut Ui, content: Rect, data: &KeyboardData) {
    let theme = ui.theme().clone();
    if data.descriptions.is_empty() {
        ui.text(content.top_strip(BODY.line_height), "None", BODY, theme.text_tertiary);
        return;
    }
    let mut rows = content;
    for (name, details) in &data.descriptions {
        let row = rows.cut_top(KEY_ROW, 0.0);
        ui.keycap(Point::new(row.x, row.y + ((KEY_ROW - KEYCAP_HEIGHT) / 2.0).round()), name);
        ui.text(row.pad(Insets::new(KEY_NAME_COLUMN, 0.0, 0.0, 0.0)), details, MONO, theme.text_primary);
    }
}

// What the Mouse card shows: the buttons held, its motion and wheel, and a
// pad that moves with its raw motion, which has no pointer acceleration.
#[derive(Clone, Default)]
struct MouseData {
    chips: Vec<ChipData>,
    pad: [f32; 2],
    fields: Vec<(&'static str, String)>,
}

fn mouse(ui: &mut Ui, rect: Rect) {
    ui.card("MouseCard", rect, Card::new("Mouse"), |ui, card| {
        let mut area = card.content;
        let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
        ui.node("MouseStatus", status, Text::caption(""));
        ui.node("MouseView", area, Canvas::new(MouseData::default(), draw_mouse));
    });
}

fn sync_mouse(ui: &mut Ui, input: &Input) {
    let view = &input.mouse;
    let state = &view.state;
    ui.get_mut::<Card>("MouseCard").set_subtitle(&view.label);
    let mut readings = format!("Readings: {}/s", view.rate.per_second);
    if let Some(rate) = view.sample_rate {
        readings += &format!("   ·   Device reports {rate} Hz");
    }
    ui.get_mut::<Text>("MouseStatus").set_text(&readings);
    let mut fields = vec![
        ("Last motion", format!("({:+}, {:+})", view.delta[0], view.delta[1])),
        ("Total motion", format!("({}, {})", state.positionX, state.positionY)),
        ("Wheel total", format!("({}, {})", state.wheelX, state.wheelY)),
    ];
    if state.positions & GameInputMouseAbsolutePosition != 0 {
        fields.push(("Absolute", format!("({}, {})", state.absolutePositionX, state.absolutePositionY)));
    }
    ui.get_mut::<Canvas<MouseData>>("MouseView").data = MouseData {
        chips: MOUSE_BUTTONS
            .iter()
            .map(|&(button, label)| ChipData {
                label: label.to_string(),
                held: state.buttons & button != 0,
                present: true,
            })
            .collect(),
        pad: view.pad,
        fields,
    };
}

fn draw_mouse(ui: &mut Ui, content: Rect, data: &MouseData) {
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let mut area = content;
    let mut pad_column = area.cut_right(MOUSE_PAD, SPACE_L);
    let caption = pad_column.cut_top(BODY.line_height, SPACE_S);
    let pad = pad_column.top_strip(MOUSE_PAD);
    let mut painter = ui.painter();
    legend(&mut painter, &theme, caption, "Raw motion", colors.mouse);
    stick_graph(&mut painter, &theme, pad, [data.pad[0], -data.pad[1]], 0.0, colors.mouse);

    let heading = area.cut_top(BODY.line_height, SPACE_S);
    ui.heading(heading, "Buttons");
    let height = draw_chips(ui, area, &data.chips);
    area.cut_top(height, SPACE_L);
    for (label, value) in &data.fields {
        ui.field(area.cut_top(BODY.line_height, SPACE_XS), label, MOUSE_LABEL, value);
    }
}

// The Devices card: a list with a row for each connected device, which
// shows a tooltip of everything known about it while the pointer rests on
// it, and scrolls if they don't all fit. The gamepad that the Gamepad card
// shows is in the accent color, with the pill of a selection, and the
// gamepads that the demo ignores are faint.
fn devices(ui: &mut Ui, rect: Rect) {
    ui.card("DevicesCard", rect, Card::new("Devices"), |ui, card| {
        ui.node("DevicesEmpty", card.content.top_strip(BODY.line_height), Text::new("None yet", BODY).color(user_interface::TextColor::Tertiary));
        ui.node(DEVICE_ROWS, card.content, ListView::new(Vec::new(), LIST_ITEM_HEIGHT).painter(device_row));
        ui.set_focusable(DEVICE_ROWS, false);
    });
}

fn sync_devices(ui: &mut Ui, input: &Input) {
    ui.get_mut::<Card>("DevicesCard").set_subtitle(&format!("{} connected", input.devices.len()));
    ui.set_visible("DevicesEmpty", input.devices.is_empty());
    let rows: Vec<ListRow> = input
        .devices
        .iter()
        .map(|device| {
            let mut details = format!("{}: {}", device.family_name(), device.kinds());
            if let Some(index) = device.xinput_index {
                details += &format!(", XInput #{index}");
            }
            if device.rumble_motors != 0 {
                details += ", rumble";
            }
            ListRow::new(&device.label)
                .icon(device_icon(device))
                .trailing(&details)
                .accent(input.gamepad.device == Some(device.id))
                .faint(!input.listens_to(device))
        })
        .collect();
    let list = ui.get_mut::<ListView>(DEVICE_ROWS);
    let changed = list.rows().len() != rows.len()
        || list.rows().iter().zip(&rows).any(|(a, b)| (a.text != b.text) || (a.trailing != b.trailing) || a.accent != b.accent || a.faint != b.faint);
    if changed {
        list.set_rows(rows);
    }
    // Everything known about the device whose row the pointer rests on.
    if let Some(index) = ui.get::<ListView>(DEVICE_ROWS).hovered_row()
        && let Some(device) = input.devices.get(index)
    {
        device_tooltip(ui, &format!("{DEVICE_ROWS}/{index}"), device);
    }
}

// A device's row in the list, in its content area: an icon for what it is,
// its label, and its family and kinds of input.
fn device_row(ui: &mut Ui, _index: usize, content: Rect, _state: RowState, row: &ListRow) {
    let theme = ui.theme().clone();
    let (style, color, icon_color) = if row.accent {
        (BODY_STRONG, theme.accent_text, theme.accent_text)
    } else if row.faint {
        (BODY, theme.text_tertiary, theme.text_tertiary)
    } else {
        (BODY, theme.text_primary, theme.text_secondary)
    };
    let mut painter = ui.painter();
    if row.accent {
        // The list's selection pill, on its row's backplate, though the
        // combo box picks the gamepad, and not the list.
        let full = Rect::new(
            content.x - LIST_ITEM_PADDING.left,
            content.y - LIST_ITEM_PADDING.top,
            content.width + LIST_ITEM_PADDING.horizontal(),
            content.height + LIST_ITEM_PADDING.vertical(),
        );
        let plate = full.pad(LIST_ITEM_BACKPLATE_MARGIN);
        let pill = Rect::new(plate.x, plate.center().y - SELECTION_PILL_LENGTH / 2.0, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
        painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
    }
    if let Some(icon) = row.icon {
        painter.icon(Point::new(content.x + ICON / 2.0, content.center().y), icon, ICON, icon_color);
    }
    let details_width = painter.measure(&row.trailing, CAPTION).width.ceil();
    let label_area = content.pad(Insets::new(ICON + SPACE_M, 0.0, details_width + SPACE_L, 0.0));
    painter.text_fit(label_area, &row.text, style, color, Align::Start);
    painter.text_in(content, &row.trailing, CAPTION, theme.text_secondary, Align::End);
}

fn device_icon(device: &Device) -> Icon {
    if device.is_gamepad() {
        Icon::Gamepad
    } else if device.supported_input & GameInputKindKeyboard != 0 {
        Icon::Keyboard
    } else {
        Icon::Mouse
    }
}

fn source_icon(source: Source) -> Icon {
    match source {
        Source::Gamepad => Icon::Gamepad,
        Source::Keyboard => Icon::Keyboard,
        Source::Mouse => Icon::Mouse,
        Source::Device | Source::Demo => Icon::Info,
    }
}

// How wide a device's tooltip may be, so that its every detail fits in the
// window's height.
pub const DEVICE_TOOLTIP_WIDTH: f32 = 720.0;

// Everything that GameInput and Windows say about a device, in a tooltip on
// its row, in three parts: what GameInput says, Windows' node for it, and
// what that's connected through. Long values, like the device's path, wrap.
fn device_tooltip(ui: &mut Ui, name: &str, device: &Device) {
    let sections = device.details();
    let mut lines = vec![TooltipLine::Heading(&device.label)];
    for section in &sections {
        lines.push(TooltipLine::Separator);
        lines.push(TooltipLine::Heading(&section.title));
        for (label, value) in &section.rows {
            lines.push(TooltipLine::Pair(label, value));
        }
    }
    ui.tooltip_for_wide(name, &lines, DEVICE_TOOLTIP_WIDTH);
}

// What the Event log card paints: the newest entries that fit, their spans'
// bounds, and their sources.
#[derive(Clone, Default)]
struct LogData {
    entries: Vec<LogLine>,
}

#[derive(Clone, PartialEq)]
struct LogLine {
    line: String,
    source_at: usize,
    text_at: usize,
    source: Source,
}

// The Event log card: everything above as it happens, newest first, in
// Cascadia Mono, so that its columns line up: when, where from, and what.
fn log(ui: &mut Ui, rect: Rect) {
    ui.card("LogCard", rect, Card::new("Event log").subtitle("Newest first   ·   a run of motion from one device shares a line"), |ui, card| {
        ui.node("Log", card.content, Canvas::new(LogData::default(), draw_log));
    });
}

fn sync_log(ui: &mut Ui, input: &Input) {
    let area = ui.node_rect("Log");
    let fit = (area.height / MONO.line_height).ceil() as usize;
    let entries: Vec<LogLine> = input
        .log
        .iter()
        .take(fit)
        .map(|entry| LogLine {
            line: entry.line.clone(),
            source_at: entry.source_at,
            text_at: entry.text_at,
            source: entry.source,
        })
        .collect();
    let canvas = ui.get_mut::<Canvas<LogData>>("Log");
    if canvas.data.entries != entries {
        canvas.data = LogData { entries };
    }
}

fn draw_log(ui: &mut Ui, content: Rect, data: &LogData) {
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let mut area = content;
    if data.entries.is_empty() {
        ui.text(area.top_strip(BODY.line_height), "Nothing yet", BODY, theme.text_tertiary);
        return;
    }
    for entry in &data.entries {
        if area.height < MONO.line_height {
            break;
        }
        let row = area.cut_top(MONO.line_height, 0.0);
        let line = &entry.line;
        // The time and the source, then what happened, cut short to fit.
        let lead = ui.measure(&line[..entry.text_at], MONO).width;
        let text = ui.painter().fit(&line[entry.text_at..], MONO, (row.width - lead).max(0.0));
        let spans = [
            Span::new(&line[..entry.source_at], MONO, theme.text_secondary),
            Span::new(&line[entry.source_at..entry.text_at], MONO, colors.source(entry.source)),
            Span::new(&text, MONO, theme.text_primary),
        ];
        ui.spans(row.origin(), &spans);
    }
}
