// Lays out and draws the showcase's cards, with a widget of each kind that
// Library/UserInterface has, and handles what the input does to them. The
// showcase supports two client areas, 2560 x 1369 maximized and 2560 x 1440
// in borderless fullscreen (see Library/Application), so the cards have
// fixed places in pixels, in four columns. In fullscreen, the cards that
// stretch, one in each column, are 71 pixels taller: the tabs, the tooltips,
// the canvas, and the list.

use user_interface::{
    Align, Button, ButtonStyle, CARD_PADDING, CONTROL_HEIGHT, Cached, Canvas, Chip, Choice, Color, Frame,
    ICON_SIZES, Icon, Lines, MARGIN, PANELS_TOP, Rect, SPACE_L, SPACE_M, SPACE_S, SPACE_XS, Sense, Severity,
    SliderRange, Stack, TextField, TextStyle, Theme, TooltipText, select_width, shadow, toggle_width, well,
};

use crate::showcase::{
    COLORS, DENSITIES, ROW_HEIGHTS, SIZES, STYLES, Showcase, TABS, TAGS, VIEWS, on_off,
};

// The cards' heights, but for the one in each column that stretches.
const BUTTONS_HEIGHT: f32 = 236.0;
const TOGGLES_HEIGHT: f32 = 336.0;
const CHOICES_HEIGHT: f32 = 236.0;
const SLIDERS_HEIGHT: f32 = 312.0;
const SELECTS_HEIGHT: f32 = 236.0;
const CHIPS_HEIGHT: f32 = 214.0;
const TEXT_FIELDS_HEIGHT: f32 = 312.0;
const INFO_BARS_HEIGHT: f32 = 288.0;
const INTERFACE_HEIGHT: f32 = 432.0;
// The tooltips card's boxes to hover over.
const HOVER_HEIGHT: f32 = 72.0;

// A row of a slider, and its label and value on each side of it.
const SLIDER_ROW: f32 = 40.0;
const SLIDER_LABEL: f32 = 96.0;
const SLIDER_VALUE: f32 = 88.0;
const PROGRESS_HEIGHT: f32 = 6.0;
const NAME_WIDTH: f32 = 360.0;
const SWATCH: f32 = 28.0;
const SELECT_WIDTH: f32 = 220.0;

