// The showcase's page: an app bar with its icon and title, a menu row under
// it with a menu bar in line with the icon and the cards, five columns of cards
// that show every widget, and a status bar. The cards at the bottom of their
// columns take the 71 pixels that borderless fullscreen adds.
//
// `view` describes the page from the showcase's state, and changes nothing:
// each widget carries the message that the input on it sends.

use user_interface::tokens::{
    BODY, BODY_LARGE, BODY_STRONG, CAPTION, CARD_GAP, CONTROL_HEIGHT, DISPLAY, ELEVATION_DIALOG, ELEVATION_FLYOUT, ELEVATION_TOOLTIP, ICON,
    ICON_LARGE, MONO, MONO_CAPTION, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S, SPACE_XS, SUBTITLE, TITLE, TITLE_LARGE,
};
use user_interface::{
    Align, AppBar, Badge, Border, Brush, Button, Canvas, Card, CheckBox, Color, ColorField, ColorPicker, ComboBox, Corners, Dialog, Expander,
    ExpanderContent, Field, Form, HyperlinkButton, Icon, IconView, InfoBar, Insets, Keycaps, Keys, ListView, MenuBar, MenuItem, NumberBox,
    Palette, Point, ProgressBar, ProgressRing, RadioGroup, Rect, Run, Segmented, SelectorBar, Sense, Severity, Size, Slider, Spans, Status,
    Text, TextBox, TextColor, TextStyle, ThemeKind, ToggleButton, ToggleSwitch, TooltipLine, Tree, Ui, View,
};

use crate::showcase::{ALIGNMENTS, BUTTONS, Message, OPTIONS, SIZES, Showcase, TABS, TOOLS};

// The theme's colors that the combo box offers, and what each is for.
pub const ROLES: [&str; 5] = ["Accent", "Success", "Caution", "Critical", "Attention"];
const ROLE_USES: [&str; 5] = [
    "What's on, chosen, or the one command that matters most",
    "What went well",
    "What needs care",
    "What went wrong",
    "What's worth knowing",
];
pub const PALETTE: [u32; 8] = [0xD13438, 0xCA5010, 0xC19C00, 0x107C10, 0x038387, 0x0078D4, 0x8764B8, 0xE3008C];

// The cards' heights; the bottom card of each column takes the rest.
const BUTTONS_HEIGHT: f32 = 260.0;
const TOGGLES_HEIGHT: f32 = 344.0;
const SLIDERS_HEIGHT: f32 = 492.0;
const CHOICES_HEIGHT: f32 = 320.0;
const INFO_HEIGHT: f32 = 324.0;
const COLOR_HEIGHT: f32 = 492.0;

// A menu item that sends its label as a command.
fn command(label: &str) -> MenuItem<Message> {
    MenuItem::new(label).on_choose(Message::Command(label.to_string()))
}

fn menus() -> Vec<(&'static str, Vec<MenuItem<Message>>)> {
    let file = vec![
        command("New").icon(Icon::Document).accelerator("Ctrl+N"),
        command("Open").icon(Icon::OpenFile).accelerator("Ctrl+O"),
        command("Save").icon(Icon::Save).accelerator("Ctrl+S"),
        MenuItem::separator(),
        command("Reset").icon(Icon::Refresh).accelerator("Ctrl+R"),
    ];
    let theme = vec![command("Dark theme").accelerator("F9"), command("Tooltips").accelerator("F1")];
    let view = vec![
        command("VSync").accelerator("F10"),
        command("Fullscreen").icon(Icon::FullScreen).accelerator("F11"),
        MenuItem::submenu("Theme", theme).icon(Icon::Color),
        MenuItem::separator(),
        MenuItem::new("Disabled").enabled(false),
    ];
    let help = vec![command("Ask a question").icon(Icon::Help)];
    vec![("File", file), ("View", view), ("Help", help)]
}

fn context_menu() -> Vec<MenuItem<Message>> {
    vec![
        command("Cut").icon(Icon::Cut).accelerator("Ctrl+X"),
        command("Copy").icon(Icon::Copy).accelerator("Ctrl+C"),
        command("Paste").icon(Icon::Paste).accelerator("Ctrl+V"),
        MenuItem::separator(),
        command("Pinned"),
    ]
}

