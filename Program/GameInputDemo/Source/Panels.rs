// Lays out and draws the demo's panels with Library/UserInterface, and
// handles what the mouse does to them: the select box that picks which
// gamepad the demo listens to, and the Devices panel's rows, each of which
// shows a tooltip about its device while the cursor is over it. The demo only
// supports two client areas, 2560 x 1369 when maximized and 2560 x 1440 in
// borderless fullscreen (see Library/Application), so the panels have fixed
// places in pixels. The bottom row and the log are taller in fullscreen, and
// the footer stays at the bottom.
//
// The widgets read the mouse from the window's messages, not from GameInput,
// because they say where the cursor is in the window. The stick graphs, the
// trigger bars, the mouse pad, and the panels' text are drawn here.

use game_input::*;
use user_interface::theme::{self, BORDER, DIM, ERROR, FAINT, TEXT, TITLE, WARNING};
use user_interface::{
    Canvas, Chip, Choice, Color, Frame, Lines, MARGIN, PADDING, PANELS_TOP, Rect, Sense,
    chip_layout, chips, panel, select_width, wrap,
};

use crate::devices::{Device, rumble_motor_names};
use crate::input::{GAMEPAD_BUTTONS, Input, Listen, MOUSE_BUTTONS, Source};

const PLOT: Color = theme::INSET;
// The left and right sticks and triggers.
const LEFT: Color = theme::ACCENT;
const RIGHT: Color = theme::ORANGE;
const PRESSED: Color = theme::SUCCESS;
const KEYBOARD: Color = theme::PURPLE;

// The layout, in pixels. Three columns: the gamepad over the keyboard and
// mouse, the latest event over the devices, and the log. MARGIN, PADDING and
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

// The select box for which gamepad to listen to. Its list is "Gamepad/List",
// and its choices "Gamepad/0", "Gamepad/1" and so on.
pub const GAMEPAD_SELECT: &str = "Gamepad";
// The Devices panel's rows are "Devices/0", "Devices/1" and so on, in the
// panel's order. Two devices can have the same label, so their rows are named
// by their place, and their tooltips by their device's label.
pub const DEVICE_ROWS: &str = "Devices";

// At most this many characters of a value fit on one of a tooltip's lines.
// Longer values wrap at spaces, and words longer than this stay whole.
const VALUE_COLUMNS: usize = 100;

// Draws the header, the footer, and the panels, and changes which gamepad
// the demo listens to when one is picked in the select box.
pub fn draw(frame: &mut Frame, input: &mut Result<Input, String>) {
    // No widget takes keyboard focus: the keys are the user's, to try the
    // keyboard panel with, and the demo's shortcuts come through GameInput.
    frame.set_focusable(false);
    let width = frame.width;
    frame.header_with(
        "GameInput Demo",
        "Gamepad, keyboard and mouse input with the GameInput v4 API",
        |canvas, area| draw_status(canvas, area, input, width),
    );
    let bottom = frame.footer(
        &[
            ("F1", "rumble test"),
            ("F2", "clear the log"),
            ("F11", "borderless fullscreen"),
        ],
        "The app's own shortcuts are read with GameInput too.",
    );
    let Ok(input) = input else {
        return;
    };

    // The gamepad panel is as tall as its contents: captions, the graphs,
    // two lines of values, the chips and a status line. The latest event
    // panel beside it is as tall. The rows under them, and the log, go down
    // to the footer.
    let line_height = frame.line_height();
    let gamepad_names: Vec<&str> = GAMEPAD_BUTTONS.iter().map(|(_, name)| *name).collect();
    let (_, gamepad_chips) =
        chip_layout(&frame.canvas, GAMEPAD_WIDTH - PADDING * 2.0, &gamepad_names);
    let title_height = PADDING * 2.0 + line_height * 1.4;
    let gamepad_height =
        title_height + line_height * 4.6 + PADDING + gamepad_chips + STICK_GRAPH_SIZE;
    let gamepad_area = Rect::new(GAMEPAD_X, PANELS_TOP, GAMEPAD_WIDTH, gamepad_height);
    let lower_y = gamepad_area.bottom() + MARGIN;
    let lower_height = bottom - lower_y;
    let keyboard_area = Rect::new(GAMEPAD_X, lower_y, KEYBOARD_WIDTH, lower_height);
    let mouse_area = Rect::new(MOUSE_X, lower_y, MOUSE_WIDTH, lower_height);
    let latest_area = Rect::new(LATEST_X, PANELS_TOP, LATEST_WIDTH, gamepad_height);
    let devices_area = Rect::new(LATEST_X, lower_y, LATEST_WIDTH, lower_height);
    let log_area = Rect::new(
        LOG_X,
        PANELS_TOP,
        width - MARGIN - LOG_X,
        bottom - PANELS_TOP,
    );

    draw_gamepad(frame, input, gamepad_area);
    draw_latest(&mut frame.canvas, input, latest_area);
    draw_keyboard(&mut frame.canvas, input, keyboard_area);
    draw_mouse(&mut frame.canvas, input, mouse_area);
    draw_devices(frame, input, devices_area);
    draw_log(&mut frame.canvas, input, log_area);
}