// What draw keeps from frame to frame: the canvas card's shapes, which
// never change while the theme doesn't, by the card's area and the theme.
#[derive(Default)]
pub struct Cache {
    shapes: Cached<([f32; 4], &'static str)>,
}

pub fn draw(showcase: &mut Showcase, frame: &mut Frame) {
    frame.header("UI Showcase", "Every widget of Library/UserInterface, after Fluent 2");
    let theme = frame.theme();
    let adapter = frame.status.adapter;
    let bottom = frame.footer(
        &[
            ("F11", "fullscreen"),
            ("Tab", "next widget"),
            ("Shift+Tab", "previous"),
            ("Esc", "close or unfocus"),
            ("F1", "tooltips"),
            ("F2", "dark theme"),
            ("Ctrl+R", "reset"),
        ],
        &[(adapter, theme.text_secondary)],
    );
    let columns = Rect::from_edges(MARGIN, PANELS_TOP, frame.width - MARGIN, bottom).columns(4, MARGIN);

    let (buttons, rest) = columns[0].split_top(BUTTONS_HEIGHT, MARGIN);
    let (toggles, rest) = rest.split_top(TOGGLES_HEIGHT, MARGIN);
    let (choices, tabs) = rest.split_top(CHOICES_HEIGHT, MARGIN);
    draw_buttons(showcase, frame, buttons);
    draw_toggles(showcase, frame, toggles);
    draw_choices(showcase, frame, choices);
    draw_tabs(showcase, frame, tabs);

    let (sliders, rest) = columns[1].split_top(SLIDERS_HEIGHT, MARGIN);
    let (selects, rest) = rest.split_top(SELECTS_HEIGHT, MARGIN);
    let (chips, tooltips) = rest.split_top(CHIPS_HEIGHT, MARGIN);
    draw_sliders(showcase, frame, sliders);
    draw_selects(showcase, frame, selects);
    draw_chips(showcase, frame, chips);
    draw_tooltips(showcase, frame, tooltips);

    let (fields, rest) = columns[2].split_top(TEXT_FIELDS_HEIGHT, MARGIN);
    let (info_bars, canvas) = rest.split_top(INFO_BARS_HEIGHT, MARGIN);
    draw_text_fields(showcase, frame, fields);
    draw_info_bars(showcase, frame, info_bars);
    draw_canvas(frame, &mut showcase.cache, canvas);

    let (interface, list) = columns[3].split_bottom(INTERFACE_HEIGHT, MARGIN);
    draw_list(showcase, frame, list);
    draw_interface(showcase, frame, interface);
}

// A tooltip about a widget, while the cursor is over it, if they're on.
fn tip(showcase: &Showcase, frame: &mut Frame, name: &str, area: Rect, hovered: bool, text: &str) {
    if showcase.tooltips && hovered {
        frame.tooltip_text(name, area, text);
    }
}

// A caption over a widget, and the area under it.
fn caption(frame: &mut Frame, stack: &mut Stack, text: &str) {
    let theme = frame.theme();
    let area = stack.next(frame.line_height(TextStyle::Body));
    frame.canvas.text([area.x, area.y], text, TextStyle::Body, theme.text_secondary);
}

fn draw_buttons(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Buttons", "act on the press, which comes before the release");
    let theme = frame.theme();
    let mut stack = Stack::down(inner, SPACE_M);
    let mut row = Stack::across(stack.next(CONTROL_HEIGHT), SPACE_S);
    let helps = [
        "A button for most actions.",
        "The action that a card is for, in the accent color.",
        "An action that throws something away.",
        "A button among others, like in a row of tools, which shows only while the cursor is over it.",
    ];
    for (index, (label, help)) in STYLES.iter().zip(helps).enumerate() {
        let style = [ButtonStyle::Standard, ButtonStyle::Accent, ButtonStyle::Danger, ButtonStyle::Subtle][index];
        let button = Button::new(label).style(style);
        let place = row.next(frame.button_width(&button).max(96.0));
        let response = frame.button(label, place, button);
        if response.clicked {
            showcase.presses[index] += 1;
            let count = showcase.presses[index];
            showcase.record(label, format!("pressed {count} time{}", if count == 1 { "" } else { "s" }));
        }
        tip(showcase, frame, label, place, response.hovered, help);
    }
    let place = row.next(96.0);
    let enabled = frame.set_enabled(false);
    let response = frame.button("Disabled", place, Button::new("Disabled"));
    frame.set_enabled(enabled);
    let help = "Disabled widgets are faint and take no input, but still show their tooltips.";
    tip(showcase, frame, "Disabled", place, response.hovered, help);

    // Buttons with icons, and icons alone, which add to and take from a
    // count.
    let mut row = Stack::across(stack.next(CONTROL_HEIGHT), SPACE_S);
    let reset = Button::new("Reset count").icon(Icon::Refresh);
    if frame.button("Reset count", row.next(frame.button_width(&reset)), reset).clicked {
        showcase.count = 0;
        showcase.record("Count", "0".to_string());
    }
    for (name, icon, step) in [("Remove", Icon::Remove, -1), ("Add", Icon::Add, 1)] {
        let place = row.next(CONTROL_HEIGHT);
        let response = frame.button(name, place, Button::icon_only(icon));
        if response.clicked {
            showcase.count += step;
            showcase.record("Count", showcase.count.to_string());
        }
        tip(showcase, frame, name, place, response.hovered, &format!("{name} one. A button can be an icon alone."));
    }
    let count = row.next(120.0);
    frame.label(count, &format!("Count: {}", showcase.count), TextStyle::BodyStrong, theme.text, Align::Start);

    let mut lines = Lines::new(stack.rest());
    let presses: Vec<String> = STYLES
        .iter()
        .zip(showcase.presses)
        .map(|(style, count)| format!("{count} {}", style.to_lowercase()))
        .collect();
    lines.pair(&mut frame.canvas, "Presses", &presses.join(", "), 64.0, theme.text_secondary, theme.text);
    lines.gap(SPACE_XS);
    lines.wrapped(
        &mut frame.canvas,
        "Space and Enter press the button with keyboard focus, which Tab moves. A press that gives the \
         window focus does only that.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

fn draw_toggles(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Checkboxes and switches", "a press, or Space with focus, flips them");
    let theme = frame.theme();
    let fonts = frame.fonts();
    let mut stack = Stack::down(inner, 0.0);
    let label = "Enable the four under this";
    let row = stack.next(CONTROL_HEIGHT);
    let row = Rect::new(row.x, row.y, toggle_width(fonts, label, false), row.height);
    if frame.checkbox("Enable", row, label, &mut showcase.enabled).changed {
        showcase.record("Enable", on_off(showcase.enabled));
    }
    let enabled = frame.set_enabled(showcase.enabled);
    let indent = 28.0;
    for name in ["Grid", "Snap", "Sound"] {
        let (label, value) = match name {
            "Grid" => ("Show the grid", &mut showcase.grid),
            "Snap" => ("Snap to the grid", &mut showcase.snap),
            _ => ("Play sounds", &mut showcase.sound),
        };
        let row = stack.next(CONTROL_HEIGHT);
        let row = Rect::new(row.x + indent, row.y, toggle_width(fonts, label, false), row.height);
        let changed = frame.checkbox(name, row, label, value).changed;
        let what = on_off(*value);
        if changed {
            showcase.record(name, what);
        }
    }
    let label = "Animate the progress bar";
    let row = stack.next(CONTROL_HEIGHT);
    let row = Rect::new(row.x + indent, row.y, toggle_width(fonts, label, true), row.height);
    if frame.switch("Animate", row, label, &mut showcase.animate).changed {
        showcase.record("Animate", on_off(showcase.animate));
    }
    frame.set_enabled(enabled);
    stack.skip(SPACE_XS);
    for (name, label, help) in [
        ("Tooltips", "Show tooltips (F1)", "Tooltips show at once, by the cursor, and never cover it."),
        ("Dark", "Dark theme (F2)", "Every color of the interface is a role, which the light and dark themes fill."),
    ] {
        let row = stack.next(CONTROL_HEIGHT);
        let row = Rect::new(row.x, row.y, toggle_width(fonts, label, true), row.height);
        let value = if name == "Dark" { &mut showcase.dark } else { &mut showcase.tooltips };
        let response = frame.switch(name, row, label, value);
        let what = on_off(*value);
        if response.changed {
            showcase.record(if name == "Dark" { "Dark theme" } else { name }, what);
        }
        tip(showcase, frame, name, row, response.hovered, help);
    }
    stack.skip(SPACE_S);
    let mut lines = Lines::new(stack.rest());
    lines.wrapped(
        &mut frame.canvas,
        "Turning off the first checkbox disables the four under it. A switch's knob slides over.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

fn draw_choices(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Radio buttons and segments", "the arrow keys move the choice with focus");
    let theme = frame.theme();
    let group = Rect::new(inner.x, inner.y, 200.0, CONTROL_HEIGHT * SIZES.len() as f32);
    if frame.radio("Size", group, &SIZES, &mut showcase.size).changed {
        showcase.record("Size", SIZES[showcase.size].to_string());
    }
    // A square of the chosen size, with the corners of a card.
    let side = [24.0, 44.0, 64.0][showcase.size];
    let middle = group.center()[1];
    let square = Rect::new(group.right() + 40.0, (middle - side * 0.5).round(), side, side);
    frame.canvas.fill(square, 8.0, theme.accent);
    let segmented = Rect::new(inner.x, group.bottom() + SPACE_L, 320.0, CONTROL_HEIGHT);
    if frame.segmented("View", segmented, &VIEWS, &mut showcase.view).changed {
        showcase.record("View", VIEWS[showcase.view].to_string());
    }
    let text = Rect::from_edges(segmented.right() + SPACE_L, segmented.y, inner.right(), segmented.bottom());
    let what = format!("Showing a {}", VIEWS[showcase.view].to_lowercase());
    frame.label(text, &what, TextStyle::Body, theme.text_secondary, Align::Start);
}

fn draw_tabs(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Tabs", "Left and Right move the choice while they have focus");
    let theme = frame.theme();
    let bar = Rect::new(inner.x - user_interface::TEXT_INSET, inner.y - SPACE_S, inner.width, user_interface::TAB_HEIGHT);
    if frame.tabs("Tabs", bar, &TABS, &mut showcase.tab).changed {
        showcase.record("Tab", TABS[showcase.tab].to_string());
    }
    frame.divider([inner.x, bar.bottom()], inner.width);
    let text = Rect::from_edges(inner.x, bar.bottom() + SPACE_M, inner.right(), inner.bottom());
    let mut lines = Lines::new(text);
    let canvas = &mut frame.canvas;
    let paragraph = |lines: &mut Lines, canvas: &mut Canvas, text: &str| {
        lines.wrapped(canvas, text, TextStyle::Body, theme.text, 0.0);
        lines.gap(SPACE_S);
    };
    match showcase.tab {
        0 => {
            paragraph(
                &mut lines,
                canvas,
                "Library/UserInterface is the interface of every KGames program on Win32, Direct3D 12 and \
                 DirectWrite. It's immediate mode: each frame, the program calls a function for each widget, \
                 with where it goes and the value it shows, and the function draws it, and says what the input \
                 did to it.",
            );
            paragraph(
                &mut lines,
                canvas,
                "It looks like Windows 11: Fluent 2's controls, type ramp and colors, in Segoe UI Variable, with \
                 Segoe Fluent Icons.",
            );
            paragraph(
                &mut lines,
                canvas,
                "Input goes to what the frame before drew, which is what was on the screen when it came. The \
                 frame reads it right before it's drawn, so that it shows the newest.",
            );
        }
        1 => {
            let rows = [
                ("Button", "standard, accent, danger, subtle, with an icon"),
                ("Checkbox", "on or off, with a label"),
                ("Switch", "on or off, with a knob that slides"),
                ("Radio", "one of a column of choices"),
                ("Segmented", "one of a row of choices"),
                ("Tabs", "one of a row of pages"),
                ("Slider", "a number, in steps, between two"),
                ("Combo box", "one of a list that opens over the cards"),
                ("Text field", "a line of text, which selects and copies"),
                ("List", "rows that scroll by pixels"),
                ("Chips", "labels that are on or off"),
                ("Info bar", "a message, with how serious it is"),
                ("Tooltip", "a card of text by the cursor"),
            ];
            for (kind, about) in rows {
                lines.pair(canvas, kind, about, 104.0, theme.text_secondary, theme.text);
            }
        }
        _ => {
            paragraph(
                &mut lines,
                canvas,
                "Each widget has a name, which is the same from frame to frame, and which scripts see in the \
                 state that they save, with the widget's area, under Interface.Targets. Scripts can press a \
                 widget by its name.",
            );
            paragraph(
                &mut lines,
                canvas,
                "Popups, like a combo box's list, and tooltips are drawn on layers over the cards, in the same \
                 draw call.",
            );
        }
    }
}

fn draw_sliders(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Sliders", "drag, scroll, or the arrow keys with focus");
    let theme = frame.theme();
    let mut stack = Stack::down(inner, 0.0);
    let ranges = [
        ("Volume", SliderRange::new(0.0, 100.0, 1.0)),
        ("Opacity", SliderRange::new(0.0, 1.0, 1000.0)),
        (
            "Angle",
            SliderRange {
                start: 0.0,
                end: 360.0,
                minimum: 0.0,
                maximum: 359.9,
                steps: 10.0,
            },
        ),
        ("Locked", SliderRange::new(0.0, 1.0, 1000.0)),
    ];
    for (name, range) in ranges {
        let value = match name {
            "Volume" => &mut showcase.volume,
            "Opacity" => &mut showcase.opacity,
            "Angle" => &mut showcase.angle,
            _ => &mut showcase.locked,
        };
        let row = stack.next(SLIDER_ROW);
        let (label, rest) = row.split_left(SLIDER_LABEL, 0.0);
        let (number, track) = rest.split_right(SLIDER_VALUE, SPACE_M);
        let locked = name == "Locked";
        let enabled = frame.set_enabled(!locked);
        let response = frame.slider(name, track, value, &range);
        frame.set_enabled(enabled);
        let text = match name {
            "Volume" => format!("{:.0}%", *value),
            "Angle" => format!("{:.1}\u{B0}", *value),
            _ => format!("{:.3}", *value),
        };
        let changed = response.changed;
        let (label_color, value_color) = if locked {
            (theme.text_disabled, theme.text_disabled)
        } else {
            (theme.text, theme.text)
        };
        frame.label(label, name, TextStyle::Body, label_color, Align::Start);
        frame.label(number, &text, TextStyle::Mono, value_color, Align::End);
        if changed {
            showcase.record(name, text);
        }
    }

    // Progress bars: one at the volume, and one that fills over 4 seconds,
    // again and again, while it's animated.
    stack.skip(SPACE_S);
    let fill = (frame.seconds() * 0.25).fract() as f32;
    let animated = if showcase.animate { fill } else { 0.0 };
    for (label, fraction, color) in [
        ("Volume", (showcase.volume / 100.0) as f32, None),
        ("Every 4 seconds", animated, Some(theme.success)),
    ] {
        let line = stack.next(frame.line_height(TextStyle::Body));
        frame.label(line, label, TextStyle::Body, theme.text_secondary, Align::Start);
        frame.label(line, &format!("{:.0}%", fraction * 100.0), TextStyle::Mono, theme.text_secondary, Align::End);
        stack.skip(SPACE_S);
        let bar = stack.next(PROGRESS_HEIGHT);
        frame.progress(bar, fraction, color);
        stack.skip(SPACE_M);
    }
}

fn draw_selects(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Combo boxes", "Space or Enter opens, Up and Down pick, Esc closes");
    let theme = frame.theme();
    let colors: Vec<Choice> = COLORS.iter().map(|&(label, detail)| Choice { label, detail }).collect();
    let densities: Vec<Choice> = DENSITIES.iter().map(|&(label, detail)| Choice { label, detail }).collect();
    let width = select_width(frame.fonts(), "Comfortable").max(SELECT_WIDTH);
    let mut stack = Stack::down(inner, SPACE_S);

    caption(frame, &mut stack, "Color");
    let row = stack.next(CONTROL_HEIGHT);
    let color_box = Rect::new(row.x, row.y, width, CONTROL_HEIGHT);
    if frame.select("Color", color_box, &colors, &mut showcase.color).changed {
        showcase.record("Color", COLORS[showcase.color].0.to_string());
    }
    let swatch = Rect::new(color_box.right() + SPACE_M, row.y + 2.0, SWATCH, SWATCH);
    frame.canvas.fill(swatch, 6.0, color_of(theme, showcase.color));
    let text = Rect::from_edges(swatch.right() + SPACE_M, row.y, inner.right(), row.bottom());
    frame.label(text, COLORS[showcase.color].1, TextStyle::Body, theme.text_secondary, Align::Start);
    stack.skip(SPACE_XS);

    caption(frame, &mut stack, "List density");
    let row = stack.next(CONTROL_HEIGHT);
    let density_box = Rect::new(row.x, row.y, width, CONTROL_HEIGHT);
    if frame.select("Density", density_box, &densities, &mut showcase.density).changed {
        showcase.record("Density", DENSITIES[showcase.density].0.to_string());
    }
    stack.skip(SPACE_XS);
    let mut lines = Lines::new(stack.rest());
    lines.wrapped(
        &mut frame.canvas,
        "A list opens over the cards under it. A press off it closes it, and does nothing else.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

// The color that the color combo box's choice stands for.
fn color_of(theme: &Theme, choice: usize) -> Color {
    [theme.accent, theme.success, theme.caution, theme.critical, theme.orange(), theme.purple()][choice]
}

fn draw_chips(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Chips", "labels that are on or off, which wrap onto rows");
    let theme = frame.theme();
    let mut stack = Stack::down(inner, SPACE_S);
    caption(frame, &mut stack, "Filter chips, which a press turns on or off");
    let rest = stack.rest();
    let (changed, height) = frame.filter_chips("Tags", rest, &TAGS, &mut showcase.tags);
    if changed {
        let on = TAGS.iter().zip(showcase.tags).filter(|(_, on)| *on).count();
        showcase.record("Tags", format!("{on} on"));
    }
    stack.skip(height + SPACE_S);
    caption(frame, &mut stack, "What's on, from the checkboxes and switches");
    let items = [
        Chip { label: "Grid", on: showcase.grid, enabled: showcase.enabled },
        Chip { label: "Snap", on: showcase.snap, enabled: showcase.enabled },
        Chip { label: "Sound", on: showcase.sound, enabled: showcase.enabled },
        Chip { label: "Animate", on: showcase.animate, enabled: showcase.enabled },
        Chip { label: "Tooltips", on: showcase.tooltips, enabled: true },
        Chip { label: "Dark", on: showcase.dark, enabled: true },
    ];
    frame.status_chips(stack.rest(), &items, theme.success);
}

// Boxes that show tooltips: a sentence, one in columns, and one at the
// card's bottom, whose tooltip goes above the cursor, as there isn't room
// under it. Each box is a widget that the showcase makes itself, which only
// takes the cursor being over it.
fn draw_tooltips(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Tooltips", "at once, by the cursor, never over it, always in the window");
    let theme = frame.theme();
    let line_height = frame.line_height(TextStyle::Body);
    let columns = Rect::new(inner.x, inner.y, inner.width, HOVER_HEIGHT).columns(2, SPACE_M);
    let bottom = Rect::new(inner.x, inner.bottom() - HOVER_HEIGHT, inner.width, HOVER_HEIGHT);
    let boxes = [
        ("Sentence", columns[0], "Hover for a sentence"),
        ("Columns", columns[1], "Hover for columns"),
        ("Bottom", bottom, "Hover here, where there's no room under the cursor"),
    ];
    for (name, area, label) in boxes {
        let response = frame.interact(name, "Hover", area, Sense::HOVER);
        let hover = frame.ease(user_interface::Id::new(name), response.hovered as u8 as f32);
        let fill = theme.well.mix(theme.subtle_hover.over(theme.well), hover);
        frame.canvas.fill_stroke(area, 6.0, fill, 1.0, theme.well_stroke);
        frame.label(area, label, TextStyle::Body, theme.text_secondary, Align::Center);
        if !(showcase.tooltips && response.hovered) {
            continue;
        }
        if name == "Columns" {
            let rows = [
                ("Layer", "the overlay, over popups and cards"),
                ("Placement", "under the cursor, or above it, or beside"),
                ("Size", "as big as its text"),
                ("Shows", "while what it's about is under the cursor"),
            ];
            let value_x = 96.0;
            let mut pieces: Vec<TooltipText<String>> =
                vec![([0.0, 0.0], "A tooltip in columns".to_string(), TextStyle::BodyStrong, theme.text)];
            for (index, (label, value)) in rows.iter().enumerate() {
                let y = line_height * (index as f32 + 1.0) + SPACE_XS;
                pieces.push(([0.0, y], label.to_string(), TextStyle::Body, theme.text_secondary));
                pieces.push(([value_x, y], value.to_string(), TextStyle::Body, theme.text));
            }
            frame.tooltip(name, area, &pieces);
        } else {
            let text = if name == "Bottom" {
                "There isn't room for this under the cursor, so it goes above it. Had there been no room above \
                 it either, it would go beside the box."
            } else {
                "A tooltip of a sentence wraps at 400 pixels, so that it's easy to read, and stays inside the \
                 window."
            };
            frame.tooltip_text(name, area, text);
        }
    }
    if !showcase.tooltips {
        let y = columns[0].bottom() + SPACE_M;
        let warning = Rect::new(inner.x, y, inner.width, 48.0);
        frame.info_bar(None, warning, Severity::Caution, "Tooltips are off", "F1 turns them on.");
    }
}

fn draw_text_fields(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Text fields", "a press puts the caret there, dragging selects");
    let theme = frame.theme();
    let mut stack = Stack::down(inner, SPACE_S);
    caption(frame, &mut stack, "Name");
    let row = stack.next(CONTROL_HEIGHT);
    let field = Rect::new(row.x, row.y, NAME_WIDTH, CONTROL_HEIGHT);
    let name = frame.text_field("Name", field, &mut showcase.name, TextField::new("Type your name").limit(40));
    let greet_button = Button::new("Greet").accent();
    let greet_width = frame.button_width(&greet_button).max(88.0);
    let greet = frame.button("Greet", Rect::new(field.right() + SPACE_S, row.y, greet_width, CONTROL_HEIGHT), greet_button);
    if name.submitted || greet.clicked {
        let name = showcase.name.trim();
        showcase.greeting = Some(if name.is_empty() {
            "Type a name first".to_string()
        } else {
            format!("Hello, {name}!")
        });
        let greeting = showcase.greeting.clone().unwrap_or_default();
        showcase.record("Greeting", greeting);
    }
    stack.skip(SPACE_XS);
    let bar = stack.next(48.0);
    match &showcase.greeting {
        Some(greeting) if greeting.starts_with("Hello") => {
            frame.info_bar(None, bar, Severity::Success, "Greeted", greeting);
        }
        Some(greeting) => {
            frame.info_bar(None, bar, Severity::Caution, "No name", greeting);
        }
        None => {
            frame.info_bar(None, bar, Severity::Info, "", "Enter or Greet says hello.");
        }
    }
    stack.skip(SPACE_XS);
    caption(frame, &mut stack, "Search the list");
    let search = stack.next(CONTROL_HEIGHT);
    let field = TextField::new("Like \"comet\" or \"07\"").limit(24).icon(Icon::Search);
    if frame.text_field("Search", search, &mut showcase.search, field).changed {
        let search = format!("\"{}\"", showcase.search);
        showcase.record("Search", search);
    }
    stack.skip(SPACE_XS);
    let mut lines = Lines::new(stack.rest());
    lines.wrapped(
        &mut frame.canvas,
        "Shift and Control select and move by words, Ctrl+A, C, X and V select all, copy, cut and paste, and \
         a double-click selects a word. Esc takes focus away.",
        TextStyle::Body,
        theme.text_secondary,
        0.0,
    );
}

fn draw_info_bars(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Info bars", "a message, with how serious it is");
    let mut stack = Stack::down(inner, SPACE_S);
    frame.info_bar(None, stack.next(48.0), Severity::Info, "Info", "Something the user may want to know.");
    frame.info_bar(None, stack.next(48.0), Severity::Success, "Success", "Something went well.");
    let caution = stack.next(48.0);
    if showcase.caution_closed {
        let button = Button::new("Show the caution again");
        let place = Rect::new(caution.x, caution.y + 8.0, frame.button_width(&button), CONTROL_HEIGHT);
        if frame.button("Show caution", place, button).clicked {
            showcase.caution_closed = false;
            showcase.record("Caution", "shown".to_string());
        }
    } else if frame
        .info_bar(Some("Close caution"), caution, Severity::Caution, "Caution", "This one closes with its button.")
        .clicked
    {
        showcase.caution_closed = true;
        showcase.record("Caution", "closed".to_string());
    }
    frame.info_bar(None, stack.next(48.0), Severity::Critical, "Critical", "Something failed.");
}

// What widgets are drawn with: the type ramp, icons, the theme's colors,
// lines, circles and rings, and shadows. They never change while the theme
// doesn't, so their instances are kept, and added again each frame.
fn draw_canvas(frame: &mut Frame, cache: &mut Cache, area: Rect) {
    let inner = frame.card(area, "Canvas", "shapes, text and icons, in one draw call with the cards");
    let theme = frame.theme();
    cache.shapes.draw(&mut frame.canvas, ([inner.x, inner.y, inner.width, inner.height], theme.name), |canvas| {
        draw_shapes(canvas, theme, inner);
    });
}

fn draw_shapes(canvas: &mut Canvas, theme: &Theme, inner: Rect) {
    // The type ramp, each style in its own size.
    let mut stack = Stack::down(inner, SPACE_XS);
    for (style, name) in [
        (TextStyle::Title, "Title 28"),
        (TextStyle::Subtitle, "Subtitle 20"),
        (TextStyle::BodyStrong, "Body strong 14"),
        (TextStyle::Body, "Body 14, the most text"),
        (TextStyle::Caption, "Caption 12, for small print"),
        (TextStyle::Mono, "Mono 13: 0123456789 +-.,"),
    ] {
        let line = stack.next(canvas.line_height(style));
        canvas.text([line.x, line.y], name, style, theme.text);
    }
    stack.skip(SPACE_S);

    // Icons, at both sizes.
    let icons = [
        Icon::Search,
        Icon::Settings,
        Icon::Save,
        Icon::Edit,
        Icon::Copy,
        Icon::Delete,
        Icon::Refresh,
        Icon::Play,
        Icon::Pause,
        Icon::Volume,
        Icon::Brightness,
        Icon::Color,
        Icon::Keyboard,
        Icon::Mouse,
        Icon::Gamepad,
        Icon::Display,
        Icon::Info,
        Icon::Warning,
        Icon::Error,
        Icon::Success,
    ];
    for size in ICON_SIZES {
        let row = stack.next(24.0);
        for (index, icon) in icons.iter().enumerate() {
            canvas.icon([row.x + 12.0 + index as f32 * 28.0, row.center()[1]], *icon, size, theme.text);
        }
    }
    stack.skip(SPACE_S);

    // The theme's colors, by their roles.
    let palette = [
        ("Text", theme.text),
        ("Secondary", theme.text_secondary),
        ("Tertiary", theme.text_tertiary),
        ("Accent", theme.accent),
        ("Success", theme.success),
        ("Caution", theme.caution),
        ("Critical", theme.critical),
        ("Orange", theme.orange()),
        ("Purple", theme.purple()),
        ("Teal", theme.teal()),
        ("Green", theme.green()),
        ("Magenta", theme.magenta()),
    ];
    let swatches = stack.next(SWATCH * 3.0 + SPACE_S * 2.0);
    let columns = swatches.columns(4, SPACE_S);
    for (index, (name, color)) in palette.iter().enumerate() {
        let column = columns[index % 4];
        let y = swatches.y + (SWATCH + SPACE_S) * (index / 4) as f32;
        canvas.fill(Rect::new(column.x, y, SWATCH, SWATCH), 6.0, *color);
        let text = Rect::from_edges(column.x + SWATCH + SPACE_S, y, column.right(), y + SWATCH);
        canvas.text_in(text, name, TextStyle::Body, theme.text, Align::Start);
    }
    stack.skip(SPACE_S);

    // Lines, circles and rings, and floating cards, in a well.
    let rest = stack.rest();
    let well_area = Rect::from_edges(rest.x, rest.y, rest.right(), rest.bottom());
    well(canvas, theme, well_area);
    let parts = well_area.inset(SPACE_L).columns(3, SPACE_L);
    let (lines, circles) = (parts[0], parts[1]);
    for (index, width) in [0.5f32, 1.0, 2.0, 4.0, 8.0].iter().enumerate() {
        let x = lines.x + 8.0 + index as f32 * 26.0;
        canvas.line([x, lines.bottom() - 10.0], [x + 48.0, lines.y + 10.0], *width, theme.accent);
    }
    let mut x = circles.x + 8.0;
    let middle = circles.center()[1];
    for radius in [5.0f32, 10.0, 18.0] {
        x += radius;
        canvas.circle([x, middle - 30.0], radius, theme.orange());
        canvas.ring([x, middle + 30.0], radius, 2.0, theme.purple());
        x += radius + 14.0;
    }
    // Cards that float at each height, with their shadows.
    let cards = parts[2].columns(2, SPACE_M);
    for (column, (name, elevation)) in cards.iter().zip([
        ("Raised", user_interface::RAISED),
        ("Flyout", user_interface::FLYOUT),
    ]) {
        let card = column.inset_by(0.0, SPACE_L);
        shadow(canvas, theme, card, 8.0, elevation);
        canvas.fill_stroke(card, 8.0, theme.flyout, 1.0, theme.flyout_stroke);
        canvas.text_in(card, name, TextStyle::Body, theme.text_secondary, Align::Center);
    }
}

fn draw_list(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let matches = showcase.matches();
    let subtitle = format!("{} of {} items match the search", matches.len(), showcase.items.len());
    let inner = frame.card(area, "List", &subtitle);
    let theme = frame.theme();
    let line_height = frame.line_height(TextStyle::Body);
    let list_area = Rect::from_edges(inner.x - CARD_PADDING + SPACE_XS, inner.y, inner.right() + CARD_PADDING - SPACE_XS, inner.bottom() - line_height - SPACE_M);
    let row_height = ROW_HEIGHTS[showcase.density];
    let mut row = matches.iter().position(|&index| Some(index) == showcase.selected);
    let items = &showcase.items;
    let view = frame.list("Items", list_area, matches.len(), row_height, &mut row, |canvas, list_row| {
        let item = matches[list_row.index];
        let text = list_row.area.inset_by(user_interface::TEXT_INSET + SPACE_XS, 0.0);
        let style = if list_row.selected { TextStyle::BodyStrong } else { TextStyle::Body };
        canvas.text_in(text, &items[item], style, theme.text, Align::Start);
    });
    if view.response.changed {
        showcase.selected = row.map(|row| matches[row]);
        if let Some(index) = showcase.selected {
            let item = showcase.items[index].clone();
            showcase.record("Selected", item);
        }
    }
    if matches.is_empty() {
        let empty = Rect::new(list_area.x, list_area.y, list_area.width, 48.0);
        frame.label(empty, "Nothing matches the search", TextStyle::Body, theme.text_tertiary, Align::Center);
    }
    if let Some((row, row_area)) = view.hovered {
        let item = matches[row];
        if view.response.double_clicked {
            showcase.opened = Some(item);
            let opened = showcase.items[item].clone();
            showcase.record("Opened", opened);
        }
        let text = format!(
            "{}. A press selects it, and a double-click opens it. Up, Down, Home, End, Page Up and Page Down move \
             the selection while the list has focus. The wheel scrolls 3 rows a notch.",
            showcase.items[item]
        );
        tip(showcase, frame, &showcase.items[item].clone(), row_area, true, &text);
    }
    let selected = match showcase.selected {
        Some(index) => showcase.items[index].as_str(),
        None => "nothing",
    };
    let opened = match showcase.opened {
        Some(index) => showcase.items[index].as_str(),
        None => "nothing",
    };
    let line = Rect::new(inner.x, inner.bottom() - line_height, inner.width, line_height);
    let mut lines = Lines::new(line);
    lines.pieces(
        &mut frame.canvas,
        &[
            ("Selected  ", theme.text_secondary),
            (selected, theme.text),
            ("     Opened  ", theme.text_secondary),
            (opened, theme.text),
        ],
        TextStyle::Body,
    );
}

// What the input is on, as the interface sees it, and the log of what the
// widgets changed.
fn draw_interface(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.card(area, "Interface", "what the input is on, and what it changed");
    let theme = frame.theme();
    let name = |name: Option<&str>| name.unwrap_or("nothing").to_string();
    let cursor = match frame.cursor() {
        Some([x, y]) => format!("{x:.0}, {y:.0}"),
        None => "not over the window, or the window doesn't have focus".to_string(),
    };
    let keys = if showcase.held_keys.is_empty() {
        "none".to_string()
    } else {
        showcase.held_keys.join(", ")
    };
    let gamepad = &showcase.gamepad;
    let (sticks, buttons) = if gamepad.connected {
        let [left_x, left_y] = gamepad.left_stick;
        let [right_x, right_y] = gamepad.right_stick;
        let names = gamepad.held_names();
        (
            format!(
                "left {left_x:+.2} {left_y:+.2}, right {right_x:+.2} {right_y:+.2}, triggers {:.2} {:.2}",
                gamepad.left_trigger, gamepad.right_trigger
            ),
            if names.is_empty() { "none".to_string() } else { names.join(", ") },
        )
    } else {
        ("not connected".to_string(), "none".to_string())
    };
    let rows = [
        ("Cursor", cursor),
        ("Hovered", name(frame.hovered_name())),
        ("Held", name(frame.held_name())),
        ("Focus", name(frame.focus_name())),
        ("Popup", name(frame.popup_name())),
        ("Targets", frame.target_count().to_string()),
        ("Keys held", keys),
        ("Gamepad", sticks),
        ("Buttons", buttons),
    ];
    let mut lines = Lines::new(inner);
    for (label, value) in &rows {
        lines.pair(&mut frame.canvas, label, value, 80.0, theme.text_secondary, theme.text);
    }
    lines.gap(SPACE_M);
    frame.divider([lines.x, lines.y], lines.width);
    lines.gap(SPACE_M);
    lines.line(&mut frame.canvas, "Log, newest first", TextStyle::BodyStrong, theme.text);
    for line in &showcase.log {
        if !lines.has_room(&frame.canvas, TextStyle::Mono) {
            break;
        }
        lines.line(&mut frame.canvas, line, TextStyle::Mono, theme.text_secondary);
    }
    if showcase.log.is_empty() {
        lines.line(&mut frame.canvas, "Nothing yet. Use a widget.", TextStyle::Body, theme.text_tertiary);
    }
}
