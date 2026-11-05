// Lays out and draws the showcase's panels, with a widget of each kind that
// Library/UserInterface has, and handles what the input does to them. The
// showcase supports two client areas, 2560 x 1369 maximized and 2560 x 1440
// in borderless fullscreen (see Library/Application), so the panels have
// fixed places in pixels, in four columns. In fullscreen, the panels that
// stretch, one in each column, are 71 pixels taller: the tabs, the
// tooltips, the canvas, and the list.

use user_interface::theme::{
    ACCENT, BORDER, DIM, ERROR, FAINT, INSET, ORANGE, PANEL, PURPLE, SUCCESS, TEXT, TITLE, WARNING,
};
use user_interface::{
    Cached, Canvas, Chip, Choice, Color, Frame, Lines, MARGIN, PADDING, PANELS_TOP, Rect, Sense, SliderRange,
    Tone, select_width, text_box,
};

use crate::showcase::{COLORS, DENSITIES, ROW_HEIGHTS, SIZES, Showcase, TABS, TAGS, on_off};

// The panels' heights, but for the one in each column that stretches.
const BUTTONS_HEIGHT: f32 = 186.0;
const TOGGLES_HEIGHT: f32 = 262.0;
const RADIO_HEIGHT: f32 = 140.0;
const SLIDERS_HEIGHT: f32 = 330.0;
const SELECTS_HEIGHT: f32 = 228.0;
const CHIPS_HEIGHT: f32 = 190.0;
const TEXT_FIELDS_HEIGHT: f32 = 300.0;
const INTERFACE_HEIGHT: f32 = 430.0;
// The tooltips panel's boxes to hover over.
const HOVER_HEIGHT: f32 = 60.0;

// A row of a checkbox or a switch.
const TOGGLE_ROW: f32 = 28.0;
// A row of a slider, and its label and value on each side of it.
const SLIDER_ROW: f32 = 40.0;
const SLIDER_LABEL: f32 = 110.0;
const SLIDER_VALUE: f32 = 120.0;
const PROGRESS_HEIGHT: f32 = 16.0;
const FIELD_HEIGHT: f32 = 30.0;
const NAME_WIDTH: f32 = 380.0;
const SWATCH: f32 = 24.0;

// The theme's colors, as the color select box offers them, in its order.
const THEME_COLORS: [Color; 6] = [ACCENT, ORANGE, PURPLE, SUCCESS, WARNING, ERROR];

// What draw keeps from frame to frame: the canvas panel's shapes, which
// never change, by the panel's area.
#[derive(Default)]
pub struct Cache {
    shapes: Cached<[f32; 4]>,
}

