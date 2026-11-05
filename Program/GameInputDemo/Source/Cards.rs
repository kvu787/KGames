// Lays out the demo's cards in fixed pixels, and draws them with
// Library/UserInterface's widgets and painter: the app bar, the status bar,
// and between them three columns of cards. The first has the Gamepad card
// over the Keyboard and Mouse cards, the second the Latest input event card
// over the Devices card, and the third the Event log. The demo supports two
// window setups, maximized and borderless fullscreen (see
// Library/UserInterface's README), whose client areas differ only in
// height, so the cards have fixed places in pixels. The 71 pixels that
// fullscreen adds go to the bottom row of cards and to the log.
//
// The combo box and the Devices card's rows read the mouse from the window's
// messages, not from GameInput, because they say where the pointer is in the
// window. Everything else that the cards show comes from GameInput.

use game_input::*;
use user_interface::tokens::{
    BODY, BODY_STRONG, CAPTION, CARD_GAP, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT,
    DURATION_FASTER, ICON, LIST_ROW_HEIGHT, MONO, RADIUS_CONTROL, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS,
    SPACE_L, SPACE_M, SPACE_S, SPACE_XS, SPACE_XXL,
};
use user_interface::{
    Align, Color, Corners, Icon, Painter, Point, Rect, Sense, Status, Stroke, TextStyle, Theme, TooltipLine, Ui,
};

use crate::NAME;
use crate::devices::{Device, rumble_motor_names};
use crate::input::{GAMEPAD_BUTTONS, Input, Listen, MOUSE_BUTTONS, Source};

// The combo box that picks which gamepad the demo listens to. The library
// names the items of its list "Gamepad/0", "Gamepad/1" and so on.
pub const GAMEPAD_COMBO_BOX: &str = "Gamepad";
// The Devices card's rows are "Devices/0", "Devices/1" and so on, in the
// card's order, each labeled with its device's label. Two devices can have
// the same label, so the rows are named by their place.
pub const DEVICE_ROWS: &str = "Devices";

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
const KEYCAP_HEIGHT: f32 = 22.0;
const KEY_NAME_COLUMN: f32 = 112.0;
// A row of the Devices card.
const DEVICE_ROW: f32 = LIST_ROW_HEIGHT;
// Where the values of the Latest input event card's and the Mouse card's
// fields start.
const LATEST_LABEL: f32 = 200.0;
const MOUSE_LABEL: f32 = 104.0;
// The combo box is as wide as its widest choice, within these.
const COMBO_BOX_MIN_WIDTH: f32 = 200.0;
const COMBO_BOX_MAX_WIDTH: f32 = 420.0;
// Around a combo box's text: its padding, and its chevron.
const COMBO_BOX_CHROME: f32 = SPACE_M + 38.0 + SPACE_S;
// A device tooltip's values wrap at this width.
const TOOLTIP_VALUE_WIDTH: f32 = 560.0;
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

// Draws the app bar, the status bar and the cards, and listens to the
// gamepad that's picked in the combo box.
pub fn draw(ui: &mut Ui, input: &mut Result<Input, String>) {
    app_bar(ui);
    status_bar(ui, input);
    let page = ui.page();
    // Without GameInput, a notice says why, and there's nothing to show.
    let Ok(input) = input else {
        return;
    };
    let mut rest = page;
    let mut first = rest.cut_left(GAMEPAD_COLUMN, CARD_GAP);
    let mut second = rest.cut_left(LATEST_COLUMN, CARD_GAP);
    let height = gamepad_height(ui);
    gamepad(ui, input, first.cut_top(height, CARD_GAP));
    let [keyboard_area, mouse_area] = first.columns::<2>(CARD_GAP);
    keyboard(ui, input, keyboard_area);
    mouse(ui, input, mouse_area);
    latest(ui, input, second.cut_top(height, CARD_GAP));
    devices(ui, input, second);
    log(ui, input, rest);
}