pub fn view(showcase: &Showcase, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    tree.set_tooltips(showcase.tooltips);
    let dark = ui.theme().kind == ThemeKind::Dark;
    let subtitle = format!("Fluent 2 · WinUI 3 · {} theme", ui.theme().kind.name());
    let stats = ui.stats();
    let fps = format!("{:.0} frames a second · {:.2} ms a frame", stats.frames_per_second, stats.frame_time);
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    let status = vec![
        Status::shortcut("F1", "Tooltips"),
        Status::shortcut("Tab", "Next widget"),
        Status::shortcut("Ctrl+R", "Reset"),
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::Spacer,
        Status::indicator(Severity::Success, fps),
        Status::text(environment.gpu.clone()),
        Status::text(display),
    ];
    let shell = tree.shell(ui, AppBar::new(Icon::Library, "UiShowcase").subtitle(subtitle), true, Some(status));
    let row = shell.menu_row.unwrap();
    let menu = MenuBar::new(menus());
    let menu_width = menu.width(ui);
    tree.add("Menu", row.left_strip(menu_width), menu);
    let theme_button = Rect::new(row.right() - 32.0, row.y, 32.0, row.height).centered(Size::square(32.0));
    tree.add("Theme", theme_button, Button::icon_only(if dark { Icon::Light } else { Icon::Moon }).subtle().on_press(Message::ToggleTheme))
        .set_tooltip("Switch the theme (F9)");
    let page = shell.page;
    let [first, second, third, fourth, fifth] = page.columns::<5>(CARD_GAP);

    let mut column = first;
    buttons(showcase, ui, &mut tree, column.cut_top(BUTTONS_HEIGHT, CARD_GAP));
    toggles(showcase, ui, &mut tree, column.cut_top(TOGGLES_HEIGHT, CARD_GAP));
    fields(showcase, ui, &mut tree, column);

    let mut column = second;
    sliders(showcase, ui, &mut tree, column.cut_top(SLIDERS_HEIGHT, CARD_GAP));
    choices(showcase, ui, &mut tree, column.cut_top(CHOICES_HEIGHT, CARD_GAP));
    menus_card(showcase, ui, &mut tree, column);

    tabs(showcase, ui, &mut tree, third);

    let mut column = fourth;
    info_bars(showcase, ui, &mut tree, column.cut_top(INFO_HEIGHT, CARD_GAP));
    colors(showcase, ui, &mut tree, column.cut_top(COLOR_HEIGHT, CARD_GAP));
    list(showcase, ui, &mut tree, column);

    interface(showcase, ui, &mut tree, fifth);
    dialog(showcase, ui, &mut tree);
    tree
}

fn buttons(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    tree.card(ui, None, rect, Card::new("Buttons"), |_, card, area| {
        let mut area = area.content;
        let [standard, accent, subtle] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        card.add(BUTTONS[0], standard, Button::new("Standard").on_press(Message::Press(0)));
        card.add(BUTTONS[1], accent, Button::new("Accent").accent().on_press(Message::Press(1)));
        card.add(BUTTONS[2], subtle, Button::new("Subtle").subtle().on_press(Message::Press(2)));
        let [disabled, disabled_accent, link] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        card.add("Disabled", disabled, Button::new("Disabled")).set_enabled(false);
        card.add("DisabledAccent", disabled_accent, Button::new("Disabled").accent()).set_enabled(false);
        card.add("Link", link, HyperlinkButton::new("Learn more").on_press(Message::Link));
        let [save, pin, more] = area.cut_top(CONTROL_HEIGHT, SPACE_S).columns::<3>(SPACE_S);
        card.add("Save", save, Button::new("Save").icon(Icon::Save).on_press(Message::Press(3))).set_tooltip("Saves nothing, but counts");
        card.add("Pin", pin, ToggleButton::new(showcase.pinned, Button::new("Pin").icon(Icon::Pin)).on_toggle(Message::Pin));
        card.add("More", more, Button::new("More").menu(context_menu()));
        let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
        let minus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
        let count = row.cut_left(48.0, SPACE_S);
        let plus = row.cut_left(CONTROL_HEIGHT, SPACE_S);
        card.add("Remove", minus, Button::icon_only(Icon::Remove).on_press(Message::Remove)).set_tooltip("One less");
        card.draw(count, Text::label(showcase.count.to_string()).center());
        card.add("Add", plus, Button::icon_only(Icon::Add).on_press(Message::Add)).set_tooltip("One more");
        let presses = format!("Pressed: Standard {}, Accent {}, Subtle {}, Save {}", showcase.presses[0], showcase.presses[1], showcase.presses[2], showcase.presses[3]);
        card.draw(area.cut_top(CAPTION.line_height, 0.0), Text::caption(presses));
    });
}