pub fn draw(showcase: &mut Showcase, frame: &mut Frame) {
    let adapter = frame.status.adapter;
    frame.header(
        "UI Showcase",
        "Every widget of Library/UserInterface, in KGames' light theme",
        &[("GPU: ", DIM), (adapter, TEXT)],
    );
    let bottom = frame.footer(
        &[
            ("F11", "borderless fullscreen"),
            ("Tab", "next widget"),
            ("Shift+Tab", "previous widget"),
            ("Esc", "close a list or unfocus"),
            ("F1", "tooltips"),
            ("Ctrl+R", "reset"),
        ],
        "",
    );
    let columns = Rect::new(MARGIN, PANELS_TOP, frame.width - MARGIN * 2.0, bottom - PANELS_TOP).columns(4, MARGIN);

    let (buttons, rest) = columns[0].split_top(BUTTONS_HEIGHT, MARGIN);
    let (toggles, rest) = rest.split_top(TOGGLES_HEIGHT, MARGIN);
    let (radio, tabs) = rest.split_top(RADIO_HEIGHT, MARGIN);
    draw_buttons(showcase, frame, buttons);
    draw_toggles(showcase, frame, toggles);
    draw_radio(showcase, frame, radio);
    draw_tabs(showcase, frame, tabs);

    let (sliders, rest) = columns[1].split_top(SLIDERS_HEIGHT, MARGIN);
    let (selects, rest) = rest.split_top(SELECTS_HEIGHT, MARGIN);
    let (chips, tooltips) = rest.split_top(CHIPS_HEIGHT, MARGIN);
    draw_sliders(showcase, frame, sliders);
    draw_selects(showcase, frame, selects);
    draw_chips(showcase, frame, chips);
    draw_tooltips(showcase, frame, tooltips);

    let (fields, canvas) = columns[2].split_top(TEXT_FIELDS_HEIGHT, MARGIN);
    draw_text_fields(showcase, frame, fields);
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

fn draw_buttons(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Buttons", "act on the press, which comes before the release");
    let mut x = inner.x;
    let tones = [
        ("Normal", Tone::Normal, "A button for most actions."),
        ("Primary", Tone::Primary, "The action that a panel is for, in the accent color."),
        ("Danger", Tone::Danger, "An action that throws something away."),
    ];
    for (index, (label, tone, help)) in tones.into_iter().enumerate() {
        let [width, height] = frame.button_size(label);
        let button = Rect::new(x, inner.y, width.max(110.0), height);
        let response = frame.button_with(label, button, label, tone);
        if response.clicked {
            showcase.presses[index] += 1;
            let count = showcase.presses[index];
            showcase.record(label, format!("pressed {count} time{}", if count == 1 { "" } else { "s" }));
        }
        tip(showcase, frame, label, button, response.hovered, help);
        x = button.right() + PADDING;
    }
    let button = Rect::new(x, inner.y, 110.0, frame.button_size("Disabled")[1]);
    let enabled = frame.set_enabled(false);
    let response = frame.button("Disabled", button, "Disabled");
    frame.set_enabled(enabled);
    tip(
        showcase,
        frame,
        "Disabled",
        button,
        response.hovered,
        "Disabled widgets are faint and take no input, but still show their tooltips.",
    );

    let line_height = frame.line_height();
    let mut lines = Lines::new(Rect::new(inner.x, button.bottom() + PADDING, inner.width, inner.bottom() - button.bottom() - PADDING), line_height);
    let [normal, primary, danger] = showcase.presses;
    lines.pair(&mut frame.canvas, "Presses: ", &format!("{normal} normal, {primary} primary, {danger} danger"), TEXT);
    lines.gap(0.5);
    lines.wrapped(
        &mut frame.canvas,
        "Space and Enter press the button with keyboard focus, which Tab moves. A press that \
         gives the window focus does only that.",
        "",
        DIM,
    );
}

// A checkbox's or a switch's row, as wide as its mark and label.
fn toggle_row(frame: &Frame, x: f32, y: f32, label: &str) -> Rect {
    Rect::new(x, y, 34.0 + 8.0 + frame.text_width(label) + 4.0, TOGGLE_ROW)
}

fn draw_toggles(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Checkboxes and switches", "a press, or Space with focus, flips them");
    let mut y = inner.y;
    let label = "Enable the four under this";
    let row = toggle_row(frame, inner.x, y, label);
    if frame.checkbox("Enable", row, label, &mut showcase.enabled).changed {
        showcase.record("Enable", on_off(showcase.enabled));
    }
    y += TOGGLE_ROW;
    let enabled = frame.set_enabled(showcase.enabled);
    let indent = inner.x + 24.0;
    for name in ["Grid", "Snap", "Sound"] {
        let (label, value) = match name {
            "Grid" => ("Show the grid", &mut showcase.grid),
            "Snap" => ("Snap to the grid", &mut showcase.snap),
            _ => ("Play sounds", &mut showcase.sound),
        };
        let row = toggle_row(frame, indent, y, label);
        let changed = frame.checkbox(name, row, label, value).changed;
        let what = on_off(*value);
        if changed {
            showcase.record(name, what);
        }
        y += TOGGLE_ROW;
    }
    let label = "Animate the progress bar";
    let row = toggle_row(frame, indent, y, label);
    if frame.switch("Animate", row, label, &mut showcase.animate).changed {
        showcase.record("Animate", on_off(showcase.animate));
    }
    y += TOGGLE_ROW;
    frame.set_enabled(enabled);
    let label = "Show tooltips (F1)";
    let row = toggle_row(frame, inner.x, y, label);
    let response = frame.switch("Tooltips", row, label, &mut showcase.tooltips);
    if response.changed {
        showcase.record("Tooltips", on_off(showcase.tooltips));
    }
    tip(showcase, frame, "Tooltips", row, response.hovered, "Tooltips show at once, by the cursor, and never cover it.");
    y += TOGGLE_ROW + PADDING * 0.5;
    let mut lines = Lines::new(Rect::new(inner.x, y, inner.width, inner.bottom() - y), frame.line_height());
    lines.wrapped(
        &mut frame.canvas,
        "Turning off the first checkbox disables the four under it.",
        "",
        DIM,
    );
}

fn draw_radio(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Radio buttons", "Up and Down move the choice while the group has focus");
    let group = Rect::new(inner.x, inner.y, 200.0, frame.control_height() * SIZES.len() as f32);
    if frame.radio("Size", group, &SIZES, &mut showcase.size).changed {
        showcase.record("Size", SIZES[showcase.size].to_string());
    }
    // A square of the chosen size.
    let side = [24.0, 44.0, 64.0][showcase.size];
    let middle = group.center()[1];
    frame.canvas.rect(group.right() + 40.0, middle - side * 0.5, side, side, ACCENT);
}

fn draw_tabs(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Tabs", "Left and Right move the choice while the row has focus");
    let bar = Rect::new(inner.x, inner.y, inner.width, frame.button_size("Tab")[1]);
    if frame.tabs("Tabs", bar, &TABS, &mut showcase.tab).changed {
        showcase.record("Tab", TABS[showcase.tab].to_string());
    }
    let text = Rect::new(inner.x, bar.bottom() + PADDING, inner.width, inner.bottom() - bar.bottom() - PADDING);
    let mut lines = Lines::new(text, frame.line_height());
    let canvas = &mut frame.canvas;
    match showcase.tab {
        0 => {
            lines.wrapped(
                canvas,
                "Library/UserInterface is the interface of every KGames program on Win32, Direct3D 12 \
                 and DirectWrite. It's immediate mode: each frame, the program calls a function for \
                 each widget, with where it goes and the value it shows, and the function draws it, \
                 and says what the input did to it.",
                "",
                TEXT,
            );
            lines.gap(0.5);
            lines.wrapped(
                canvas,
                "Input goes to what the frame before drew, which is what was on the screen when it \
                 came. The frame reads it right before it's drawn, so that it shows the newest.",
                "",
                TEXT,
            );
        }
        1 => {
            for (kind, about) in [
                ("Button", "normal, primary, danger, or disabled"),
                ("Checkbox", "on or off, with a label"),
                ("Switch", "on or off, with a label"),
                ("Radio", "one of a column of choices"),
                ("Tabs", "one of a row of choices"),
                ("Slider", "a number, in steps, between two"),
                ("Select", "one of a list that opens over the panels"),
                ("Text field", "a line of text, with a caret"),
                ("List", "rows that scroll, of which one is selected"),
                ("Chips", "labels that are on or off"),
                ("Progress", "a bar filled to a fraction"),
                ("Tooltip", "a box of text by the cursor"),
                ("Panel", "a box with a title, around widgets"),
            ] {
                lines.pair(canvas, &format!("{kind:<12}"), about, TEXT);
            }
        }
        _ => {
            lines.wrapped(
                canvas,
                "Each widget has a name, which is the same from frame to frame, and which scripts see \
                 in the state that they save, with the widget's area, under Interface.Targets.",
                "",
                TEXT,
            );
            lines.gap(0.5);
            lines.wrapped(
                canvas,
                "Popups, like a select box's list, and tooltips are drawn on layers over the panels, \
                 in the same draw call.",
                "",
                TEXT,
            );
        }
    }
}

fn draw_sliders(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Sliders", "drag, scroll, or Left and Right with focus");
    let line_height = frame.line_height();
    let mut y = inner.y;
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
        let text_y = y + ((SLIDER_ROW - line_height) * 0.5).round();
        let track = Rect::new(inner.x + SLIDER_LABEL, y, inner.width - SLIDER_LABEL - SLIDER_VALUE, SLIDER_ROW);
        let locked = name == "Locked";
        let enabled = frame.set_enabled(!locked);
        let response = frame.slider(name, track, value, &range);
        frame.set_enabled(enabled);
        let text = match name {
            "Volume" => format!("{:.0}%", *value),
            "Angle" => format!("{:.1} deg", *value),
            _ => format!("{:.3}", *value),
        };
        let changed = response.changed;
        frame.canvas.text(inner.x, text_y, name, if locked { FAINT } else { TEXT });
        frame.canvas.text(track.right() + PADDING, text_y, &text, if locked { FAINT } else { TITLE });
        if changed {
            showcase.record(name, text);
        }
        y += SLIDER_ROW;
    }

    // Progress bars: one at the volume, and one that fills over 4 seconds,
    // again and again, while it's animated.
    y += PADDING * 0.5;
    let fill = (frame.seconds() * 0.25).fract() as f32;
    let animated = if showcase.animate { fill } else { 0.0 };
    for (label, fraction, color) in [
        ("Volume", (showcase.volume / 100.0) as f32, ACCENT),
        ("Every 4 seconds", animated, SUCCESS),
    ] {
        frame.canvas.text(inner.x, y, label, DIM);
        y += line_height + 4.0;
        frame.progress(Rect::new(inner.x, y, inner.width, PROGRESS_HEIGHT), fraction, color);
        y += PROGRESS_HEIGHT + PADDING * 0.5;
    }
}