// The app bar: what the demo is, whether the window has focus, which it
// needs to read input, and the frame rate.
fn app_bar(ui: &mut Ui) {
    let mut bar = ui.app_bar(Icon::Gamepad, NAME);
    let theme = ui.theme().clone();
    let stats = ui.stats();
    let rate = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let width = ui.measure(&rate, CAPTION).width.ceil();
    ui.text(bar.cut_right(width, SPACE_XXL), &rate, CAPTION, theme.text_secondary);

    let (status, color, mark) = if ui.window_focused() {
        ("Focused: reading input", theme.success, Icon::StatusSuccess)
    } else {
        ("Not focused: input is ignored", theme.caution, Icon::StatusWarning)
    };
    let width = ICON + SPACE_S + ui.measure(status, CAPTION).width.ceil();
    let area = bar.cut_right(width, SPACE_XXL);
    let mut painter = ui.painter();
    let center = Point::new(area.x + ICON / 2.0, area.center().y);
    painter.icon(center, Icon::StatusCircle, ICON, color);
    painter.icon(center, mark, ICON, theme.text_on_accent);
    painter.text_in(area.inset_by(ICON + SPACE_S, 0.0, 0.0, 0.0), status, CAPTION, theme.text_primary, Align::Start);
    ui.text(bar, "Gamepads, keyboards and mice, as GameInput reads them", CAPTION, theme.text_secondary);
}

// The status bar: the shortcuts, the GameInput runtime, how many readings
// the demo handled, and the display that the window is on.
fn status_bar(ui: &mut Ui, input: &Result<Input, String>) {
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
    // A runtime deep in folders is cut short, so that the rest fits.
    let runtime = ui.painter().fit(&runtime, CAPTION, 1000.0);
    let mut items = vec![
        Status::Shortcut("F1", "Rumble test"),
        Status::Shortcut("F2", "Clear the log"),
        Status::Shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::Text(&runtime),
    ];
    for text in [&readings, &display] {
        if !text.is_empty() {
            items.push(Status::Text(text));
        }
    }
    ui.status_bar(&items);
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

// The Gamepad card: a graph for each stick, a bar for each trigger, the raw
// values, and a chip for each button, which lights up while it's held.
fn gamepad(ui: &mut Ui, input: &mut Input, rect: Rect) {
    let card = ui.card(rect, "Gamepad");
    gamepad_combo_box(ui, input, card.header);
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let view = &input.gamepad;
    let state = view.state;

    let mut area = card.content;
    let captions = area.cut_top(BODY.line_height, SPACE_S);
    let mut plots = area.cut_top(STICK_GRAPH, SPACE_S);
    let values = area.cut_top(MONO.line_height * 2.0, SPACE_L);
    let status = area.cut_bottom(CAPTION.line_height, SPACE_L);
    let left = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
    let right = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
    let left_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);
    let right_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);
    let sticks = [
        [state.leftThumbstickX, state.leftThumbstickY],
        [state.rightThumbstickX, state.rightThumbstickY],
    ];

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
    stick_graph(&mut painter, &theme, left, sticks[0], input.rumble_dead_zone, colors.left);
    stick_graph(&mut painter, &theme, right, sticks[1], input.rumble_dead_zone, colors.right);
    trigger_bar(&mut painter, &theme, left_trigger, state.leftTrigger, colors.left);
    trigger_bar(&mut painter, &theme, right_trigger, state.rightTrigger, colors.right);

    // The raw values, in Cascadia Mono, so that their digits stay put.
    for (column, [x, y]) in [(left, sticks[0]), (right, sticks[1])] {
        painter.text(Point::new(column.x, values.y), &format!("x {x:+.3}  y {y:+.3}"), MONO, theme.text_primary);
        let distance = x.hypot(y);
        let degrees = y.atan2(x).to_degrees();
        let polar = format!("r {distance:.3}  {degrees:+.0}°");
        painter.text(Point::new(column.x, values.y + MONO.line_height), &polar, MONO, theme.text_secondary);
    }
    for (column, value) in [(left_trigger, state.leftTrigger), (right_trigger, state.rightTrigger)] {
        painter.text(Point::new(column.x, values.y), &format!("{value:.3}"), MONO, theme.text_primary);
    }

    // A chip for each button. Those that the gamepad says it lacks are faint.
    let chips: Vec<Chip> = GAMEPAD_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            held: state.buttons & button != 0,
            present: view.layout.is_none_or(|layout| layout & button != 0),
        })
        .collect();
    draw_chips(ui, area, &chips);

    let rumble = if input.rumble_enabled { "on" } else { "off" };
    let line = format!(
        "Readings: {}/s   ·   Rumble test: {rumble} (F1)   ·   Motors: {}",
        view.rate.per_second,
        rumble_motor_names(view.rumble_motors)
    );
    ui.text(status, &line, CAPTION, theme.text_secondary);
}

