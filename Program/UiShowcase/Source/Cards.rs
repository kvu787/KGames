// The showcase's cards, laid out in fixed pixels in five columns, and their
// widgets. Of the 71 pixels that borderless fullscreen adds to the height,
// the choices, the tooltips, the tabs, the list and the interface card get
// all of them, as each is at the bottom of its column.

use user_interface::tokens::{
    BODY, BODY_LARGE, BODY_STRONG, CAPTION, CARD_GAP, CONTROL_HEIGHT, CURVE_DECELERATE, DISPLAY, ELEVATION_2, ELEVATION_8,
    ELEVATION_16, ELEVATION_28, ICON, ICON_LARGE, MONO, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S, SPACE_XS,
    SPACE_XXL, SUBTITLE, TITLE, TITLE_LARGE,
};
use user_interface::{
    Align, Button, Color, Corners, Icon, InfoBar, Point, Rect, Severity, Size, Slider, Status, Stroke, TextBox, TextStyle,
    Theme, TooltipLine, Ui,
};

use crate::showcase::{ALIGNMENTS, BUTTONS, DENSITIES, OPTIONS, SIZES, Showcase, TABS, TOOLS};

// The theme's roles that the color combo box picks from, and what each is for.
pub const ROLES: [&str; 8] = [
    "Accent",
    "Text primary",
    "Text secondary",
    "Success",
    "Caution",
    "Critical",
    "Card stroke",
    "Divider",
];

fn role(theme: &Theme, index: usize) -> (Color, &'static str) {
    match index {
        0 => (theme.accent, "What's on, chosen, or the one command that matters most"),
        1 => (theme.text_primary, "Text that matters most, like labels and values"),
        2 => (theme.text_secondary, "Text that explains, like captions and hints"),
        3 => (theme.success, "Something that went well"),
        4 => (theme.caution, "Something to look into"),
        5 => (theme.critical, "Something that went wrong"),
        6 => (theme.card_stroke, "The outline of a card"),
        _ => (theme.divider, "A line between groups"),
    }
}

// The rows of the list, for each density.
pub const ROW_HEIGHTS: [f32; 3] = [28.0, 36.0, 44.0];

// The heights of the cards that aren't at the bottom of their column.
pub const BUTTONS_HEIGHT: f32 = 284.0;
pub const TOGGLES_HEIGHT: f32 = 372.0;
pub const SLIDERS_HEIGHT: f32 = 488.0;
pub const COMBO_HEIGHT: f32 = 236.0;
pub const TEXT_HEIGHT: f32 = 268.0;
pub const INFO_BARS_HEIGHT: f32 = 340.0;

pub fn draw(showcase: &mut Showcase, ui: &mut Ui) {
    app_bar(showcase, ui);
    status_bar(ui);
    let page = ui.page();
    let [first, second, third, fourth, fifth] = page.columns::<5>(CARD_GAP);

    let mut column = first;
    buttons(showcase, ui, column.cut_top(BUTTONS_HEIGHT, CARD_GAP));
    toggles(showcase, ui, column.cut_top(TOGGLES_HEIGHT, CARD_GAP));
    choices(showcase, ui, column);

    let mut column = second;
    sliders(showcase, ui, column.cut_top(SLIDERS_HEIGHT, CARD_GAP));
    combo_boxes(showcase, ui, column.cut_top(COMBO_HEIGHT, CARD_GAP));
    text_boxes(showcase, ui, column.cut_top(TEXT_HEIGHT, CARD_GAP));
    tooltips(ui, column);

    tabs(showcase, ui, third);

    let mut column = fourth;
    info_bars(showcase, ui, column.cut_top(INFO_BARS_HEIGHT, CARD_GAP));
    list(showcase, ui, column);

    interface(showcase, ui, fifth);
}

fn app_bar(showcase: &mut Showcase, ui: &mut Ui) {
    let bar = ui.app_bar(Icon::Library, "UiShowcase");
    let caption = format!("Fluent 2 · {} theme", if showcase.dark { "Dark" } else { "Light" });
    let color = ui.theme().text_secondary;
    ui.text(bar.left_strip(400.0), &caption, CAPTION, color);
    let theme_button = bar.right_strip(CONTROL_HEIGHT);
    if ui.button("Theme", theme_button, Button::icon_only(Icon::Light).subtle()).clicked {
        showcase.dark = !showcase.dark;
        showcase.log(format!("{} theme", if showcase.dark { "Dark" } else { "Light" }));
    }
    ui.tooltip("Theme", "Switches between the light and dark themes (F2)");
}

