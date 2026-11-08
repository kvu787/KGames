// Describes the modeler's interface from its state, with
// Library/UserInterface's widgets: the app bar with its menu bar, the status
// bar, and the cards, which have fixed places in pixels for the two client
// areas that programs support, 2560 x 1369 maximized and 2560 x 1440 in
// borderless fullscreen. The view and the cards at the bottom of the side
// columns get the 71 pixels that fullscreen adds.
//
// Left: Select, with the selection mode and the tool; Edit, with the edits;
// Add, with primitives; and Model, with what the model has. Middle: the
// view, which the modeler's render fills with the mesh. Right: Shading, with
// the shader and subdivision, and Paints, with the paints and SimplePaint's
// settings for the one picked.

use simple_paint::{INTERIOR_MAXIMUM, MARGIN};
use user_interface::tokens::{
    APP_BAR_HEIGHT, BODY, CAPTION, CARD_GAP, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, CONTROL_HEIGHT, MONO, PAGE_PADDING, SPACE_M,
    SPACE_S, SPACE_XS, STATUS_BAR_HEIGHT,
};
use user_interface::{
    AppBar, Border, Button, Canvas, Card, ColorField, ColorPicker, ComboBox, Cursor, Dialog, Field, Form, Icon, Insets, ListView, MenuBar,
    MenuItem, Rect, Segmented, Severity, Size, Slider, Status, Swatch, Text, TextBox, ToggleSwitch, Tree, Ui, View,
};

use crate::camera::Viewpoint;
use crate::editing::{Mode, Primitive, Tool};
use crate::modeler::{Command, MAXIMUM_LEVELS, Message, Modeler, PaintEdit, Prompt, TITLE};
use crate::viewport::Shader;

pub const LEFT_WIDTH: f32 = 328.0;
pub const RIGHT_WIDTH: f32 = 400.0;
// Rows of the paints' list that show at once.
const PAINT_ROWS: f32 = 4.0;
const ROW_HEIGHT: f32 = 40.0;
const PICKER_WIDTH: f32 = 176.0;
const SWATCH: f32 = 20.0;
const FIELD_LABEL_WIDTH: f32 = 128.0;
// The Open dialog's list.
const OPEN_ROWS: f32 = 6.0;

// Where the view card's content is, from the window's bounds alone, with no
// notices under the app bar: Prepare places the scene with it before the
// frame's view runs, and the view card lays out the same way.
pub fn view_content(bounds: Rect) -> Rect {
    let top = bounds.y + APP_BAR_HEIGHT + PAGE_PADDING;
    let bottom = bounds.bottom() - STATUS_BAR_HEIGHT - PAGE_PADDING;
    let mut page = Rect::new(PAGE_PADDING, top, bounds.width - PAGE_PADDING * 2.0, bottom - top);
    page.cut_left(LEFT_WIDTH, CARD_GAP);
    page.cut_right(RIGHT_WIDTH, CARD_GAP);
    let inner = page.inset(CARD_PADDING);
    inner.pad(Insets::new(0.0, CARD_HEADER_HEIGHT + CARD_HEADER_GAP, 0.0, 0.0))
}

pub fn view(modeler: &Modeler, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    let status = status_items(modeler);
    let shell = tree.shell(ui, app_bar(modeler), false, Some(status));
    menu_bar(modeler, ui, &mut tree, shell.commands);

    let mut page = shell.page;
    let left = page.cut_left(LEFT_WIDTH, CARD_GAP);
    let right = page.cut_right(RIGHT_WIDTH, CARD_GAP);
    left_column(modeler, ui, &mut tree, left);
    view_card(modeler, ui, &mut tree, page);
    right_column(modeler, ui, &mut tree, right);

    // A modal edit: a cover over the whole window takes the pointer from
    // everything else while it's under way, and hears it every frame.
    if modeler.editor.operation.is_some() {
        tree.add("Operation", ui.bounds(), View::new().cursor(Cursor::Move).every_frame().on_input(Message::Operation));
    }
    prompt(modeler, ui, &mut tree);
    tree
}