// The combo box in the Gamepad card's header, which shows which gamepad the
// demo listens to, and lists each connected gamepad, a picked one while it's
// disconnected, and none, to pick another. After it, a note on what the card
// shows, if it isn't the gamepad.
fn gamepad_combo_box(ui: &mut Ui, input: &mut Input, header: Rect) {
    let connected = match &input.listen {
        Listen::Gamepad(device) => input.gamepads().any(|info| info.id == device.id),
        Listen::NoGamepad => true,
    };
    let (picked, combo_box) = {
        let mut choices: Vec<Option<&Device>> = input.gamepads().map(Some).collect();
        if let Listen::Gamepad(device) = &input.listen
            && !connected
        {
            choices.push(Some(device));
        }
        choices.push(None);
        let labels: Vec<&str> = choices.iter().map(|choice| choice.map_or(NO_GAMEPAD, |device| device.label.as_str())).collect();
        let chosen = match &input.listen {
            Listen::Gamepad(device) => Some(device.id),
            Listen::NoGamepad => None,
        };
        let mut selected = choices.iter().position(|choice| choice.map(|device| device.id) == chosen).unwrap_or(0);
        let widest = labels.iter().map(|label| ui.measure(label, BODY).width).fold(0.0, f32::max);
        let width = (widest + COMBO_BOX_CHROME).clamp(COMBO_BOX_MIN_WIDTH, COMBO_BOX_MAX_WIDTH).min(header.width).ceil();
        let rect = Rect::new(header.x, header.y, width, CONTROL_HEIGHT);
        let response = ui.combo_box(GAMEPAD_COMBO_BOX, rect, &mut selected, &labels);
        let picked = response.changed.then(|| match choices[selected] {
            Some(device) => Listen::Gamepad(device.clone()),
            None => Listen::NoGamepad,
        });
        (picked, rect)
    };
    if let Some(listen) = picked {
        input.pick_listen(listen);
    }

    // Until a choice is picked, the demo switches to a gamepad as soon as one
    // connects.
    let theme = ui.theme().clone();
    let (note, caution) = match &input.listen {
        Listen::Gamepad(_) if !connected && input.listen_picked => ("Disconnected: waiting for it to reconnect", true),
        Listen::Gamepad(_) if !connected => ("Disconnected: waiting for a gamepad to connect", true),
        Listen::Gamepad(_) => ("", false),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected", true),
        Listen::NoGamepad => ("Gamepad input is ignored", false),
    };
    let mut area = Rect::new(combo_box.right() + SPACE_M, header.y, (header.right() - combo_box.right() - SPACE_M).max(0.0), header.height);
    if caution {
        let icon = area.cut_left(ICON, SPACE_S);
        ui.icon(icon, Icon::Warning, ICON, theme.caution);
    }
    ui.text(area, note, BODY, if caution { theme.caution } else { theme.text_secondary });
}