// The left of the header's status line: the GameInput runtime, where it was
// loaded from, cut short to fit, and how many readings the demo handled.
fn draw_status(canvas: &mut Canvas, area: Rect, input: &Result<Input, String>, width: f32) {
    let status_y = area.y;
    match input {
        Ok(input) => {
            let mut x = canvas.text(area.x, status_y, "Runtime: ", DIM);
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
            x = canvas.text(x, status_y, &runtime, TEXT);
            let count = input.reading_count.to_string();
            let count_label = "   Readings handled: ";
            let count_width = canvas.text_width(count_label) + canvas.text_width(&count);
            let location_width = (area.right() - x - count_width).min(width * 0.45);
            x = canvas.text_clipped(x, status_y, &location, location_width, DIM);
            x = canvas.text(x, status_y, count_label, DIM);
            canvas.text(x, status_y, &count, TEXT);
        }
        Err(message) => {
            canvas.text_clipped(
                area.x,
                status_y,
                &format!("GameInput isn't available: {message}"),
                area.width,
                ERROR,
            );
        }
    }
}

fn draw_gamepad(frame: &mut Frame, input: &mut Input, area: Rect) {
    let line_height = frame.line_height();
    let (graph, bar_width, gap) = (STICK_GRAPH_SIZE, TRIGGER_BAR_WIDTH, GAMEPAD_GAP);
    let inner = frame.panel(area, "Gamepad", "");
    draw_gamepad_select(frame, input, area);
    let canvas = &mut frame.canvas;
    let view = &input.gamepad;
    let state = &view.state;

    // Stick graphs and trigger bars, with a caption above and values below.
    let plots_y = inner.y + line_height * 1.1;
    let left_x = inner.x;
    let right_x = left_x + graph + gap;
    let left_trigger_x = right_x + graph + gap;
    let right_trigger_x = left_trigger_x + bar_width + gap;
    canvas.text(left_x, inner.y, "Left stick", LEFT);
    canvas.text(right_x, inner.y, "Right stick", RIGHT);
    canvas.text(left_trigger_x, inner.y, "LT", LEFT);
    canvas.text(right_trigger_x, inner.y, "RT", RIGHT);

    let left = [state.leftThumbstickX, state.leftThumbstickY];
    let right = [state.rightThumbstickX, state.rightThumbstickY];
    stick_graph(
        canvas,
        Rect::new(left_x, plots_y, graph, graph),
        left,
        input.rumble_dead_zone,
        LEFT,
    );
    stick_graph(
        canvas,
        Rect::new(right_x, plots_y, graph, graph),
        right,
        input.rumble_dead_zone,
        RIGHT,
    );
    trigger_bar(
        canvas,
        Rect::new(left_trigger_x, plots_y, bar_width, graph),
        state.leftTrigger,
        LEFT,
    );
    trigger_bar(
        canvas,
        Rect::new(right_trigger_x, plots_y, bar_width, graph),
        state.rightTrigger,
        RIGHT,
    );

    let values_y = plots_y + graph + line_height * 0.3;
    for (x, value) in [(left_x, left), (right_x, right)] {
        let mut lines = Lines::new(
            Rect::new(x, values_y, graph + gap, line_height * 2.0),
            line_height,
        );
        lines.line(
            canvas,
            &format!("x {:+.3}  y {:+.3}", value[0], value[1]),
            TEXT,
        );
        let magnitude = (value[0] * value[0] + value[1] * value[1]).sqrt();
        let degrees = value[1].atan2(value[0]).to_degrees();
        lines.line(
            canvas,
            &format!("r {magnitude:.3}  {degrees:+.0}\u{B0}"),
            DIM,
        );
    }
    canvas.text(
        left_trigger_x,
        values_y,
        &format!("{:.3}", state.leftTrigger),
        TEXT,
    );
    canvas.text(
        right_trigger_x,
        values_y,
        &format!("{:.3}", state.rightTrigger),
        TEXT,
    );

    // A chip per button. Buttons the gamepad says it lacks are faint.
    let chips_y = values_y + line_height * 2.0 + PADDING;
    let chips_area = Rect::new(
        inner.x,
        chips_y,
        inner.width,
        inner.bottom() - line_height * 1.2 - chips_y,
    );
    let buttons: Vec<_> = GAMEPAD_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            on: state.buttons & button != 0,
            enabled: view.layout.is_none_or(|layout| layout & button != 0),
        })
        .collect();
    chips(canvas, chips_area, &buttons, PRESSED);

    let mut status = format!("Readings: {}/s   Rumble test: ", view.rate.per_second);
    status += if input.rumble_enabled { "on" } else { "off" };
    status += " (F1)   Motors: ";
    status += &rumble_motor_names(view.rumble_motors);
    canvas.text_clipped(
        inner.x,
        inner.bottom() - line_height,
        &status,
        inner.width,
        DIM,
    );
}