fn status_bar(ui: &mut Ui) {
    let stats = ui.stats();
    let rate = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    ui.status_bar(&[
        Status::Shortcut("F1", "Tooltips"),
        Status::Shortcut("F2", "Theme"),
        Status::Shortcut("Tab", "Next widget"),
        Status::Shortcut("Ctrl+R", "Reset"),
        Status::Shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::Text(&rate),
        Status::Text(&environment.gpu),
        Status::Text(&display),
    ]);
}

fn buttons(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Buttons");
    let mut area = card.content;
    let [standard, accent, subtle] = area.cut_top(CONTROL_HEIGHT, SPACE_M).columns::<3>(SPACE_S);
    for (index, (rect, button, tip)) in [
        (standard, Button::new("Standard"), "A standard button, for most commands"),
        (accent, Button::new("Accent").accent(), "An accent button, for the one command that matters most"),
        (subtle, Button::new("Subtle").subtle(), "A subtle button, which shows its fill under the pointer"),
    ]
    .into_iter()
    .enumerate()
    {
        if ui.button(BUTTONS[index], rect, button).clicked {
            showcase.presses[index] += 1;
            showcase.log(format!("Pressed {}", BUTTONS[index]));
        }
        ui.tooltip(BUTTONS[index], tip);
    }

    let [disabled, disabled_accent, _] = area.cut_top(CONTROL_HEIGHT, SPACE_M).columns::<3>(SPACE_S);
    ui.set_enabled(false);
    ui.button("Disabled", disabled, Button::new("Disabled"));
    ui.button("DisabledAccent", disabled_accent, Button::new("Disabled").accent());
    ui.set_enabled(true);

    let [save, pin, link] = area.cut_top(CONTROL_HEIGHT, SPACE_M).columns::<3>(SPACE_S);
    if ui.button("Save", save, Button::new("Save").icon(Icon::Save)).clicked {
        showcase.presses[3] += 1;
        showcase.log("Pressed Save");
    }
    ui.tooltip("Save", "A button with an icon and text");
    let pinned = showcase.pinned;
    if ui.toggle_button("Pin", pin, &mut showcase.pinned, Button::new(if pinned { "Pinned" } else { "Pin" }).icon(Icon::Pin)).changed {
        let line = format!("Pin {}", if showcase.pinned { "on" } else { "off" });
        showcase.log(line);
    }
    ui.tooltip("Pin", "A toggle button, in the accent color while it's on");
    if ui.hyperlink("Link", link, "Learn more").clicked {
        showcase.link_clicks += 1;
        showcase.log("Followed the link");
    }

    let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_M);
    if ui.button("Remove", row.cut_left(CONTROL_HEIGHT, SPACE_S), Button::icon_only(Icon::Remove)).clicked {
        showcase.count -= 1;
    }
    ui.tooltip("Remove", "Takes one away");
    let count = row.cut_left(48.0, SPACE_S);
    let text = ui.theme().text_primary;
    ui.text_aligned(count, &showcase.count.to_string(), BODY_STRONG, text, Align::Center);
    if ui.button("Add", row.cut_left(CONTROL_HEIGHT, SPACE_S), Button::icon_only(Icon::Add)).clicked {
        showcase.count += 1;
    }
    ui.tooltip("Add", "Adds one");

    let presses = format!(
        "Pressed: Standard {}, Accent {}, Subtle {}, Save {}",
        showcase.presses[0], showcase.presses[1], showcase.presses[2], showcase.presses[3]
    );
    ui.caption(area.cut_top(CAPTION.line_height, 0.0), &presses);
}