fn toggles(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let dark = ui.theme().kind == ThemeKind::Dark;
    tree.card(ui, None, rect, Card::new("Checkboxes and switches"), |_, card, area| {
        let mut area = area.content;
        card.add("OptionsEnabled", area.cut_top(CONTROL_HEIGHT, 0.0), CheckBox::new(showcase.options_enabled, "Enable the options under this").on_change(Message::OptionsEnabled));
        for (index, option) in OPTIONS.iter().enumerate() {
            let row = area.cut_top(CONTROL_HEIGHT, 0.0);
            card.add(&format!("Option/{option}"), Rect::new(row.x + 28.0, row.y, row.width - 28.0, row.height), CheckBox::new(showcase.options[index], *option).on_change(move |on| Message::Option(index, on)))
                .set_enabled(showcase.options_enabled);
        }
        area.cut_top(SPACE_S, 0.0);
        card.add("Tooltips", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(showcase.tooltips, "Tooltips (F1)").on_change(Message::Tooltips));
        card.add("DarkTheme", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(dark, "Dark theme (F9)").on_change(|_| Message::ToggleTheme));
        card.add("Filling", area.cut_top(CONTROL_HEIGHT, 0.0), ToggleSwitch::new(showcase.filling, "Fill the progress bar").on_change(Message::Filling));
    });
}

fn choices(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let accent = ui.theme().accent;
    let fill = ui.theme().control_alt_fill_secondary;
    tree.card(ui, None, rect, Card::new("Radio buttons and segments"), |_, card, area| {
        let mut area = area.content;
        let mut radios = area.cut_top(RadioGroup::<Message>::height(SIZES.len()), SPACE_M);
        let preview = radios.cut_right(96.0, 0.0);
        card.add("Size", radios, RadioGroup::new(showcase.size, &SIZES).on_select(Message::Size));
        let side = 24.0 + 16.0 * showcase.size as f32;
        card.draw(preview, Canvas::new(move |painter, rect| painter.rounded(rect.centered(Size::square(side)), RADIUS_CONTROL, accent)));
        card.add("Alignment", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(showcase.alignment, &ALIGNMENTS).on_select(Message::Alignment));
        let line = area.cut_top(CONTROL_HEIGHT, SPACE_M);
        let align = [Align::Start, Align::Center, Align::End][showcase.alignment];
        card.draw(line, Canvas::new(move |painter, rect| painter.rounded(rect, RADIUS_CONTROL, fill)));
        card.draw(line.inset(SPACE_M), Text::label("Aligned text").align(align));
        let tools: Vec<(Icon, &str)> = [Icon::Brush, Icon::Fill, Icon::Erase, Icon::Eyedropper].into_iter().zip(TOOLS).collect();
        card.add("Tool", area.cut_top(CONTROL_HEIGHT, SPACE_S), Segmented::new(showcase.tool, &tools).on_select(Message::Tool));
        card.draw(area.cut_top(CAPTION.line_height, 0.0), Text::caption(format!("{}: segments with icons, like a tool's modes", TOOLS[showcase.tool])));
    });
}