// The select box after the gamepad panel's title, which shows which gamepads
// the demo listens to, and after it, what the panel shows. Picking a choice
// in its list listens to that.
fn draw_gamepad_select(frame: &mut Frame, input: &mut Input, area: Rect) {
    let line_height = frame.line_height();
    let char_width = frame.char_width();
    let height = frame.control_height();
    let text_y = area.y + PADDING;
    let x = area.x + PADDING + frame.text_width("Gamepad") + char_width * 2.0;
    // So that the box's text is on the title's line.
    let y = text_y - ((height - line_height) * 0.5).round();
    let width =
        select_width(&frame.canvas, input.listen.label()).min(area.right() - PADDING - x);
    let select_box = Rect::new(x, y, width, height);

    // Each choice's gamepad, or none, and a few words about it: each
    // connected gamepad, a chosen one while it's disconnected, which stays
    // in the list, and no gamepad.
    let connected = match &input.listen {
        Listen::Gamepad(device) => input.gamepads().any(|info| info.id == device.id),
        Listen::NoGamepad => true,
    };
    let mut gamepads: Vec<(Option<&Device>, &str)> = input
        .gamepads()
        .map(|info| (Some(info), info.family_name()))
        .collect();
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

    // Until a choice is picked, the demo switches to a gamepad as soon as one connects.
    let (note, color) = match &input.listen {
        Listen::Gamepad(_) if !connected => (
            if input.listen_picked {
                "Disconnected: waiting for it to reconnect"
            } else {
                "Disconnected: waiting for a gamepad to connect"
            },
            WARNING,
        ),
        Listen::Gamepad(_) => ("", DIM),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected", WARNING),
        Listen::NoGamepad => ("Gamepad input is ignored", DIM),
    };
    let note_x = select_box.right() + char_width * 2.0;
    frame.canvas.text_clipped(
        note_x,
        text_y,
        note,
        area.right() - PADDING - note_x,
        color,
    );
}