fn toggles(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Checkboxes and switches");
    let mut area = card.content;
    let row = |area: &mut Rect| area.cut_top(CONTROL_HEIGHT, SPACE_XS);
    if ui.checkbox("OptionsEnabled", row(&mut area), &mut showcase.options_enabled, "Enable the options under this").changed {
        let line = format!("Options {}", if showcase.options_enabled { "enabled" } else { "disabled" });
        showcase.log(line);
    }
    ui.set_enabled(showcase.options_enabled);
    for (index, label) in OPTIONS.iter().enumerate() {
        let rect = row(&mut area).inset_by(28.0, 0.0, 0.0, 0.0);
        if ui.checkbox(&format!("Option/{label}"), rect, &mut showcase.options[index], label).changed {
            let line = format!("{label} {}", if showcase.options[index] { "on" } else { "off" });
            showcase.log(line);
        }
    }
    ui.set_enabled(true);
    area.cut_top(SPACE_S, 0.0);
    if ui.toggle_switch("Tooltips", row(&mut area), &mut showcase.tooltips, "Tooltips (F1)").changed {
        let line = format!("Tooltips {}", if showcase.tooltips { "on" } else { "off" });
        showcase.log(line);
    }
    if ui.toggle_switch("Dark", row(&mut area), &mut showcase.dark, "Dark theme (F2)").changed {
        let line = format!("{} theme", if showcase.dark { "Dark" } else { "Light" });
        showcase.log(line);
    }
    let time = ui.time();
    if ui.toggle_switch("Filling", row(&mut area), &mut showcase.filling, "Fill the progress bar").changed {
        showcase.fill_started = time;
    }
}

fn choices(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Radio buttons and segments");
    let mut area = card.content;
    let mut radios = area.cut_top(CONTROL_HEIGHT * 3.0 + SPACE_XS * 2.0, SPACE_L);
    let preview = radios.cut_right(72.0, 0.0);
    for (index, label) in SIZES.iter().enumerate() {
        let rect = radios.cut_top(CONTROL_HEIGHT, SPACE_XS);
        if ui.radio(&format!("Size/{label}"), rect, &mut showcase.size, index, label).changed {
            showcase.log(format!("Size {label}"));
        }
    }
    let side = [24.0, 40.0, 56.0][showcase.size];
    let side = ui.animate(user_interface::id("Size"), 0, side, user_interface::tokens::DURATION_FAST, CURVE_DECELERATE);
    let theme = ui.theme().clone();
    let square = Rect::from_center(preview.center(), Size::new(side, side));
    ui.painter().shape(square, Corners::all(RADIUS_CONTROL), theme.accent, None);

    if ui.segmented("Alignment", area.cut_top(CONTROL_HEIGHT, SPACE_S), &mut showcase.alignment, &ALIGNMENTS).changed {
        let line = format!("Aligned {}", ALIGNMENTS[showcase.alignment]);
        showcase.log(line);
    }
    let sample = area.cut_top(36.0, SPACE_L);
    let align = [Align::Start, Align::Center, Align::End][showcase.alignment];
    let mut painter = ui.painter();
    painter.shape(sample, Corners::all(RADIUS_CONTROL), theme.control_alt_fill_secondary, Some(Stroke::new(theme.control_stroke, 1.0)));
    painter.text_in(sample.inset_by(SPACE_M, 0.0, SPACE_M, 0.0), "Aligned text", BODY, theme.text_primary, align);

    let tools = [(Icon::Brush, TOOLS[0]), (Icon::Fill, TOOLS[1]), (Icon::Erase, TOOLS[2]), (Icon::Eyedropper, TOOLS[3])];
    if ui.segmented_icons("Tool", area.cut_top(CONTROL_HEIGHT, SPACE_S), &mut showcase.tool, &tools).changed {
        let line = format!("Tool {}", TOOLS[showcase.tool]);
        showcase.log(line);
    }
    let caption = format!("{}: segments with icons, like a tool's modes", TOOLS[showcase.tool]);
    ui.caption(area.cut_top(CAPTION.line_height, 0.0), &caption);
}

fn labeled_value(ui: &mut Ui, rect: Rect, label: &str, value: &str) {
    let theme = ui.theme().clone();
    let mut painter = ui.painter();
    painter.text_in(rect, label, BODY, theme.text_primary, Align::Start);
    painter.text_in(rect, value, BODY, theme.text_secondary, Align::End);
}

