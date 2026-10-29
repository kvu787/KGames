// Lays out and draws the demo's panels onto a Canvas. Sizes come from the
// font's cell size, so the layout scales with the monitor's DPI.

use std::collections::VecDeque;

use crate::canvas::{Canvas, Color};
use crate::frame_counter::FrameCounter;
use crate::game_input::*;
use crate::input::{
    GAMEPAD_BUTTONS, Input, LEFT_DEAD_ZONE, Listen, MOUSE_BUTTONS, RIGHT_DEAD_ZONE, Source,
    describe_key,
};

pub const BACKGROUND: [f32; 4] = [0.071, 0.078, 0.094, 1.0];
const PANEL: Color = Color::rgb(0x1C1F26);
const PLOT: Color = Color::rgb(0x15171C);
const BORDER: Color = Color::rgb(0x2E333D);
const TITLE: Color = Color::rgb(0xFFFFFF);
const TEXT: Color = Color::rgb(0xD7DAE0);
const DIM: Color = Color::rgb(0x8B93A1);
const FAINT: Color = Color::rgb(0x4A515E);
const LEFT: Color = Color::rgb(0x4FC1FF);
const RIGHT: Color = Color::rgb(0xFF9F43);
const PRESSED: Color = Color::rgb(0x3DDC84);
const PRESSED_TEXT: Color = Color::rgb(0x0B2414);
const WARNING: Color = Color::rgb(0xFFCC66);
const ERROR: Color = Color::rgb(0xFF6B6B);

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    fn bottom(&self) -> f32 {
        self.y + self.height
    }

    fn right(&self) -> f32 {
        self.x + self.width
    }

    fn contains(&self, [x, y]: [f32; 2]) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

// What a click does at a place in the window.
pub enum Click {
    // Opens or closes the gamepad select box's list.
    Picker,
    Choose(Listen),
}

// The places in the last frame drawn that react to clicks, topmost last.
#[derive(Default)]
pub struct Targets(Vec<(Rect, Click)>);

impl Targets {
    pub fn at(&self, point: [f32; 2]) -> Option<&Click> {
        self.0
            .iter()
            .rev()
            .find(|(area, _)| area.contains(point))
            .map(|(_, click)| click)
    }
}

// Writes lines of text downward, and stops once they no longer fit.
struct Lines {
    x: f32,
    y: f32,
    bottom: f32,
    width: f32,
    line_height: f32,
}

impl Lines {
    fn new(area: Rect, line_height: f32) -> Self {
        Self {
            x: area.x,
            y: area.y,
            bottom: area.bottom(),
            width: area.width,
            line_height,
        }
    }

    fn has_room(&self) -> bool {
        self.y + self.line_height <= self.bottom + 0.5
    }

    fn line(&mut self, canvas: &mut Canvas, text: &str, color: Color) {
        if self.has_room() {
            canvas.text_clipped(self.x, self.y, text, self.width, color);
            self.y += self.line_height;
        }
    }

    // A label in one color followed by a value in another.
    fn pair(&mut self, canvas: &mut Canvas, label: &str, value: &str, color: Color) {
        if self.has_room() {
            let end = canvas.text(self.x, self.y, label, DIM);
            canvas.text_clipped(end, self.y, value, self.right() - end, color);
            self.y += self.line_height;
        }
    }

    fn wrapped(&mut self, canvas: &mut Canvas, text: &str, indent: &str, color: Color) {
        let columns = (self.width / canvas.char_width()).floor().max(8.0) as usize;
        for (index, line) in wrap(text, columns, indent.chars().count())
            .iter()
            .enumerate()
        {
            let prefix = if index == 0 { "" } else { indent };
            self.line(canvas, &format!("{prefix}{line}"), color);
        }
    }

    fn gap(&mut self, fraction: f32) {
        self.y += self.line_height * fraction;
    }

    fn right(&self) -> f32 {
        self.x + self.width
    }
}