fn stick_graph(
    canvas: &mut Canvas,
    area: Rect,
    value: [f32; 2],
    dead_zone: f32,
    color: Color,
) {
    canvas.rect(area.x, area.y, area.width, area.height, PLOT);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    let center = [area.x + area.width * 0.5, area.y + area.height * 0.5];
    let radius = area.width * 0.5 - 6.0;
    // Stick y is up, screen y is down.
    let to_screen =
        |point: [f32; 2]| [center[0] + point[0] * radius, center[1] - point[1] * radius];

    canvas.line(
        [area.x + 1.0, center[1]],
        [area.right() - 1.0, center[1]],
        1.0,
        FAINT,
    );
    canvas.line(
        [center[0], area.y + 1.0],
        [center[0], area.bottom() - 1.0],
        1.0,
        FAINT,
    );
    canvas.ring(center, radius, 1.5, FAINT);
    canvas.ring(center, radius * 0.5, 1.0, FAINT.alpha(0.5));
    // The mouse pad has no dead zone, and a ring of radius 0 would be a dot.
    if dead_zone > 0.0 {
        canvas.ring(center, radius * dead_zone, 1.0, WARNING.alpha(0.45));
    }

    let position = to_screen(value);
    canvas.line(center, position, 1.5, color.alpha(0.7));
    canvas.circle(position, (canvas.line_height() * 0.35).max(4.0), color);
}

fn trigger_bar(canvas: &mut Canvas, area: Rect, value: f32, color: Color) {
    canvas.rect(area.x, area.y, area.width, area.height, PLOT);
    let filled = (area.height - 2.0) * value.clamp(0.0, 1.0);
    canvas.rect(
        area.x + 1.0,
        area.bottom() - 1.0 - filled,
        area.width - 2.0,
        filled,
        color,
    );
    for quarter in 1..4 {
        let y = (area.y + area.height * quarter as f32 / 4.0).round();
        canvas.rect(area.x, y, area.width * 0.25, 1.0, FAINT);
    }
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
}

fn draw_latest(canvas: &mut Canvas, input: &Input, area: Rect) {
    let padding = PADDING;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, "Latest input event", "");
    // A text box: an inset area that holds the description.
    canvas.rect(inner.x, inner.y, inner.width, inner.height, PLOT);
    canvas.rect_outline(inner.x, inner.y, inner.width, inner.height, 1.0, BORDER);
    let text_area = Rect::new(
        inner.x + padding,
        inner.y + padding * 0.6,
        inner.width - padding * 2.0,
        inner.height - padding * 1.2,
    );
    let mut lines = Lines::new(text_area, line_height);

    let Some(event) = &input.latest else {
        lines.wrapped(
            canvas,
            "Waiting for input. Press a key, move the mouse, or use a gamepad.",
            "",
            DIM,
        );
        return;
    };
    let heading = format!(
        "{} reading #{}  at {:.6} s",
        event.source.name(),
        event.number,
        event.time
    );
    lines.line(canvas, &heading, source_color(event.source));
    lines.pair(canvas, "Device: ", &event.device, TEXT);
    lines.pair(canvas, "Reading has: ", &event.kinds, TEXT);
    lines.pair(
        canvas,
        "Timestamp: ",
        &format!("{} \u{B5}s", event.timestamp),
        TEXT,
    );
    match event.interval {
        Some(interval) => lines.pair(
            canvas,
            "Since its previous reading: ",
            &format!("{interval:.3} ms"),
            TEXT,
        ),
        None => lines.wrapped(
            canvas,
            "No earlier reading from this device, so this is compared with an idle state.",
            "",
            DIM,
        ),
    }
    lines.gap(0.4);
    lines.line(canvas, "Changed:", DIM);
    for change in &event.changes {
        lines.wrapped(canvas, &format!("- {change}"), "  ", TEXT);
    }
    lines.gap(0.4);
    lines.line(canvas, "State now:", DIM);
    for state in &event.state {
        lines.wrapped(canvas, state, "  ", TEXT);
    }
}