fn item(modeler: &Modeler, command: Command, icon: Option<Icon>) -> MenuItem<Message> {
    let mut item = MenuItem::new(command.label()).enabled(modeler.can(command)).on_choose(Message::Command(command));
    if let Some(shortcut) = command.shortcut() {
        item = item.accelerator(shortcut);
    }
    if let Some(icon) = icon {
        item = item.icon(icon);
    }
    if let Some(checked) = modeler.checked(command) {
        item = item.checked(checked);
    }
    item
}

fn app_bar(modeler: &Modeler) -> AppBar {
    let dirty = if modeler.document.dirty { " · Not saved" } else { "" };
    AppBar::new(Icon::Edit, TITLE).subtitle(format!("{}{dirty}", modeler.document.name))
}

fn menu_bar(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, bar: Rect) {
    let m = modeler;
    let file = vec![
        item(m, Command::New, Some(Icon::Document)),
        item(m, Command::Open, Some(Icon::OpenFile)),
        item(m, Command::Save, Some(Icon::Save)),
        item(m, Command::SaveAs, Some(Icon::SaveAs)),
        MenuItem::separator(),
        item(m, Command::ExportObj, Some(Icon::Export)),
    ];
    let edit = vec![
        item(m, Command::Undo, Some(Icon::Undo)),
        item(m, Command::Redo, Some(Icon::Redo)),
        MenuItem::separator(),
        item(m, Command::SelectAll, Some(Icon::SelectAll)),
        item(m, Command::SelectNone, None),
        item(m, Command::InvertSelection, None),
        MenuItem::separator(),
        item(m, Command::VertexMode, None),
        item(m, Command::EdgeMode, None),
        item(m, Command::FaceMode, None),
        MenuItem::separator(),
        item(m, Command::SelectTool, None),
        item(m, Command::PaintTool, Some(Icon::Brush)),
        item(m, Command::LoopCutTool, Some(Icon::Cut)),
    ];
    let primitives: Vec<MenuItem<Message>> = Primitive::ALL.iter().map(|primitive| item(m, Command::Add(*primitive), None)).collect();
    let mesh = vec![
        item(m, Command::Move, Some(Icon::Move)),
        item(m, Command::Rotate, Some(Icon::Rotate)),
        item(m, Command::Scale, None),
        MenuItem::separator(),
        item(m, Command::Extrude, None),
        item(m, Command::Inset, None),
        item(m, Command::Fill, Some(Icon::Fill)),
        item(m, Command::Merge, None),
        item(m, Command::Subdivide, Some(Icon::Tiles)),
        MenuItem::separator(),
        item(m, Command::Delete, Some(Icon::Delete)),
        item(m, Command::DeleteFaces, None),
        MenuItem::separator(),
        item(m, Command::FlipNormals, None),
        item(m, Command::RecalculateNormals, None),
        MenuItem::separator(),
        item(m, Command::Crease, None),
        item(m, Command::ClearCrease, None),
        MenuItem::separator(),
        MenuItem::submenu("Add", primitives).icon(Icon::Add),
    ];
    let paint = vec![
        item(m, Command::AssignPaint, Some(Icon::Brush)),
        item(m, Command::PaintTool, None),
        MenuItem::separator(),
        item(m, Command::AddPaint, Some(Icon::Add)),
        item(m, Command::RemovePaint, Some(Icon::Remove)),
    ];
    let viewpoints: Vec<MenuItem<Message>> = [Viewpoint::Front, Viewpoint::Back, Viewpoint::Right, Viewpoint::Left, Viewpoint::Top, Viewpoint::Bottom]
        .iter()
        .map(|viewpoint| item(m, Command::View(*viewpoint), None))
        .collect();
    let levels: Vec<MenuItem<Message>> = (0..=MAXIMUM_LEVELS).map(|levels| item(m, Command::Level(levels), None)).collect();
    let view = vec![
        item(m, Command::FrameAll, Some(Icon::Home)),
        item(m, Command::FrameSelected, Some(Icon::Search)),
        MenuItem::submenu("Look from", viewpoints).icon(Icon::Camera),
        MenuItem::separator(),
        MenuItem::submenu("Subdivision", levels).icon(Icon::Tiles),
        item(m, Command::ToggleShader, None),
        item(m, Command::ToggleSmooth, None),
        item(m, Command::ToggleXray, None),
        item(m, Command::ToggleGrid, None),
        MenuItem::separator(),
        item(m, Command::ToggleTheme, Some(Icon::Moon)),
        item(m, Command::VerticalSync, None).checked(ui.vertical_sync()),
        item(m, Command::Fullscreen, Some(Icon::FullScreen)),
    ];
    let menus = vec![("File", file), ("Edit", edit), ("Mesh", mesh), ("Paint", paint), ("View", view)];
    let menu = MenuBar::new(menus);
    let width = menu.width(ui);
    tree.add("Menu", Rect::new(bar.x, bar.y, width, bar.height), menu);
}

