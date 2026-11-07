// The showcase's page: an app bar with its icon and title, a menu row under
// it with a menu bar in line with the icon and the cards, five columns of cards
// that show every widget, and a status bar. The cards at the bottom of their
// columns take the 71 pixels that borderless fullscreen adds.

use user_interface::tokens::{
    BODY, BODY_LARGE, BODY_STRONG, CAPTION, CARD_GAP, CONTROL_HEIGHT, DISPLAY, ELEVATION_DIALOG, ELEVATION_FLYOUT, ELEVATION_TOOLTIP, ICON, ICON_LARGE, MONO, MONO_CAPTION, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S, SPACE_XS,
    SUBTITLE, TITLE, TITLE_LARGE,
};
use user_interface::{
    Align, AppBar, Border, Brush, Button, Card, CheckBox, Color, ColorField, ColorPicker, ComboBox, Corners, Dialog, DialogResult, Expander,
    Form, HyperlinkButton, Icon, InfoBar, ListView, MenuBar, MenuItem, NumberBox, Point, RadioGroup, Rect, Segmented, SelectorBar,
    Severity, Size, Slider, Span, Status, Swatch, TextBox, TextStyle, ThemeKind, ToggleButton, ToggleSwitch, TooltipLine, Ui,
};

use crate::showcase::{ALIGNMENTS, OPTIONS, SIZES, Showcase, TABS, TOOLS};

// The theme's colors that the combo box offers, and what each is for.
pub const ROLES: [&str; 5] = ["Accent", "Success", "Caution", "Critical", "Attention"];
const ROLE_USES: [&str; 5] = [
    "What's on, chosen, or the one command that matters most",
    "What went well",
    "What needs care",
    "What went wrong",
    "What's worth knowing",
];
const PALETTE: [u32; 8] = [0xD13438, 0xCA5010, 0xC19C00, 0x107C10, 0x038387, 0x0078D4, 0x8764B8, 0xE3008C];

const FILE_MENU: &[MenuItem] = &[
    MenuItem::new("New").icon(Icon::Document).accelerator("Ctrl+N"),
    MenuItem::new("Open").icon(Icon::OpenFile).accelerator("Ctrl+O"),
    MenuItem::new("Save").icon(Icon::Save).accelerator("Ctrl+S"),
    MenuItem::separator(),
    MenuItem::new("Reset").icon(Icon::Refresh).accelerator("Ctrl+R"),
];
const VIEW_MENU: &[MenuItem] = &[
    MenuItem::new("VSync").accelerator("F10"),
    MenuItem::new("Fullscreen").icon(Icon::FullScreen).accelerator("F11"),
    MenuItem::submenu("Theme", THEME_MENU).icon(Icon::Color),
    MenuItem::separator(),
    MenuItem::new("Disabled").enabled(false),
];
const THEME_MENU: &[MenuItem] = &[MenuItem::new("Dark theme").accelerator("F9"), MenuItem::new("Tooltips").accelerator("F1")];
const HELP_MENU: &[MenuItem] = &[MenuItem::new("Ask a question").icon(Icon::Help)];
const MENUS: &[(&str, &[MenuItem])] = &[("File", FILE_MENU), ("View", VIEW_MENU), ("Help", HELP_MENU)];
const CONTEXT_MENU: &[MenuItem] = &[
    MenuItem::new("Cut").icon(Icon::Cut).accelerator("Ctrl+X"),
    MenuItem::new("Copy").icon(Icon::Copy).accelerator("Ctrl+C"),
    MenuItem::new("Paste").icon(Icon::Paste).accelerator("Ctrl+V"),
    MenuItem::separator(),
    MenuItem::new("Pinned"),
];

// The cards' heights; the bottom card of each column takes the rest.
const BUTTONS_HEIGHT: f32 = 260.0;
const TOGGLES_HEIGHT: f32 = 344.0;
const SLIDERS_HEIGHT: f32 = 492.0;
const CHOICES_HEIGHT: f32 = 320.0;
const INFO_HEIGHT: f32 = 324.0;
const COLOR_HEIGHT: f32 = 492.0;