fn sliders(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let time = ui.time();
    tree.card(ui, None, rect, Card::new("Sliders and progress"), |_, card, area| {
        let mut form = Form::new(area.content);
        let height = Slider::<Message>::height(true);
        card.add("Volume", form.take(height), Slider::new(showcase.volume, 0.0, 100.0).step(1.0).header("Volume").show_value(0).on_change(Message::Volume));
        card.add("Opacity", form.take(height), Slider::new(showcase.opacity, 0.0, 1.0).step(0.001).large_step(0.1).header("Opacity").show_value(3).on_change(Message::Opacity));
        card.add("Angle", form.take(height), Slider::new(showcase.angle, 0.0, 360.0).step(0.1).large_step(15.0).header("Angle").show_value(1).suffix("°").on_change(Message::Angle));
        card.add("DisabledSlider", form.take(height), Slider::<Message>::new(30.0, 0.0, 100.0).header("Disabled").show_value(0)).set_enabled(false);
        let fill = if showcase.filling { ((time - showcase.fill_started) % 4.0 / 4.0) as f32 } else { 0.78 };
        for (label, value) in [("At the volume", Some(showcase.volume / 100.0)), ("Fills every 4 seconds", Some(fill)), ("Of unknown length", None)] {
            let row = form.take(BODY.line_height + SPACE_XS + 4.0);
            card.draw(row.top_strip(BODY.line_height), Text::label(label).secondary());
            if let Some(value) = value {
                card.draw(row.top_strip(BODY.line_height), Text::label(format!("{:.0}%", value * 100.0)).secondary().end());
            }
            card.draw(row.bottom_strip(4.0), ProgressBar::new(value));
        }
        let rings = form.take(40.0);
        card.draw(Rect::new(rings.x, rings.y, 40.0, rings.height), ProgressRing::new(Some(showcase.volume / 100.0)));
        card.draw(Rect::new(rings.x + 48.0, rings.y, 40.0, rings.height), ProgressRing::new(None));
        card.draw(Rect::new(rings.x + 104.0, rings.y, rings.width - 104.0, rings.height), Text::caption("Progress rings, at the volume, and spinning"));
    });
}

fn fields(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let theme = ui.theme();
    let role_color = [theme.accent, theme.success, theme.caution, theme.critical, theme.attention][showcase.role];
    tree.card(ui, None, rect, Card::new("Combo boxes and fields"), |ui, card, area| {
        let mut form = Form::new(area.content);
        let mut row = form.control(card, "A color of the theme");
        let swatch = row.cut_right(CONTROL_HEIGHT, SPACE_S);
        card.add("Role", row, ComboBox::new(showcase.role, &ROLES).on_select(Message::Role));
        card.draw(swatch, Canvas::new(move |painter, rect| painter.rounded(rect, RADIUS_CONTROL, role_color)));
        form.caption(ui, card, ROLE_USES[showcase.role]);
        let mut row = form.control(card, "Your name");
        let greet = row.cut_right(96.0, SPACE_S);
        card.add("Name", row, TextBox::new(&showcase.name).placeholder("Type a name").max_length(40).on_change(Message::Name).on_submit(Message::Greet));
        card.add("Greet", greet, Button::new("Greet").accent().on_press(Message::Greet));
        let row = form.control(card, "Search the list");
        card.add("Search", row, TextBox::new(&showcase.search).placeholder("Search").button(Icon::Search).on_change(Message::Search));
        let [quantity, scale] = form.take(BODY.line_height + 8.0 + CONTROL_HEIGHT).columns::<2>(SPACE_S);
        card.draw(quantity.top_strip(BODY.line_height), Text::label("Quantity"));
        card.add("Quantity", quantity.bottom_strip(CONTROL_HEIGHT), NumberBox::new(showcase.quantity as f64).range(0.0, 99.0).on_change(Message::Quantity));
        card.draw(scale.top_strip(BODY.line_height), Text::label("Scale"));
        card.add("Scale", scale.bottom_strip(CONTROL_HEIGHT), NumberBox::new(showcase.scale).range(0.25, 4.0).step(0.25).decimals(2).on_change(Message::Scale));
        if let Some(greeting) = &showcase.greeting {
            card.add("Greeting", form.take(48.0), InfoBar::new(Severity::Success, "", greeting).on_close(Message::CloseGreeting));
        }
    });
}