// A dot in a series' color, and its name.
fn legend(painter: &mut Painter, theme: &Theme, area: Rect, text: &str, color: Color) {
    painter.circle(Point::new(area.x + 4.0, area.center().y), 4.0, color);
    painter.text_in(area.inset_by(SPACE_M, 0.0, 0.0, 0.0), text, BODY_STRONG, theme.text_primary, Align::Start);
}

// A stick's position, from -1 to 1 across and up, in a square with rings at
// its edge, half way, and the dead zone, if it has one.
fn stick_graph(painter: &mut Painter, theme: &Theme, area: Rect, value: [f32; 2], dead_zone: f32, color: Color) {
    painter.shape(area, Corners::all(RADIUS_CONTROL), theme.control_alt_fill_secondary, Some(Stroke::new(theme.control_stroke, 1.0)));
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
    painter.shape(area, Corners::all(RADIUS_CONTROL), theme.control_alt_fill_secondary, Some(Stroke::new(theme.control_stroke, 1.0)));
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

// A button's chip, which is in the accent color while it's held.
struct Chip<'a> {
    label: &'a str,
    held: bool,
    // Whether the device has the button.
    present: bool,
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
fn draw_chips(ui: &mut Ui, area: Rect, chips: &[Chip]) -> f32 {
    let labels: Vec<&str> = chips.iter().map(|chip| chip.label).collect();
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
        painter.shape(*rect, Corners::all(RADIUS_CONTROL), fill, Some(Stroke::new(stroke, 1.0)));
        painter.text_in(*rect, chip.label, CAPTION, text, Align::Center);
    }
    rects.last().map_or(0.0, |rect| rect.bottom() - area.y)
}

// The Latest input event card: the newest reading that changed something,
// described in full.
fn latest(ui: &mut Ui, input: &Input, rect: Rect) {
    let card = ui.card(rect, "Latest input event");
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let mut area = card.content;
    let Some(event) = &input.latest else {
        let text = "Waiting for input. Press a key, move the mouse, or use a gamepad.";
        ui.paragraph(area, text, BODY, theme.text_secondary);
        return;
    };
    ui.text_aligned(card.header, &format!("at {:.6} s", event.time), CAPTION, theme.text_secondary, Align::End);
    ui.painter().push_clip(area);

    let mut heading = area.cut_top(BODY.line_height, SPACE_S);
    let icon = heading.cut_left(ICON, SPACE_S);
    ui.icon(icon, source_icon(event.source), ICON, colors.source(event.source));
    let title = format!("{} reading #{}", event.source.name(), event.number);
    ui.text(heading, &title, BODY_STRONG, theme.text_primary);
    for (label, value) in [
        ("Device", event.device.as_str()),
        ("Reading has", event.kinds.as_str()),
        ("Timestamp", &format!("{} µs", event.timestamp)),
    ] {
        ui.field(area.cut_top(BODY.line_height, SPACE_XS), label, LATEST_LABEL, value);
    }
    match event.interval {
        Some(interval) => {
            let value = format!("{interval:.3} ms");
            ui.field(area.cut_top(BODY.line_height, SPACE_XS), "Since its previous reading", LATEST_LABEL, &value);
        }
        None => {
            let text = "No earlier reading from this device, so this is compared with an idle state.";
            let height = ui.paragraph(area, text, BODY, theme.text_secondary);
            area.cut_top(height, SPACE_XS);
        }
    }

    ui.divider(area.cut_top(SPACE_L, SPACE_XS));
    ui.text(area.cut_top(BODY.line_height, SPACE_XS), "Changed", BODY_STRONG, theme.text_primary);
    for change in &event.changes {
        let bullet = area.inset_by(SPACE_XS, 0.0, 0.0, 0.0);
        ui.text(bullet.top_strip(BODY.line_height), "•", BODY, theme.text_secondary);
        let height = ui.paragraph(bullet.inset_by(SPACE_M, 0.0, 0.0, 0.0), change, BODY, theme.text_primary);
        area.cut_top(height, SPACE_XS);
    }

    ui.divider(area.cut_top(SPACE_L, SPACE_XS));
    ui.text(area.cut_top(BODY.line_height, SPACE_XS), "State now", BODY_STRONG, theme.text_primary);
    for line in &event.state {
        ui.text(area.cut_top(MONO.line_height, 0.0), line, MONO, theme.text_primary);
    }
    ui.painter().pop_clip();
}