// What the header and footer show besides input.
pub struct Status<'a> {
    pub focused: bool,
    pub frames: &'a FrameCounter,
    // The display the window is on, like "KXGR16 (PA278QGV)".
    pub display: &'a str,
    // Problems with how the demo was started, one per paragraph.
    pub notices: &'a [String],
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
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let margin = (line_height * 0.6).round();
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
            let version = input.runtime.version.map(|part| part.to_string()).join(".");
            x = canvas.text(margin, status_y, "Runtime: ", DIM);
            x = canvas.text(
                x,
                status_y,
                &format!("{} {}", file_name(&input.runtime.path), version),
                TEXT,
            );
            let location = input
                .runtime
                .path
                .parent()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            let count = input.reading_count.to_string();
            let count_label = "   Readings handled: ";
            let count_width = canvas.text_width(count_label) + canvas.text_width(&count);
            let location_width = (status_right - x - count_width).min(width * 0.45);
            x = canvas.text_clipped(x, status_y, &format!(" in {location}"), location_width, DIM);
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
        "Window doesn't have focus: GameInput stops keyboard and mouse input"
    };
    let focus_width = canvas.text_width(focus_text);
    canvas.text(
        width - margin - focus_width,
        margin,
        focus_text,
        if focused { PRESSED } else { WARNING },
    );

    // Notices go under the status, wrapped to the window's width.
    let mut header_bottom = status_y + line_height;
    let columns = ((width - margin * 2.0) / char_width).floor().max(8.0) as usize;
    for notice in status.notices {
        for (index, line) in wrap(notice, columns, 2).iter().enumerate() {
            let indent = if index == 0 { "" } else { "  " };
            canvas.text(margin, header_bottom, &format!("{indent}{line}"), WARNING);
            header_bottom += line_height;
        }
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

    let top = margin + header_bottom;
    let bottom = footer_y - margin;
    let Ok(input) = input else {
        return Targets::default();
    };

    // Columns: gamepad, keyboard and mouse on the left; the latest event and
    // the devices on the right; the log across the bottom.
    let right_width = (width * 0.36).clamp(char_width * 40.0, char_width * 72.0);
    let left_width = width - margin * 3.0 - right_width;
    let right_x = margin * 2.0 + left_width;
    let padding = (line_height * 0.5).round();
    // A panel's padding above and below, and its title.
    let title_height = padding * 2.0 + line_height * 1.4;

    // Rows are as tall as their contents, including however many rows of
    // chips the width needs. The stick graphs grow into the space that's
    // left, and the log gets the rest.
    let gamepad_names: Vec<&str> = GAMEPAD_BUTTONS.iter().map(|(_, name)| *name).collect();
    let (_, gamepad_chips) = chip_layout(canvas, left_width - padding * 2.0, &gamepad_names);
    // Captions, two lines of values and a status line, around the graphs and chips.
    let gamepad_height_without_graph = title_height + line_height * 4.6 + padding + gamepad_chips;

    let keyboard_width = ((left_width - margin) * 0.45).floor();
    let mouse_width = left_width - margin - keyboard_width;
    let mouse_names: Vec<&str> = MOUSE_BUTTONS.iter().map(|(_, name)| *name).collect();
    let (_, mouse_chips) = chip_layout(
        canvas,
        mouse_text_width(mouse_width - padding * 2.0, line_height, padding),
        &mouse_names,
    );
    // The mouse's chips, four lines of values and a status line, or its
    // motion pad and caption, whichever is taller.
    let middle_height = title_height + (mouse_chips + line_height * 5.2).max(line_height * 6.1);

    let bar_width = (char_width * 5.0).round();
    let inner_gap = padding * 2.0;
    let available = bottom - top;
    let log_minimum = title_height + line_height * 3.0;
    let graph_by_width = (left_width - padding * 2.0 - bar_width * 2.0 - inner_gap * 3.0) / 2.0;
    let graph_by_height =
        (available - gamepad_height_without_graph - middle_height - margin * 2.0 - log_minimum)
            .min(available * 0.3);
    // At least wide enough for the values under it, like "x +0.000  y +0.000".
    let graph = graph_by_width
        .min(graph_by_height)
        .clamp(char_width * 16.0, line_height * 14.0)
        .floor();

    let gamepad_height = gamepad_height_without_graph + graph;
    let gamepad_area = Rect::new(margin, top, left_width, gamepad_height);
    let middle_y = gamepad_area.bottom() + margin;
    let keyboard_area = Rect::new(margin, middle_y, keyboard_width, middle_height);
    let mouse_area = Rect::new(
        margin * 2.0 + keyboard_width,
        middle_y,
        mouse_width,
        middle_height,
    );
    let latest_area = Rect::new(right_x, top, right_width, gamepad_height);
    let devices_area = Rect::new(right_x, middle_y, right_width, middle_height);
    let log_y = middle_y + middle_height + margin;
    let log_area = Rect::new(margin, log_y, width - margin * 2.0, bottom - log_y);

    let mut targets = Targets::default();
    let picker = draw_gamepad(
        canvas,
        input,
        status,
        gamepad_area,
        padding,
        graph,
        bar_width,
        inner_gap,
    );
    targets.0.push((picker, Click::Picker));
    draw_latest(canvas, input, latest_area, padding);
    draw_keyboard(canvas, input, keyboard_area, padding);
    draw_mouse(canvas, input, mouse_area, padding);
    draw_devices(canvas, input, devices_area, padding);
    if log_area.height > line_height * 2.0 {
        draw_log(canvas, input, log_area, padding);
    }
    // Last, so the list covers the panels under it.
    if status.picker_open {
        draw_picker_list(canvas, input, status, picker, width - margin, &mut targets);
    }
    targets
}

