// Describes the demo's cards in fixed pixels from what the demo read, with
// Library/UserInterface's widgets and canvases: the app bar, the status bar,
// and between them three columns of cards. The first has the Gamepad card
// over the Keyboard and Mouse cards, the second the Latest input event card
// over the Devices card, and the third the Event log. The demo supports two
// window setups, maximized and borderless fullscreen (see
// Library/UserInterface's README), whose client areas differ only in
// height, so the cards have fixed places in pixels. The 71 pixels that
// fullscreen adds go to the bottom row of cards and to the log.
//
// The combo box and the Devices card's rows are the library's widgets, which
// read the mouse's buttons from GameInput, and the pointer from where Windows'
// cursor is in the window. Everything else that the cards show comes from the
// devices' readings.

use game_input::*;
use user_interface::controls::{
    INFO_BADGE_HEIGHT, LIST_ITEM_BACKPLATE_MARGIN, LIST_ITEM_HEIGHT, SCROLL_BAR_SIZE, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS,
};
use user_interface::tokens::{
    BODY, BODY_STRONG, CAPTION, CARD_GAP, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, ICON, MONO, RADIUS_CONTROL,
    SPACE_L, SPACE_M, SPACE_S, SPACE_XS, SPACE_XXL,
};
use user_interface::{
    Align, AppBar, Badge, Border, Canvas, Card, Color, ComboBox, Divider, Field, Icon, IconView, Insets, KEYCAP_HEIGHT, Keycaps, ListView,
    Painter, Point, Rect, ScrollArea, Severity, Size, Span, Status, StatusIcon, Text, TextColor, Theme, Tooltip, TooltipLine, Tree, Ui,
};

use crate::NAME;
use crate::demo::Message;
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
// Taller than any card's details, to measure them in.
const UNBOUNDED: f32 = 100_000.0;

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

// The app bar, the status bar and the cards, from what the demo read.
pub fn view(ui: &mut Ui, input: &Result<Input, String>) -> Tree<Message> {
    let mut tree = Tree::new();
    // No widget takes keyboard focus, so that the keys stay the user's, to
    // try the Keyboard card with. The combo box's list still takes the
    // arrow keys while it's open.
    tree.set_focusable(false);
    let status = status_items(ui, input);
    let shell = tree.shell(
        ui,
        AppBar::new(Icon::Gamepad, NAME).subtitle("Gamepads, keyboards and mice, as GameInput reads them"),
        false,
        Some(status),
    );
    app_bar_status(ui, &mut tree, shell.commands);
    // Without GameInput, a notice says why, and there's nothing to show.
    let Ok(input) = input else {
        return tree;
    };
    let mut rest = shell.page;
    let mut first = rest.cut_left(GAMEPAD_COLUMN, CARD_GAP);
    let mut second = rest.cut_left(LATEST_COLUMN, CARD_GAP);
    let height = gamepad_height(ui);
    gamepad(ui, &mut tree, input, first.cut_top(height, CARD_GAP));
    let [keyboard_area, mouse_area] = first.columns::<2>(CARD_GAP);
    keyboard(ui, &mut tree, input, keyboard_area);
    mouse(ui, &mut tree, input, mouse_area);
    latest(ui, &mut tree, input, second.cut_top(height, CARD_GAP));
    devices(ui, &mut tree, input, second);
    log(ui, &mut tree, input, rest);
    tree
}

// After the app bar's title: whether the window has focus, which the demo
// needs to read input, and the frame rate.
fn app_bar_status(ui: &mut Ui, tree: &mut Tree<Message>, mut bar: Rect) {
    let stats = ui.stats();
    let rate = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let width = ui.measure(&rate, CAPTION).width.ceil();
    tree.draw(bar.cut_right(width, SPACE_XXL), Text::caption(rate));

    let (status, severity) = if ui.window_focused() {
        ("Focused: reading input", Severity::Success)
    } else {
        ("Not focused: input is ignored", Severity::Warning)
    };
    let width = ICON + SPACE_S + ui.measure(status, CAPTION).width.ceil();
    let mut area = bar.cut_right(width, SPACE_XXL);
    let icon = area.cut_left(ICON, SPACE_S);
    tree.draw(icon, StatusIcon::new(severity, ICON));
    tree.draw(area, Text::label(status).style(CAPTION));
}