// The Keyboard card: the keys held on the keyboard that last changed them,
// each with its codes, and the character it types.
fn keyboard(ui: &mut Ui, input: &Input, rect: Rect) {
    let view = &input.keyboard;
    let card = ui.card(rect, "Keyboard");
    let theme = ui.theme().clone();
    ui.text_aligned(card.header, &view.label, CAPTION, theme.text_secondary, Align::End);
    let mut area = card.content;
    let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
    ui.text(status, &format!("Readings: {}/s", view.rate.per_second), CAPTION, theme.text_secondary);

    let heading = area.cut_top(BODY.line_height, SPACE_S);
    let title = ui.text(heading, "Keys down", BODY_STRONG, theme.text_primary);
    if view.keys.is_empty() {
        ui.text(area.top_strip(BODY.line_height), "None", BODY, theme.text_tertiary);
        return;
    }
    ui.badge(Point::new(title.right() + SPACE_S, heading.center().y), &view.keys.len().to_string());
    // If they don't all fit, the last row says how many more there are.
    let fits = (area.height / KEY_ROW).floor() as usize;
    let shown = if view.keys.len() > fits { fits.saturating_sub(1) } else { view.keys.len() };
    for (name, details) in view.descriptions.iter().take(shown) {
        let row = area.cut_top(KEY_ROW, 0.0);
        ui.keycap(Point::new(row.x, row.y + ((KEY_ROW - KEYCAP_HEIGHT) / 2.0).round()), name);
        ui.text(row.inset_by(KEY_NAME_COLUMN, 0.0, 0.0, 0.0), details, MONO, theme.text_primary);
    }
    if shown < view.keys.len() {
        let more = format!("and {} more", view.keys.len() - shown);
        ui.text(area.top_strip(KEY_ROW), &more, BODY, theme.text_secondary);
    }
}

// The Mouse card: the buttons held on the mouse that last changed, its
// motion and wheel, and a pad that moves with its raw motion, which has no
// pointer acceleration.
fn mouse(ui: &mut Ui, input: &Input, rect: Rect) {
    let view = &input.mouse;
    let state = &view.state;
    let card = ui.card(rect, "Mouse");
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    ui.text_aligned(card.header, &view.label, CAPTION, theme.text_secondary, Align::End);
    let mut area = card.content;
    let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
    let mut readings = format!("Readings: {}/s", view.rate.per_second);
    if let Some(rate) = view.sample_rate {
        readings += &format!("   ·   Device reports {rate} Hz");
    }
    ui.text(status, &readings, CAPTION, theme.text_secondary);

    let mut pad_column = area.cut_right(MOUSE_PAD, SPACE_L);
    let caption = pad_column.cut_top(BODY.line_height, SPACE_S);
    let pad = pad_column.top_strip(MOUSE_PAD);
    let mut painter = ui.painter();
    legend(&mut painter, &theme, caption, "Raw motion", colors.mouse);
    stick_graph(&mut painter, &theme, pad, [view.pad[0], -view.pad[1]], 0.0, colors.mouse);

    let heading = area.cut_top(BODY.line_height, SPACE_S);
    ui.text(heading, "Buttons", BODY_STRONG, theme.text_primary);
    let chips: Vec<Chip> = MOUSE_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            held: state.buttons & button != 0,
            present: true,
        })
        .collect();
    let height = draw_chips(ui, area, &chips);
    area.cut_top(height, SPACE_L);
    let mut fields = vec![
        ("Last motion", format!("({:+}, {:+})", view.delta[0], view.delta[1])),
        ("Total motion", format!("({}, {})", state.positionX, state.positionY)),
        ("Wheel total", format!("({}, {})", state.wheelX, state.wheelY)),
    ];
    if state.positions & GameInputMouseAbsolutePosition != 0 {
        fields.push(("Absolute", format!("({}, {})", state.absolutePositionX, state.absolutePositionY)));
    }
    for (label, value) in fields {
        ui.field(area.cut_top(BODY.line_height, SPACE_XS), label, MOUSE_LABEL, &value);
    }
}

