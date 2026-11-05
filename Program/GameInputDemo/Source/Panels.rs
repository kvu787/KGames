// Lays out and draws the demo's cards with Library/UserInterface, and
// handles what the mouse does to them: the combo box that picks which
// gamepad the demo listens to, and the Devices card's rows, each of which
// shows a tooltip about its device while the cursor is over it. The demo only
// supports two client areas, 2560 x 1369 when maximized and 2560 x 1440 in
// borderless fullscreen (see Library/Application), so the cards have fixed
// places in pixels. The bottom row and the log are taller in fullscreen, and
// the status bar stays at the bottom.
//
// The widgets read the mouse from the window's messages, not from GameInput,
// because they say where the cursor is in the window. The stick graphs, the
// trigger bars, the mouse pad, and the cards' text are drawn here.

use game_input::*;
use user_interface::{
    Align, CARD_PADDING, Canvas, Chip, Choice, Color, Frame, Lines, MARGIN, PANELS_TOP, RADIUS_CONTROL, Rect,
    SPACE_M, SPACE_S, SPACE_XS, Sense, Stack, TextStyle, Theme, TooltipText, card, chips_height, select_width,
    status_chips, well,
};

use crate::devices::{Device, rumble_motor_names};
use crate::input::{GAMEPAD_BUTTONS, Input, Listen, MOUSE_BUTTONS, Source};

// The layout, in pixels. Three columns: the gamepad over the keyboard and
// mouse, the latest event over the devices, and the log. MARGIN and
// PANELS_TOP come from Library/UserInterface, which all programs share.
const GAMEPAD_X: f32 = MARGIN;
const GAMEPAD_WIDTH: f32 = 980.0;
const LATEST_X: f32 = GAMEPAD_X + GAMEPAD_WIDTH + MARGIN;
const LATEST_WIDTH: f32 = 640.0;
const LOG_X: f32 = LATEST_X + LATEST_WIDTH + MARGIN;
const KEYBOARD_WIDTH: f32 = (GAMEPAD_WIDTH - MARGIN) / 2.0;
const MOUSE_X: f32 = GAMEPAD_X + KEYBOARD_WIDTH + MARGIN;
const MOUSE_WIDTH: f32 = GAMEPAD_WIDTH - KEYBOARD_WIDTH - MARGIN;
// The stick graphs are square, and the trigger bars as tall.
const STICK_GRAPH_SIZE: f32 = 400.0;
const TRIGGER_BAR_WIDTH: f32 = 40.0;
const GAMEPAD_GAP: f32 = 24.0;
const MOUSE_PAD_SIZE: f32 = 200.0;
// A line of text, and a row of the Devices card.
const LINE: f32 = 20.0;
const DEVICE_ROW: f32 = 28.0;

// The combo box for which gamepad to listen to. Its list is "Gamepad/List",
// and its choices "Gamepad/0", "Gamepad/1" and so on.
pub const GAMEPAD_SELECT: &str = "Gamepad";
// The Devices card's rows are "Devices/0", "Devices/1" and so on, in the
// card's order. Two devices can have the same label, so their rows are named
// by their place, and their tooltips by their device's label.
pub const DEVICE_ROWS: &str = "Devices";

// A tooltip's values wrap at this width. Words wider than it stay whole.
const VALUE_WIDTH: f32 = 720.0;

// What the colors mean: the left and right sticks and triggers, what's held,
// and each source of input.
struct Palette {
    left: Color,
    right: Color,
    pressed: Color,
    keyboard: Color,
    mouse: Color,
}

fn palette(theme: &Theme) -> Palette {
    Palette {
        left: theme.blue(),
        right: theme.orange(),
        pressed: theme.success,
        keyboard: theme.purple(),
        mouse: theme.teal(),
    }
}