// The status bar: the shortcuts, the GameInput runtime, how many readings
// the demo handled, and the display that the window is on. A runtime deep
// in folders is cut short where it meets the items on the right.
fn status_items(ui: &Ui, input: &Result<Input, String>) -> Vec<Status> {
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
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::text(runtime),
        Status::Spacer,
    ];
    for text in [readings, display] {
        if !text.is_empty() {
            items.push(Status::text(text));
        }
    }
    items
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
fn gamepad(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let view = &input.gamepad;
    let state = view.state;
    let rumble_dead_zone = input.rumble_dead_zone;
    let rumble = if input.rumble_enabled { "on" } else { "off" };
    let status_line = format!(
        "Readings: {:.0}/s over the last 2 s   ·   Rumble test: {rumble} (F1)   ·   Motors: {}",
        view.rate.per_second(),
        rumble_motor_names(view.rumble_motors)
    );
    let chips: Vec<Chip> = GAMEPAD_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            held: state.buttons & button != 0,
            present: view.layout.is_none_or(|layout| layout & button != 0),
        })
        .collect();

    tree.card(ui, None, rect, Card::new("Gamepad"), |ui, card, area| {
        gamepad_combo_box(ui, card, input, area.header);
        let mut content = area.content;
        let captions = content.cut_top(BODY.line_height, SPACE_S);
        let mut plots = content.cut_top(STICK_GRAPH, SPACE_S);
        let values = content.cut_top(MONO.line_height * 2.0, SPACE_L);
        let status = content.cut_bottom(CAPTION.line_height, SPACE_L);
        let left = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
        let right = plots.cut_left(STICK_GRAPH, GRAPH_GAP);
        let left_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);
        let right_trigger = plots.cut_left(TRIGGER_BAR, GRAPH_GAP);
        let sticks = [
            [state.leftThumbstickX, state.leftThumbstickY],
            [state.rightThumbstickX, state.rightThumbstickY],
        ];
        card.draw(
            area.content,
            Canvas::new(move |painter, _| {
                let theme = painter.theme().clone();
                let colors = Palette::of(&theme);
                for (column, caption, color) in [
                    (left, "Left stick", colors.left),
                    (right, "Right stick", colors.right),
                    (left_trigger, "LT", colors.left),
                    (right_trigger, "RT", colors.right),
                ] {
                    let area = Rect::new(column.x, captions.y, column.width + GRAPH_GAP, captions.height);
                    legend(painter, &theme, area, caption, color);
                }
                stick_graph(painter, &theme, left, sticks[0], rumble_dead_zone, colors.left);
                stick_graph(painter, &theme, right, sticks[1], rumble_dead_zone, colors.right);
                trigger_bar(painter, &theme, left_trigger, state.leftTrigger, colors.left);
                trigger_bar(painter, &theme, right_trigger, state.rightTrigger, colors.right);

                // The raw values, in Cascadia Mono, so that their digits
                // stay put.
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
            }),
        );
        // A chip for each button. Those that the gamepad says it lacks are
        // faint.
        draw_chips(ui, card, content, chips);
        card.draw(status, Text::caption(status_line));
    });
}