fn sliders(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Sliders and progress");
    let mut area = card.content;
    let label = |area: &mut Rect| area.cut_top(BODY.line_height, 0.0);
    labeled_value(ui, label(&mut area), "Volume", &format!("{:.0}", showcase.volume));
    ui.slider("Volume", area.cut_top(CONTROL_HEIGHT, SPACE_S), &mut showcase.volume, Slider::new(0.0, 100.0).step(1.0).show_value(0));
    ui.tooltip("Volume", "Whole numbers, from 0 to 100");
    labeled_value(ui, label(&mut area), "Opacity", &format!("{:.3}", showcase.opacity));
    ui.slider(
        "Opacity",
        area.cut_top(CONTROL_HEIGHT, SPACE_S),
        &mut showcase.opacity,
        Slider::new(0.0, 1.0).step(0.001).large_step(0.1).show_value(3),
    );
    labeled_value(ui, label(&mut area), "Angle", &format!("{:.1}°", showcase.angle));
    ui.slider(
        "Angle",
        area.cut_top(CONTROL_HEIGHT, SPACE_S),
        &mut showcase.angle,
        Slider::new(0.0, 359.9).step(0.1).large_step(10.0).show_value(1),
    );
    ui.set_enabled(false);
    labeled_value(ui, label(&mut area), "Disabled", "30");
    let mut disabled = 30.0;
    ui.slider("DisabledSlider", area.cut_top(CONTROL_HEIGHT, SPACE_M), &mut disabled, Slider::new(0.0, 100.0));
    ui.set_enabled(true);

    labeled_value(ui, label(&mut area), "At the volume", &format!("{:.0}%", showcase.volume));
    ui.progress_bar(area.cut_top(SPACE_M, SPACE_S), Some(showcase.volume / 100.0));
    let time = ui.time();
    let fill = if showcase.filling { (((time - showcase.fill_started) % 4.0) / 4.0) as f32 } else { 0.0 };
    labeled_value(ui, label(&mut area), "Fills every 4 seconds", &format!("{:.0}%", fill * 100.0));
    ui.progress_bar(area.cut_top(SPACE_M, SPACE_S), Some(fill));
    labeled_value(ui, label(&mut area), "Of unknown length", "");
    ui.progress_bar(area.cut_top(SPACE_M, SPACE_M), None);
    let mut rings = area.cut_top(40.0, 0.0);
    let first = rings.cut_left(40.0, SPACE_L);
    ui.progress_ring(first.center(), 32.0, Some(showcase.volume / 100.0));
    let second = rings.cut_left(40.0, SPACE_L);
    ui.progress_ring(second.center(), 32.0, None);
    let color = ui.theme().text_secondary;
    ui.text(rings, "Progress rings, at the volume, and spinning", CAPTION, color);
}

fn combo_boxes(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Combo boxes");
    let mut area = card.content;
    ui.label(area.cut_top(BODY.line_height, SPACE_S), "A color of the theme");
    let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
    let swatch = row.cut_right(CONTROL_HEIGHT, SPACE_S);
    if ui.combo_box("Role", row, &mut showcase.role, &ROLES).changed {
        showcase.log(format!("Color {}", ROLES[showcase.role]));
    }
    let (color, purpose) = role(ui.theme(), showcase.role);
    let theme = ui.theme().clone();
    ui.painter().shape(swatch, Corners::all(RADIUS_CONTROL), color, Some(Stroke::new(theme.control_stroke, 1.0)));
    ui.caption(area.cut_top(CAPTION.line_height, SPACE_M), purpose);
    ui.label(area.cut_top(BODY.line_height, SPACE_S), "The list's density");
    if ui.combo_box("Density", area.cut_top(CONTROL_HEIGHT, 0.0), &mut showcase.density, &DENSITIES).changed {
        showcase.log(format!("Density {}", DENSITIES[showcase.density]));
    }
}

fn text_boxes(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Text fields");
    let mut area = card.content;
    ui.label(area.cut_top(BODY.line_height, SPACE_S), "Your name");
    let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
    let greet = row.cut_right(80.0, SPACE_S);
    let name = ui.text_box("Name", row, &mut showcase.name, TextBox::new().placeholder("Type a name"));
    let greet = ui.button("Greet", greet, Button::new("Greet").accent());
    if (name.submitted || greet.clicked) && !showcase.name.trim().is_empty() {
        let greeting = format!("Hello, {}!", showcase.name.trim());
        showcase.log(greeting.clone());
        showcase.greeting = Some(greeting);
    }
    let bar = area.cut_top(48.0, SPACE_M);
    if let Some(greeting) = showcase.greeting.clone()
        && ui.info_bar("Greeting", bar, InfoBar::new(Severity::Success, "Greeted", &greeting).closable()).clicked
    {
        showcase.greeting = None;
    }
    ui.label(area.cut_top(BODY.line_height, SPACE_S), "Search the list");
    let search = ui.text_box(
        "Search",
        area.cut_top(CONTROL_HEIGHT, 0.0),
        &mut showcase.search,
        TextBox::new().placeholder("Search").button(Icon::Search),
    );
    if search.changed {
        showcase.selected = None;
    }
}