// Draws the app bar, the status bar, and the cards, and changes which
// gamepad the demo listens to when one is picked in the combo box.
pub fn draw(frame: &mut Frame, input: &mut Result<Input, String>) {
    // No widget takes keyboard focus: the keys are the user's, to try the
    // keyboard card with, and the demo's shortcuts come through GameInput.
    frame.set_focusable(false);
    let theme = frame.theme();
    frame.header("GameInput Demo", "Gamepad, keyboard and mouse input with the GameInput v4 API");
    let bottom = frame.footer_with(
        &[("F1", "rumble test"), ("F2", "clear the log"), ("F11", "fullscreen")],
        |canvas, area| draw_status(canvas, theme, area, input),
    );
    let Ok(input) = input else {
        return;
    };

    // The gamepad card is as tall as its contents: captions, the graphs, two
    // lines of values, the chips and a status line. The latest event card
    // beside it is as tall. The rows under them, and the log, go down to the
    // status bar.
    let gamepad_names: Vec<&str> = GAMEPAD_BUTTONS.iter().map(|(_, name)| *name).collect();
    let chip_rows = chips_height(frame.fonts(), GAMEPAD_WIDTH - CARD_PADDING * 2.0, &gamepad_names);
    let gamepad_height = user_interface::CARD_HEADER
        + LINE
        + SPACE_XS
        + STICK_GRAPH_SIZE
        + SPACE_S
        + LINE * 2.0
        + SPACE_M
        + chip_rows
        + SPACE_M
        + LINE
        + CARD_PADDING;
    let gamepad_area = Rect::new(GAMEPAD_X, PANELS_TOP, GAMEPAD_WIDTH, gamepad_height);
    let lower_y = gamepad_area.bottom() + MARGIN;
    let keyboard_area = Rect::from_edges(GAMEPAD_X, lower_y, GAMEPAD_X + KEYBOARD_WIDTH, bottom);
    let mouse_area = Rect::from_edges(MOUSE_X, lower_y, MOUSE_X + MOUSE_WIDTH, bottom);
    let latest_area = Rect::new(LATEST_X, PANELS_TOP, LATEST_WIDTH, gamepad_height);
    let devices_area = Rect::from_edges(LATEST_X, lower_y, LATEST_X + LATEST_WIDTH, bottom);
    let log_area = Rect::from_edges(LOG_X, PANELS_TOP, frame.width - MARGIN, bottom);

    draw_gamepad(frame, input, gamepad_area);
    draw_latest(&mut frame.canvas, theme, input, latest_area);
    draw_keyboard(&mut frame.canvas, theme, input, keyboard_area);
    draw_mouse(&mut frame.canvas, theme, input, mouse_area);
    draw_devices(frame, input, devices_area);
    draw_log(&mut frame.canvas, theme, input, log_area);
}

// What the status bar says about the demo: the GameInput runtime, where it
// was loaded from, cut short to fit, and how many readings the demo handled,
// on the right of the area.
fn draw_status(canvas: &mut Canvas, theme: &Theme, area: Rect, input: &Result<Input, String>) {
    let style = TextStyle::Caption;
    match input {
        Ok(input) => {
            let (runtime, location) = match input.runtime() {
                Some(runtime) => {
                    let version = runtime.version.map(|part| part.to_string()).join(".");
                    let location = runtime
                        .path
                        .parent()
                        .map(|path| format!(" in {}", path.display()))
                        .unwrap_or_default();
                    (format!("{} {}", file_name(&runtime.path), version), location)
                }
                None => ("none".to_string(), ", the input comes from a script".to_string()),
            };
            let count = format!("   Readings handled: {}", input.reading_count);
            let fixed = canvas.measure("Runtime: ", style) + canvas.measure(&runtime, style) + canvas.measure(&count, style);
            let location = canvas.fit(&location, style, (area.width - fixed).max(0.0));
            let total = fixed + canvas.measure(&location, style);
            let mut x = (area.right() - total).max(area.x);
            for (text, color) in [
                ("Runtime: ", theme.text_secondary),
                (runtime.as_str(), theme.text),
                (location.as_str(), theme.text_secondary),
                (count.as_str(), theme.text_secondary),
            ] {
                x = canvas.text([x, area.y], text, style, color);
            }
        }
        Err(message) => {
            let message = format!("GameInput isn't available: {message}");
            let fitted = canvas.fit(&message, style, area.width);
            let x = (area.right() - canvas.measure(&fitted, style)).max(area.x);
            canvas.text([x, area.y], &fitted, style, theme.critical);
        }
    }
}