// The combo box in the Gamepad card's header, which shows which gamepad the
// demo listens to, always a connected one, and lists each connected gamepad,
// and none, to pick another. After it, a note on what the card shows, if it
// isn't a gamepad.
fn gamepad_combo_box(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, header: Rect) {
    let mut choices: Vec<Option<Device>> = input.gamepads().cloned().map(Some).collect();
    choices.push(None);
    let labels: Vec<&str> = choices.iter().map(|choice| choice.as_ref().map_or(NO_GAMEPAD, |device| device.label.as_str())).collect();
    let chosen = match &input.listen {
        Listen::Gamepad(device) => Some(device.id),
        Listen::NoGamepad => None,
    };
    let selected = choices.iter().position(|choice| choice.as_ref().map(|device| device.id) == chosen).unwrap_or(0);
    let width = ComboBox::<Message>::width(ui, &labels).clamp(GAMEPAD_COMBO_BOX_MIN_WIDTH, GAMEPAD_COMBO_BOX_MAX_WIDTH).min(header.width);
    let rect = Rect::new(header.x, header.y, width, CONTROL_HEIGHT);
    let combo = ComboBox::new(selected, &labels).on_select(move |index| {
        Message::Listen(match &choices[index] {
            Some(device) => Listen::Gamepad(device.clone()),
            None => Listen::NoGamepad,
        })
    });
    tree.add(GAMEPAD_COMBO_BOX, rect, combo);

    // Unless none is picked, the demo switches to a gamepad as soon as one
    // connects, so it has none only while none is connected.
    let (note, caution) = match &input.listen {
        Listen::Gamepad(_) => ("", false),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected", true),
        Listen::NoGamepad => ("Gamepad input is ignored", false),
    };
    let mut area = Rect::new(rect.right() + SPACE_M, header.y, (header.right() - rect.right() - SPACE_M).max(0.0), header.height);
    if caution {
        let icon = area.cut_left(ICON, SPACE_S);
        tree.draw(icon, StatusIcon::new(Severity::Warning, ICON));
    }
    let text = if caution { Text::label(note) } else { Text::label(note).secondary() };
    tree.draw(area, text);
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

// A button's chip, which is in the accent color while it's held.
#[derive(Clone, Copy)]
struct Chip {
    label: &'static str,
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
fn draw_chips(ui: &mut Ui, tree: &mut Tree<Message>, area: Rect, chips: Vec<Chip>) -> f32 {
    let labels: Vec<&str> = chips.iter().map(|chip| chip.label).collect();
    let rects = chip_rects(ui, area, &labels);
    let height = rects.last().map_or(0.0, |rect| rect.bottom() - area.y);
    let placed: Vec<(Rect, Chip)> = rects.into_iter().zip(chips).collect();
    tree.draw(
        area,
        Canvas::new(move |painter, _| {
            let theme = painter.theme().clone();
            for (rect, chip) in &placed {
                let (fill, stroke, text) = if chip.held {
                    (theme.accent, theme.control_stroke_on_accent, theme.text_on_accent)
                } else if chip.present {
                    (theme.control_alt_fill_secondary, theme.control_stroke, theme.text_secondary)
                } else {
                    (Color::TRANSPARENT, theme.control_stroke, theme.text_disabled)
                };
                painter.shape(*rect, RADIUS_CONTROL, fill, Some(Border::new(stroke, 1.0)));
                painter.text_in(*rect, chip.label, CAPTION, text, Align::Center);
            }
        }),
    );
    height
}

// The Latest input event card: the newest reading that changed something,
// described in full. Details that don't fit scroll.
fn latest(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let time = input.latest.as_ref().map(|event| format!("at {:.6} s", event.time));
    let card = match time {
        Some(time) => Card::new("Latest input event").subtitle(time),
        None => Card::new("Latest input event"),
    };
    tree.card(ui, None, rect, card, |ui, card, area| {
        let area = area.content;
        let Some(event) = &input.latest else {
            card.draw(area, Text::paragraph("Waiting for input. Press a key, move the mouse, or use a gamepad.").secondary());
            return;
        };
        let mut height = latest_details(ui, event, Rect::new(area.x, area.y, area.width, UNBOUNDED), None);
        if height > area.height {
            let width = area.width - SCROLL_BAR_SIZE;
            height = latest_details(ui, event, Rect::new(area.x, area.y, width, UNBOUNDED), None);
        }
        let content = ScrollArea::content(ui, LATEST_DETAILS, area, height);
        card.container(Some(LATEST_DETAILS), area, ScrollArea::new(height), |scroll| {
            latest_details(ui, event, content, Some(scroll));
        });
    });
}

// Lays the event's details out from the top of the area, into the tree if
// one is given, so that they can be measured first. Returns how tall they
// are.
fn latest_details(ui: &mut Ui, event: &LatestEvent, area: Rect, mut tree: Option<&mut Tree<Message>>) -> f32 {
    let mut rest = area;

    let mut heading = rest.cut_top(BODY.line_height, SPACE_S);
    if let Some(tree) = tree.as_deref_mut() {
        let icon = heading.cut_left(ICON, SPACE_S);
        let color = Palette::of(ui.theme()).source(event.source);
        tree.draw(icon, IconView::new(source_icon(event.source), ICON).color(TextColor::Custom(color)));
        tree.draw(heading, Text::heading(format!("{} reading #{}", event.source.name(), event.number)));
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
        if let Some(tree) = tree.as_deref_mut() {
            tree.draw(row, Field::new(label, LATEST_LABEL, value));
        }
    }
    if interval.is_none() {
        let text = "No earlier reading from this device, so this is compared with an idle state.";
        let height = ui.paragraph_height(rest.width, text, BODY);
        let row = rest.cut_top(height, SPACE_XS);
        if let Some(tree) = tree.as_deref_mut() {
            tree.draw(row, Text::paragraph(text).secondary());
        }
    }

    let divider = rest.cut_top(SPACE_L, SPACE_XS);
    let title = rest.cut_top(BODY.line_height, SPACE_XS);
    if let Some(tree) = tree.as_deref_mut() {
        tree.draw(divider.bottom_strip(1.0), Divider);
        tree.draw(title, Text::heading("Changed"));
    }
    for change in &event.changes {
        // A bullet, and the change after it, which may wrap.
        let text = rest.pad(Insets::new(SPACE_XS + SPACE_M, 0.0, 0.0, 0.0));
        let height = ui.paragraph_height(text.width, change, BODY);
        let row = rest.cut_top(height, SPACE_XS);
        if let Some(tree) = tree.as_deref_mut() {
            tree.draw(Rect::new(row.x + SPACE_XS, row.y, SPACE_M, BODY.line_height), Text::label("•").secondary());
            tree.draw(Rect::new(text.x, row.y, text.width, height), Text::paragraph(change));
        }
    }

    let divider = rest.cut_top(SPACE_L, SPACE_XS);
    let title = rest.cut_top(BODY.line_height, SPACE_XS);
    if let Some(tree) = tree.as_deref_mut() {
        tree.draw(divider.bottom_strip(1.0), Divider);
        tree.draw(title, Text::heading("State now"));
    }
    for line in &event.state {
        let row = rest.cut_top(MONO.line_height, 0.0);
        if let Some(tree) = tree.as_deref_mut() {
            tree.draw(row, Text::label(line).style(MONO));
        }
    }
    rest.y - area.y
}

// The Keyboard card: the keys held on the keyboard that last changed them,
// each with its codes, and the character it types. Keys that don't fit
// scroll.
fn keyboard(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let view = &input.keyboard;
    tree.card(ui, None, rect, Card::new("Keyboard").subtitle(&view.label), |ui, card, area| {
        let mut area = area.content;
        let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
        card.draw(status, Text::caption(format!("Readings: {}/s", view.rate.per_second)));

        let heading = area.cut_top(BODY.line_height, SPACE_S);
        let title_width = ui.measure("Keys down", BODY_STRONG).width;
        card.draw(heading, Text::heading("Keys down"));
        if view.keys.is_empty() {
            card.draw(area.top_strip(BODY.line_height), Text::label("None").tertiary());
            return;
        }
        // The badge is centered on its point, and a count of one digit is
        // round.
        let center = Point::new(heading.x + title_width + SPACE_S + INFO_BADGE_HEIGHT / 2.0, heading.center().y);
        card.draw(Rect::from_center(center, Size::square(INFO_BADGE_HEIGHT * 2.0)), Badge::new(view.keys.len().to_string()));
        let height = view.descriptions.len() as f32 * KEY_ROW;
        let mut rows = ScrollArea::content(ui, KEY_ROWS, area, height);
        card.container(Some(KEY_ROWS), area, ScrollArea::new(height), |scroll| {
            for (name, details) in &view.descriptions {
                let row = rows.cut_top(KEY_ROW, 0.0);
                let cap_top = row.y + ((KEY_ROW - KEYCAP_HEIGHT) / 2.0).round();
                scroll.draw(Rect::new(row.x, cap_top, KEY_NAME_COLUMN, KEYCAP_HEIGHT), Keycaps::new(name));
                scroll.draw(row.pad(Insets::new(KEY_NAME_COLUMN, 0.0, 0.0, 0.0)), Text::label(details).style(MONO));
            }
        });
    });
}

// The Mouse card: the buttons held on the mouse that last changed, its
// motion and wheel, and a pad that moves with its raw motion, which has no
// pointer acceleration.
fn mouse(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let view = &input.mouse;
    let state = view.state;
    let pad_value = [view.pad[0], -view.pad[1]];
    tree.card(ui, None, rect, Card::new("Mouse").subtitle(&view.label), |ui, card, area| {
        let mut area = area.content;
        let status = area.cut_bottom(CAPTION.line_height, SPACE_M);
        let mut readings = format!("Readings: {}/s", view.rate.per_second);
        if let Some(rate) = view.sample_rate {
            readings += &format!("   ·   Device reports {rate} Hz");
        }
        card.draw(status, Text::caption(readings));

        let mut pad_column = area.cut_right(MOUSE_PAD, SPACE_L);
        let caption = pad_column.cut_top(BODY.line_height, SPACE_S);
        let pad = pad_column.top_strip(MOUSE_PAD);
        card.draw(
            Rect::new(caption.x, caption.y, caption.width, pad.bottom() - caption.y),
            Canvas::new(move |painter, _| {
                let theme = painter.theme().clone();
                let colors = Palette::of(&theme);
                legend(painter, &theme, caption, "Raw motion", colors.mouse);
                stick_graph(painter, &theme, pad, pad_value, 0.0, colors.mouse);
            }),
        );

        let heading = area.cut_top(BODY.line_height, SPACE_S);
        card.draw(heading, Text::heading("Buttons"));
        let chips: Vec<Chip> = MOUSE_BUTTONS
            .iter()
            .map(|&(button, label)| Chip {
                label,
                held: state.buttons & button != 0,
                present: true,
            })
            .collect();
        let height = draw_chips(ui, card, area, chips);
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
            card.draw(area.cut_top(BODY.line_height, SPACE_XS), Field::new(label, MOUSE_LABEL, value));
        }
    });
}

// The Devices card: a list with a row for each connected device, which
// shows a tooltip of everything known about it while the pointer rests on
// it, and scrolls if they don't all fit. The gamepad that the Gamepad card
// shows is in the accent color, with the pill of a selection, and the
// gamepads that the demo ignores are faint.
fn devices(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let count = format!("{} connected", input.devices.len());
    tree.card(ui, None, rect, Card::new("Devices").subtitle(count), |ui, card, area| {
        if input.devices.is_empty() {
            card.draw(area.content.top_strip(BODY.line_height), Text::label("None yet").tertiary());
            return;
        }
        let labels: Vec<String> = input.devices.iter().map(|device| device.label.clone()).collect();
        let tooltip_devices = input.devices.clone();
        let list = ListView::new(input.devices.len(), LIST_ITEM_HEIGHT)
            .labels(move |index| labels[index].clone())
            .tooltips(move |index| Some(device_tooltip(&tooltip_devices[index])));
        let rows = list.rows(ui, DEVICE_ROWS, area.content);
        card.container(Some(DEVICE_ROWS), area.content, list, |items| {
            for index in rows.indices() {
                device_row(ui, items, input, &input.devices[index], rows.row(index), rows.content(index));
            }
        });
    });
}

// A device's row in the list: an icon for what it is, its label, and its
// family and kinds of input, in its content area.
fn device_row(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, device: &Device, row: Rect, content: Rect) {
    let theme = ui.theme();
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

    if listened {
        // The list's selection pill, on its row's backplate, though the
        // combo box picks the gamepad, and not the list.
        let plate = row.pad(LIST_ITEM_BACKPLATE_MARGIN);
        let pill = Rect::new(plate.x, plate.center().y - SELECTION_PILL_LENGTH / 2.0, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
        let accent = theme.accent;
        tree.draw(pill, Canvas::new(move |painter, pill| painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, accent)));
    }
    tree.draw(
        Rect::new(content.x, content.y, ICON, content.height),
        IconView::new(device_icon(device), ICON).color(TextColor::Custom(icon_color)),
    );
    let details_width = ui.measure(&details, CAPTION).width.ceil();
    let label_area = content.pad(Insets::new(ICON + SPACE_M, 0.0, details_width + SPACE_L, 0.0));
    tree.draw(label_area, Text::label(&device.label).style(style).color(TextColor::Custom(color)));
    tree.draw(content, Text::caption(details).end());
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
fn device_tooltip(device: &Device) -> Tooltip {
    let mut lines = vec![TooltipLine::heading(&device.label)];
    for section in device.details() {
        lines.push(TooltipLine::Separator);
        lines.push(TooltipLine::heading(section.title));
        for (label, value) in section.rows {
            lines.push(TooltipLine::Pair(label, value));
        }
    }
    Tooltip {
        lines,
        max_width: DEVICE_TOOLTIP_WIDTH,
    }
}

// The Event log card: everything above as it happens, newest first, in
// Cascadia Mono, so that its columns line up: when, where from, and what.
fn log(ui: &mut Ui, tree: &mut Tree<Message>, input: &Input, rect: Rect) {
    let entries: Vec<(String, usize, usize, Source)> = input
        .log
        .iter()
        .map(|entry| (entry.line.clone(), entry.source_at, entry.text_at, entry.source))
        .collect();
    tree.card(ui, None, rect, Card::new("Event log").subtitle("Newest first   ·   a run of motion from one device shares a line"), |_, card, area| {
        let area = area.content;
        if entries.is_empty() {
            card.draw(area.top_strip(BODY.line_height), Text::label("Nothing yet").tertiary());
            return;
        }
        card.draw(
            area,
            Canvas::new(move |painter, _| {
                let theme = painter.theme().clone();
                let colors = Palette::of(&theme);
                let mut area = area;
                for (line, source_at, text_at, source) in &entries {
                    if area.height < MONO.line_height {
                        break;
                    }
                    let row = area.cut_top(MONO.line_height, 0.0);
                    // The time and the source, then what happened, cut short
                    // to fit.
                    let lead = painter.measure(&line[..*text_at], MONO).width;
                    let text = painter.fit(&line[*text_at..], MONO, (row.width - lead).max(0.0)).into_owned();
                    let spans = [
                        Span::new(&line[..*source_at], MONO, theme.text_secondary),
                        Span::new(&line[*source_at..*text_at], MONO, colors.source(*source)),
                        Span::new(&text, MONO, theme.text_primary),
                    ];
                    painter.spans(row.origin(), &spans);
                }
            }),
        );
    });
}