fn menus_card(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let theme = ui.theme();
    let (fill, stroke) = (theme.control_alt_fill_secondary, theme.control_strong_stroke);
    tree.card(ui, None, rect, Card::new("Menus and dialogs"), |_, card, area| {
        let mut form = Form::new(area.content);
        let [ask, choice] = form.columns::<2>(CONTROL_HEIGHT);
        card.add("Ask", ask, Button::new("Ask a question").icon(Icon::Help).on_press(Message::Ask));
        let result = showcase.dialog_result.clone().unwrap_or_else(|| "Not asked yet".to_string());
        card.draw(choice, Text::caption(format!("Answer: {result}")));
        let area = form.take(64.0);
        card.draw(area, Canvas::new(move |painter, rect| painter.shape(rect, RADIUS_CONTROL, fill, Some(Border::new(stroke.fade(0.5), 1.0)))));
        card.draw(area, Text::label("Right-click here for a context menu").secondary().center());
        card.add("ContextArea", area, View::new().sense(Sense::new(true, true, false, false, Keys::None)).kind("Area")).set_context_menu(context_menu());
        let chosen = showcase.menu_choice.clone().unwrap_or_else(|| "Nothing yet".to_string());
        card.draw(form.take(CAPTION.line_height), Text::caption(format!("Chose from a menu: {chosen}")));
        let header = form.take(48.0);
        card.add("Expander", header, Expander::new(showcase.expanded, "An expander").caption("Shows and hides what's under it").on_toggle(Message::Expand));
        if showcase.expanded {
            let rest = form.rest();
            let content = Rect::new(header.x, header.bottom(), header.width, rest.bottom() - header.bottom());
            card.container(None, content, ExpanderContent, |expander| {
                expander.draw(ExpanderContent::inner(content), Text::paragraph("Expander content sits on the secondary card fill, under its header, as WinUI's does.").secondary());
            });
        }
    });
}

fn dialog(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>) {
    if !showcase.dialog_open {
        return;
    }
    let dialog = Dialog::new("Save your work?", "This is a content dialog: it covers the page with smoke until you choose.")
        .primary("Save")
        .secondary("Don't save")
        .close("Cancel")
        .on_result(Message::Answer);
    let area = dialog.area(ui);
    tree.add("Question", area.rect, dialog);
}

fn tabs(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    tree.card(ui, None, rect, Card::new(""), |ui, card, area| {
        let mut area = area.content;
        card.add("Tabs", area.cut_top(48.0, SPACE_S), SelectorBar::new(showcase.tab, &TABS).on_select(Message::Tab));
        match showcase.tab {
            0 => type_ramp(card, area),
            1 => icons(card, area),
            2 => theme_colors(ui, card, area),
            _ => shapes(ui, card, area),
        }
    });
}

fn type_ramp(tree: &mut Tree<Message>, mut area: Rect) {
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
    for (label, style, sample) in styles {
        let row = area.cut_top(style.line_height, SPACE_S);
        tree.draw(row.left_strip(148.0), Text::caption(label));
        tree.draw(Rect::new(row.x + 148.0, row.y, row.width - 148.0, row.height), Text::label(sample).style(style));
    }
    tree.draw(
        Rect::new(area.x, area.y + SPACE_S, area.width, BODY.line_height),
        Spans::new(vec![
            Run::new("Spans: ", BODY, TextColor::Secondary),
            Run::new("several styles ", BODY_STRONG, TextColor::Primary),
            Run::new("and colors ", BODY, TextColor::Accent),
            Run::new("on a line", CAPTION, TextColor::Secondary),
        ]),
    );
}