pub fn draw(showcase: &mut Showcase, ui: &mut Ui) {
    let dark = ui.theme().kind == ThemeKind::Dark;
    let subtitle = format!("Fluent 2 · WinUI 3 · {} theme", ui.theme().kind.name());
    ui.app_bar(AppBar::new(Icon::Library, "UiShowcase").subtitle(&subtitle));
    let row = ui.menu_row();
    let menu = MenuBar::new(MENUS);
    let menu_width = menu.width(ui);
    if let Some(label) = ui.add("Menu", row.left_strip(menu_width), menu) {
        showcase.command(ui, label);
    }
    let theme_button = Rect::new(row.right() - 32.0, row.y, 32.0, row.height).centered(Size::square(32.0));
    let response = ui.add("Theme", theme_button, Button::icon_only(if dark { Icon::Light } else { Icon::Moon }).subtle());
    ui.tooltip(&response, "Switch the theme (F9)");
    if response.clicked {
        ui.toggle_theme();
    }
    let stats = ui.stats();
    let fps = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    ui.status_bar(&[
        Status::Shortcut("F1", "Tooltips"),
        Status::Shortcut("Tab", "Next widget"),
        Status::Shortcut("Ctrl+R", "Reset"),
        Status::Shortcut("F9", "Theme"),
        Status::Shortcut("F10", "VSync"),
        Status::Shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::Indicator(Severity::Success, &fps),
        Status::Text(&environment.gpu),
        Status::Text(&display),
    ]);
    let page = ui.page();
    let [first, second, third, fourth, fifth] = page.columns::<5>(CARD_GAP);

    let mut column = first;
    buttons(showcase, ui, column.cut_top(BUTTONS_HEIGHT, CARD_GAP));
    toggles(showcase, ui, column.cut_top(TOGGLES_HEIGHT, CARD_GAP));
    fields(showcase, ui, column);

    let mut column = second;
    sliders(showcase, ui, column.cut_top(SLIDERS_HEIGHT, CARD_GAP));
    choices(showcase, ui, column.cut_top(CHOICES_HEIGHT, CARD_GAP));
    menus(showcase, ui, column);

    tabs(showcase, ui, third);

    let mut column = fourth;
    info_bars(showcase, ui, column.cut_top(INFO_HEIGHT, CARD_GAP));
    colors(showcase, ui, column.cut_top(COLOR_HEIGHT, CARD_GAP));
    list(showcase, ui, column);

    interface(showcase, ui, fifth);
    dialog(showcase, ui);
}

fn buttons(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Buttons"));
    let mut area = card.content;
    let [standard, accent, subtle] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
    for (index, (rect, button)) in [(standard, Button::new("Standard")), (accent, Button::new("Accent").accent()), (subtle, Button::new("Subtle").subtle())]
        .into_iter()
        .enumerate()
    {
        if ui.add(crate::showcase::BUTTONS[index], rect, button).clicked {
            showcase.presses[index] += 1;
            showcase.log(format!("Pressed {}", crate::showcase::BUTTONS[index]));
        }
    }
    let [disabled, disabled_accent, link] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
    ui.with_enabled(false, |ui| {
        ui.add("Disabled", disabled, Button::new("Disabled"));
        ui.add("DisabledAccent", disabled_accent, Button::new("Disabled").accent());
    });
    if ui.add("Link", link, HyperlinkButton::new("Learn more")).clicked {
        showcase.link_clicks += 1;
        showcase.log("Followed the link");
    }
    let [save, pin, more] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
    let response = ui.add("Save", save, Button::new("Save").icon(Icon::Save));
    ui.tooltip(&response, "Saves nothing, but counts");
    if response.clicked {
        showcase.presses[3] += 1;
    }
    if ui.add("Pin", pin, ToggleButton::new(&mut showcase.pinned, Button::new("Pin").icon(Icon::Pin))).changed {
        showcase.log(if showcase.pinned { "Pinned" } else { "Unpinned" });
    }
    let response = ui.add("More", more, Button::new("More").chevron());
    if let Some(label) = ui.flyout_menu("MoreMenu", &response, CONTEXT_MENU) {
        showcase.command(ui, label);
        if label == "Pinned" {
            showcase.pinned = !showcase.pinned;
        }
    }
    let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
    let minus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
    let count = row.cut_left(48.0, SPACE_S);
    let plus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
    let response = ui.add("Remove", minus, Button::icon_only(Icon::Remove));
    ui.tooltip(&response, "One less");
    if response.clicked {
        showcase.count -= 1;
    }
    let primary = ui.theme().text_primary;
    ui.text_aligned(count, &showcase.count.to_string(), BODY, primary, Align::Center);
    let response = ui.add("Add", plus, Button::icon_only(Icon::Add));
    ui.tooltip(&response, "One more");
    if response.clicked {
        showcase.count += 1;
    }
    let presses = format!("Pressed: Standard {}, Accent {}, Subtle {}, Save {}", showcase.presses[0], showcase.presses[1], showcase.presses[2], showcase.presses[3]);
    ui.caption(area.cut_top(CAPTION.line_height, 0.0), &presses);
}