fn status_items(modeler: &Modeler) -> Vec<Status> {
    let message = modeler.editor.message.as_ref().map(|message| (Severity::Warning, message.clone())).or_else(|| modeler.message.clone());
    let mut items = Vec::new();
    if let Some(operation) = &modeler.editor.operation {
        let amount = format!("{} · {} · {}", operation.name, operation.axis.name(), operation.amount);
        items.extend([
            Status::shortcut("X", "X axis"),
            Status::shortcut("Y", "Y axis"),
            Status::shortcut("Z", "Z axis"),
            Status::shortcut("N", "Normal"),
            Status::shortcut("Ctrl", "Snap"),
            Status::shortcut("Enter", "Keep"),
            Status::shortcut("Esc", "Cancel"),
            Status::Spacer,
            Status::text(amount),
        ]);
    } else {
        items.extend([
            Status::shortcut("1", "Vertices"),
            Status::shortcut("2", "Edges"),
            Status::shortcut("3", "Faces"),
            Status::shortcut("G", "Move"),
            Status::shortcut("R", "Rotate"),
            Status::shortcut("S", "Scale"),
            Status::shortcut("E", "Extrude"),
            Status::shortcut("Ctrl+R", "Loop cut"),
            Status::shortcut("Alt", "Click a loop"),
            Status::shortcut("Middle", "Orbit"),
            Status::shortcut("Shift+Middle", "Pan"),
            Status::Spacer,
        ]);
        if let Some((severity, text)) = message {
            items.push(Status::indicator(severity, text));
        }
    }
    items
}

fn left_column(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, mut column: Rect) {
    let bottom = select_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = edit_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = add_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, CARD_GAP);
    model_card(modeler, ui, tree, column);
}

// A button for a command, with its shortcut in its tooltip.
fn command_button(modeler: &Modeler, tree: &mut Tree<Message>, rect: Rect, command: Command, label: &str, icon: Option<Icon>) {
    let mut button = Button::new(label).on_press(Message::Command(command));
    if let Some(icon) = icon {
        button = button.icon(icon);
    }
    let tip = match command.shortcut() {
        Some(shortcut) => format!("{} ({shortcut})", command.label()),
        None => command.label().to_string(),
    };
    tree.add(command.label(), rect, button).set_enabled(modeler.can(command)).set_tooltip(tip);
}

fn select_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> Rect {
    tree.sized_card(ui, None, rect, Card::new("Select"), |_, card, area| {
        let mut form = Form::new(area.content);
        let modes: Vec<&str> = Mode::ALL
            .iter()
            .map(|mode| match mode {
                Mode::Vertices => "Vertices",
                Mode::Edges => "Edges",
                Mode::Faces => "Faces",
            })
            .collect();
        let mode = Mode::ALL.iter().position(|mode| *mode == modeler.editor.mode).unwrap_or(2);
        let row = form.control(card, "Selection mode");
        card.add("Mode", row, Segmented::new(mode, &modes).on_select(|index| Message::Command([Command::VertexMode, Command::EdgeMode, Command::FaceMode][index])));
        let tools = [(Icon::Edit, "Select"), (Icon::Brush, "Paint"), (Icon::Cut, "Loop cut")];
        let tool = Tool::ALL.iter().position(|tool| *tool == modeler.editor.tool).unwrap_or(0);
        let row = form.control(card, "Tool");
        card.add("Tool", row, Segmented::new(tool, &tools).on_select(|index| Message::Command([Command::SelectTool, Command::PaintTool, Command::LoopCutTool][index])));
        let [all, none, invert] = form.columns::<3>(CONTROL_HEIGHT);
        command_button(modeler, card, all, Command::SelectAll, "All", None);
        command_button(modeler, card, none, Command::SelectNone, "None", None);
        command_button(modeler, card, invert, Command::InvertSelection, "Invert", None);
        form.bottom()
    })
}