fn draw_keyboard(canvas: &mut Canvas, input: &Input, area: Rect) {
    let view = &input.keyboard;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, "Keyboard", &view.label);
    let mut lines = Lines::new(
        Rect::new(inner.x, inner.y, inner.width, inner.height - line_height),
        line_height,
    );
    lines.line(canvas, &format!("Keys down ({}):", view.keys.len()), DIM);
    if view.keys.is_empty() {
        lines.line(canvas, "  none", FAINT);
    }
    // Under the heading, and leaving a line for how many more there are.
    let rows = (inner.height / line_height).floor() as usize;
    let room = if view.keys.len() + 2 <= rows {
        view.keys.len()
    } else {
        rows.saturating_sub(3).max(1)
    };
    for description in view.descriptions.iter().take(room) {
        lines.line(canvas, &format!("  {description}"), PRESSED);
    }
    if view.keys.len() > room {
        lines.line(
            canvas,
            &format!("  and {} more", view.keys.len() - room),
            DIM,
        );
    }
    canvas.text_clipped(
        inner.x,
        inner.bottom() - line_height,
        &format!("Readings: {}/s", view.rate.per_second),
        inner.width,
        DIM,
    );
}

fn draw_mouse(canvas: &mut Canvas, input: &Input, area: Rect) {
    let view = &input.mouse;
    let state = &view.state;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, "Mouse", &view.label);

    // Raw relative motion, without the pointer acceleration the cursor has.
    let pad_size = MOUSE_PAD_SIZE;
    let pad_area = Rect::new(
        inner.right() - pad_size,
        inner.y + line_height * 1.1,
        pad_size,
        pad_size,
    );
    canvas.text_clipped(pad_area.x, inner.y, "Raw motion", pad_size, DIM);
    stick_graph(
        canvas,
        pad_area,
        [view.pad[0], -view.pad[1]],
        0.0,
        WARNING,
    );

    let text_area = Rect::new(
        inner.x,
        inner.y,
        inner.width - pad_size - PADDING,
        inner.height - line_height,
    );
    let buttons: Vec<_> = MOUSE_BUTTONS
        .iter()
        .map(|&(button, label)| Chip {
            label,
            on: state.buttons & button != 0,
            enabled: true,
        })
        .collect();
    let values_y =
        text_area.y + chips(canvas, text_area, &buttons, PRESSED) + line_height * 0.2;
    let mut lines = Lines::new(
        Rect::new(
            text_area.x,
            values_y,
            text_area.width,
            text_area.bottom() - values_y,
        ),
        line_height,
    );
    lines.pair(
        canvas,
        "Last motion: ",
        &format!("({:+}, {:+})", view.delta[0], view.delta[1]),
        TEXT,
    );
    lines.pair(
        canvas,
        "Total motion: ",
        &format!("({}, {})", state.positionX, state.positionY),
        TEXT,
    );
    lines.pair(
        canvas,
        "Wheel total: ",
        &format!("({}, {})", state.wheelX, state.wheelY),
        TEXT,
    );
    if state.positions & GameInputMouseAbsolutePosition != 0 {
        lines.pair(
            canvas,
            "Absolute: ",
            &format!("({}, {})", state.absolutePositionX, state.absolutePositionY),
            TEXT,
        );
    }
    let mut status = format!("Readings: {}/s", view.rate.per_second);
    if let Some(rate) = view.sample_rate {
        status += &format!("   Device reports {rate} Hz");
    }
    canvas.text_clipped(
        inner.x,
        inner.bottom() - line_height,
        &status,
        text_area.width,
        DIM,
    );
}