fn draw_gamepad(frame: &mut Frame, input: &mut Input, area: Rect) {
    let theme = frame.theme();
    let colors = palette(theme);
    let inner = frame.card(area, "Gamepad", "");
    draw_gamepad_select(frame, input, area);
    let canvas = &mut frame.canvas;
    let view = &input.gamepad;
    let state = &view.state;
    let (graph, bar_width, gap) = (STICK_GRAPH_SIZE, TRIGGER_BAR_WIDTH, GAMEPAD_GAP);

    // Stick graphs and trigger bars, with a caption above and values below.
    let plots_y = inner.y + LINE + SPACE_XS;
    let left_x = inner.x;
    let right_x = left_x + graph + gap;
    let left_trigger_x = right_x + graph + gap;
    let right_trigger_x = left_trigger_x + bar_width + gap;
    for (x, caption, color) in [
        (left_x, "Left stick", colors.left),
        (right_x, "Right stick", colors.right),
        (left_trigger_x, "LT", colors.left),
        (right_trigger_x, "RT", colors.right),
    ] {
        canvas.text([x, inner.y], caption, TextStyle::BodyStrong, color);
    }

    let left = [state.leftThumbstickX, state.leftThumbstickY];
    let right = [state.rightThumbstickX, state.rightThumbstickY];
    stick_graph(canvas, theme, Rect::new(left_x, plots_y, graph, graph), left, input.rumble_dead_zone, colors.left);
    stick_graph(canvas, theme, Rect::new(right_x, plots_y, graph, graph), right, input.rumble_dead_zone, colors.right);
    trigger_bar(canvas, theme, Rect::new(left_trigger_x, plots_y, bar_width, graph), state.leftTrigger, colors.left);
    trigger_bar(canvas, theme, Rect::new(right_trigger_x, plots_y, bar_width, graph), state.rightTrigger, colors.right);

    let values_y = plots_y + graph + SPACE_S;
    for (x, value) in [(left_x, left), (right_x, right)] {
        let mut lines = Lines::new(Rect::new(x, values_y, graph + gap, LINE * 2.0));
        lines.line(canvas, &format!("x {:+.3}  y {:+.3}", value[0], value[1]), TextStyle::Mono, theme.text);
        let magnitude = (value[0] * value[0] + value[1] * value[1]).sqrt();
        let degrees = value[1].atan2(value[0]).to_degrees();
        lines.line(canvas, &format!("r {magnitude:.3}  {degrees:+.0}\u{B0}"), TextStyle::Mono, theme.text_secondary);
    }
    canvas.text([left_trigger_x, values_y], &format!("{:.3}", state.leftTrigger), TextStyle::Mono, theme.text);
    canvas.text([right_trigger_x, values_y], &format!("{:.3}", state.rightTrigger), TextStyle::Mono, theme.text);

    // A chip per button. Buttons the gamepad says it lacks are faint.
    let chips_y = values_y + LINE * 2.0 + SPACE_M;
    let chips_area = Rect::from_edges(inner.x, chips_y, inner.right(), inner.bottom() - LINE - SPACE_M);
    let buttons: Vec<_> = GAMEPAD_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            on: state.buttons & button != 0,
            enabled: view.layout.is_none_or(|layout| layout & button != 0),
        })
        .collect();
    status_chips(canvas, theme, chips_area, &buttons, colors.pressed);

    let mut status = format!("Readings: {}/s   Rumble test: ", view.rate.per_second);
    status += if input.rumble_enabled { "on" } else { "off" };
    status += " (F1)   Motors: ";
    status += &rumble_motor_names(view.rumble_motors);
    canvas.text_clipped([inner.x, inner.bottom() - LINE], &status, inner.width, TextStyle::Body, theme.text_secondary);
}