fn toggles(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Checkboxes and switches"));
    let mut area = card.content;
    if ui.add("OptionsEnabled", area.cut_top(CONTROL_HEIGHT, 0.0), CheckBox::new(&mut showcase.options_enabled, "Enable the options under this")).changed {
        showcase.log(format!("Options {}", if showcase.options_enabled { "enabled" } else { "disabled" }));
    }
    ui.with_enabled(showcase.options_enabled, |ui| {
        for (index, option) in OPTIONS.iter().enumerate() {
            let row = area.cut_top(CONTROL_HEIGHT, 0.0);
            ui.add(&format!("Option/{option}"), Rect::new(row.x + 28.0, row.y, row.width - 28.0, row.height), CheckBox::new(&mut showcase.options[index], option));
        }
    });
    area.cut_top(SPACE_S, 0.0);
    ui.add("Tooltips", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(&mut showcase.tooltips, "Tooltips (F1)"));
    let mut dark = ui.theme().kind == ThemeKind::Dark;
    if ui.add("DarkTheme", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(&mut dark, "Dark theme (F9)")).changed {
        ui.toggle_theme();
    }
    if ui.add("Filling", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(&mut showcase.filling, "Fill the progress bar")).changed {
        showcase.fill_started = ui.time();
    }
}

fn choices(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Radio buttons and segments"));
    let mut area = card.content;
    let mut radios = area.cut_top(RadioGroup::height(SIZES.len()), SPACE_M);
    let preview = radios.cut_right(96.0, 0.0);
    ui.add("Size", radios, RadioGroup::new(&mut showcase.size, &SIZES));
    let side = 24.0 + 16.0 * showcase.size as f32;
    let accent = ui.theme().accent;
    ui.painter().rounded(preview.centered(Size::square(side)), RADIUS_CONTROL, accent);
    ui.add("Alignment", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(&mut showcase.alignment, &ALIGNMENTS));
    let line = area.cut_top(CONTROL_HEIGHT, SPACE_M);
    let align = [Align::Start, Align::Center, Align::End][showcase.alignment];
    let (fill, primary) = (ui.theme().control_alt_fill_secondary, ui.theme().text_primary);
    ui.painter().rounded(line, RADIUS_CONTROL, fill);
    ui.text_aligned(line.inset(SPACE_M), "Aligned text", BODY, primary, align);
    let tools: Vec<(Icon, &str)> = [Icon::Brush, Icon::Fill, Icon::Erase, Icon::Eyedropper].into_iter().zip(TOOLS).collect();
    if ui.add("Tool", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(&mut showcase.tool, &tools)).changed {
        showcase.log(format!("Tool: {}", TOOLS[showcase.tool]));
    }
    ui.caption(area.cut_top(CAPTION.line_height, 0.0), &format!("{}: segments with icons, like a tool's modes", TOOLS[showcase.tool]));
}