fn edit_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> Rect {
    tree.sized_card(ui, None, rect, Card::new("Edit"), |_, card, area| {
        let mut form = Form::new(area.content).gap(SPACE_S);
        let rows: [[(Command, &str, Option<Icon>); 2]; 6] = [
            [(Command::Move, "Move", Some(Icon::Move)), (Command::Rotate, "Rotate", Some(Icon::Rotate))],
            [(Command::Scale, "Scale", None), (Command::Extrude, "Extrude", None)],
            [(Command::Inset, "Inset", None), (Command::LoopCutTool, "Loop cut", Some(Icon::Cut))],
            [(Command::Fill, "Fill", Some(Icon::Fill)), (Command::Merge, "Merge", None)],
            [(Command::Subdivide, "Subdivide", Some(Icon::Tiles)), (Command::Delete, "Delete", Some(Icon::Delete))],
            [(Command::FlipNormals, "Flip", None), (Command::RecalculateNormals, "Recalculate", None)],
        ];
        for row in rows {
            let [first, second] = form.columns::<2>(CONTROL_HEIGHT);
            for ((command, label, icon), rect) in row.into_iter().zip([first, second]) {
                command_button(modeler, card, rect, command, label, icon);
            }
        }
        form.space(SPACE_S);
        let rect = form.take(Slider::<Message>::height(true));
        card.add("Sharpness", rect, Slider::new(modeler.crease, 0.0, 10.0).step(0.25).header("Crease sharpness").show_value(2).on_change(Message::Crease));
        let [crease, clear] = form.columns::<2>(CONTROL_HEIGHT);
        command_button(modeler, card, crease, Command::Crease, "Crease", None);
        command_button(modeler, card, clear, Command::ClearCrease, "Clear crease", None);
        form.bottom()
    })
}

fn add_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> Rect {
    tree.sized_card(ui, None, rect, Card::new("Add"), |_, card, area| {
        let mut form = Form::new(area.content);
        let mut row = form.control(card, "Primitive");
        let button = row.cut_right(96.0, SPACE_S);
        let names: Vec<&str> = Primitive::ALL.iter().map(|primitive| Command::Add(*primitive).label()).collect();
        card.add("Primitive", row, ComboBox::new(modeler.primitive, &names).on_select(Message::Primitive));
        let primitive = Primitive::ALL[modeler.primitive.min(Primitive::ALL.len() - 1)];
        card.add("AddPrimitive", button, Button::new("Add").icon(Icon::Add).on_press(Message::Command(Command::Add(primitive))))
            .set_tooltip("Adds it at the origin, in the paint picked, and selects it");
        form.bottom()
    })
}

fn model_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    tree.card(ui, None, rect, Card::new("Model"), |_, card, area| {
        let mut area = area.content;
        let document = &modeler.document;
        let mesh = &document.mesh;
        let selection = &document.selection;
        let surface_faces = modeler.surface().map_or(mesh.faces.len(), |surface| surface.mesh.faces.len());
        let fields = [
            ("Name", document.name.clone()),
            ("Saved", if document.dirty { "No".to_string() } else { "Yes".to_string() }),
            ("Vertices", format!("{} · {} selected", mesh.positions.len(), selection.vertices.len())),
            ("Edges", format!("{} · {} selected", document.edges().len(), selection.edges.len())),
            ("Faces", format!("{} · {} selected", mesh.faces.len(), selection.faces.len())),
            ("Creased edges", mesh.creases.len().to_string()),
            ("Surface faces", surface_faces.to_string()),
            ("Undo", document.undo_name().unwrap_or("Nothing").to_string()),
            ("Redo", document.redo_name().unwrap_or("Nothing").to_string()),
        ];
        for (label, value) in fields {
            if area.height < BODY.line_height {
                break;
            }
            let row = area.cut_top(BODY.line_height, SPACE_XS);
            card.draw(row, Field::new(label, FIELD_LABEL_WIDTH, value));
        }
    });
}

