// Lays out and draws the demo's panels onto a Canvas. The demo only supports
// two client areas, 2560 x 1369 when maximized and 2560 x 1440 in borderless
// fullscreen (see Library/Application), so the panels have fixed places in pixels.
// The bottom row and the log are taller in fullscreen, and the footer stays
// at the bottom.

use std::collections::VecDeque;

use application::FrameCounter;
use canvas::theme::{BORDER, DIM, ERROR, FAINT, PANEL, TEXT, TITLE, WARNING};
use canvas::{
    Canvas, Color, EDGE, Lines, MARGIN, PADDING, Rect, draw_window_problems, panel, theme, wrap,
};
use game_input::*;

use crate::devices::{Device, rumble_motor_names};
use crate::input::{GAMEPAD_BUTTONS, Input, Listen, MOUSE_BUTTONS, Source};

const PLOT: Color = theme::INSET;
// The left and right sticks and triggers.
const LEFT: Color = theme::ACCENT;
const RIGHT: Color = theme::ORANGE;
const PRESSED: Color = theme::SUCCESS;
const PRESSED_TEXT: Color = theme::ON_COLOR;
const KEYBOARD: Color = theme::PURPLE;

// The layout, in pixels. Three columns: the gamepad over the keyboard and
// mouse, the latest event over the devices, and the log. MARGIN, EDGE and
// PADDING come from the canvas, which all programs share.
//
// The header has a title, a status line, and room for this many lines of
// notices, so the panels start at the same place whether there are any.
const NOTICE_LINES: usize = 2;
const PANELS_TOP: f32 = 96.0;
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

// At most this many characters of a value fit on one of a tooltip's lines.
// Longer values wrap at spaces, and words longer than this stay whole.
const TOOLTIP_COLUMNS: usize = 100;

// What a click does at a place in the window.
pub enum Click {
    // Opens or closes the gamepad select box's list.
    Picker,
    Choose(Listen),
}

// The places in the last frame drawn that react to the mouse: where clicks
// do something, topmost last, and the Devices panel's rows, each of which
// shows a tooltip about its device while the cursor is over it.
#[derive(Default)]
pub struct Targets {
    clicks: Vec<(Rect, Click)>,
    // Each row's area, and its device's label.
    device_rows: Vec<(Rect, String)>,
    // The device whose tooltip the frame shows, and the tooltip's area.
    tooltip: Option<(String, Rect)>,
}

impl Targets {
    pub fn at(&self, point: [f32; 2]) -> Option<&Click> {
        self.clicks
            .iter()
            .rev()
            .find(|(area, _)| area.contains(point))
            .map(|(_, click)| click)
    }

    // Where the clicks go, for scripted runs to save, for writing scripts.
    pub fn state(&self) -> json::Value {
        json::Value::Array(
            self.clicks
                .iter()
                .map(|(area, click)| {
                    let name = match click {
                        Click::Picker => "Select box".to_string(),
                        Click::Choose(listen) => format!("Choose {}", listen.label()),
                    };
                    area_state(name, *area)
                })
                .collect(),
        )
    }

    // Where the device rows are, for scripted runs to save, for moving the
    // cursor onto one.
    pub fn device_rows_state(&self) -> json::Value {
        json::Value::Array(
            self.device_rows
                .iter()
                .map(|(area, label)| area_state(label.clone(), *area))
                .collect(),
        )
    }

    // The device whose tooltip shows, and where, or null.
    pub fn tooltip_state(&self) -> json::Value {
        self.tooltip
            .as_ref()
            .map(|(label, area)| area_state(label.clone(), *area))
            .into()
    }
}

fn area_state(name: String, area: Rect) -> json::Value {
    json::object([
        ("Name", name.into()),
        ("Area", [area.x, area.y, area.width, area.height].into()),
    ])
}