fn sliders(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Sliders and progress"));
    let mut form = Form::new(card.content);
    let height = Slider::<f32>::height(true);
    ui.add("Volume", form.take(height), Slider::new(&mut showcase.volume, 0.0, 100.0).step(1.0).header("Volume").show_value(0));
    ui.add("Opacity", form.take(height), Slider::new(&mut showcase.opacity, 0.0, 1.0).step(0.001).large_step(0.1).header("Opacity").show_value(3));
    ui.add("Angle", form.take(height), Slider::new(&mut showcase.angle, 0.0, 360.0).step(0.1).large_step(15.0).header("Angle").show_value(1).suffix("°"));
    let mut disabled = 30.0f32;
    ui.add_enabled(false, "DisabledSlider", form.take(height), Slider::new(&mut disabled, 0.0, 100.0).header("Disabled").show_value(0));
    let secondary = ui.theme().text_secondary;
    let fill = if showcase.filling { ((ui.time() - showcase.fill_started) % 4.0 / 4.0) as f32 } else { 0.78 };
    for (label, value) in [("At the volume", Some(showcase.volume / 100.0)), ("Fills every 4 seconds", Some(fill)), ("Of unknown length", None)] {
        let row = form.take(BODY.line_height + SPACE_XS + 4.0);
        ui.text(row.top_strip(BODY.line_height), label, BODY, secondary);
        if let Some(value) = value {
            ui.text_aligned(row.top_strip(BODY.line_height), &format!("{:.0}%", value * 100.0), BODY, secondary, Align::End);
        }
        ui.progress_bar(row.bottom_strip(4.0), value);
    }
    let rings = form.take(40.0);
    ui.progress_ring(Point::new(rings.x + 20.0, rings.center().y), 32.0, Some(showcase.volume / 100.0));
    ui.progress_ring(Point::new(rings.x + 68.0, rings.center().y), 32.0, None);
    ui.caption(Rect::new(rings.x + 104.0, rings.y, rings.width - 104.0, rings.height), "Progress rings, at the volume, and spinning");
}

fn fields(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Combo boxes and fields"));
    let mut form = Form::new(card.content);
    let mut row = form.control(ui, "A color of the theme");
    let swatch = row.cut_right(CONTROL_HEIGHT, SPACE_S);
    if ui.add("Role", row, ComboBox::new(&mut showcase.role, &ROLES)).changed {
        showcase.log(format!("Role: {}", ROLES[showcase.role]));
    }
    let theme = ui.theme();
    let color = [theme.accent, theme.success, theme.caution, theme.critical, theme.attention][showcase.role];
    ui.painter().rounded(swatch, RADIUS_CONTROL, color);
    form.caption(ui, ROLE_USES[showcase.role]);
    let mut row = form.control(ui, "Your name");
    let greet = row.cut_right(96.0, SPACE_S);
    let response = ui.add("Name", row, TextBox::new(&mut showcase.name).placeholder("Type a name").max_length(40));
    let greet_clicked = ui.add("Greet", greet, Button::new("Greet").accent()).clicked;
    if response.submitted || greet_clicked {
        let name = showcase.name.trim();
        showcase.greeting = Some(if name.is_empty() { "Hello, whoever you are!".to_string() } else { format!("Hello, {name}!") });
    }
    let row = form.control(ui, "Search the list");
    ui.add("Search", row, TextBox::new(&mut showcase.search).placeholder("Search").button(Icon::Search));
    let [quantity, scale] = form.take(BODY.line_height + 8.0 + CONTROL_HEIGHT).columns::<2>(SPACE_S);
    let (primary, _) = (ui.theme().text_primary, ());
    ui.text(quantity.top_strip(BODY.line_height), "Quantity", BODY, primary);
    ui.add("Quantity", quantity.bottom_strip(CONTROL_HEIGHT), NumberBox::new(&mut showcase.quantity).range(0.0, 99.0));
    ui.text(scale.top_strip(BODY.line_height), "Scale", BODY, primary);
    ui.add("Scale", scale.bottom_strip(CONTROL_HEIGHT), NumberBox::new(&mut showcase.scale).range(0.25, 4.0).step(0.25).decimals(2));
    if let Some(greeting) = showcase.greeting.clone() {
        let bar = InfoBar::new(Severity::Success, "", &greeting).closable();
        if ui.add("Greeting", form.take(48.0), bar).closed {
            showcase.greeting = None;
        }
    }
}