// Draws a panel's background and title, and returns the area under the title.
fn panel(canvas: &mut Canvas, area: Rect, padding: f32, title: &str, subtitle: &str) -> Rect {
    panel_with(canvas, area, padding, title, subtitle, DIM)
}

fn panel_with(
    canvas: &mut Canvas,
    area: Rect,
    padding: f32,
    title: &str,
    subtitle: &str,
    color: Color,
) -> Rect {
    canvas.rect(area.x, area.y, area.width, area.height, PANEL);
    canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
    let x = canvas.text(area.x + padding, area.y + padding, title, TITLE);
    if !subtitle.is_empty() {
        canvas.text_clipped(
            x,
            area.y + padding,
            &format!("  {subtitle}"),
            area.right() - padding - x,
            color,
        );
    }
    let top = area.y + padding + canvas.line_height() * 1.4;
    Rect::new(
        area.x + padding,
        top,
        area.width - padding * 2.0,
        area.bottom() - padding - top,
    )
}

// Returns the area of the select box for which gamepads to listen to.
fn draw_gamepad(
    canvas: &mut Canvas,
    input: &Input,
    status: &Status,
    area: Rect,
    padding: f32,
    graph: f32,
    bar_width: f32,
    gap: f32,
) -> Rect {
    let view = &input.gamepad;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, padding, "Gamepad", "");
    let picker = draw_picker_box(canvas, input, status, area, padding);
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
        LEFT_DEAD_ZONE,
        LEFT,
    );
    stick_graph(
        canvas,
        Rect::new(right_x, plots_y, graph, graph),
        right,
        &view.right_trail,
        RIGHT_DEAD_ZONE,
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
    let chips_y = values_y + line_height * 2.0 + padding;
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
fn draw_picker_box(
    canvas: &mut Canvas,
    input: &Input,
    status: &Status,
    area: Rect,
    padding: f32,
) -> Rect {
    let line_height = canvas.line_height();
    let char_width = canvas.char_width();
    let text_y = area.y + padding;
    let height = chip_height(canvas);
    let x = area.x + padding + canvas.text_width("Gamepad") + char_width * 2.0;
    let y = (text_y - (height - line_height) * 0.5).round();
    let inset = (line_height * 0.4).round();
    let arrow_width = line_height;
    let text = listen_label(input, &input.listen);
    let width =
        (canvas.text_width(&text) + inset * 2.0 + arrow_width).min(area.right() - padding - x);
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
    canvas.text_clipped(x + inset, text_y, &text, width - inset - arrow_width, TEXT);
    chevron(
        canvas,
        [picker.right() - arrow_width * 0.5, y + height * 0.5],
        (line_height * 0.2).round(),
        status.picker_open,
        DIM,
    );

    let (note, color) = match &input.listen {
        Listen::AnyGamepad => match &input.gamepad.device {
            Some(_) => (format!("showing {}", input.gamepad.label), DIM),
            None => (
                "No input yet: connect a gamepad, then press a button or move a stick".to_string(),
                WARNING,
            ),
        },
        Listen::Gamepad(device) if !input.gamepads().any(|info| info.device == *device) => (
            "Disconnected: waiting for it to reconnect".to_string(),
            WARNING,
        ),
        Listen::Gamepad(_) => (String::new(), DIM),
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
    let mut choices = vec![(
        Listen::AnyGamepad,
        listen_label(input, &Listen::AnyGamepad),
        "follows the one used last".to_string(),
    )];
    for info in input.gamepads() {
        let listen = Listen::Gamepad(info.device.clone());
        choices.push((listen, info.label.clone(), info.family_name().to_string()));
    }
    // A chosen gamepad stays in the list while it's disconnected.
    if let Listen::Gamepad(device) = &input.listen
        && !input.gamepads().any(|info| info.device == *device)
    {
        choices.push((
            input.listen.clone(),
            listen_label(input, &input.listen),
            "disconnected".to_string(),
        ));
    }
    choices.push((
        Listen::NoGamepad,
        listen_label(input, &Listen::NoGamepad),
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
        Color::rgb(0x000000).alpha(0.5),
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
        targets.0.push((row, Click::Choose(listen)));
    }
}

fn listen_label(input: &Input, listen: &Listen) -> String {
    match listen {
        Listen::AnyGamepad => "Any gamepad".to_string(),
        Listen::Gamepad(device) => input.label_of(Some(device)),
        Listen::NoGamepad => "None".to_string(),
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
    canvas.ring(center, radius * dead_zone, 1.0, WARNING.alpha(0.45));

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

fn draw_latest(canvas: &mut Canvas, input: &Input, area: Rect, padding: f32) {
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, padding, "Latest input event", "");
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

fn draw_keyboard(canvas: &mut Canvas, input: &Input, area: Rect, padding: f32) {
    let view = &input.keyboard;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, padding, "Keyboard", &view.label);
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
    for key in view.keys.iter().take(room) {
        lines.line(canvas, &format!("  {}", describe_key(key)), PRESSED);
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

fn draw_mouse(canvas: &mut Canvas, input: &Input, area: Rect, padding: f32) {
    let view = &input.mouse;
    let state = &view.state;
    let line_height = canvas.line_height();
    let inner = panel(canvas, area, padding, "Mouse", &view.label);

    // Raw relative motion, without the pointer acceleration the cursor has.
    let pad_size = mouse_pad_size(inner.width, line_height)
        .min(inner.height - line_height * 1.1)
        .floor();
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
        inner.width - pad_size - padding,
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

// The mouse panel's motion pad is at most 5 lines tall, so that the panel
// isn't taller than its values need.
fn mouse_pad_size(inner_width: f32, line_height: f32) -> f32 {
    (inner_width * 0.4).min(line_height * 5.0).floor()
}

// The width beside the motion pad, for the mouse's chips and values.
fn mouse_text_width(inner_width: f32, line_height: f32, padding: f32) -> f32 {
    inner_width - mouse_pad_size(inner_width, line_height) - padding
}

fn draw_devices(canvas: &mut Canvas, input: &Input, area: Rect, padding: f32) {
    let line_height = canvas.line_height();
    let inner = panel(
        canvas,
        area,
        padding,
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
    // The gamepad the panel shows is highlighted, and those ignored are faint.
    for info in input.devices.iter().take(shown) {
        let color = if input.gamepad.device.as_ref() == Some(&info.device) {
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
}

fn draw_log(canvas: &mut Canvas, input: &Input, area: Rect, padding: f32) {
    let line_height = canvas.line_height();
    let inner = panel(
        canvas,
        area,
        padding,
        "Event log",
        "newest first; repeated motion from one device shares a line",
    );
    let mut lines = Lines::new(inner, line_height);
    for entry in &input.log {
        if !lines.has_room() {
            break;
        }
        let mut x = canvas.text(lines.x, lines.y, &format!("{:>8.3}  ", entry.time), DIM);
        x = canvas.text(
            x,
            lines.y,
            &format!("{:<9}", entry.source.name()),
            source_color(entry.source),
        );
        let mut text = entry.text.clone();
        if entry.count > 1 {
            text += &format!("  (x{})", entry.count);
        }
        canvas.text_clipped(x, lines.y, &text, inner.right() - x, TEXT);
        lines.y += line_height;
    }
}

fn source_color(source: Source) -> Color {
    match source {
        Source::Gamepad => LEFT,
        Source::Keyboard => Color::rgb(0xC792EA),
        Source::Mouse => WARNING,
        Source::Device => PRESSED,
        Source::Demo => DIM,
    }
}

fn rumble_motor_names(motors: GameInputRumbleMotors) -> String {
    let names: Vec<&str> = [
        (GameInputRumbleLowFrequency, "low frequency"),
        (GameInputRumbleHighFrequency, "high frequency"),
        (GameInputRumbleLeftTrigger, "left trigger"),
        (GameInputRumbleRightTrigger, "right trigger"),
    ]
    .iter()
    .filter(|(motor, _)| motors & motor != 0)
    .map(|(_, name)| *name)
    .collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

// Splits text into lines of at most `columns` characters at spaces. Lines
// after the first are `indent` characters narrower, for a hanging indent.
fn wrap(text: &str, columns: usize, indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let limit = if lines.is_empty() {
            columns
        } else {
            columns.saturating_sub(indent).max(1)
        };
        let length = line.chars().count();
        if length > 0 && length + 1 + word.chars().count() > limit {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push(line);
    lines
}