// What the header and footer show besides input.
pub struct Status<'a> {
    pub focused: bool,
    pub frames: &'a FrameCounter,
    // The display the window is on, like "KXGR16 (PA278QGV)".
    pub display: &'a str,
    // Problems with how the demo was started, one per paragraph.
    pub notices: &'a [String],
    // Which window setups the demo supports, for when it isn't in one.
    pub supported: &'a str,
    // What's wrong with the window's setup, if it isn't one of the two that
    // the demo supports. Then only these are drawn.
    pub window_problems: &'a [String],
    // Where the mouse cursor is in the window, if it's over the window.
    pub cursor: Option<[f32; 2]>,
    // Whether the gamepad select box's list is open.
    pub picker_open: bool,
}

// Returns where clicks do something in what it drew.
pub fn draw(
    canvas: &mut Canvas,
    input: &Result<Input, String>,
    status: &Status,
    width: f32,
    height: f32,
) -> Targets {
    if !status.window_problems.is_empty() {
        draw_window_problems(
            canvas,
            status.supported,
            status.window_problems,
            width,
        );
        return Targets::default();
    }
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let margin = EDGE;
    let frames = status.frames;
    let focused = status.focused;
    // Header and footer.
    let mut x = canvas.text(margin, margin, "GameInput Demo", TITLE);
    canvas.text(
        x,
        margin,
        "  Gamepad, keyboard and mouse input with the GameInput v4 API",
        DIM,
    );
    let status_y = margin + line_height;
    // The frame rate goes on the right, under the focus text. It's far above
    // the monitor's refresh rate, because frames don't wait for vertical sync.
    let (rate, frame_time) = match frames.per_second {
        Some(rate) => (
            format!("{rate:.0}"),
            format!("  ({:.2} ms per frame)", 1000.0 / rate),
        ),
        None => ("measuring".to_string(), String::new()),
    };
    let rate_x = width - margin - canvas.text_width(&format!("FPS: {rate}{frame_time}"));
    x = canvas.text(rate_x, status_y, "FPS: ", DIM);
    x = canvas.text(x, status_y, &rate, TEXT);
    canvas.text(x, status_y, &frame_time, DIM);
    // The status on the left is cut short to leave room for it.
    let status_right = rate_x - char_width * 3.0;
    match input {
        Ok(input) => {
            x = canvas.text(margin, status_y, "Runtime: ", DIM);
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
            let location_width = (status_right - x - count_width).min(width * 0.45);
            x = canvas.text_clipped(x, status_y, &location, location_width, DIM);
            x = canvas.text(x, status_y, count_label, DIM);
            canvas.text(x, status_y, &count, TEXT);
        }
        Err(message) => {
            canvas.text_clipped(
                margin,
                status_y,
                &format!("GameInput isn't available: {message}"),
                status_right - margin,
                ERROR,
            );
        }
    }
    let focus_text = if focused {
        "Window has focus"
    } else {
        "Window doesn't have focus: input is ignored"
    };
    let focus_width = canvas.text_width(focus_text);
    canvas.text(
        width - margin - focus_width,
        margin,
        focus_text,
        if focused { PRESSED } else { WARNING },
    );

    // Notices go under the status, wrapped to the window's width, in the
    // lines the header has for them.
    let mut lines = Vec::new();
    let columns = ((width - margin * 2.0) / char_width).floor() as usize;
    for notice in status.notices {
        for (index, line) in wrap(notice, columns, 2).into_iter().enumerate() {
            lines.push(if index == 0 {
                line
            } else {
                format!("  {line}")
            });
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

    // The display goes on the right of the footer.
    let footer_y = height - margin - line_height;
    let display_x =
        width - margin - canvas.text_width("Display: ") - canvas.text_width(status.display);
    x = canvas.text(display_x, footer_y, "Display: ", DIM);
    canvas.text(x, footer_y, status.display, TEXT);
    x = canvas.text(margin, footer_y, "F1", TEXT);
    x = canvas.text(x, footer_y, " rumble test   ", DIM);
    x = canvas.text(x, footer_y, "F2", TEXT);
    x = canvas.text(x, footer_y, " clear the log   ", DIM);
    x = canvas.text(x, footer_y, "F11", TEXT);
    canvas.text_clipped(
        x,
        footer_y,
        " borderless fullscreen   The app's own shortcuts are read with GameInput too.",
        display_x - char_width * 3.0 - x,
        DIM,
    );

    let bottom = footer_y - margin;
    let Ok(input) = input else {
        return Targets::default();
    };

    // The gamepad panel is as tall as its contents: captions, the graphs,
    // two lines of values, the chips and a status line. The latest event
    // panel beside it is as tall. The rows under them, and the log, go down
    // to the footer.
    let gamepad_names: Vec<&str> = GAMEPAD_BUTTONS.iter().map(|(_, name)| *name).collect();
    let (_, gamepad_chips) = chip_layout(canvas, GAMEPAD_WIDTH - PADDING * 2.0, &gamepad_names);
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

    let mut targets = Targets::default();
    let picker = draw_gamepad(canvas, input, status, gamepad_area);
    targets.clicks.push((picker, Click::Picker));
    draw_latest(canvas, input, latest_area);
    draw_keyboard(canvas, input, keyboard_area);
    draw_mouse(canvas, input, mouse_area);
    let hovered = draw_devices(canvas, input, status, devices_area, &mut targets);
    draw_log(canvas, input, log_area);
    // Last, so the list covers the panels under it.
    if status.picker_open {
        draw_picker_list(canvas, input, status, picker, width - MARGIN, &mut targets);
    }
    // And the tooltip covers everything, unless the cursor is on the list.
    if let (Some(device), Some(cursor)) = (hovered, status.cursor)
        && targets.at(cursor).is_none()
    {
        let area = draw_tooltip(canvas, device, cursor, width, height);
        targets.tooltip = Some((device.label.clone(), area));
    }
    targets
}

// Returns the area of the select box for which gamepad to listen to.
fn draw_gamepad(canvas: &mut Canvas, input: &Input, status: &Status, area: Rect) -> Rect {
    let view = &input.gamepad;
    let line_height = canvas.line_height();
    let (graph, bar_width, gap) = (STICK_GRAPH_SIZE, TRIGGER_BAR_WIDTH, GAMEPAD_GAP);
    let inner = panel(canvas, area, "Gamepad", "");
    let picker = draw_picker_box(canvas, input, status, area);
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
        &view.left_trail,
        input.rumble_dead_zone,
        LEFT,
    );
    stick_graph(
        canvas,
        Rect::new(right_x, plots_y, graph, graph),
        right,
        &view.right_trail,
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
        .map(|&(button, name)| {
            let supported = view.layout.is_none_or(|layout| layout & button != 0);
            (name, state.buttons & button != 0, supported)
        })
        .collect();
    chips(canvas, chips_area, &buttons);

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
    picker
}

// The select box after the gamepad panel's title, which shows which gamepads
// the demo listens to, and after it, what the panel shows. Returns its area.
fn draw_picker_box(canvas: &mut Canvas, input: &Input, status: &Status, area: Rect) -> Rect {
    let padding = PADDING;
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let text_y = area.y + padding;
    let height = chip_height(canvas);
    let x = area.x + padding + canvas.text_width("Gamepad") + char_width * 2.0;
    let y = (text_y - (height - line_height) * 0.5).round();
    let inset = (line_height * 0.4).round();
    let arrow_width = line_height;
    let text = input.listen.label();
    let width =
        (canvas.text_width(text) + inset * 2.0 + arrow_width).min(area.right() - padding - x);
    let picker = Rect::new(x, y, width, height);

    let hovered = status.cursor.is_some_and(|point| picker.contains(point));
    canvas.rect(picker.x, picker.y, picker.width, picker.height, PLOT);
    canvas.rect_outline(
        picker.x,
        picker.y,
        picker.width,
        picker.height,
        1.0,
        if hovered || status.picker_open {
            DIM
        } else {
            BORDER
        },
    );
    canvas.text_clipped(x + inset, text_y, text, width - inset - arrow_width, TEXT);
    chevron(
        canvas,
        [picker.right() - arrow_width * 0.5, y + height * 0.5],
        (line_height * 0.2).round(),
        status.picker_open,
        DIM,
    );

    // Until a choice is picked, the demo switches to a gamepad as soon as one connects.
    let (note, color) = match &input.listen {
        Listen::Gamepad(device) if !input.gamepads().any(|info| info.id == device.id) => (
            if input.listen_picked {
                "Disconnected: waiting for it to reconnect"
            } else {
                "Disconnected: waiting for a gamepad to connect"
            }
            .to_string(),
            WARNING,
        ),
        Listen::Gamepad(_) => (String::new(), DIM),
        Listen::NoGamepad if !input.listen_picked => ("No gamepad connected".to_string(), WARNING),
        Listen::NoGamepad => ("Gamepad input is ignored".to_string(), DIM),
    };
    let note_x = picker.right() + char_width * 2.0;
    canvas.text_clipped(
        note_x,
        text_y,
        &note,
        area.right() - padding - note_x,
        color,
    );
    picker
}

// The select box's list of choices, under the box. Adds a target for each.
fn draw_picker_list(
    canvas: &mut Canvas,
    input: &Input,
    status: &Status,
    picker: Rect,
    right: f32,
    targets: &mut Targets,
) {
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let row_height = chip_height(canvas);
    let inset = (line_height * 0.4).round();

    // Each choice, and a few words about it.
    let mut choices = Vec::new();
    for info in input.gamepads() {
        let listen = Listen::Gamepad(info.clone());
        choices.push((listen, info.label.clone(), info.family_name().to_string()));
    }
    // A chosen gamepad stays in the list while it's disconnected.
    if let Listen::Gamepad(device) = &input.listen
        && !input.gamepads().any(|info| info.id == device.id)
    {
        choices.push((
            input.listen.clone(),
            input.listen.label().to_string(),
            "disconnected".to_string(),
        ));
    }
    choices.push((
        Listen::NoGamepad,
        Listen::NoGamepad.label().to_string(),
        "ignores all gamepads".to_string(),
    ));

    let label_width = choices
        .iter()
        .map(|(_, label, _)| canvas.text_width(label))
        .fold(0.0, f32::max);
    let detail_width = choices
        .iter()
        .map(|(_, _, detail)| canvas.text_width(detail))
        .fold(0.0, f32::max);
    // With a pixel on each side for the outline.
    let width = (inset * 2.0 + label_width + char_width * 2.0 + detail_width + 2.0)
        .max(picker.width)
        .min(right - picker.x);
    let list = Rect::new(
        picker.x,
        picker.bottom() + 2.0,
        width,
        row_height * choices.len() as f32 + 2.0,
    );
    // A shadow, so the list stands out from the panels under it.
    canvas.rect(
        list.x + 3.0,
        list.y + 3.0,
        list.width,
        list.height,
        Color::rgb(0x000000).alpha(0.15),
    );
    canvas.rect(list.x, list.y, list.width, list.height, PANEL);
    canvas.rect_outline(list.x, list.y, list.width, list.height, 1.0, DIM);

    let detail_x = list.x + inset + label_width + char_width * 2.0;
    for (index, (listen, label, detail)) in choices.into_iter().enumerate() {
        let row = Rect::new(
            list.x + 1.0,
            list.y + 1.0 + row_height * index as f32,
            list.width - 2.0,
            row_height,
        );
        let chosen = listen == input.listen;
        if status.cursor.is_some_and(|point| row.contains(point)) {
            canvas.rect(row.x, row.y, row.width, row.height, BORDER);
        }
        if chosen {
            canvas.rect(row.x, row.y, 3.0, row.height, LEFT);
        }
        let text_y = row.y + ((row_height - line_height) * 0.5).round();
        canvas.text_clipped(
            row.x + inset,
            text_y,
            &label,
            detail_x - char_width - row.x - inset,
            if chosen { LEFT } else { TEXT },
        );
        canvas.text_clipped(
            detail_x,
            text_y,
            &detail,
            row.right() - inset - detail_x,
            DIM,
        );
        targets.clicks.push((row, Click::Choose(listen)));
    }
}

// A small V that points down, or up while the list it opens is open.
fn chevron(canvas: &mut Canvas, [x, y]: [f32; 2], size: f32, up: bool, color: Color) {
    let tip = if up { -size * 0.5 } else { size * 0.5 };
    canvas.line([x - size, y - tip], [x, y + tip], 1.5, color);
    canvas.line([x, y + tip], [x + size, y - tip], 1.5, color);
}

fn stick_graph(
    canvas: &mut Canvas,
    area: Rect,
    value: [f32; 2],
    trail: &VecDeque<[f32; 2]>,
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

    // The recent path fades out with age.
    let points: Vec<[f32; 2]> = trail.iter().map(|&point| to_screen(point)).collect();
    for (index, pair) in points.windows(2).enumerate() {
        let age = 1.0 - index as f32 / points.len() as f32;
        canvas.line(pair[0], pair[1], 2.0, color.alpha(0.6 * (1.0 - age)));
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

fn chip_height(canvas: &Canvas) -> f32 {
    (canvas.line_height() * 1.3).round()
}

// Places chips with these names left to right, wrapping onto new rows to fit
// in `width`. Returns each chip's offset from the top left and its width, and
// the height of all the rows.
fn chip_layout(canvas: &Canvas, width: f32, names: &[&str]) -> (Vec<([f32; 2], f32)>, f32) {
    let line_height = canvas.line_height();
    let gap = (line_height * 0.3).round();
    let (mut x, mut y) = (0.0, 0.0);
    let mut chips = Vec::new();
    for name in names {
        let chip_width = canvas.text_width(name) + line_height * 0.8;
        if x + chip_width > width && x > 0.0 {
            x = 0.0;
            y += chip_height(canvas) + gap;
        }
        chips.push(([x, y], chip_width));
        x += chip_width + gap;
    }
    (chips, y + chip_height(canvas))
}

// Draws labeled chips as chip_layout places them, and returns their height.
fn chips(canvas: &mut Canvas, area: Rect, items: &[(&str, bool, bool)]) -> f32 {
    let line_height = canvas.line_height();
    let chip_height = chip_height(canvas);
    let names: Vec<&str> = items.iter().map(|(name, ..)| *name).collect();
    let (layout, height) = chip_layout(canvas, area.width, &names);
    for (&(name, pressed, supported), ([x, y], chip_width)) in items.iter().zip(layout) {
        let (x, y) = (area.x + x, area.y + y);
        if y + chip_height > area.bottom() + 0.5 {
            break;
        }
        let text_y = y + ((chip_height - line_height) * 0.5).round();
        if pressed {
            canvas.rect(x, y, chip_width, chip_height, PRESSED);
            canvas.text(x + line_height * 0.4, text_y, name, PRESSED_TEXT);
        } else {
            canvas.rect_outline(x, y, chip_width, chip_height, 1.0, BORDER);
            canvas.text(
                x + line_height * 0.4,
                text_y,
                name,
                if supported { DIM } else { FAINT },
            );
        }
    }
    height
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
        &view.pad_trail,
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
        .map(|&(button, name)| (name, state.buttons & button != 0, true))
        .collect();
    let values_y = text_area.y + chips(canvas, text_area, &buttons) + line_height * 0.2;
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

// Adds a target for each device's row, and returns the device whose row the
// cursor is over, if any.
fn draw_devices<'a>(
    canvas: &mut Canvas,
    input: &'a Input,
    status: &Status,
    area: Rect,
    targets: &mut Targets,
) -> Option<&'a Device> {
    let line_height = canvas.line_height();
    let inner = panel(
        canvas,
        area,
        "Devices",
        &format!("{} connected", input.devices.len()),
    );
    let mut lines = Lines::new(inner, line_height);
    if input.devices.is_empty() {
        lines.line(canvas, "None yet", FAINT);
    }
    // One line each: the name, then what kind of device it is. If they don't
    // all fit, the last line says how many more there are.
    let rows = ((inner.height + 0.5) / line_height).floor() as usize;
    let shown = if input.devices.len() > rows {
        rows.saturating_sub(1)
    } else {
        input.devices.len()
    };
    let mut hovered = None;
    // The gamepad the panel shows is highlighted, and those ignored are faint.
    for info in input.devices.iter().take(shown) {
        // Each row reaches halfway into the panel's padding on both sides.
        let row = Rect::new(
            inner.x - PADDING * 0.5,
            lines.y,
            inner.width + PADDING,
            line_height,
        );
        if status.cursor.is_some_and(|point| row.contains(point)) {
            canvas.rect(row.x, row.y, row.width, row.height, BORDER);
            hovered = Some(info);
        }
        targets.device_rows.push((row, info.label.clone()));
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
        lines.y += line_height;
    }
    if shown < input.devices.len() {
        lines.line(
            canvas,
            &format!("and {} more", input.devices.len() - shown),
            DIM,
        );
    }
    hovered
}

// What's known about a device, in a box by the cursor: under it, or above
// it if there isn't room, and always inside the window. Returns its area.
fn draw_tooltip(
    canvas: &mut Canvas,
    device: &Device,
    cursor: [f32; 2],
    width: f32,
    height: f32,
) -> Rect {
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let indent = char_width * 2.0;
    let section_gap = (line_height * 0.4).round();

    // Lay out the text first, from the box's top left, to know its size.
    let mut texts: Vec<([f32; 2], String, Color)> = Vec::new();
    let mut y = 0.0;
    let mut right: f32 = canvas.text_width(&device.label);
    texts.push(([0.0, y], device.label.clone(), TITLE));
    y += line_height;
    for section in device.details() {
        y += section_gap;
        right = right.max(canvas.text_width(&section.title));
        texts.push(([0.0, y], section.title, theme::ACCENT));
        y += line_height;
        // The values line up after the section's longest name.
        let value_x = indent
            + section
                .rows
                .iter()
                .map(|(name, _)| canvas.text_width(name))
                .fold(0.0, f32::max)
            + char_width * 2.0;
        for (name, value) in section.rows {
            texts.push(([indent, y], name, DIM));
            for part in value.split('\n') {
                for line in wrap(part, TOOLTIP_COLUMNS, 0) {
                    right = right.max(value_x + canvas.text_width(&line));
                    texts.push(([value_x, y], line, TEXT));
                    y += line_height;
                }
            }
        }
    }

    let (box_width, box_height) = (right + PADDING * 2.0, y + PADDING * 2.0);
    let below = cursor[1] + line_height * 1.4;
    let top = if below + box_height <= height - MARGIN {
        below
    } else {
        (cursor[1] - line_height * 0.6 - box_height).max(MARGIN)
    };
    let left = (cursor[0] + char_width)
        .min(width - MARGIN - box_width)
        .max(MARGIN);
    let area = Rect::new(left, top, box_width, box_height);
    // A shadow, like the select box's list has.
    canvas.rect(
        area.x + 3.0,
        area.y + 3.0,
        area.width,
        area.height,
        Color::rgb(0x000000).alpha(0.15),
    );
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, DIM);
    for ([x, y], text, color) in texts {
        canvas.text(area.x + PADDING + x, area.y + PADDING + y, &text, color);
    }
    area
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