fn draw_selects(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Select boxes", "Space or Enter opens, Up and Down pick, Esc closes");
    let line_height = frame.line_height();
    let colors: Vec<Choice> = COLORS.iter().map(|&(label, detail)| Choice { label, detail }).collect();
    let densities: Vec<Choice> = DENSITIES.iter().map(|&(label, detail)| Choice { label, detail }).collect();
    let width = ["Comfortable", "Success"]
        .iter()
        .map(|label| select_width(&frame.canvas, label))
        .fold(0.0, f32::max)
        + 40.0;
    let height = frame.control_height();

    let mut y = inner.y;
    frame.canvas.text(inner.x, y, "Color", DIM);
    y += line_height + 4.0;
    let color_box = Rect::new(inner.x, y, width, height);
    if frame.select("Color", color_box, &colors, &mut showcase.color).changed {
        showcase.record("Color", COLORS[showcase.color].0.to_string());
    }
    let swatch = Rect::new(color_box.right() + PADDING, y, SWATCH * 2.0, height);
    frame.canvas.rect(swatch.x, swatch.y, swatch.width, swatch.height, THEME_COLORS[showcase.color]);
    frame.canvas.text(swatch.right() + PADDING, y + ((height - line_height) * 0.5).round(), COLORS[showcase.color].1, DIM);
    y += height + PADDING;

    frame.canvas.text(inner.x, y, "List density", DIM);
    y += line_height + 4.0;
    let density_box = Rect::new(inner.x, y, width, height);
    if frame.select("Density", density_box, &densities, &mut showcase.density).changed {
        showcase.record("Density", DENSITIES[showcase.density].0.to_string());
    }
    y += height + PADDING;
    let mut lines = Lines::new(Rect::new(inner.x, y, inner.width, inner.bottom() - y), line_height);
    lines.wrapped(
        &mut frame.canvas,
        "A list opens over the panels under it. A press off it closes it, and does nothing else.",
        "",
        DIM,
    );
}