// A row for each device, which shows a tooltip about it while the cursor is
// over it.
fn draw_devices(frame: &mut Frame, input: &Input, area: Rect) {
    let line_height = frame.line_height();
    let inner = frame.panel(
        area,
        "Devices",
        &format!("{} connected", input.devices.len()),
    );
    let mut lines = Lines::new(inner, line_height);
    if input.devices.is_empty() {
        lines.line(&mut frame.canvas, "None yet", FAINT);
    }
    // One line each: the name, then what kind of device it is. If they don't
    // all fit, the last line says how many more there are.
    let rows = ((inner.height + 0.5) / line_height).floor() as usize;
    let shown = if input.devices.len() > rows {
        rows.saturating_sub(1)
    } else {
        input.devices.len()
    };
    // The gamepad the panel shows is highlighted, and those ignored are faint.
    for (index, info) in input.devices.iter().take(shown).enumerate() {
        // Each row reaches halfway into the panel's padding on both sides.
        let row = Rect::new(
            inner.x - PADDING * 0.5,
            lines.y,
            inner.width + PADDING,
            line_height,
        );
        let name = format!("{DEVICE_ROWS}/{index}");
        let response = frame.interact_labeled(&name, "Row", &info.label, row, Sense::HOVER);
        let canvas = &mut frame.canvas;
        if response.hovered {
            canvas.rect(row.x, row.y, row.width, row.height, BORDER);
        }
        let color = if input.gamepad.device == Some(info.id) {
            LEFT
        } else if !input.listens_to(info) {
            FAINT
        } else {
            TEXT
        };
        let x = canvas.text_clipped(lines.x, lines.y, &info.label, lines.width, color);
        let mut details = format!("  {}: {}", info.family_name(), info.kinds());
        if let Some(index) = info.xinput_index {
            details += &format!(", XInput #{index}");
        }
        if info.rumble_motors != 0 {
            details += ", rumble";
        }
        canvas.text_clipped(x, lines.y, &details, lines.right() - x, DIM);
        if response.hovered {
            draw_tooltip(frame, info, row);
        }
        lines.y += line_height;
    }
    if shown < input.devices.len() {
        lines.line(
            &mut frame.canvas,
            &format!("and {} more", input.devices.len() - shown),
            DIM,
        );
    }
}

// What's known about a device, in a tooltip by the cursor, on the device's
// row. Library/UserInterface places it: under the cursor, or above it if
// there isn't room, or else beside the row, like for a device far down
// Windows' device tree, and always inside the window. It's named after the
// device's label, for scripts.
fn draw_tooltip(frame: &mut Frame, device: &Device, row: Rect) {
    let line_height = frame.line_height();
    let char_width = frame.char_width();
    let indent = char_width * 2.0;
    let section_gap = (line_height * 0.4).round();

    // Each piece of text, from the box's top left.
    let mut texts: Vec<([f32; 2], String, Color)> = Vec::new();
    let mut y = 0.0;
    texts.push(([0.0, y], device.label.clone(), TITLE));
    y += line_height;
    for section in device.details() {
        y += section_gap;
        texts.push(([0.0, y], section.title, theme::ACCENT));
        y += line_height;
        // The values line up after the section's longest name.
        let value_x = indent
            + section
                .rows
                .iter()
                .map(|(name, _)| frame.text_width(name))
                .fold(0.0, f32::max)
            + char_width * 2.0;
        for (name, value) in section.rows {
            texts.push(([indent, y], name, DIM));
            for part in value.split('\n') {
                for line in wrap(part, VALUE_COLUMNS, 0) {
                    texts.push(([value_x, y], line, TEXT));
                    y += line_height;
                }
            }
        }
    }
    frame.tooltip(&device.label, row, &texts);
}

fn draw_log(canvas: &mut Canvas, input: &Input, area: Rect) {
    let line_height = canvas.line_height();
    let inner = panel(
        canvas,
        area,
        "Event log",
        "newest first; repeated motion from one device shares a line",
    );
    let mut lines = Lines::new(inner, line_height);
    for entry in &input.log {
        if !lines.has_room() {
            break;
        }
        let line = &entry.line;
        let mut x = canvas.text(lines.x, lines.y, &line[..entry.source_at], DIM);
        x = canvas.text(
            x,
            lines.y,
            &line[entry.source_at..entry.text_at],
            source_color(entry.source),
        );
        canvas.text_clipped(x, lines.y, &line[entry.text_at..], inner.right() - x, TEXT);
        lines.y += line_height;
    }
}

fn source_color(source: Source) -> Color {
    match source {
        Source::Gamepad => LEFT,
        Source::Keyboard => KEYBOARD,
        Source::Mouse => WARNING,
        Source::Device => PRESSED,
        Source::Demo => DIM,
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