fn menus(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Menus and dialogs"));
    let mut form = Form::new(card.content);
    let [ask, choice] = form.columns::<2>(CONTROL_HEIGHT);
    if ui.add("Ask", ask, Button::new("Ask a question").icon(Icon::Help)).clicked {
        showcase.dialog_open = true;
    }
    let result = showcase.dialog_result.clone().unwrap_or_else(|| "Not asked yet".to_string());
    ui.caption(choice, &format!("Answer: {result}"));
    let area = form.take(64.0);
    let response = ui.interact("ContextArea", area, user_interface::Sense::new(true, true, false, false, user_interface::Keys::None), "Area");
    let theme = ui.theme();
    let (fill, stroke, text) = (theme.control_alt_fill_secondary, theme.control_strong_stroke, theme.text_secondary);
    ui.painter().shape(area, RADIUS_CONTROL, fill, Some(Border::new(stroke.fade(0.5), 1.0)));
    ui.text_aligned(area, "Right-click here for a context menu", BODY, text, Align::Center);
    if let Some(label) = ui.context_menu("ContextMenu", &response, CONTEXT_MENU) {
        showcase.command(ui, label);
    }
    let chosen = showcase.menu_choice.clone().unwrap_or_else(|| "Nothing yet".to_string());
    ui.caption(form.take(CAPTION.line_height), &format!("Chose from a menu: {chosen}"));
    let header = form.take(48.0);
    ui.add("Expander", header, Expander::new(&mut showcase.expanded, "An expander").caption("Shows and hides what's under it"));
    if showcase.expanded {
        let rest = form.rest();
        let content = Rect::new(header.x, header.bottom(), header.width, rest.bottom() - header.bottom());
        let inner = ui.expander_content(content);
        let secondary = ui.theme().text_secondary;
        ui.paragraph(inner, "Expander content sits on the secondary card fill, under its header, as WinUI's does.", BODY, secondary);
    }
}

fn dialog(showcase: &mut Showcase, ui: &mut Ui) {
    if !showcase.dialog_open {
        return;
    }
    let dialog = Dialog::new("Save your work?", "This is a content dialog: it covers the page with smoke until you choose.").primary("Save").secondary("Don't save").close("Cancel");
    if let Some(result) = ui.dialog("Question", dialog) {
        let answer = match result {
            DialogResult::Primary => "Save",
            DialogResult::Secondary => "Don't save",
            DialogResult::Close => "Cancel",
        };
        showcase.dialog_result = Some(answer.to_string());
        showcase.dialog_open = false;
        showcase.log(format!("Answered {answer}"));
    }
}

fn tabs(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new(""));
    let mut area = card.content;
    ui.add("Tabs", area.cut_top(48.0, SPACE_S), SelectorBar::new(&mut showcase.tab, &TABS));
    match showcase.tab {
        0 => type_ramp(ui, area),
        1 => icons(ui, area),
        2 => theme_colors(ui, area),
        _ => shapes(ui, area),
    }
}

fn type_ramp(ui: &mut Ui, mut area: Rect) {
    let styles: [(&str, TextStyle, &str); 11] = [
        ("Caption · 12 on 16", CAPTION, "The quick brown fox jumps"),
        ("Body · 14 on 20", BODY, "The quick brown fox jumps"),
        ("Body strong · 14 on 20", BODY_STRONG, "The quick brown fox jumps"),
        ("Body large · 18 on 24", BODY_LARGE, "The quick brown fox jumps"),
        ("Subtitle · 20 on 28", SUBTITLE, "The quick brown fox jumps"),
        ("Title · 28 on 36", TITLE, "The quick brown fox jumps"),
        ("Title large · 40 on 52", TITLE_LARGE, "Fluent"),
        ("Display · 68 on 92", DISPLAY, "Fluent"),
        ("Mono · 13 on 20", MONO, "The quick brown fox jumps"),
        ("Mono strong · 13 on 20", MONO.weight(user_interface::Weight::Semibold), "The quick brown fox jumps"),
        ("Mono caption · 12 on 16", MONO_CAPTION, "The quick brown fox jumps"),
    ];
    let (secondary, primary) = (ui.theme().text_secondary, ui.theme().text_primary);
    for (label, style, sample) in styles {
        let row = area.cut_top(style.line_height, SPACE_S);
        ui.text(row.left_strip(148.0), label, CAPTION, secondary);
        ui.text(Rect::new(row.x + 148.0, row.y, row.width - 148.0, row.height), sample, style, primary);
    }
    let accent = ui.theme().accent_text;
    ui.spans(
        Point::new(area.x, area.y + SPACE_S),
        &[
            Span::new("Spans: ", BODY, secondary),
            Span::new("several styles ", BODY_STRONG, primary),
            Span::new("and colors ", BODY, accent),
            Span::new("on a line", CAPTION, secondary),
        ],
    );
}