fn draw_chips(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Chips", "labels that are on or off, which wrap onto rows");
    let line_height = frame.line_height();
    let mut y = inner.y;
    frame.canvas.text(inner.x, y, "Tags, which a press turns on or off", DIM);
    y += line_height + 4.0;
    let (changed, height) = frame.chip_switches("Tags", Rect::new(inner.x, y, inner.width, inner.bottom() - y), &TAGS, &mut showcase.tags);
    if changed {
        let on = TAGS.iter().zip(showcase.tags).filter(|(_, on)| *on).count();
        showcase.record("Tags", format!("{on} on"));
    }
    y += height + PADDING;
    frame.canvas.text(inner.x, y, "What's on, from the checkboxes and switches", DIM);
    y += line_height + 4.0;
    let items = [
        Chip { label: "Grid", on: showcase.grid, enabled: showcase.enabled },
        Chip { label: "Snap", on: showcase.snap, enabled: showcase.enabled },
        Chip { label: "Sound", on: showcase.sound, enabled: showcase.enabled },
        Chip { label: "Animate", on: showcase.animate, enabled: showcase.enabled },
        Chip { label: "Tooltips", on: showcase.tooltips, enabled: true },
    ];
    frame.chips(Rect::new(inner.x, y, inner.width, inner.bottom() - y), &items);
}