fn tooltips(ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Tooltips");
    let mut area = card.content;
    let [sentence, details] = area.cut_top(CONTROL_HEIGHT, SPACE_M).columns::<2>(SPACE_S);
    ui.button("TipSentence", sentence, Button::new("A sentence"));
    ui.tooltip("TipSentence", "Tooltips show after the pointer rests on a widget for half a second");
    ui.button("TipDetails", details, Button::new("Details").icon(Icon::Info));
    ui.tooltip_lines(
        "TipDetails",
        &[
            TooltipLine::Heading("Segoe UI Variable"),
            TooltipLine::Pair("Caption", "12 on 16, Small"),
            TooltipLine::Pair("Body", "14 on 20, Text"),
            TooltipLine::Pair("Subtitle", "20 on 28, Display"),
            TooltipLine::Separator,
            TooltipLine::Mono("Cascadia Mono lines up"),
        ],
    );
    let theme = ui.theme().clone();
    let mut keys = area.cut_top(22.0, SPACE_M);
    for key in ["Ctrl", "Shift", "S"] {
        let width = ui.keycap(keys.origin(), key);
        keys.cut_left(width, SPACE_XS);
    }
    ui.text(keys, "Keycaps, for shortcuts", CAPTION, theme.text_secondary);
    let mut badges = area.cut_top(22.0, SPACE_M);
    let width = ui.badge(Point::new(badges.x, badges.center().y), "3");
    badges.cut_left(width, SPACE_XS);
    let width = ui.badge(Point::new(badges.x, badges.center().y), "New");
    badges.cut_left(width, SPACE_S);
    ui.text(badges, "Badges, for counts", CAPTION, theme.text_secondary);
    // A button at the bottom, whose tooltip has no room under the pointer.
    let low = area.bottom_strip(CONTROL_HEIGHT);
    ui.button("TipAbove", low, Button::new("At the bottom: the tooltip goes above"));
    ui.tooltip_lines(
        "TipAbove",
        &[
            TooltipLine::Heading("Above the pointer"),
            TooltipLine::Text("There's no room under the pointer, so the tooltip goes above it, inside the window."),
        ],
    );
}

fn tabs(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "");
    let mut area = card.content;
    if ui.selector_bar("Tabs", area.cut_top(40.0, SPACE_L), &mut showcase.tab, &TABS).changed {
        showcase.log(format!("Tab {}", TABS[showcase.tab]));
    }
    match showcase.tab {
        0 => type_ramp(ui, area),
        1 => icons(ui, area),
        2 => colors(ui, area),
        _ => shapes(ui, area),
    }
}

fn type_ramp(ui: &mut Ui, mut area: Rect) {
    let theme = ui.theme().clone();
    let styles: [(&str, TextStyle); 10] = [
        ("Caption", CAPTION),
        ("Body", BODY),
        ("Body strong", BODY_STRONG),
        ("Body large", BODY_LARGE),
        ("Subtitle", SUBTITLE),
        ("Title", TITLE),
        ("Title large", TITLE_LARGE),
        ("Display", DISPLAY),
        ("Mono", MONO),
        ("Mono strong", user_interface::tokens::MONO_STRONG),
    ];
    for (name, style) in styles {
        let rect = area.cut_top(style.line_height, SPACE_S);
        let label = format!("{name} · {} on {}", style.size, style.line_height);
        let mut painter = ui.painter();
        painter.text_in(rect.left_strip(140.0), &label, CAPTION, theme.text_secondary, Align::Start);
        let sample = if style.size > 30.0 { "Fluent" } else { "The quick brown fox jumps" };
        painter.text_fit(rect.inset_by(148.0, 0.0, 0.0, 0.0), sample, style, theme.text_primary, Align::Start);
    }
}

fn icons(ui: &mut Ui, area: Rect) {
    let theme = ui.theme().clone();
    let columns = 5;
    let cell = Size::new(area.width / columns as f32, 64.0);
    for (index, icon) in Icon::ALL.iter().enumerate() {
        let x = area.x + (index % columns) as f32 * cell.width;
        let y = area.y + (index / columns) as f32 * cell.height;
        if y + cell.height > area.bottom() {
            break;
        }
        let rect = Rect::new(x, y, cell.width, cell.height);
        let mut painter = ui.painter();
        painter.icon(Point::new(rect.center().x, rect.y + 20.0), *icon, ICON_LARGE, theme.text_primary);
        painter.text_fit(Rect::new(rect.x, rect.y + 36.0, rect.width, 20.0), &icon.name(), CAPTION, theme.text_secondary, Align::Center);
    }
}