fn icons(ui: &mut Ui, area: Rect) {
    let columns = 4;
    let cell = Size::new(area.width / columns as f32, 40.0);
    let primary = ui.theme().text_primary;
    let secondary = ui.theme().text_secondary;
    for (index, icon) in user_interface::Icon::ALL.iter().enumerate() {
        let x = area.x + cell.width * (index % columns) as f32;
        let y = area.y + cell.height * (index / columns) as f32;
        if y + cell.height > area.bottom() {
            break;
        }
        ui.icon(Rect::new(x, y, 32.0, cell.height), *icon, ICON_LARGE, primary);
        ui.text(Rect::new(x + 36.0, y, cell.width - 40.0, cell.height), icon.name(), CAPTION, secondary);
    }
}

fn theme_colors(ui: &mut Ui, area: Rect) {
    let roles = ui.theme().roles();
    let row_height = 26.0;
    let secondary = ui.theme().text_secondary;
    let stroke = ui.theme().control_stroke_secondary;
    let [left, right] = area.columns::<2>(SPACE_L);
    let per_column = (area.height / row_height).floor() as usize;
    for (index, (name, color)) in roles.iter().enumerate() {
        let column = if index < per_column { left } else { right };
        let y = column.y + row_height * (index % per_column) as f32;
        if index >= per_column * 2 {
            break;
        }
        let swatch = Rect::new(column.x, y + 3.0, 20.0, 20.0);
        ui.painter().shape(swatch, RADIUS_CONTROL, *color, Some(Border::new(stroke, 1.0)));
        ui.text(Rect::new(column.x + 28.0, y, column.width - 120.0, row_height), name, CAPTION, secondary);
        ui.text_aligned(Rect::new(column.x, y, column.width, row_height), &color.to_hex(), CAPTION, secondary, Align::End);
    }
}