// Boxes that show tooltips: a sentence, one in columns, and one at the
// panel's bottom, whose tooltip goes above the cursor, as there isn't room
// under it. Each box is a widget that the showcase makes itself, which only
// takes the cursor being over it.
fn draw_tooltips(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Tooltips", "at once, by the cursor, never over it, always in the window");
    let line_height = frame.line_height();
    let columns = Rect::new(inner.x, inner.y, inner.width, HOVER_HEIGHT).columns(2, PADDING);
    let bottom = Rect::new(inner.x, inner.bottom() - HOVER_HEIGHT, inner.width, HOVER_HEIGHT);
    let boxes = [
        ("Sentence", columns[0], "Hover for a sentence"),
        ("Columns", columns[1], "Hover for columns"),
        ("Bottom", bottom, "Hover here, where there's no room under the cursor"),
    ];
    for (name, area, label) in boxes {
        let response = frame.interact(name, "Hover", area, Sense::HOVER);
        let fill = if response.hovered { INSET } else { PANEL };
        frame.canvas.rect(area.x, area.y, area.width, area.height, fill);
        frame.canvas.rect_outline(area.x, area.y, area.width, area.height, 1.0, BORDER);
        let text_x = (area.center()[0] - frame.text_width(label) * 0.5).round();
        let text_y = (area.center()[1] - line_height * 0.5).round();
        frame.canvas.text(text_x, text_y, label, DIM);
        if !(showcase.tooltips && response.hovered) {
            continue;
        }
        if name == "Columns" {
            let char_width = frame.char_width();
            let rows = [
                ("Layer", "tooltips, over popups and panels"),
                ("Placement", "under the cursor, or above it, or beside"),
                ("Size", "as big as its text"),
                ("Shows", "while what it's about is under the cursor"),
            ];
            let value_x = char_width * 12.0;
            let mut pieces = vec![([0.0, 0.0], "A tooltip in columns".to_string(), TITLE)];
            for (index, (label, value)) in rows.iter().enumerate() {
                let y = line_height * (index as f32 + 1.4);
                pieces.push(([char_width * 2.0, y], label.to_string(), DIM));
                pieces.push(([value_x, y], value.to_string(), TEXT));
            }
            frame.tooltip(name, area, &pieces);
        } else {
            let text = if name == "Bottom" {
                "There isn't room for this under the cursor, so it goes above it. Had there been no \
                 room above it either, it would go beside the box."
            } else {
                "A tooltip of a sentence wraps at 72 characters, so that it's easy to read, and stays \
                 inside the window."
            };
            frame.tooltip_text(name, area, text);
        }
    }
    if !showcase.tooltips {
        let y = columns[0].bottom() + PADDING;
        frame.canvas.text(inner.x, y, "Tooltips are off. F1 turns them on.", WARNING);
    }
}