// The combo box after the gamepad card's title, which shows which gamepads
// the demo listens to, and after it, what the card shows. Picking a choice
// in its list listens to that.
fn draw_gamepad_select(frame: &mut Frame, input: &mut Input, area: Rect) {
    let theme = frame.theme();
    let trailing = frame.card_trailing(area, "Gamepad");
    let width = select_width(frame.fonts(), input.listen.label()).max(160.0).min(trailing.width);
    let select_box = Rect::new(trailing.x, trailing.y, width, trailing.height);

    // Each choice's gamepad, or none, and a few words about it: each
    // connected gamepad, a chosen one while it's disconnected, which stays
    // in the list, and no gamepad.
    let connected = match &input.listen {
        Listen::Gamepad(device) => input.gamepads().any(|info| info.id == device.id),
        Listen::NoGamepad => true,
    };
    let mut gamepads: Vec<(Option<&Device>, &str)> =
        input.gamepads().map(|info| (Some(info), info.family_name())).collect();
    if let Listen::Gamepad(device) = &input.listen
        && !connected
    {
        gamepads.push((Some(device), "disconnected"));
    }
    gamepads.push((None, "ignores all gamepads"));
    let none = Listen::NoGamepad;
    let choices: Vec<Choice> = gamepads
        .iter()
        .map(|&(device, detail)| Choice {
            label: device.map_or(none.label(), |device| &device.label),
            detail,
        })
        .collect();
    let chosen = match &input.listen {
        Listen::Gamepad(device) => Some(device.id),
        Listen::NoGamepad => None,
    };
    let mut selected = gamepads
        .iter()
        .position(|(device, _)| device.map(|device| device.id) == chosen)
        .unwrap_or(0);
    let response = frame.select(GAMEPAD_SELECT, select_box, &choices, &mut selected);
    let picked = response.changed.then(|| match gamepads[selected].0 {
        Some(device) => Listen::Gamepad(device.clone()),
        None => Listen::NoGamepad,
    });
    if let Some(listen) = picked {
        input.pick_listen(listen);
    }

    // Until a choice is picked, the demo switches to a gamepad as soon as one
    // connects.
    let (note, color) = match &input.listen {
        Listen::Gamepad(_) if !connected => (
            if input.listen_picked {
                "Disconnected: waiting for it to reconnect"
            } else {
                "Disconnected: waiting for a gamepad to connect"
            },
            theme.caution,
        ),
        Listen::Gamepad(_) => ("", theme.text_secondary),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected", theme.caution),
        Listen::NoGamepad => ("Gamepad input is ignored", theme.text_secondary),
    };
    let note_area = Rect::from_edges(select_box.right() + SPACE_M, trailing.y, trailing.right(), trailing.bottom());
    frame.label(note_area, note, TextStyle::Body, color, Align::Start);
}

// A stick's position in a well, from -1 to 1 across and down, with rings at
// its edge, at half way, and at the dead zone, if it has one.
fn stick_graph(canvas: &mut Canvas, theme: &Theme, area: Rect, value: [f32; 2], dead_zone: f32, color: Color) {
    well(canvas, theme, area);
    let center = area.center();
    let radius = area.width * 0.5 - 8.0;
    // Stick y is up, screen y is down.
    let to_screen = |point: [f32; 2]| [center[0] + point[0] * radius, center[1] - point[1] * radius];

    canvas.rect(Rect::new(area.x + 1.0, center[1].round(), area.width - 2.0, 1.0), theme.divider);
    canvas.rect(Rect::new(center[0].round(), area.y + 1.0, 1.0, area.height - 2.0), theme.divider);
    canvas.ring(center, radius, 1.5, theme.text_tertiary);
    canvas.ring(center, radius * 0.5, 1.0, theme.text_tertiary.alpha(0.5));
    // The mouse pad has no dead zone, and a ring of radius 0 would be a dot.
    if dead_zone > 0.0 {
        canvas.ring(center, radius * dead_zone, 1.0, theme.caution.alpha(0.5));
    }

    let position = to_screen(value);
    canvas.line(center, position, 2.0, color.alpha(0.6));
    canvas.circle(position, 7.0, color);
    canvas.ring(position, 7.0, 2.0, theme.card);
}