fn shapes(ui: &mut Ui, area: Rect) {
    let [elevations, rest] = area.rows::<2>(SPACE_L);
    let theme = ui.theme().clone();
    let tiles = elevations.top_strip(120.0).inset(SPACE_L).columns::<3>(SPACE_L * 2.0);
    let names = ["Tooltip · 16", "Flyout · 32", "Dialog · 128"];
    for ((tile, elevation), name) in tiles.iter().zip([ELEVATION_TOOLTIP, ELEVATION_FLYOUT, ELEVATION_DIALOG]).zip(names) {
        let mut painter = ui.painter();
        painter.shadow(*tile, RADIUS_OVERLAY, elevation);
        painter.shape(*tile, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
        painter.text_in(*tile, name, CAPTION, theme.text_secondary, Align::Center);
    }
    let mut painter = ui.painter();
    let mut y = elevations.y + 160.0;
    let data = theme.data.all();
    for (index, color) in data.iter().enumerate() {
        let x = elevations.x + 24.0 + index as f32 * 52.0;
        painter.circle(Point::new(x, y), 16.0, *color);
    }
    y += 52.0;
    painter.line(Point::new(elevations.x + 8.0, y), Point::new(elevations.right() - 8.0, y + 24.0), 2.0, theme.accent);
    painter.polyline(&[Point::new(elevations.x + 8.0, y + 56.0), Point::new(elevations.x + 120.0, y + 36.0), Point::new(elevations.x + 240.0, y + 72.0), Point::new(elevations.x + 360.0, y + 40.0)], 2.0, theme.data.orange);
    let center = Point::new(rest.x + 48.0, rest.y + 48.0);
    painter.ring(center, 40.0, 4.0, theme.control_strong_stroke);
    painter.arc(center, 38.0, 4.0, 0.0, 4.0, theme.accent);
    painter.polygon(&[Point::new(rest.x + 140.0, rest.y + 88.0), Point::new(rest.x + 180.0, rest.y + 8.0), Point::new(rest.x + 220.0, rest.y + 88.0)], theme.data.green);
    let hexagon: Vec<Point> = (0..6).map(|index| {
        let angle = std::f32::consts::TAU * index as f32 / 6.0;
        Point::new(rest.x + 300.0 + 40.0 * angle.cos(), rest.y + 48.0 + 40.0 * angle.sin())
    }).collect();
    painter.polygon(&hexagon, theme.data.purple);
    let gradient = Rect::new(rest.x, rest.y + 120.0, rest.width, 32.0);
    painter.shape(gradient, RADIUS_CONTROL, Brush::horizontal(gradient, theme.accent, theme.data.pink), None);
    let vertical = Rect::new(rest.x, rest.y + 168.0, 120.0, 80.0);
    painter.shape(vertical, Corners::top(16.0), Brush::vertical(vertical, theme.data.teal, theme.data.blue), None);
    let bordered = Rect::new(rest.x + 140.0, rest.y + 168.0, 120.0, 80.0);
    painter.shape(bordered, RADIUS_CONTROL, theme.control_fill, Some(user_interface::Elevation::Control.border(&theme, bordered)));
    painter.text_in(bordered, "Elevation border", CAPTION, theme.text_secondary, Align::Center);
}

fn info_bars(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Info bars"));
    let mut area = card.content;
    let bars = [
        ("Informational", InfoBar::new(Severity::Informational, "Informational", "Something to know.")),
        ("Success", InfoBar::new(Severity::Success, "Success", "It went well.")),
        ("Caution", InfoBar::new(Severity::Warning, "Caution", "Close me.").closable()),
        ("Error", InfoBar::new(Severity::Error, "Error", "It went wrong.")),
    ];
    for (name, bar) in bars {
        if name == "Caution" && !showcase.caution_shown {
            let row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
            if ui.add("BringBack", row.left_strip(160.0), Button::new("Bring it back")).clicked {
                showcase.caution_shown = true;
            }
            continue;
        }
        let height = ui.info_bar_height(area.width, &bar);
        if ui.add(name, area.cut_top(height, SPACE_S), bar).closed {
            showcase.caution_shown = false;
            showcase.log("Closed the caution");
        }
    }
}

fn colors(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Color"));
    let mut area = card.content;
    let picker_width = 256.0f32.min(area.width);
    let picker = area.cut_top(ColorPicker::height(picker_width), SPACE_M).left_strip(picker_width);
    if ui.add("Picker", picker, ColorPicker::new(&mut showcase.color)).changed {
        showcase.log(format!("Color {}", showcase.color.to_hex()));
    }
    let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_M);
    let field = row.cut_left(160.0, SPACE_M);
    ui.add("Hex", field, ColorField::new(&mut showcase.color));
    for (index, color) in PALETTE.iter().enumerate() {
        let swatch = row.cut_left(CONTROL_HEIGHT - 4.0, SPACE_XS).centered(Size::square(24.0));
        let color = Color::hex(*color);
        if ui.add(&format!("Palette/{index}"), swatch, Swatch::new(color, showcase.color == color)).clicked {
            showcase.color = color;
        }
    }
}

fn list(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let matches = showcase.matches();
    let subtitle = format!("{} of {} · Double-click to open", matches.len(), showcase.items.len());
    let card = ui.card(rect, Card::new("List").subtitle(&subtitle));
    let items = showcase.items.clone();
    let mut selected = showcase.selected.and_then(|item| matches.iter().position(|index| *index == item));
    let response = ui.add(
        "Items",
        card.content,
        ListView::new(matches.len(), 40.0, |ui, row, rect, _| {
            let item = matches[row];
            let theme = ui.theme();
            let (primary, tertiary) = (theme.text_primary, theme.text_tertiary);
            ui.icon(Rect::new(rect.x, rect.y, ICON, rect.height), Icon::Document, ICON, primary);
            ui.text(Rect::new(rect.x + ICON + SPACE_M, rect.y, rect.width - 80.0, rect.height), &items[item], BODY, primary);
            ui.text_aligned(rect, &format!("#{}", item + 1), CAPTION, tertiary, Align::End);
            ui.set_label(&format!("Items/{row}"), &items[item]);
        })
        .selection(&mut selected),
    );
    showcase.selected = selected.map(|row| matches[row]);
    if let Some(row) = response.activated {
        showcase.opened = Some(matches[row]);
        showcase.log(format!("Opened {}", items[matches[row]]));
    }
}