// The view: a card whose content the modeler's render fills with the mesh,
// with what's under way over its top left, and the box being selected.
fn view_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let subtitle = format!(
        "{} mode · {} tool · {} shader{}{}",
        modeler.editor.mode.name(),
        modeler.editor.tool.name(),
        modeler.shader.name(),
        if modeler.levels > 0 { format!(" · Subdivided {}×", modeler.levels) } else { String::new() },
        if modeler.xray { " · X-ray" } else { "" },
    );
    let selecting_box = modeler.editor.selecting_box;
    // What's under way, at the view's top left.
    let label = match &modeler.editor.operation {
        Some(operation) => Some(format!("{} · {} · {}", operation.name, operation.axis.name(), operation.amount)),
        None => match modeler.editor.tool {
            Tool::LoopCut => Some("Click an edge to cut the loop across it".to_string()),
            Tool::Paint => Some(format!("Painting with {}", modeler.document.paints[modeler.editor.active_paint].name)),
            Tool::Select => None,
        },
    };
    let items = context_items(modeler);
    tree.card(ui, None, rect, Card::new("View").subtitle(subtitle).with_view(), |_, card, area| {
        card.add("View", area.content, View::new().every_frame().on_input(Message::View)).set_context_menu(items);
        // The box being selected.
        if let Some((a, b)) = selecting_box {
            card.draw(
                Rect::from_corners(a, b),
                Canvas::new(|painter, rect| {
                    let accent = painter.theme().accent;
                    painter.shape(rect, 0.0_f32, accent.fade(0.12), Some(Border::new(accent, 1.0)));
                }),
            );
        }
        if let Some(label) = label {
            let at = area.content.pad(Insets::all(SPACE_M));
            card.draw(Rect::new(at.x, at.y, at.width, MONO.line_height), Text::label(label).style(MONO).secondary());
        }
    });
}

fn context_items(modeler: &Modeler) -> Vec<MenuItem<Message>> {
    let m = modeler;
    vec![
        item(m, Command::Move, Some(Icon::Move)),
        item(m, Command::Rotate, Some(Icon::Rotate)),
        item(m, Command::Scale, None),
        MenuItem::separator(),
        item(m, Command::Extrude, None),
        item(m, Command::Inset, None),
        item(m, Command::LoopCutTool, Some(Icon::Cut)),
        item(m, Command::Fill, Some(Icon::Fill)),
        item(m, Command::Merge, None),
        item(m, Command::Subdivide, Some(Icon::Tiles)),
        MenuItem::separator(),
        item(m, Command::Crease, None),
        item(m, Command::AssignPaint, Some(Icon::Brush)),
        MenuItem::separator(),
        item(m, Command::Delete, Some(Icon::Delete)),
    ]
}

fn right_column(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, mut column: Rect) {
    let bottom = shading_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, CARD_GAP);
    paints_card(modeler, ui, tree, column);
}

fn shading_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> Rect {
    tree.sized_card(ui, None, rect, Card::new("Shading"), |_, card, area| {
        let mut form = Form::new(area.content);
        let shader = if modeler.shader == Shader::SimplePaint { 0 } else { 1 };
        let row = form.control(card, "Shader");
        card.add("Shader", row, Segmented::new(shader, &["SimplePaint", "Flat"]).on_select(|_| Message::Command(Command::ToggleShader)));
        let rect = form.take(Slider::<Message>::height(true));
        card.add(
            "Subdivision",
            rect,
            Slider::new(modeler.levels as f64, 0.0, MAXIMUM_LEVELS as f64)
                .step(1.0)
                .header("Catmull-Clark subdivision levels")
                .show_value(0)
                .on_change(|value| Message::Command(Command::Level(value.round() as u32))),
        );
        for (name, label, command) in [
            ("Smooth", "Smooth shading", Command::ToggleSmooth),
            ("XRay", "X-ray", Command::ToggleXray),
            ("Grid", "Grid", Command::ToggleGrid),
        ] {
            let value = modeler.checked(command).unwrap_or(false);
            let row = form.take(CONTROL_HEIGHT);
            card.add(name, row, ToggleSwitch::new(value, label).on_change(move |_| Message::Command(command)));
        }
        form.bottom()
    })
}

