// The showcase's page: an app bar with its icon and title, a menu row under
// it with a menu bar in line with the icon and the cards, five columns of cards
// that show every widget, and a status bar. The cards at the bottom of their
// columns take the 71 pixels that borderless fullscreen adds.
//
// `build` makes the tree, and `sync` keeps what the controls show true to
// the showcase's state, every frame.

use user_interface::tokens::{
    BODY, BODY_LARGE, BODY_STRONG, CAPTION, CARD_GAP, CONTROL_HEIGHT, CURVE_FAST_OUT_SLOW_IN, DISPLAY, DURATION_FAST, ELEVATION_DIALOG,
    ELEVATION_FLYOUT, ELEVATION_TOOLTIP, ICON_LARGE, MONO, MONO_CAPTION, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S, SPACE_XS,
    SUBTITLE, TITLE, TITLE_LARGE,
};
use user_interface::{
    Align, AppBar, Border, Brush, Button, Canvas, Card, CheckBox, Color, ColorField, ColorPicker, ComboBox, Corners, Dialog, Expander, Field,
    Form, Gamepad, HyperlinkButton, Icon, InfoBar, ListRow, ListView, MenuBar, MenuItem, NumberBox, Point, ProgressBar, RadioGroup, Rect,
    Segmented, SelectorBar, Severity, Size, Slider, Span, Status, Swatch, Text, TextBox, TextStyle, Tip, ToggleButton, ToggleSwitch,
    Ui, id,
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

fn file_menu() -> Vec<MenuItem> {
    vec![
        MenuItem::new("New").icon(Icon::Document).accelerator("Ctrl+N"),
        MenuItem::new("Open").icon(Icon::OpenFile).accelerator("Ctrl+O"),
        MenuItem::new("Save").icon(Icon::Save).accelerator("Ctrl+S"),
        MenuItem::separator(),
        MenuItem::new("Reset").icon(Icon::Refresh).accelerator("Ctrl+R"),
    ]
}

fn view_menu() -> Vec<MenuItem> {
    vec![
        MenuItem::new("VSync").accelerator("F10"),
        MenuItem::new("Fullscreen").icon(Icon::FullScreen).accelerator("F11"),
        MenuItem::submenu("Theme", theme_menu()).icon(Icon::Color),
        MenuItem::separator(),
        MenuItem::new("Disabled").enabled(false),
    ]
}

fn theme_menu() -> Vec<MenuItem> {
    vec![MenuItem::new("Dark theme").accelerator("F2"), MenuItem::new("Tooltips").accelerator("F1")]
}

fn help_menu() -> Vec<MenuItem> {
    vec![MenuItem::new("Ask a question").icon(Icon::Help)]
}

pub fn context_menu() -> Vec<MenuItem> {
    vec![
        MenuItem::new("Cut").icon(Icon::Cut).accelerator("Ctrl+X"),
        MenuItem::new("Copy").icon(Icon::Copy).accelerator("Ctrl+C"),
        MenuItem::new("Paste").icon(Icon::Paste).accelerator("Ctrl+V"),
        MenuItem::separator(),
        MenuItem::new("Pinned"),
    ]
}

// The cards' heights; the bottom card of each column takes the rest.
const BUTTONS_HEIGHT: f32 = 260.0;
const TOGGLES_HEIGHT: f32 = 344.0;
const SLIDERS_HEIGHT: f32 = 492.0;
const CHOICES_HEIGHT: f32 = 320.0;
const INFO_HEIGHT: f32 = 324.0;
const COLOR_HEIGHT: f32 = 492.0;

pub fn build(showcase: &Showcase, ui: &mut Ui) {
    ui.app_bar(AppBar::new(Icon::Library, "UiShowcase").subtitle(&subtitle(showcase)));
    let row = ui.menu_row();
    let menu = MenuBar::new(vec![("File", file_menu()), ("View", view_menu()), ("Help", help_menu())]);
    let menu_width = menu.width(ui);
    ui.node("Menu", row.left_strip(menu_width), menu);
    let theme_button = Rect::new(row.right() - 32.0, row.y, 32.0, row.height).centered(Size::square(32.0));
    ui.node("Theme", theme_button, Button::icon_only(if showcase.dark { Icon::Light } else { Icon::Moon }).subtle());
    ui.set_tooltip("Theme", "Switch the theme (F2)");
    ui.status_bar(status_items(ui.stats(), ui.environment().clone()));
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

fn subtitle(showcase: &Showcase) -> String {
    let theme_name = if showcase.dark { "Dark theme" } else { "Light theme" };
    format!("Fluent 2 · WinUI 3 · {theme_name}")
}

fn status_items(stats: user_interface::FrameStats, environment: user_interface::Environment) -> Vec<Status> {
    let fps = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let display = format!("{} {}", environment.display, environment.display_name);
    vec![
        Status::shortcut("F1", "Tooltips"),
        Status::shortcut("F2", "Theme"),
        Status::shortcut("Tab", "Next widget"),
        Status::shortcut("Ctrl+R", "Reset"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::indicator(Severity::Success, &fps),
        Status::text(&environment.gpu),
        Status::text(&display),
    ]
}

// Keeps what the controls show true to the showcase's state: the mirrors of
// values that shortcuts, menus and other controls change too.
pub fn sync(showcase: &Showcase, ui: &mut Ui) {
    ui.get_mut::<AppBar>("AppBar").set_subtitle(&subtitle(showcase));
    ui.get_mut::<Button>("Theme").set_icon(if showcase.dark { Icon::Light } else { Icon::Moon });
    let (stats, environment) = (ui.stats(), ui.environment().clone());
    ui.get_mut::<user_interface::StatusBar>("StatusBar").set_items(status_items(stats, environment));

    // Buttons.
    ui.get_mut::<ToggleButton>("Pin").set_on(showcase.pinned);
    ui.get_mut::<Text>("Count").set_text(&showcase.count.to_string());
    let presses = format!(
        "Pressed: Standard {}, Accent {}, Subtle {}, Save {}",
        showcase.presses[0], showcase.presses[1], showcase.presses[2], showcase.presses[3]
    );
    ui.get_mut::<Text>("Presses").set_text(&presses);

    // Toggles.
    ui.get_mut::<CheckBox>("OptionsEnabled").set_checked(showcase.options_enabled);
    for (index, option) in OPTIONS.iter().enumerate() {
        let name = format!("Option/{option}");
        ui.get_mut::<CheckBox>(&name).set_checked(showcase.options[index]);
        ui.set_enabled(&name, showcase.options_enabled);
    }
    ui.get_mut::<ToggleSwitch>("Tooltips").set_on(showcase.tooltips);
    ui.get_mut::<ToggleSwitch>("DarkTheme").set_on(showcase.dark);
    ui.get_mut::<ToggleSwitch>("Filling").set_on(showcase.filling);

    // Sliders and progress.
    ui.get_mut::<Slider>("Volume").set_value(showcase.volume);
    ui.get_mut::<Slider>("Opacity").set_value(showcase.opacity);
    ui.get_mut::<Slider>("Angle").set_value(showcase.angle);
    let time = ui.time();
    let fill = if showcase.filling { ((time - showcase.fill_started) % 4.0 / 4.0) as f32 } else { 0.78 };
    ui.get_mut::<Text>("VolumeBarValue").set_text(&format!("{:.0}%", showcase.volume));
    ui.get_mut::<ProgressBar>("VolumeBar").set_value(Some(showcase.volume / 100.0));
    ui.get_mut::<Text>("FillBarValue").set_text(&format!("{:.0}%", fill * 100.0));
    ui.get_mut::<ProgressBar>("FillBar").set_value(Some(fill));
    ui.get_mut::<Canvas<f32>>("Rings").data = showcase.volume / 100.0;

    // Choices.
    ui.get_mut::<RadioGroup>("Size").set_selected(showcase.size);
    ui.get_mut::<Canvas<usize>>("SizePreview").data = showcase.size;
    ui.get_mut::<Segmented>("Alignment").set_selected(showcase.alignment);
    ui.get_mut::<Canvas<usize>>("AlignedText").data = showcase.alignment;
    ui.get_mut::<Segmented>("Tool").set_selected(showcase.tool);
    ui.get_mut::<Text>("ToolCaption").set_text(&format!("{}: segments with icons, like a tool's modes", TOOLS[showcase.tool]));

    // Fields.
    ui.get_mut::<ComboBox>("Role").set_selected(showcase.role);
    ui.get_mut::<Canvas<usize>>("RoleSwatch").data = showcase.role;
    ui.get_mut::<Text>("RoleUse").set_text(ROLE_USES[showcase.role]);
    if ui.get::<TextBox>("Name").text() != showcase.name {
        ui.get_mut::<TextBox>("Name").set_text(&showcase.name);
    }
    if ui.get::<TextBox>("Search").text() != showcase.search {
        ui.get_mut::<TextBox>("Search").set_text(&showcase.search);
    }
    ui.get_mut::<NumberBox>("Quantity").set_value_if_settled(showcase.quantity);
    ui.get_mut::<NumberBox>("Scale").set_value_if_settled(showcase.scale);
    match &showcase.greeting {
        Some(greeting) => {
            ui.get_mut::<InfoBar>("Greeting").set_message(Severity::Success, "", greeting);
            ui.set_visible("Greeting", true);
        }
        None => ui.set_visible("Greeting", false),
    }

    // Menus and dialogs.
    let result = showcase.dialog_result.clone().unwrap_or_else(|| "Not asked yet".to_string());
    ui.get_mut::<Text>("Answer").set_text(&format!("Answer: {result}"));
    let chosen = showcase.menu_choice.clone().unwrap_or_else(|| "Nothing yet".to_string());
    ui.get_mut::<Text>("MenuChosen").set_text(&format!("Chose from a menu: {chosen}"));
    ui.get_mut::<Expander>("Expander").set_expanded(showcase.expanded);
    ui.set_visible("Question", showcase.dialog_open);

    // Tabs.
    ui.get_mut::<SelectorBar>("Tabs").set_selected(showcase.tab);
    for (index, tab) in TABS.iter().enumerate() {
        ui.set_visible(&format!("Tab/{tab}"), showcase.tab == index);
    }

    // Colors.
    ui.get_mut::<ColorPicker>("Picker").set_color(showcase.color);
    let field = ui.get_mut::<ColorField>("Hex");
    if field.color() != showcase.color {
        field.set_color(showcase.color);
    }
    for (index, color) in PALETTE.iter().enumerate() {
        ui.get_mut::<Swatch>(&format!("Palette/{index}")).set_selected(showcase.color == Color::hex(*color));
    }

    // The list.
    let matches = showcase.matches();
    let rows: Vec<ListRow> = matches
        .iter()
        .map(|item| ListRow::new(&showcase.items[*item]).icon(Icon::Document).trailing(&format!("#{}", item + 1)))
        .collect();
    let list = ui.get_mut::<ListView>("Items");
    let rows_changed = list.rows().len() != rows.len() || list.rows().iter().zip(&rows).any(|(a, b)| a.text != b.text);
    if rows_changed {
        list.set_rows(rows);
    }
    let selected = showcase.selected.and_then(|item| matches.iter().position(|index| *index == item));
    list.set_selected(selected);
    let subtitle = format!("{} of {} · Double-click to open", matches.len(), showcase.items.len());
    ui.get_mut::<Card>("ListCard").set_subtitle(&subtitle);

    // The interface card.
    interface_sync(showcase, ui);
}

fn buttons(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("ButtonsCard", rect, Card::new("Buttons"), |ui, card| {
        let mut area = card.content;
        let [standard, accent, subtle] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        ui.node(crate::showcase::BUTTONS[0], standard, Button::new("Standard"));
        ui.node(crate::showcase::BUTTONS[1], accent, Button::new("Accent").accent());
        ui.node(crate::showcase::BUTTONS[2], subtle, Button::new("Subtle").subtle());
        let [disabled, disabled_accent, link] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        ui.node("Disabled", disabled, Button::new("Disabled"));
        ui.set_enabled("Disabled", false);
        ui.node("DisabledAccent", disabled_accent, Button::new("Disabled").accent());
        ui.set_enabled("DisabledAccent", false);
        ui.node("Link", link, HyperlinkButton::new("Learn more"));
        let [save, pin, more] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        ui.node("Save", save, Button::new("Save").icon(Icon::Save));
        ui.set_tooltip("Save", "Saves nothing, but counts");
        ui.node("Pin", pin, ToggleButton::new(showcase.pinned, Button::new("Pin").icon(Icon::Pin)));
        ui.node("More", more, Button::new("More").flyout("MoreMenu", context_menu()));
        let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
        let minus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
        let count = row.cut_left(48.0, SPACE_S);
        let plus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
        ui.node("Remove", minus, Button::icon_only(Icon::Remove));
        ui.set_tooltip("Remove", "One less");
        ui.node("Count", count, Text::label(&showcase.count.to_string()).align(Align::Center));
        ui.node("Add", plus, Button::icon_only(Icon::Add));
        ui.set_tooltip("Add", "One more");
        ui.node("Presses", area.cut_top(CAPTION.line_height, 0.0), Text::caption(""));
    });
}

fn toggles(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("TogglesCard", rect, Card::new("Checkboxes and switches"), |ui, card| {
        let mut area = card.content;
        ui.node("OptionsEnabled", area.cut_top(CONTROL_HEIGHT, 0.0), CheckBox::new(showcase.options_enabled, "Enable the options under this"));
        for (index, option) in OPTIONS.iter().enumerate() {
            let row = area.cut_top(CONTROL_HEIGHT, 0.0);
            ui.node(&format!("Option/{option}"), Rect::new(row.x + 28.0, row.y, row.width - 28.0, row.height), CheckBox::new(showcase.options[index], option));
        }
        area.cut_top(SPACE_S, 0.0);
        ui.node("Tooltips", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(showcase.tooltips, "Tooltips (F1)"));
        ui.node("DarkTheme", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(showcase.dark, "Dark theme (F2)"));
        ui.node("Filling", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(showcase.filling, "Fill the progress bar"));
    });
}

fn choices(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("ChoicesCard", rect, Card::new("Radio buttons and segments"), |ui, card| {
        let mut area = card.content;
        let mut radios = area.cut_top(CONTROL_HEIGHT * 3.0, SPACE_M);
        let preview = radios.cut_right(96.0, 0.0);
        ui.node("Size", radios, RadioGroup::new(&SIZES, showcase.size));
        ui.node("SizePreview", preview, Canvas::new(showcase.size, draw_size_preview));
        ui.node("Alignment", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(&ALIGNMENTS, showcase.alignment));
        let line = area.cut_top(CONTROL_HEIGHT, SPACE_M);
        ui.node("AlignedText", line, Canvas::new(showcase.alignment, draw_aligned_text));
        let tools: Vec<(Icon, &str)> = [Icon::Brush, Icon::Fill, Icon::Erase, Icon::Eyedropper].into_iter().zip(TOOLS).collect();
        ui.node("Tool", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(&tools, showcase.tool));
        ui.node("ToolCaption", area.cut_top(CAPTION.line_height, 0.0), Text::caption(""));
    });
}

fn draw_size_preview(ui: &mut Ui, rect: Rect, size: &usize) {
    let side = ui.animate(id("SizePreview"), 0, 24.0 + 16.0 * *size as f32, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
    let accent = ui.theme().accent;
    ui.painter().rounded(rect.centered(Size::square(side)), RADIUS_CONTROL, accent);
}

fn draw_aligned_text(ui: &mut Ui, rect: Rect, alignment: &usize) {
    let align = [Align::Start, Align::Center, Align::End][*alignment];
    let (fill, primary) = (ui.theme().control_alt_fill_secondary, ui.theme().text_primary);
    ui.painter().rounded(rect, RADIUS_CONTROL, fill);
    ui.text_aligned(rect.inset(SPACE_M), "Aligned text", BODY, primary, align);
}

fn sliders(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("SlidersCard", rect, Card::new("Sliders and progress"), |ui, card| {
        let mut form = Form::new("Sliders", card.content);
        let height = Slider::height(true);
        ui.node("Volume", form.take(height), Slider::new(showcase.volume, 0.0, 100.0).step(1.0).header("Volume").show_value(0));
        ui.node("Opacity", form.take(height), Slider::new(showcase.opacity, 0.0, 1.0).step(0.001).large_step(0.1).header("Opacity").show_value(3));
        ui.node("Angle", form.take(height), Slider::new(showcase.angle, 0.0, 360.0).step(0.1).large_step(15.0).header("Angle").show_value(1).suffix("°"));
        ui.node("DisabledSlider", form.take(height), Slider::new(30.0f32, 0.0, 100.0).header("Disabled").show_value(0));
        ui.set_enabled("DisabledSlider", false);
        for (name, label, value_name) in [
            ("VolumeBar", "At the volume", Some("VolumeBarValue")),
            ("FillBar", "Fills every 4 seconds", Some("FillBarValue")),
            ("UnknownBar", "Of unknown length", None),
        ] {
            let row = form.take(BODY.line_height + SPACE_XS + 4.0);
            ui.node(&format!("{name}Label"), row.top_strip(BODY.line_height), Text::new(label, BODY).secondary());
            if let Some(value_name) = value_name {
                ui.node(value_name, row.top_strip(BODY.line_height), Text::new("", BODY).secondary().align(Align::End));
            }
            let value = if name == "UnknownBar" { None } else { Some(0.0) };
            ui.node(name, row.bottom_strip(4.0), ProgressBar::new(value));
        }
        let rings = form.take(40.0);
        ui.node("Rings", rings, Canvas::new(showcase.volume / 100.0, draw_rings));
    });
}

// The progress rings, at the volume, and spinning, with their caption.
fn draw_rings(ui: &mut Ui, rect: Rect, volume: &f32) {
    ui.progress_ring(Point::new(rect.x + 20.0, rect.center().y), 32.0, Some(*volume));
    ui.progress_ring(Point::new(rect.x + 68.0, rect.center().y), 32.0, None);
    ui.caption(Rect::new(rect.x + 104.0, rect.y, rect.width - 104.0, rect.height), "Progress rings, at the volume, and spinning");
}

fn fields(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("FieldsCard", rect, Card::new("Combo boxes and fields"), |ui, card| {
        let mut form = Form::new("Fields", card.content);
        let mut row = form.control(ui, "A color of the theme");
        let swatch = row.cut_right(CONTROL_HEIGHT, SPACE_S);
        ui.node("Role", row, ComboBox::new(&ROLES, showcase.role));
        ui.node("RoleSwatch", swatch, Canvas::new(showcase.role, draw_role_swatch));
        let caption_height = ui.paragraph_height(form.width(), ROLE_USES[0], CAPTION);
        ui.node("RoleUse", form.take(caption_height), Text::new("", CAPTION).secondary().wrapped());
        let mut row = form.control(ui, "Your name");
        let greet = row.cut_right(96.0, SPACE_S);
        ui.node("Name", row, TextBox::new(&showcase.name).placeholder("Type a name").max_length(40));
        ui.node("Greet", greet, Button::new("Greet").accent());
        let row = form.control(ui, "Search the list");
        ui.node("Search", row, TextBox::new(&showcase.search).placeholder("Search").button(Icon::Search));
        let [quantity, scale] = form.take(BODY.line_height + 8.0 + CONTROL_HEIGHT).columns::<2>(SPACE_S);
        ui.node("QuantityLabel", quantity.top_strip(BODY.line_height), Text::label("Quantity"));
        ui.node("Quantity", quantity.bottom_strip(CONTROL_HEIGHT), NumberBox::new(showcase.quantity).range(0.0, 99.0));
        ui.node("ScaleLabel", scale.top_strip(BODY.line_height), Text::label("Scale"));
        ui.node("Scale", scale.bottom_strip(CONTROL_HEIGHT), NumberBox::new(showcase.scale).range(0.25, 4.0).step(0.25).decimals(2));
        ui.node("Greeting", form.take(48.0), InfoBar::new(Severity::Success, "", "").closable());
        ui.set_visible("Greeting", showcase.greeting.is_some());
    });
}

fn draw_role_swatch(ui: &mut Ui, rect: Rect, role: &usize) {
    let theme = ui.theme();
    let color = [theme.accent, theme.success, theme.caution, theme.critical, theme.attention][*role];
    ui.painter().rounded(rect, RADIUS_CONTROL, color);
}

fn menus(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("MenusCard", rect, Card::new("Menus and dialogs"), |ui, card| {
        let mut form = Form::new("Menus", card.content);
        let [ask, choice] = form.columns::<2>(CONTROL_HEIGHT);
        ui.node("Ask", ask, Button::new("Ask a question").icon(Icon::Help));
        ui.node("Answer", choice, Text::caption(""));
        let area = form.take(64.0);
        ui.node("ContextArea", area, Canvas::new((), draw_context_area).sense(user_interface::Sense::new(true, true, false, false, user_interface::Keys::None)).kind("Area"));
        ui.set_context_menu("ContextArea", "ContextMenu", context_menu());
        ui.node("MenuChosen", form.take(CAPTION.line_height), Text::caption(""));
        let header = form.take(48.0);
        let rest = form.rest();
        let expander_rect = Rect::new(header.x, header.y, header.width, rest.bottom() - header.y);
        ui.group("Expander", expander_rect, Expander::new(showcase.expanded, "An expander").caption("Shows and hides what's under it"), |ui| {
            let inner = Expander::content_area(expander_rect, 48.0);
            ui.node("ExpanderText", inner, Text::new("Expander content sits on the secondary card fill, under its header, as WinUI's does.", BODY).secondary().wrapped());
        });
    });
}

fn draw_context_area(ui: &mut Ui, rect: Rect, _: &()) {
    let theme = ui.theme();
    let (fill, stroke, text) = (theme.control_alt_fill_secondary, theme.control_strong_stroke, theme.text_secondary);
    ui.painter().shape(rect, RADIUS_CONTROL, fill, Some(Border::new(stroke.fade(0.5), 1.0)));
    ui.text_aligned(rect, "Right-click here for a context menu", BODY, text, Align::Center);
}

fn dialog(showcase: &Showcase, ui: &mut Ui) {
    let dialog = Dialog::new("Save your work?", "This is a content dialog: it covers the page with smoke until you choose.")
        .primary("Save")
        .secondary("Don't save")
        .close("Cancel");
    ui.node("Question", Rect::default(), dialog);
    ui.set_visible("Question", showcase.dialog_open);
}

fn tabs(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("TabsCard", rect, Card::new(""), |ui, card| {
        let mut area = card.content;
        ui.node("Tabs", area.cut_top(48.0, SPACE_S), SelectorBar::new(&TABS, showcase.tab));
        for (index, tab) in TABS.iter().enumerate() {
            let paint = [draw_type_ramp, draw_icons, draw_theme_colors, draw_shapes][index];
            ui.node(&format!("Tab/{tab}"), area, Canvas::new((), paint));
            ui.set_visible(&format!("Tab/{tab}"), showcase.tab == index);
        }
    });
}

fn draw_type_ramp(ui: &mut Ui, area: Rect, _: &()) {
    let mut area = area;
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

fn draw_icons(ui: &mut Ui, area: Rect, _: &()) {
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

fn draw_theme_colors(ui: &mut Ui, area: Rect, _: &()) {
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

fn draw_shapes(ui: &mut Ui, area: Rect, _: &()) {
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

fn info_bars(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("InfoBarsCard", rect, Card::new("Info bars"), |ui, card| {
        let mut area = card.content;
        let bars = [
            ("Informational", Severity::Informational, "Informational", "Something to know.", false),
            ("Success", Severity::Success, "Success", "It went well.", false),
            ("Caution", Severity::Warning, "Caution", "Close me.", true),
            ("Error", Severity::Error, "Error", "It went wrong.", false),
        ];
        for (name, severity, title, message, closable) in bars {
            if name == "Caution" && !showcase.caution_shown {
                let row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
                ui.node("BringBack", row.left_strip(160.0), Button::new("Bring it back"));
                continue;
            }
            let mut bar = InfoBar::new(severity, title, message);
            if closable {
                bar = bar.closable();
            }
            let height = bar.height(ui, area.width);
            ui.node(name, area.cut_top(height, SPACE_S), bar);
        }
    });
}

fn colors(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("ColorCard", rect, Card::new("Color"), |ui, card| {
        let mut area = card.content;
        let picker_width = 256.0f32.min(area.width);
        let picker = area.cut_top(ColorPicker::height(picker_width), SPACE_M).left_strip(picker_width);
        ui.node("Picker", picker, ColorPicker::new(showcase.color));
        let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_M);
        let field = row.cut_left(160.0, SPACE_M);
        ui.node("Hex", field, ColorField::new(showcase.color));
        for (index, color) in PALETTE.iter().enumerate() {
            let swatch = row.cut_left(CONTROL_HEIGHT - 4.0, SPACE_XS).centered(Size::square(24.0));
            let color = Color::hex(*color);
            ui.node(&format!("Palette/{index}"), swatch, Swatch::new(color, showcase.color == color));
        }
    });
}

// The palette's color at the swatch's index.
pub fn palette(index: usize) -> Color {
    Color::hex(PALETTE[index])
}

fn list(showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    let matches = showcase.matches();
    let subtitle = format!("{} of {} · Double-click to open", matches.len(), showcase.items.len());
    ui.card("ListCard", rect, Card::new("List").subtitle(&subtitle), |ui, card| {
        let rows: Vec<ListRow> = matches
            .iter()
            .map(|item| ListRow::new(&showcase.items[*item]).icon(Icon::Document).trailing(&format!("#{}", item + 1)))
            .collect();
        let mut list = ListView::new(rows, 40.0).selectable();
        list.set_selected(showcase.selected.and_then(|item| matches.iter().position(|index| *index == item)));
        ui.node("Items", card.content, list);
    });
}

fn interface(_showcase: &Showcase, ui: &mut Ui, rect: Rect) {
    ui.card("InterfaceCard", rect, Card::new("Interface"), |ui, card| {
        let mut area = card.content;
        for name in ["WindowFocus", "Pointer", "Hovered", "Pressed", "Focus", "Popup", "Tooltip", "Dialog"] {
            let label = match name {
                "WindowFocus" => "Window focus",
                "Pointer" => "Pointer",
                "Hovered" => "Under the pointer",
                "Pressed" => "Pressed",
                "Focus" => "Keyboard focus",
                "Popup" => "Open popup",
                "Tooltip" => "Tooltip",
                _ => "Dialog",
            };
            ui.node(&format!("Interface/{name}"), area.cut_top(24.0, 0.0), Field::new(label, 160.0, ""));
        }
        ui.node("Interface/Keys", area.cut_top(24.0, SPACE_S), Field::new("Keys held", 160.0, ""));
        ui.node("GamepadHeading", area.cut_top(BODY_STRONG.line_height, SPACE_S), Text::heading("Gamepad"));
        let sticks = area.cut_top(96.0, SPACE_S);
        ui.node("GamepadView", sticks, Canvas::new(Gamepad::default(), draw_gamepad));
        ui.node("GamepadButtons", area.cut_top(CAPTION.line_height, SPACE_L), Text::caption(""));
        let badges = area.cut_top(24.0, SPACE_L);
        ui.node("Badges", badges, Canvas::new((), draw_badges));
        ui.node("Details", area.cut_top(CONTROL_HEIGHT, SPACE_L).left_strip(200.0), Button::new("Details").icon(Icon::Info));
        ui.node("LogHeading", area.cut_top(BODY_STRONG.line_height, SPACE_S), Text::heading("Log"));
        ui.node("Log", area, Canvas::new(Vec::new(), draw_log));
    });
}

fn interface_sync(showcase: &Showcase, ui: &mut Ui) {
    let pointer = ui.pointer().map_or("Not in the window".to_string(), |point| format!("{:.0}, {:.0}", point.x, point.y));
    let values = [
        ("WindowFocus", if ui.window_focused() { "Yes".to_string() } else { "No".to_string() }),
        ("Pointer", pointer),
        ("Hovered", ui.hovered_name().unwrap_or("None").to_string()),
        ("Pressed", ui.pressed_name().unwrap_or("None").to_string()),
        ("Focus", ui.focused_name().unwrap_or("None").to_string()),
        ("Popup", ui.popup_name().unwrap_or("None").to_string()),
        ("Tooltip", ui.tooltip_name().unwrap_or("None").to_string()),
        ("Dialog", if ui.modal() { "Open".to_string() } else { "None".to_string() }),
    ];
    for (name, value) in values {
        ui.get_mut::<Field>(&format!("Interface/{name}")).set_value(&value);
    }
    let held: Vec<String> = ui.input().keys_held().iter().map(|key| key.name()).collect();
    let held = if held.is_empty() { "None".to_string() } else { held.join(", ") };
    ui.get_mut::<Field>("Interface/Keys").set_value(&held);
    let gamepad = *ui.gamepad();
    ui.get_mut::<Canvas<Gamepad>>("GamepadView").data = gamepad;
    let buttons = gamepad.buttons.names();
    let pressed = if !gamepad.connected { "No gamepad".to_string() } else if buttons.is_empty() { "No buttons held".to_string() } else { buttons.join(", ") };
    ui.get_mut::<Text>("GamepadButtons").set_text(&pressed);
    ui.set_tooltip_tips(
        "Details",
        vec![
            Tip::heading("Details"),
            Tip::pair("Theme", if showcase.dark { "Dark" } else { "Light" }),
            Tip::Separator,
            Tip::text("Tooltips can have lines, columns and separators."),
        ],
    );
    let log: Vec<String> = showcase.log.iter().cloned().collect();
    ui.get_mut::<Canvas<Vec<String>>>("Log").data = log;
}

fn draw_gamepad(ui: &mut Ui, area: Rect, gamepad: &Gamepad) {
    let theme = ui.theme().clone();
    let mut sticks = area;
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
}

fn draw_badges(ui: &mut Ui, area: Rect, _: &()) {
    let mut badges = area;
    let primary = ui.theme().text_secondary;
    let width = ui.keys(Point::new(badges.x, badges.center().y - 10.0), "Ctrl+Shift+S");
    badges.cut_left(width + SPACE_M, 0.0);
    let size = ui.badge(Point::new(badges.x + 12.0, badges.center().y), "3");
    badges.cut_left(size.width + SPACE_S + 8.0, 0.0);
    ui.badge(Point::new(badges.x + 6.0, badges.center().y), "");
    ui.text(Rect::new(badges.x + 20.0, badges.y, badges.width - 20.0, badges.height), "Keycaps, badges and a dot", CAPTION, primary);
}

fn draw_log(ui: &mut Ui, area: Rect, log: &Vec<String>) {
    let mut area = area;
    let secondary = ui.theme().text_secondary;
    let lines = ((area.height / 20.0).floor() as usize).min(log.len());
    for line in log.iter().take(lines) {
        let row = area.cut_top(20.0, 0.0);
        ui.text(row, line, MONO, secondary);
    }
}