fn interface(showcase: &mut Showcase, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Interface"));
    let mut area = card.content;
    let pointer = ui.pointer().map_or("Not in the window".to_string(), |point| format!("{:.0}, {:.0}", point.x, point.y));
    let rows = [
        ("Window focus", if ui.window_focused() { "Yes".to_string() } else { "No".to_string() }),
        ("Pointer", pointer),
        ("Under the pointer", ui.hovered_name().unwrap_or("None").to_string()),
        ("Pressed", ui.pressed_name().unwrap_or("None").to_string()),
        ("Keyboard focus", ui.focused_name().unwrap_or("None").to_string()),
        ("Open popup", ui.popup_name().unwrap_or("None").to_string()),
        ("Tooltip", ui.tooltip_name().unwrap_or("None").to_string()),
        ("Dialog", if ui.modal() { "Open".to_string() } else { "None".to_string() }),
    ];
    for (label, value) in rows {
        ui.field(area.cut_top(24.0, 0.0), label, 160.0, &value);
    }
    let held: Vec<String> = ui.input().keys_held().iter().map(|key| key.name()).collect();
    ui.field(area.cut_top(24.0, SPACE_S), "Keys held", 160.0, &if held.is_empty() { "None".to_string() } else { held.join(", ") });
    let theme = ui.theme().clone();
    ui.heading(area.cut_top(BODY_STRONG.line_height, SPACE_S), "Gamepad");
    let gamepad = *ui.gamepad();
    let mut sticks = area.cut_top(96.0, SPACE_S);
    for stick in [gamepad.left_stick, gamepad.right_stick] {
        let box_rect = sticks.cut_left(96.0, SPACE_M);
        let center = box_rect.center();
        let mut painter = ui.painter();
        painter.ring(center, 44.0, 1.0, theme.control_strong_stroke);
        painter.line(Point::new(center.x - 44.0, center.y), Point::new(center.x + 44.0, center.y), 1.0, theme.divider);
        painter.line(Point::new(center.x, center.y - 44.0), Point::new(center.x, center.y + 44.0), 1.0, theme.divider);
        painter.circle(Point::new(center.x + stick.x * 40.0, center.y - stick.y * 40.0), 5.0, theme.accent);
    }
    for trigger in [gamepad.left_trigger, gamepad.right_trigger] {
        let bar = sticks.cut_left(12.0, SPACE_S);
        let mut painter = ui.painter();
        painter.rounded(bar, 6.0, theme.control_alt_fill_tertiary);
        let height = bar.height * trigger.clamp(0.0, 1.0);
        painter.rounded(Rect::new(bar.x, bar.bottom() - height, bar.width, height), 6.0, theme.accent);
    }
    let buttons = gamepad.buttons.names();
    let pressed = if !gamepad.connected { "No gamepad".to_string() } else if buttons.is_empty() { "No buttons held".to_string() } else { buttons.join(", ") };
    ui.caption(area.cut_top(CAPTION.line_height, SPACE_L), &pressed);
    let mut badges = area.cut_top(24.0, SPACE_L);
    let primary = theme.text_secondary;
    let width = ui.keys(Point::new(badges.x, badges.center().y - 10.0), "Ctrl+Shift+S");
    badges.cut_left(width + SPACE_M, 0.0);
    let size = ui.badge(Point::new(badges.x + 12.0, badges.center().y), "3");
    badges.cut_left(size.width + SPACE_S + 8.0, 0.0);
    ui.badge(Point::new(badges.x + 6.0, badges.center().y), "");
    ui.text(Rect::new(badges.x + 20.0, badges.y, badges.width - 20.0, badges.height), "Keycaps, badges and a dot", CAPTION, primary);
    let response = ui.add("Details", area.cut_top(CONTROL_HEIGHT, SPACE_L).left_strip(200.0), Button::new("Details").icon(Icon::Info));
    ui.tooltip_lines(&response, &[TooltipLine::Heading("Details"), TooltipLine::Pair("Theme", ui.theme().kind.name()), TooltipLine::Separator, TooltipLine::Text("Tooltips can have lines, columns and separators.")]);
    ui.heading(area.cut_top(BODY_STRONG.line_height, SPACE_S), "Log");
    let lines = ((area.height / 20.0).floor() as usize).min(showcase.log.len());
    for line in showcase.log.iter().take(lines) {
        let row = area.cut_top(20.0, 0.0);
        ui.text(row, line, MONO, theme.text_secondary);
    }
}