fn paints_card(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let document = &modeler.document;
    let counts: Vec<usize> = (0..document.paints.len())
        .map(|paint| document.mesh.faces.iter().filter(|face| face.material as usize == paint).count())
        .collect();
    let paints: Vec<(String, user_interface::Color)> = document.paints.iter().map(|paint| (paint.name.clone(), paint.color())).collect();
    let index = modeler.editor.active_paint;
    let labels: Vec<String> = paints.iter().map(|(name, _)| name.clone()).collect();
    let list = ListView::new(paints.len(), ROW_HEIGHT)
        .selection(Some(index))
        .on_select(Message::PickPaint)
        .labels(move |row| labels[row].clone());

    tree.card(ui, None, rect, Card::new("Paints").subtitle("Each in SimplePaint's settings"), |ui, card, area| {
        let mut form = Form::new(area.content);

        // The paints, of which the one picked is what Paint and new
        // primitives use.
        let list_rect = form.take(ROW_HEIGHT * PAINT_ROWS);
        let rows = list.rows(ui, "Paints", list_rect);
        card.container(Some("Paints"), list_rect, list, |items| {
            for row in rows.indices() {
                let rect = rows.content(row);
                let (name, color) = &paints[row];
                let swatch = Rect::new(rect.x, rect.center().y - SWATCH / 2.0, SWATCH, SWATCH);
                items.draw(swatch, Swatch::new(*color));
                let mut text = Rect::new(rect.x + SWATCH + SPACE_M, rect.y, rect.width - SWATCH - SPACE_M, rect.height);
                let count = if counts[row] == 1 { "1 face".to_string() } else { format!("{} faces", counts[row]) };
                let count_rect = text.cut_right(96.0, SPACE_S);
                items.draw(text, Text::label(name));
                items.draw(count_rect, Text::caption(count).end());
            }
        });
        let [add, remove, assign] = form.columns::<3>(CONTROL_HEIGHT);
        command_button(modeler, card, add, Command::AddPaint, "Add", Some(Icon::Add));
        command_button(modeler, card, remove, Command::RemovePaint, "Remove", Some(Icon::Remove));
        command_button(modeler, card, assign, Command::AssignPaint, "Paint", Some(Icon::Brush));

        // The paint picked: its color, beside its name and its color's field.
        let mut row = form.take(ColorPicker::<Message>::height(PICKER_WIDTH));
        let picker = row.cut_left(PICKER_WIDTH, SPACE_M);
        let color = document.paints[index].color();
        card.add("Paint/Color", picker, ColorPicker::new(color).on_change(|color| Message::Paint(PaintEdit::Color(color))));
        let mut side = Form::new(row);
        let name_row = side.control(card, "Name");
        card.add("Paint/Name", name_row, TextBox::new(&document.paints[index].name).max_length(32).on_change(|name| Message::Paint(PaintEdit::Name(name))));
        let hex_row = side.control(card, "Base color");
        card.add("Paint/Hex", hex_row, ColorField::new(color).on_change(|color| Message::Paint(PaintEdit::Color(color))));

        // SimplePaint's other settings.
        let parameters = document.paints[index].parameters;
        type Edit = fn(f64) -> PaintEdit;
        let sliders: [(&str, &str, f64, f64, f64, usize, &str, Edit); 5] = [
            ("Paint/Brightness", "Brightness", parameters.brightness, MARGIN, INTERIOR_MAXIMUM, 2, "", PaintEdit::Brightness),
            ("Paint/Shift", "Shift", parameters.shift, 0.0, INTERIOR_MAXIMUM, 2, "", PaintEdit::Shift),
            ("Paint/Rotation", "Rotation", parameters.rotation_degrees, 0.0, 359.0, 0, "°", PaintEdit::Rotation),
            ("Paint/Dark", "Dark point", parameters.dark_point, 0.0, INTERIOR_MAXIMUM, 2, "", PaintEdit::Dark),
            ("Paint/Light", "Light point", parameters.light_point, MARGIN, 1.0, 2, "", PaintEdit::Light),
        ];
        for (name, header, value, minimum, maximum, decimals, suffix, edit) in sliders {
            let rect = form.take(Slider::<Message>::height(true));
            let step = if decimals == 0 { 1.0 } else { 0.01 };
            let slider = Slider::new(value, minimum, maximum)
                .step(step)
                .header(header)
                .show_value(decimals)
                .suffix(suffix)
                .on_change(move |value| Message::Paint(edit(value)));
            card.add(name, rect, slider);
        }
    });
}