fn icons(tree: &mut Tree<Message>, area: Rect) {
    let columns = 4;
    let cell = Size::new(area.width / columns as f32, 40.0);
    for (index, icon) in Icon::ALL.iter().enumerate() {
        let x = area.x + cell.width * (index % columns) as f32;
        let y = area.y + cell.height * (index / columns) as f32;
        if y + cell.height > area.bottom() {
            break;
        }
        tree.draw(Rect::new(x, y, 32.0, cell.height), IconView::new(*icon, ICON_LARGE));
        tree.draw(Rect::new(x + 36.0, y, cell.width - 40.0, cell.height), Text::caption(icon.name()));
    }
}

fn theme_colors(ui: &mut Ui, tree: &mut Tree<Message>, area: Rect) {
    let roles = ui.theme().roles();
    let row_height = 26.0;
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
        let color = *color;
        tree.draw(swatch, Canvas::new(move |painter, rect| painter.shape(rect, RADIUS_CONTROL, color, Some(Border::new(stroke, 1.0)))));
        tree.draw(Rect::new(column.x + 28.0, y, column.width - 120.0, row_height), Text::caption(*name));
        tree.draw(Rect::new(column.x, y, column.width, row_height), Text::caption(color.to_hex()).end());
    }
}

fn shapes(ui: &mut Ui, tree: &mut Tree<Message>, area: Rect) {
    let theme = ui.theme().clone();
    tree.draw(
        area,
        Canvas::new(move |painter, area| {
            let [elevations, rest] = area.rows::<2>(SPACE_L);
            let tiles = elevations.top_strip(120.0).inset(SPACE_L).columns::<3>(SPACE_L * 2.0);
            let names = ["Tooltip · 16", "Flyout · 32", "Dialog · 128"];
            for ((tile, elevation), name) in tiles.iter().zip([ELEVATION_TOOLTIP, ELEVATION_FLYOUT, ELEVATION_DIALOG]).zip(names) {
                painter.shadow(*tile, RADIUS_OVERLAY, elevation);
                painter.shape(*tile, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
                painter.text_in(*tile, name, CAPTION, theme.text_secondary, Align::Center);
            }
            let mut y = elevations.y + 160.0;
            let data = theme.data.all();
            for (index, color) in data.iter().enumerate() {
                let x = elevations.x + 24.0 + index as f32 * 52.0;
                painter.circle(Point::new(x, y), 16.0, *color);
            }
            y += 52.0;
            painter.line(Point::new(elevations.x + 8.0, y), Point::new(elevations.right() - 8.0, y + 24.0), 2.0, theme.accent);
            painter.polyline(
                &[Point::new(elevations.x + 8.0, y + 56.0), Point::new(elevations.x + 120.0, y + 36.0), Point::new(elevations.x + 240.0, y + 72.0), Point::new(elevations.x + 360.0, y + 40.0)],
                2.0,
                theme.data.orange,
            );
            let center = Point::new(rest.x + 48.0, rest.y + 48.0);
            painter.ring(center, 40.0, 4.0, theme.control_strong_stroke);
            painter.arc(center, 38.0, 4.0, 0.0, 4.0, theme.accent);
            painter.polygon(&[Point::new(rest.x + 140.0, rest.y + 88.0), Point::new(rest.x + 180.0, rest.y + 8.0), Point::new(rest.x + 220.0, rest.y + 88.0)], theme.data.green);
            let hexagon: Vec<Point> = (0..6)
                .map(|index| {
                    let angle = std::f32::consts::TAU * index as f32 / 6.0;
                    Point::new(rest.x + 300.0 + 40.0 * angle.cos(), rest.y + 48.0 + 40.0 * angle.sin())
                })
                .collect();
            painter.polygon(&hexagon, theme.data.purple);
            let gradient = Rect::new(rest.x, rest.y + 120.0, rest.width, 32.0);
            painter.shape(gradient, RADIUS_CONTROL, Brush::horizontal(gradient, theme.accent, theme.data.pink), None);
            let vertical = Rect::new(rest.x, rest.y + 168.0, 120.0, 80.0);
            painter.shape(vertical, Corners::top(16.0), Brush::vertical(vertical, theme.data.teal, theme.data.blue), None);
            let bordered = Rect::new(rest.x + 140.0, rest.y + 168.0, 120.0, 80.0);
            painter.shape(bordered, RADIUS_CONTROL, theme.control_fill, Some(user_interface::Elevation::Control.border(&theme, bordered)));
            painter.text_in(bordered, "Elevation border", CAPTION, theme.text_secondary, Align::Center);
        }),
    );
}

fn info_bars(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    tree.card(ui, None, rect, Card::new("Info bars"), |ui, card, area| {
        let mut area = area.content;
        let bars = [
            ("Informational", InfoBar::new(Severity::Informational, "Informational", "Something to know.")),
            ("Success", InfoBar::new(Severity::Success, "Success", "It went well.")),
            ("Caution", InfoBar::new(Severity::Warning, "Caution", "Close me.").on_close(Message::CloseCaution)),
            ("Error", InfoBar::new(Severity::Error, "Error", "It went wrong.")),
        ];
        for (name, bar) in bars {
            if name == "Caution" && !showcase.caution_shown {
                let row = area.cut_top(CONTROL_HEIGHT, SPACE_S);
                card.add("BringBack", row.left_strip(160.0), Button::new("Bring it back").on_press(Message::BringBackCaution));
                continue;
            }
            let height = bar.height(ui, area.width);
            card.add(name, area.cut_top(height, SPACE_S), bar);
        }
    });
}

fn colors(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    tree.card(ui, None, rect, Card::new("Color"), |_, card, area| {
        let mut area = area.content;
        let picker_width = 256.0f32.min(area.width);
        let picker = area.cut_top(ColorPicker::<Message>::height(picker_width), SPACE_M).left_strip(picker_width);
        card.add("Picker", picker, ColorPicker::new(showcase.color).on_change(Message::Color));
        let mut row = area.cut_top(CONTROL_HEIGHT, SPACE_M);
        let field = row.cut_left(160.0, SPACE_M);
        card.add("Hex", field, ColorField::new(showcase.color).on_change(Message::Color));
        // Swatches 24 pixels square and 8 apart, in the middle of the row's
        // height. The one picked is the one that's the color, if one is.
        let colors = PALETTE.map(Color::hex);
        let picked = colors.iter().position(|color| *color == showcase.color);
        let swatches = row.pad(Insets::new(2.0, 4.0, 0.0, 4.0));
        card.add("Palette", swatches, Palette::new(picked, &colors).size(24.0).gap(SPACE_S).on_pick(Message::Swatch));
    });
}

fn list(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let matches = showcase.matches();
    let subtitle = format!("{} of {} · Double-click to open", matches.len(), showcase.items.len());
    let names: Vec<String> = matches.iter().map(|item| showcase.items[*item].clone()).collect();
    let labels = names.clone();
    let selected = showcase.selected.and_then(|item| matches.iter().position(|index| *index == item));
    // The messages carry the item each row showed, the gesture's own fact,
    // not a row number for apply time to re-resolve.
    let picks = matches.clone();
    let opens = matches.clone();
    let list = ListView::new(matches.len(), 40.0)
        .selection(selected)
        .on_select(move |row| Message::Select(picks.get(row).copied().unwrap_or(usize::MAX)))
        .on_activate(move |row| Message::Activate(opens.get(row).copied().unwrap_or(usize::MAX)))
        .labels(move |row| labels[row].clone());
    tree.card(ui, None, rect, Card::new("List").subtitle(subtitle), |ui, card, area| {
        let rows = list.rows(ui, "Items", area.content);
        card.container(Some("Items"), area.content, list, |items| {
            for row in rows.indices() {
                let rect = rows.content(row);
                items.draw(Rect::new(rect.x, rect.y, ICON, rect.height), IconView::new(Icon::Document, ICON));
                items.draw(Rect::new(rect.x + ICON + SPACE_M, rect.y, rect.width - 80.0, rect.height), Text::label(&names[row]));
                items.draw(rect, Text::caption(format!("#{}", matches[row] + 1)).tertiary().end());
            }
        });
    });
}

fn interface(showcase: &Showcase, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
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
    let held: Vec<String> = ui.input().keys_held().iter().map(|key| key.name()).collect();
    let held = if held.is_empty() { "None".to_string() } else { held.join(", ") };
    let theme = ui.theme().clone();
    let gamepad = *ui.gamepad();
    let theme_name = theme.kind.name();
    tree.card(ui, None, rect, Card::new("Interface"), |ui, card, area| {
        let mut area = area.content;
        for (label, value) in rows {
            card.draw(area.cut_top(24.0, 0.0), Field::new(label, 160.0, value));
        }
        card.draw(area.cut_top(24.0, SPACE_S), Field::new("Keys held", 160.0, held));
        card.draw(area.cut_top(BODY_STRONG.line_height, SPACE_S), Text::heading("Gamepad"));
        let mut sticks = area.cut_top(96.0, SPACE_S);
        for stick in [gamepad.left_stick, gamepad.right_stick] {
            let box_rect = sticks.cut_left(96.0, SPACE_M);
            let (ring, divider, accent) = (theme.control_strong_stroke, theme.divider, theme.accent);
            card.draw(
                box_rect,
                Canvas::new(move |painter, rect| {
                    let center = rect.center();
                    painter.ring(center, 44.0, 1.0, ring);
                    painter.line(Point::new(center.x - 44.0, center.y), Point::new(center.x + 44.0, center.y), 1.0, divider);
                    painter.line(Point::new(center.x, center.y - 44.0), Point::new(center.x, center.y + 44.0), 1.0, divider);
                    painter.circle(Point::new(center.x + stick.x * 40.0, center.y - stick.y * 40.0), 5.0, accent);
                }),
            );
        }
        for trigger in [gamepad.left_trigger, gamepad.right_trigger] {
            let bar = sticks.cut_left(12.0, SPACE_S);
            let (track, accent) = (theme.control_alt_fill_tertiary, theme.accent);
            card.draw(
                bar,
                Canvas::new(move |painter, bar| {
                    painter.rounded(bar, 6.0, track);
                    let height = bar.height * trigger.clamp(0.0, 1.0);
                    painter.rounded(Rect::new(bar.x, bar.bottom() - height, bar.width, height), 6.0, accent);
                }),
            );
        }
        let buttons = gamepad.buttons.names();
        let pressed = if !gamepad.connected { "No gamepad".to_string() } else if buttons.is_empty() { "No buttons held".to_string() } else { buttons.join(", ") };
        card.draw(area.cut_top(CAPTION.line_height, SPACE_L), Text::caption(pressed));
        let mut badges = area.cut_top(24.0, SPACE_L);
        let width = Keycaps::width(ui.fonts(), "Ctrl+Shift+S");
        card.draw(Rect::new(badges.x, badges.center().y - 10.0, width, 20.0), Keycaps::new("Ctrl+Shift+S"));
        badges.cut_left(width + SPACE_M, 0.0);
        let size = Badge::size(ui.fonts(), "3");
        card.draw(Rect::new(badges.x, badges.y, 24.0, badges.height), Badge::new("3"));
        badges.cut_left(size.width + SPACE_S + 8.0, 0.0);
        card.draw(Rect::new(badges.x, badges.y, 12.0, badges.height), Badge::dot());
        card.draw(Rect::new(badges.x + 20.0, badges.y, badges.width - 20.0, badges.height), Text::caption("Keycaps, badges and a dot"));
        card.add("Details", area.cut_top(CONTROL_HEIGHT, SPACE_L).left_strip(200.0), Button::new("Details").icon(Icon::Info)).set_tooltip_lines(vec![
            TooltipLine::heading("Details"),
            TooltipLine::pair("Theme", theme_name),
            TooltipLine::Separator,
            TooltipLine::text("Tooltips can have lines, columns and separators."),
        ]);
        card.draw(area.cut_top(BODY_STRONG.line_height, SPACE_S), Text::heading("Log"));
        let lines = ((area.height / 20.0).floor() as usize).min(showcase.log.len());
        for line in showcase.log.iter().take(lines) {
            let row = area.cut_top(20.0, 0.0);
            card.draw(row, Text::label(line).style(MONO).secondary());
        }
    });
}