fn colors(ui: &mut Ui, mut area: Rect) {
    let theme = ui.theme().clone();
    for (name, color) in theme.roles() {
        if area.height < 28.0 {
            break;
        }
        let mut row = area.cut_top(28.0, 4.0);
        let swatch = row.cut_left(44.0, SPACE_M);
        let mut painter = ui.painter();
        painter.shape(swatch, Corners::all(RADIUS_CONTROL), color, Some(Stroke::new(theme.control_stroke, 1.0)));
        painter.text_in(row, name, BODY, theme.text_primary, Align::Start);
        let [r, g, b, a] = color.to_bytes();
        painter.text_in(row, &format!("#{a:02X}{r:02X}{g:02X}{b:02X}"), MONO, theme.text_secondary, Align::End);
    }
}

fn shapes(ui: &mut Ui, mut area: Rect) {
    let theme = ui.theme().clone();
    let data = theme.data.clone();
    let mut painter = ui.painter();
    // Elevation: surfaces at each level of Fluent's shadows.
    painter.text(area.origin(), "Elevation", CAPTION, theme.text_secondary);
    area.cut_top(CAPTION.line_height + SPACE_S, 0.0);
    let mut row = area.cut_top(72.0, SPACE_XXL);
    for (name, elevation) in [("2", ELEVATION_2), ("8", ELEVATION_8), ("16", ELEVATION_16), ("28", ELEVATION_28)] {
        let surface = row.cut_left(72.0, SPACE_XXL);
        painter.shadow(surface, RADIUS_OVERLAY, elevation);
        painter.shape(surface, Corners::all(RADIUS_OVERLAY), theme.flyout, Some(Stroke::new(theme.surface_stroke_flyout, 1.0)));
        painter.text_in(surface, name, BODY_STRONG, theme.text_primary, Align::Center);
    }
    // Lines of each width, and a curve.
    painter.text(area.origin(), "Lines", CAPTION, theme.text_secondary);
    area.cut_top(CAPTION.line_height + SPACE_S, 0.0);
    let lines = area.cut_top(96.0, SPACE_L);
    for (index, width) in [0.5f32, 1.0, 2.0, 4.0, 8.0].iter().enumerate() {
        let x = lines.x + index as f32 * 24.0 + 8.0;
        painter.line(Point::new(x, lines.y + 8.0), Point::new(x + 12.0, lines.bottom() - 8.0), *width, theme.text_primary);
    }
    let curve: Vec<Point> = (0..=64)
        .map(|step| {
            let t = step as f32 / 64.0;
            Point::new(lines.x + 140.0 + t * (lines.width - 150.0), lines.center().y - (t * std::f32::consts::TAU).sin() * 36.0)
        })
        .collect();
    painter.polyline(&curve, 2.0, data.blue);
    // Circles, rings and arcs.
    painter.text(area.origin(), "Circles, rings and arcs", CAPTION, theme.text_secondary);
    area.cut_top(CAPTION.line_height + SPACE_S, 0.0);
    let mut circles = area.cut_top(64.0, SPACE_L);
    let spot = circles.cut_left(64.0, SPACE_L);
    painter.circle(spot.center(), 28.0, data.red);
    let spot = circles.cut_left(64.0, SPACE_L);
    painter.ring(spot.center(), 28.0, 4.0, data.green);
    let spot = circles.cut_left(64.0, SPACE_L);
    painter.arc(spot.center(), 24.0, 8.0, 0.0, std::f32::consts::PI * 1.5, data.purple);
    let spot = circles.cut_left(64.0, SPACE_L);
    painter.arc(spot.center(), 24.0, 4.0, -std::f32::consts::FRAC_PI_2, std::f32::consts::PI, data.orange);
    // Polygons: a hexagon of each orientation, and a triangle.
    painter.text(area.origin(), "Polygons", CAPTION, theme.text_secondary);
    area.cut_top(CAPTION.line_height + SPACE_S, 0.0);
    let mut polygons = area.cut_top(72.0, SPACE_L);
    for pointy in [true, false] {
        let spot = polygons.cut_left(72.0, SPACE_L);
        let corners: Vec<Point> = (0..6)
            .map(|corner| {
                let angle = (corner as f32 * 60.0 + if pointy { 30.0 } else { 0.0 }).to_radians();
                spot.center() + Point::new(angle.cos(), angle.sin()) * 32.0
            })
            .collect();
        painter.polygon(&corners, if pointy { data.teal } else { data.yellow });
    }
    let spot = polygons.cut_left(72.0, SPACE_L);
    painter.polygon(
        &[Point::new(spot.center().x, spot.y + 4.0), Point::new(spot.right() - 4.0, spot.bottom() - 4.0), Point::new(spot.x + 4.0, spot.bottom() - 4.0)],
        data.pink,
    );
    // Gradients and corners.
    painter.text(area.origin(), "Gradients and corners", CAPTION, theme.text_secondary);
    area.cut_top(CAPTION.line_height + SPACE_S, 0.0);
    let mut gradients = area.cut_top(56.0, SPACE_L);
    painter.gradient(gradients.cut_left(160.0, SPACE_L), RADIUS_CONTROL, data.blue, data.green, false);
    painter.gradient(gradients.cut_left(56.0, SPACE_L), RADIUS_CONTROL, theme.accent, theme.background, true);
    painter.shape(gradients.cut_left(56.0, SPACE_L), Corners::top(16.0), data.orange, None);
    painter.shape(
        gradients.cut_left(56.0, SPACE_L),
        Corners::all(RADIUS_OVERLAY),
        theme.control_fill,
        Some(Stroke::elevated(theme.control_stroke, theme.control_stroke_secondary)),
    );
    let _ = ICON;
}