// A trigger's value as a bar that fills a well from the bottom.
fn trigger_bar(canvas: &mut Canvas, theme: &Theme, area: Rect, value: f32, color: Color) {
    well(canvas, theme, area);
    for quarter in 1..4 {
        let y = (area.y + area.height * quarter as f32 / 4.0).round();
        canvas.rect(Rect::new(area.x, y, area.width * 0.25, 1.0), theme.text_tertiary);
    }
    let filled = (area.height - 4.0) * value.clamp(0.0, 1.0);
    if filled > 0.0 {
        let bar = Rect::from_edges(area.x + 2.0, area.bottom() - 2.0 - filled, area.right() - 2.0, area.bottom() - 2.0);
        canvas.fill(bar, RADIUS_CONTROL - 1.0, color);
    }
}

fn draw_latest(canvas: &mut Canvas, theme: &Theme, input: &Input, area: Rect) {
    let inner = card(canvas, theme, area, "Latest input event", "");
    // A well that holds the description.
    well(canvas, theme, inner);
    let mut lines = Lines::new(inner.inset_by(SPACE_M, SPACE_S));
    let value_x = 200.0;

    let Some(event) = &input.latest else {
        lines.wrapped(
            canvas,
            "Waiting for input. Press a key, move the mouse, or use a gamepad.",
            TextStyle::Body,
            theme.text_secondary,
            0.0,
        );
        return;
    };
    let heading = format!("{} reading #{}  at {:.6} s", event.source.name(), event.number, event.time);
    lines.line(canvas, &heading, TextStyle::BodyStrong, source_color(theme, event.source));
    lines.gap(SPACE_XS);
    let pair = |lines: &mut Lines, canvas: &mut Canvas, label: &str, value: &str| {
        lines.pair(canvas, label, value, value_x, theme.text_secondary, theme.text);
    };
    pair(&mut lines, canvas, "Device", &event.device);
    pair(&mut lines, canvas, "Reading has", &event.kinds);
    pair(&mut lines, canvas, "Timestamp", &format!("{} \u{B5}s", event.timestamp));
    match event.interval {
        Some(interval) => pair(&mut lines, canvas, "Since its previous reading", &format!("{interval:.3} ms")),
        None => lines.wrapped(
            canvas,
            "No earlier reading from this device, so this is compared with an idle state.",
            TextStyle::Body,
            theme.text_secondary,
            0.0,
        ),
    }
    lines.gap(SPACE_S);
    lines.line(canvas, "Changed", TextStyle::BodyStrong, theme.text);
    for change in &event.changes {
        lines.wrapped(canvas, &format!("\u{2022} {change}"), TextStyle::Body, theme.text, 12.0);
    }
    lines.gap(SPACE_S);
    lines.line(canvas, "State now", TextStyle::BodyStrong, theme.text);
    for state in &event.state {
        lines.wrapped(canvas, state, TextStyle::Body, theme.text, 12.0);
    }
}

fn draw_keyboard(canvas: &mut Canvas, theme: &Theme, input: &Input, area: Rect) {
    let view = &input.keyboard;
    let colors = palette(theme);
    let inner = card(canvas, theme, area, "Keyboard", &view.label);
    let (status, list) = inner.split_bottom(LINE, SPACE_S);
    let mut lines = Lines::new(list);
    lines.line(canvas, &format!("Keys down ({})", view.keys.len()), TextStyle::BodyStrong, theme.text);
    if view.keys.is_empty() {
        lines.line(canvas, "None", TextStyle::Body, theme.text_tertiary);
    }
    // Under the heading, and leaving a line for how many more there are.
    let rows = ((list.height - LINE) / LINE).floor() as usize;
    let room = if view.keys.len() <= rows { view.keys.len() } else { rows.saturating_sub(1).max(1) };
    for description in view.descriptions.iter().take(room) {
        lines.line(canvas, description, TextStyle::Mono, colors.pressed);
    }
    if view.keys.len() > room {
        lines.line(canvas, &format!("and {} more", view.keys.len() - room), TextStyle::Body, theme.text_secondary);
    }
    canvas.text_clipped(
        [status.x, status.y],
        &format!("Readings: {}/s", view.rate.per_second),
        status.width,
        TextStyle::Body,
        theme.text_secondary,
    );
}