fn draw_text_fields(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Text fields", "a press puts the caret there, Enter submits");
    let line_height = frame.line_height();
    let mut y = inner.y;
    frame.canvas.text(inner.x, y, "Name", DIM);
    y += line_height + 4.0;
    let field = Rect::new(inner.x, y, NAME_WIDTH, FIELD_HEIGHT);
    let name = frame.text_field("Name", field, &mut showcase.name, "Type your name", 40);
    let [width, height] = frame.button_size("Greet");
    let greet = frame.button_with("Greet", Rect::new(field.right() + PADDING, y, width.max(90.0), height), "Greet", Tone::Primary);
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
    y += FIELD_HEIGHT + 6.0;
    match &showcase.greeting {
        Some(greeting) => frame.canvas.text_clipped(inner.x, y, greeting, inner.width, SUCCESS),
        None => frame.canvas.text(inner.x, y, "Enter or Greet says hello", FAINT),
    };
    y += line_height + PADDING;

    frame.canvas.text(inner.x, y, "Search the list", DIM);
    y += line_height + 4.0;
    let search = Rect::new(inner.x, y, inner.width, FIELD_HEIGHT);
    if frame.text_field("Search", search, &mut showcase.search, "Like \"comet\" or \"07\"", 24).changed {
        let search = format!("\"{}\"", showcase.search);
        showcase.record("Search", search);
    }
    y += FIELD_HEIGHT + PADDING;
    let mut lines = Lines::new(Rect::new(inner.x, y, inner.width, inner.bottom() - y), line_height);
    lines.wrapped(
        &mut frame.canvas,
        "Backspace, Delete, Left, Right, Home and End edit. While a field has focus, it takes the \
         keys that type, so they don't set off shortcuts, and Esc takes its focus away.",
        "",
        DIM,
    );
}

// What widgets are drawn with: the theme's colors, lines, circles and rings,
// and a box of text, as over a 3D view. They never change, so their
// vertices are kept, and added again each frame.
fn draw_canvas(frame: &mut Frame, cache: &mut Cache, area: Rect) {
    let inner = frame.panel(area, "Canvas", "shapes and text, in one draw call with the panels");
    cache.shapes.draw(&mut frame.canvas, [inner.x, inner.y, inner.width, inner.height], |canvas| {
        draw_shapes(canvas, inner);
    });
}

fn draw_shapes(canvas: &mut Canvas, inner: Rect) {
    let line_height = canvas.line_height();
    let palette = [
        ("TITLE", TITLE),
        ("TEXT", TEXT),
        ("DIM", DIM),
        ("FAINT", FAINT),
        ("BORDER", BORDER),
        ("INSET", INSET),
        ("ACCENT", ACCENT),
        ("ORANGE", ORANGE),
        ("PURPLE", PURPLE),
        ("SUCCESS", SUCCESS),
        ("WARNING", WARNING),
        ("ERROR", ERROR),
    ];
    let columns = inner.columns(3, PADDING);
    for (index, (name, color)) in palette.iter().enumerate() {
        let column = columns[index % 3];
        let y = inner.y + (SWATCH + 6.0) * (index / 3) as f32;
        canvas.rect(column.x, y, SWATCH, SWATCH, *color);
        canvas.rect_outline(column.x, y, SWATCH, SWATCH, 1.0, BORDER);
        canvas.text(column.x + SWATCH + 8.0, y + ((SWATCH - line_height) * 0.5).round(), name, TEXT);
    }

    // The text box under the well is 3 lines tall, with padding.
    let top = inner.y + (SWATCH + 6.0) * 4.0 + PADDING;
    let text_box_height = line_height * 3.0 + PADDING * 2.0;
    let well = Rect::new(inner.x, top, inner.width, inner.bottom() - top - text_box_height - PADDING);
    canvas.rect(well.x, well.y, well.width, well.height, INSET);
    canvas.rect_outline(well.x, well.y, well.width, well.height, 1.0, BORDER);
    // Antialiased lines from thin to thick, fanning out.
    for (index, width) in [0.5f32, 1.0, 2.0, 4.0, 8.0].iter().enumerate() {
        let x = well.x + 30.0 + index as f32 * 36.0;
        canvas.line([x, well.bottom() - 40.0], [x + 60.0, well.y + 24.0], *width, ACCENT);
    }
    // Circles and rings.
    let center_y = well.center()[1];
    let mut x = well.x + 300.0;
    for radius in [6.0f32, 14.0, 28.0] {
        canvas.circle([x, center_y - 40.0], radius, ORANGE);
        canvas.ring([x, center_y + 40.0], radius, 2.0, PURPLE);
        x += radius * 2.0 + 30.0;
    }
    canvas.text(well.x + 12.0, well.bottom() - line_height - 4.0, "Lines", DIM);
    canvas.text(well.x + 300.0 - 6.0, well.bottom() - line_height - 4.0, "Circles and rings", DIM);

    // A box of text, as over a 3D view.
    let piece = |text: &str, color| (text.to_string(), color);
    text_box(
        canvas,
        inner.x,
        well.bottom() + PADDING,
        &[
            vec![piece("Text box", TITLE), piece("  over a 3D view, like the scene tests'", DIM)],
            vec![piece("FPS: ", DIM), piece("1000", TEXT)],
            vec![piece("Window has focus", SUCCESS)],
        ],
    );
}