// The Devices card: a row for each connected device, which shows a tooltip
// of everything known about it while the pointer rests on it. The gamepad
// that the Gamepad card shows is in the accent color, and the gamepads that
// the demo ignores are faint.
fn devices(ui: &mut Ui, input: &Input, rect: Rect) {
    let card = ui.card(rect, "Devices");
    let theme = ui.theme().clone();
    let count = format!("{} connected", input.devices.len());
    ui.text_aligned(card.header, &count, CAPTION, theme.text_secondary, Align::End);
    let content = card.content;
    if input.devices.is_empty() {
        ui.text(content.top_strip(BODY.line_height), "None yet", BODY, theme.text_tertiary);
        return;
    }
    // The rows reach nearer the card's edges than its content does, like a
    // list's, and their text lines up with the content.
    let mut rows = content.inset_by(-SPACE_S, 0.0, -SPACE_S, 0.0);
    // If they don't all fit, the last row says how many more there are.
    let fits = (rows.height / DEVICE_ROW).floor() as usize;
    let shown = if input.devices.len() > fits { fits.saturating_sub(1) } else { input.devices.len() };
    for (index, device) in input.devices.iter().take(shown).enumerate() {
        let row = rows.cut_top(DEVICE_ROW, 0.0);
        let name = format!("{DEVICE_ROWS}/{index}");
        let response = ui.interact(&name, row, Sense::NONE, "DeviceRow");
        ui.set_label(&name, &device.label);
        let fill = if response.hovered { theme.subtle_fill_secondary } else { Color::TRANSPARENT };
        let fill = ui.animate_color(response.id, 0, fill, DURATION_FASTER);
        let listened = input.gamepad.device == Some(device.id);
        let (style, color, icon_color) = if listened {
            (BODY_STRONG, theme.accent_text, theme.accent_text)
        } else if !input.listens_to(device) {
            (BODY, theme.text_tertiary, theme.text_tertiary)
        } else {
            (BODY, theme.text_primary, theme.text_secondary)
        };
        let mut details = format!("{}: {}", device.family_name(), device.kinds());
        if let Some(index) = device.xinput_index {
            details += &format!(", XInput #{index}");
        }
        if device.rumble_motors != 0 {
            details += ", rumble";
        }

        let background = row.inset_by(0.0, 2.0, 0.0, 2.0);
        let inner = row.inset_by(SPACE_S, 0.0, SPACE_S, 0.0);
        let mut painter = ui.painter();
        painter.shape(background, Corners::all(RADIUS_CONTROL), fill, None);
        if listened {
            let pill = Rect::new(
                background.x,
                (background.center().y - SELECTION_PILL_LENGTH / 2.0).round(),
                SELECTION_PILL_THICKNESS,
                SELECTION_PILL_LENGTH,
            );
            painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
        }
        painter.icon(Point::new(inner.x + ICON / 2.0, row.center().y), device_icon(device), ICON, icon_color);
        let details_width = painter.measure(&details, CAPTION).width.ceil();
        let label_area = inner.inset_by(ICON + SPACE_M, 0.0, details_width + SPACE_L, 0.0);
        painter.text_fit(label_area, &device.label, style, color, Align::Start);
        painter.text_in(inner, &details, CAPTION, theme.text_secondary, Align::End);
        if response.hovered {
            device_tooltip(ui, &name, device);
        }
    }
    if shown < input.devices.len() {
        let more = format!("and {} more", input.devices.len() - shown);
        ui.text(rows.top_strip(DEVICE_ROW).inset_by(SPACE_S, 0.0, SPACE_S, 0.0), &more, BODY, theme.text_secondary);
    }
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

// A line of a device's tooltip, before it borrows its text.
enum Line {
    Heading(String),
    Pair(String, String),
    Separator,
}

// Everything that GameInput and Windows say about a device, in a tooltip on
// its row, in three parts: what GameInput says, Windows' node for it, and
// what that's connected through. Long values, like the device's path, wrap.
fn device_tooltip(ui: &mut Ui, name: &str, device: &Device) {
    let mut lines = vec![Line::Heading(device.label.clone())];
    for section in device.details() {
        lines.push(Line::Separator);
        lines.push(Line::Heading(section.title));
        for (label, value) in section.rows {
            let mut label = Some(label);
            for part in value.split('\n') {
                for line in wrap(ui, part, CAPTION, TOOLTIP_VALUE_WIDTH) {
                    lines.push(Line::Pair(label.take().unwrap_or_default(), line));
                }
            }
        }
    }
    let lines: Vec<TooltipLine> = lines
        .iter()
        .map(|line| match line {
            Line::Heading(text) => TooltipLine::Heading(text),
            Line::Pair(label, value) => TooltipLine::Pair(label, value),
            Line::Separator => TooltipLine::Separator,
        })
        .collect();
    ui.tooltip_lines(name, &lines);
}

// The text in lines no wider than `width`, broken at spaces, or where it must
// be in a word that's wider on its own, like a device's path.
fn wrap(ui: &mut Ui, text: &str, style: TextStyle, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_inclusive(' ') {
        let joined = format!("{line}{word}");
        if ui.measure(joined.trim_end(), style).width <= width {
            line = joined;
            continue;
        }
        if !line.trim_end().is_empty() {
            lines.push(line.trim_end().to_string());
        }
        let mut rest = word;
        loop {
            let ends: Vec<usize> = rest.char_indices().skip(1).map(|(index, _)| index).chain([rest.len()]).collect();
            let fitting = ends.partition_point(|&end| ui.measure(rest[..end].trim_end(), style).width <= width);
            if fitting == ends.len() {
                break;
            }
            // At least a character a line.
            let end = ends[fitting.saturating_sub(1)];
            lines.push(rest[..end].to_string());
            rest = &rest[end..];
        }
        line = rest.to_string();
    }
    if !line.trim_end().is_empty() || lines.is_empty() {
        lines.push(line.trim_end().to_string());
    }
    lines
}

// The Event log card: everything above as it happens, newest first, in
// Cascadia Mono, so that its columns line up: when, where from, and what.
fn log(ui: &mut Ui, input: &Input, rect: Rect) {
    let card = ui.card(rect, "Event log");
    let theme = ui.theme().clone();
    let colors = Palette::of(&theme);
    let note = "Newest first   ·   a run of motion from one device shares a line";
    ui.text_aligned(card.header, note, CAPTION, theme.text_secondary, Align::End);
    let mut area = card.content;
    if input.log.is_empty() {
        ui.text(area.top_strip(BODY.line_height), "Nothing yet", BODY, theme.text_tertiary);
        return;
    }
    let mut painter = ui.painter();
    for entry in &input.log {
        if area.height < MONO.line_height {
            break;
        }
        let row = area.cut_top(MONO.line_height, 0.0);
        let line = &entry.line;
        let mut x = row.x;
        x += painter.text(Point::new(x, row.y), &line[..entry.source_at], MONO, theme.text_secondary).width;
        x += painter.text(Point::new(x, row.y), &line[entry.source_at..entry.text_at], MONO, colors.source(entry.source)).width;
        let text = Rect::new(x, row.y, (row.right() - x).max(0.0), row.height);
        painter.text_fit(text, &line[entry.text_at..], MONO, theme.text_primary, Align::Start);
    }
}