fn draw_mouse(canvas: &mut Canvas, theme: &Theme, input: &Input, area: Rect) {
    let view = &input.mouse;
    let state = &view.state;
    let colors = palette(theme);
    let inner = card(canvas, theme, area, "Mouse", &view.label);

    // Raw relative motion, without the pointer acceleration the cursor has.
    let pad_area = Rect::new(inner.right() - MOUSE_PAD_SIZE, inner.y + LINE + SPACE_XS, MOUSE_PAD_SIZE, MOUSE_PAD_SIZE);
    canvas.text_clipped([pad_area.x, inner.y], "Raw motion", MOUSE_PAD_SIZE, TextStyle::BodyStrong, colors.mouse);
    stick_graph(canvas, theme, pad_area, [view.pad[0], -view.pad[1]], 0.0, colors.mouse);

    let (status, rest) = inner.split_bottom(LINE, SPACE_S);
    let text_area = Rect::from_edges(rest.x, rest.y, pad_area.x - SPACE_M, rest.bottom());
    let buttons: Vec<_> = MOUSE_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            on: state.buttons & button != 0,
            enabled: true,
        })
        .collect();
    let chips = status_chips(canvas, theme, text_area, &buttons, colors.pressed);
    let mut lines = Lines::new(Rect::from_edges(text_area.x, text_area.y + chips + SPACE_M, text_area.right(), text_area.bottom()));
    let value_x = 112.0;
    let mut pair = |label: &str, value: String| {
        lines.pair(canvas, label, &value, value_x, theme.text_secondary, theme.text);
    };
    pair("Last motion", format!("({:+}, {:+})", view.delta[0], view.delta[1]));
    pair("Total motion", format!("({}, {})", state.positionX, state.positionY));
    pair("Wheel total", format!("({}, {})", state.wheelX, state.wheelY));
    if state.positions & GameInputMouseAbsolutePosition != 0 {
        pair("Absolute", format!("({}, {})", state.absolutePositionX, state.absolutePositionY));
    }
    let mut readings = format!("Readings: {}/s", view.rate.per_second);
    if let Some(rate) = view.sample_rate {
        readings += &format!("   Device reports {rate} Hz");
    }
    canvas.text_clipped([status.x, status.y], &readings, status.width, TextStyle::Body, theme.text_secondary);
}

// A row for each device, which shows a tooltip about it while the cursor is
// over it.
fn draw_devices(frame: &mut Frame, input: &Input, area: Rect) {
    let theme = frame.theme();
    let inner = frame.card(area, "Devices", &format!("{} connected", input.devices.len()));
    if input.devices.is_empty() {
        frame.label(Rect::new(inner.x, inner.y, inner.width, LINE), "None yet", TextStyle::Body, theme.text_tertiary, Align::Start);
    }
    // One row each: the name, then what kind of device it is. If they don't
    // all fit, the last line says how many more there are.
    let rows = ((inner.height + 0.5) / DEVICE_ROW).floor() as usize;
    let shown = if input.devices.len() > rows {
        rows.saturating_sub(1)
    } else {
        input.devices.len()
    };
    // Each row reaches from the card's edges, like a list's.
    let mut stack = Stack::down(Rect::from_edges(area.x + SPACE_XS, inner.y, area.right() - SPACE_XS, inner.bottom()), 0.0);
    // The gamepad the card shows is highlighted, and those ignored are faint.
    for (index, info) in input.devices.iter().take(shown).enumerate() {
        let row = stack.next(DEVICE_ROW);
        let name = format!("{DEVICE_ROWS}/{index}");
        let response = frame.interact_labeled(&name, "Row", &info.label, row, Sense::HOVER);
        let hover = frame.ease(user_interface::Id::new(&name), response.hovered as u8 as f32);
        frame.canvas.fill(row, RADIUS_CONTROL, theme.subtle_hover.alpha(hover));
        let listened = input.gamepad.device == Some(info.id);
        let (style, color) = if listened {
            (TextStyle::BodyStrong, theme.accent_text)
        } else if !input.listens_to(info) {
            (TextStyle::Body, theme.text_tertiary)
        } else {
            (TextStyle::Body, theme.text)
        };
        let text = row.inset_by(CARD_PADDING - SPACE_XS, 0.0);
        let x = frame.label(text, &info.label, style, color, Align::Start);
        let mut details = format!("{}: {}", info.family_name(), info.kinds());
        if let Some(index) = info.xinput_index {
            details += &format!(", XInput #{index}");
        }
        if info.rumble_motors != 0 {
            details += ", rumble";
        }
        let rest = Rect::from_edges(x + SPACE_S, row.y, text.right(), row.bottom());
        frame.label(rest, &details, TextStyle::Body, theme.text_secondary, Align::Start);
        if response.hovered {
            draw_tooltip(frame, info, row);
        }
    }
    if shown < input.devices.len() {
        let line = stack.next(DEVICE_ROW).inset_by(CARD_PADDING - SPACE_XS, 0.0);
        let more = format!("and {} more", input.devices.len() - shown);
        frame.label(line, &more, TextStyle::Body, theme.text_secondary, Align::Start);
    }
}