fn draw_list(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let matches = showcase.matches();
    let subtitle = format!("{} of {} items match the search", matches.len(), showcase.items.len());
    let inner = frame.panel(area, "List", &subtitle);
    let line_height = frame.line_height();
    let list_area = Rect::new(inner.x, inner.y, inner.width, inner.height - line_height - PADDING);
    let row_height = ROW_HEIGHTS[showcase.density];
    let mut row = matches.iter().position(|&index| Some(index) == showcase.selected);
    let view = frame.list("Items", list_area, matches.len(), row_height, &mut row);
    if view.response.changed {
        showcase.selected = row.map(|row| matches[row]);
        if let Some(index) = showcase.selected {
            let item = showcase.items[index].clone();
            showcase.record("Selected", item);
        }
    }
    for &(row, area) in &view.rows {
        let item = matches[row];
        let text_y = area.y + ((area.height - line_height) * 0.5).round();
        let color = if Some(item) == showcase.selected { ACCENT } else { TEXT };
        frame.canvas.text_clipped(area.x + 10.0, text_y, &showcase.items[item], area.width - 20.0, color);
    }
    if matches.is_empty() {
        frame.canvas.text(list_area.x + 10.0, list_area.y + 8.0, "Nothing matches the search", FAINT);
    }
    if let Some(row) = view.hovered
        && let Some(&(_, area)) = view.rows.iter().find(|(index, _)| *index == row)
    {
        let text = format!(
            "{}. A press selects it, and Up, Down, Home and End move the selection while the list has \
             focus. The wheel scrolls 3 rows a notch.",
            showcase.items[matches[row]]
        );
        tip(showcase, frame, &showcase.items[matches[row]].clone(), area, true, &text);
    }
    let selected = match showcase.selected {
        Some(index) => showcase.items[index].as_str(),
        None => "nothing",
    };
    let y = inner.bottom() - line_height;
    let end = frame.canvas.text(inner.x, y, "Selected: ", DIM);
    frame.canvas.text_clipped(end, y, selected, inner.right() - end, TEXT);
}

// What the input is on, as the interface sees it, and the log of what the
// widgets changed.
fn draw_interface(showcase: &mut Showcase, frame: &mut Frame, area: Rect) {
    let inner = frame.panel(area, "Interface", "what the input is on, and what it changed");
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
        ("Cursor:    ", cursor),
        ("Hovered:   ", name(frame.hovered_name())),
        ("Held:      ", name(frame.held_name())),
        ("Focus:     ", name(frame.focus_name())),
        ("Popup:     ", name(frame.popup_name())),
        ("Targets:   ", frame.target_count().to_string()),
        ("Keys held: ", keys),
        ("Gamepad:   ", sticks),
        ("Buttons:   ", buttons),
    ];
    let mut lines = Lines::new(inner, frame.line_height());
    for (label, value) in &rows {
        lines.pair(&mut frame.canvas, label, value, TEXT);
    }
    lines.gap(0.5);
    lines.line(&mut frame.canvas, "Log, newest first", DIM);
    for line in &showcase.log {
        if !lines.has_room() {
            break;
        }
        lines.line(&mut frame.canvas, line, TEXT);
    }
    if showcase.log.is_empty() {
        lines.line(&mut frame.canvas, "Nothing yet. Use a widget.", FAINT);
    }
}