fn info_bars(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Info bars");
    let mut area = card.content;
    let bar = |area: &mut Rect| area.cut_top(48.0, SPACE_S);
    ui.info_bar("Informational", bar(&mut area), InfoBar::new(Severity::Informational, "Informational", "Something to know."));
    ui.info_bar("Success", bar(&mut area), InfoBar::new(Severity::Success, "Success", "It went well."));
    let caution = bar(&mut area);
    if showcase.caution_shown {
        if ui.info_bar("Caution", caution, InfoBar::new(Severity::Warning, "Caution", "Close me.").closable()).clicked {
            showcase.caution_shown = false;
            showcase.log("Closed the caution");
        }
    } else if ui.button("ShowCaution", caution.inset_by(0.0, 8.0, 0.0, 8.0), Button::new("Show the caution again")).clicked {
        showcase.caution_shown = true;
        showcase.log("Showed the caution");
    }
    ui.info_bar("Error", bar(&mut area), InfoBar::new(Severity::Error, "Error", "It went wrong."));
}

fn list(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "List");
    let matches = showcase.matches();
    let theme = ui.theme().clone();
    let summary = match showcase.opened {
        Some(index) => format!("{} of {} · Opened {}", matches.len(), showcase.items.len(), showcase.items[index]),
        None => format!("{} of {} · Double-click to open", matches.len(), showcase.items.len()),
    };
    ui.text_aligned(card.header, &summary, CAPTION, theme.text_secondary, Align::End);
    let row_height = ROW_HEIGHTS[showcase.density];
    // The selection is a position among the matches.
    let mut selected = showcase.selected.and_then(|item| matches.iter().position(|index| *index == item));
    let items = showcase.items.clone();
    let response = ui.list_view("Items", card.content, matches.len(), row_height, &mut selected, |ui, position, row, _| {
        let item = matches[position];
        let name = format!("Items/{position}");
        ui.set_label(&name, &items[item]);
        let color = theme.text_primary;
        let mut painter = ui.painter();
        let content = row.inset_by(SPACE_L, 0.0, SPACE_L, 0.0);
        painter.icon(Point::new(content.x + ICON / 2.0, row.center().y), Icon::Document, ICON, theme.text_secondary);
        painter.text_fit(content.inset_by(ICON + SPACE_M, 0.0, 60.0, 0.0), &items[item], BODY, color, Align::Start);
        painter.text_in(content, &format!("#{}", item + 1), CAPTION, theme.text_tertiary, Align::End);
        ui.tooltip(&name, &format!("{}, item {} of {}", items[item], item + 1, items.len()));
    });
    showcase.selected = selected.map(|position| matches[position]);
    if response.changed
        && let Some(item) = showcase.selected
    {
        showcase.log(format!("Selected {}", showcase.items[item]));
    }
    if let Some(position) = response.activated {
        let item = matches[position];
        showcase.opened = Some(item);
        showcase.log(format!("Opened {}", showcase.items[item]));
    }
}