// What's known about a device, in a tooltip by the cursor, on the device's
// row. Library/UserInterface places it: under the cursor, or above it if
// there isn't room, or else beside the row, like for a device far down
// Windows' device tree, and always inside the window. It's named after the
// device's label, for scripts.
fn draw_tooltip(frame: &mut Frame, device: &Device, row: Rect) {
    let theme = frame.theme();
    let fonts = frame.fonts();
    let indent = 12.0;
    let section_gap = 6.0;

    // Each piece of text, from the box's top left.
    let mut texts: Vec<TooltipText<String>> = Vec::new();
    let mut y = 0.0;
    texts.push(([0.0, y], device.label.clone(), TextStyle::BodyStrong, theme.text));
    y += LINE;
    for section in device.details() {
        y += section_gap;
        texts.push(([0.0, y], section.title, TextStyle::BodyStrong, theme.accent_text));
        y += LINE;
        // The values line up after the section's longest name.
        let value_x = indent
            + section
                .rows
                .iter()
                .map(|(name, _)| fonts.measure(name, TextStyle::Body))
                .fold(0.0, f32::max)
                .ceil()
            + SPACE_M;
        for (name, value) in section.rows {
            texts.push(([indent, y], name, TextStyle::Body, theme.text_secondary));
            for part in value.split('\n') {
                for line in fonts.wrap(part, TextStyle::Body, VALUE_WIDTH, 0.0) {
                    texts.push(([value_x, y], line, TextStyle::Body, theme.text));
                    y += LINE;
                }
            }
        }
    }
    frame.tooltip(&device.label, row, &texts);
}

// The log, newest first, in Cascadia Mono, so that its columns line up: when,
// where from, and what.
fn draw_log(canvas: &mut Canvas, theme: &Theme, input: &Input, area: Rect) {
    let inner = card(canvas, theme, area, "Event log", "newest first; repeated motion from one device shares a line");
    let mut lines = Lines::new(inner);
    for entry in &input.log {
        if !lines.has_room(canvas, TextStyle::Mono) {
            break;
        }
        let Some(row) = lines.next(canvas, TextStyle::Mono) else {
            break;
        };
        let line = &entry.line;
        let mut x = canvas.text([row.x, row.y], &line[..entry.source_at], TextStyle::Mono, theme.text_secondary);
        x = canvas.text(
            [x, row.y],
            &line[entry.source_at..entry.text_at],
            TextStyle::Mono,
            source_color(theme, entry.source),
        );
        canvas.text_clipped([x, row.y], &line[entry.text_at..], row.right() - x, TextStyle::Mono, theme.text);
    }
}

fn source_color(theme: &Theme, source: Source) -> Color {
    let colors = palette(theme);
    match source {
        Source::Gamepad => colors.left,
        Source::Keyboard => colors.keyboard,
        Source::Mouse => colors.mouse,
        Source::Device => theme.success,
        Source::Demo => theme.text_secondary,
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