// The dialog that's open, if any.
fn prompt(modeler: &Modeler, ui: &mut Ui, tree: &mut Tree<Message>) {
    let Some(prompt) = modeler.prompt else {
        return;
    };
    match prompt {
        Prompt::Unsaved(_) => {
            let message = format!("{} has changes that aren't saved.", modeler.document.name);
            let dialog = Dialog::new("Save your changes?", message)
                .primary("Save")
                .secondary("Don't save")
                .close("Cancel")
                .on_result(Message::Unsaved);
            let area = dialog.area(ui);
            tree.add(prompt.name(), area.rect, dialog);
        }
        Prompt::SaveAs => {
            let error = modeler.prompt_error.clone();
            let height = BODY.line_height + SPACE_XS + CONTROL_HEIGHT + if error.is_some() { SPACE_S + CAPTION.line_height * 2.0 } else { 0.0 };
            let dialog = Dialog::new("Save as", "Models are saved in the program's Models folder.")
                .primary("Save")
                .close("Cancel")
                .content_height(height)
                .on_result(Message::SaveAsResult);
            let area = dialog.area(ui);
            let draft = modeler.name_draft.clone();
            tree.container(Some(prompt.name()), area.rect, dialog, |dialog| {
                let mut form = Form::new(area.content).gap(SPACE_S);
                let row = form.control(dialog, "Name");
                dialog.add("SaveAs/Name", row, TextBox::new(&draft).max_length(64).placeholder("Name the model").on_change(Message::NameDraft));
                if let Some(error) = &error {
                    dialog.draw(form.rest(), Text::paragraph(error).style(CAPTION).color(user_interface::TextColor::Custom(ui.theme().critical)));
                }
            });
        }
        Prompt::Open => {
            let names = modeler.saved_names.clone();
            let message = if names.is_empty() {
                "There are no models in the program's Models folder yet."
            } else {
                "Pick a model from the program's Models folder."
            };
            let mut dialog = Dialog::new("Open a model", message).close("Cancel").on_result(Message::OpenResult);
            if !names.is_empty() {
                dialog = dialog.primary("Open");
            }
            let height = if names.is_empty() { 0.0 } else { ROW_HEIGHT * OPEN_ROWS };
            let dialog = dialog.content_height(height);
            let area = dialog.area(ui);
            if names.is_empty() {
                tree.add(prompt.name(), area.rect, dialog);
                return;
            }
            let list = ListView::new(names.len(), ROW_HEIGHT)
                .selection(modeler.open_choice)
                .on_select(Message::OpenChoice)
                .on_activate(|_| Message::OpenActivated)
                .labels({
                    let names = names.clone();
                    move |row| names[row].clone()
                });
            let list_rect = area.content;
            let rows = list.rows(ui, "OpenModel/List", list_rect);
            tree.container(Some(prompt.name()), area.rect, dialog, |dialog| {
                dialog.container(Some("OpenModel/List"), list_rect, list, |items| {
                    for row in rows.indices() {
                        items.draw(rows.content(row), Text::label(&names[row]));
                    }
                });
            });
        }
    }
}

// The size of the paint list's swatches, for tests.
pub const SWATCH_SIZE: Size = Size::new(SWATCH, SWATCH);