fn interface(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, "Interface");
    let mut area = card.content;
    let theme = ui.theme().clone();
    let state = ui.state();
    let name = |member: &str| match state.get(member) {
        Some(user_interface::json::Value::String(name)) => name.clone(),
        Some(user_interface::json::Value::Object(_)) => state
            .get(member)
            .and_then(|tooltip| tooltip.get("Name"))
            .map(|name| name.to_json().trim_matches('"').to_string())
            .unwrap_or_default(),
        _ => "None".to_string(),
    };
    let pointer = match ui.pointer() {
        Some(point) => format!("{:.0}, {:.0}", point.x, point.y),
        None => "Not in the window".to_string(),
    };
    let label_width = 140.0;
    let fields = [
        ("Window focus", if ui.window_focused() { "Yes".to_string() } else { "No".to_string() }),
        ("Pointer", pointer),
        ("Under the pointer", name("Hovered")),
        ("Pressed", name("Pressed")),
        ("Keyboard focus", name("Focused")),
        ("Open popup", name("Popup")),
        ("Tooltip", name("Tooltip")),
    ];
    for (label, value) in fields {
        ui.field(area.cut_top(BODY.line_height, SPACE_XS), label, label_width, &value);
    }
    area.cut_top(SPACE_S, 0.0);
    let held: Vec<String> = ui.input().keys_held().iter().map(|key| key.name()).collect();
    let held = if held.is_empty() { "None".to_string() } else { held.join(" ") };
    ui.field(area.cut_top(BODY.line_height, SPACE_M), "Keys held", label_width, &held);

    // The gamepad: its sticks, its triggers, and the buttons held.
    let gamepad = *ui.gamepad();
    ui.text(area.cut_top(BODY.line_height, SPACE_S), "Gamepad", BODY_STRONG, theme.text_primary);
    let mut pad = area.cut_top(96.0, SPACE_S);
    let mut painter = ui.painter();
    for stick in [gamepad.left_stick, gamepad.right_stick] {
        let spot = pad.cut_left(96.0, SPACE_M);
        let radius = 44.0;
        painter.circle(spot.center(), radius, theme.control_alt_fill_secondary);
        painter.ring(spot.center(), radius, 1.0, theme.control_strong_stroke);
        painter.line(Point::new(spot.center().x - radius, spot.center().y), Point::new(spot.center().x + radius, spot.center().y), 1.0, theme.divider);
        painter.line(Point::new(spot.center().x, spot.center().y - radius), Point::new(spot.center().x, spot.center().y + radius), 1.0, theme.divider);
        let dot = spot.center() + Point::new(stick.x, -stick.y) * radius;
        painter.circle(dot, 6.0, theme.accent);
    }
    for trigger in [gamepad.left_trigger, gamepad.right_trigger] {
        let bar = pad.cut_left(16.0, SPACE_S);
        painter.rounded(bar, RADIUS_CONTROL, theme.control_alt_fill_secondary);
        let fill = Rect::new(bar.x, bar.bottom() - bar.height * trigger, bar.width, bar.height * trigger);
        painter.rounded(fill, RADIUS_CONTROL, theme.accent);
    }
    let buttons = gamepad.buttons.names();
    let buttons = if !gamepad.connected {
        "No gamepad".to_string()
    } else if buttons.is_empty() {
        "No buttons held".to_string()
    } else {
        buttons.join(" ")
    };
    let line = area.cut_top(CAPTION.line_height, SPACE_L);
    painter.text_fit(line, &buttons, CAPTION, theme.text_secondary, Align::Start);

    // The log, newest first.
    painter.text(area.origin(), "Log", BODY_STRONG, theme.text_primary);
    area.cut_top(BODY.line_height + SPACE_S, 0.0);
    painter.push_clip(area);
    for (index, line) in showcase.log.iter().enumerate() {
        let row = area.cut_top(MONO.line_height, 0.0);
        if row.height < MONO.line_height {
            break;
        }
        let color = if index == 0 { theme.text_primary } else { theme.text_secondary };
        painter.text_fit(row, line, MONO, color, Align::Start);
    }
    painter.pop_clip();
}
