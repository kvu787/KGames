// Lays out and draws the modeler's interface with Library/UserInterface: the
// app bar with its menu bar, the status bar, and the cards, which have fixed
// places in pixels for the two client areas that programs support, 2560 x
// 1369 maximized and 2560 x 1440 in borderless fullscreen. The view and the
// cards at the bottom of the side columns get the 71 pixels that fullscreen
// adds.
//
// Left: Select, with the selection mode and the tool; Edit, with the edits;
// Add, with primitives; and Model, with what the model has. Middle: the
// view, which the modeler's render fills with the mesh. Right: Shading, with
// the shader and subdivision, and Paints, with the paints and SimplePaint's
// settings for the one picked.

use simple_paint::{INTERIOR_MAXIMUM, MARGIN};
use user_interface::tokens::{BODY, CAPTION, CARD_GAP, CONTROL_HEIGHT, MONO, SPACE_M, SPACE_S, SPACE_XS};
use user_interface::{
    AppBar, Border, Button, Card, ColorField, ColorPicker, ComboBox, Dialog, DialogResult, Form, Icon, ListView, MenuBar, MenuItem, Rect,
    Response, Segmented, Severity, Size, Slider, Status, TextBox, ToggleSwitch, Ui, View,
};

use crate::camera::Viewpoint;
use crate::editing::{Mode, Primitive, Tool};
use crate::interaction;
use crate::modeler::{Command, MAXIMUM_LEVELS, Modeler, Pending, Prompt, TITLE};
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

pub fn draw(modeler: &mut Modeler, ui: &mut Ui) {
    // A paint's edit ends once the pointer lets go and its name has no focus.
    if modeler.paint_edit.is_some() && !ui.input().any_button_down() && !ui.is_focused("Paint/Name") {
        modeler.end_paint_edit();
    }
    let paints = modeler.document.paints.len();
    modeler.editor.active_paint = modeler.editor.active_paint.min(paints - 1);

    interaction::keys(modeler, ui);
    app_bar(modeler, ui);
    status_bar(modeler, ui);

    let mut page = ui.page();
    let left = page.cut_left(LEFT_WIDTH, CARD_GAP);
    let right = page.cut_right(RIGHT_WIDTH, CARD_GAP);
    left_column(modeler, ui, left);
    view_card(modeler, ui, page);
    right_column(modeler, ui, right);

    interaction::operation(modeler, ui);
    prompt(modeler, ui);
}

fn item(modeler: &Modeler, command: Command, icon: Option<Icon>) -> MenuItem<'static> {
    let mut item = MenuItem::new(command.label()).enabled(modeler.can(command));
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

fn app_bar(modeler: &mut Modeler, ui: &mut Ui) {
    let dirty = if modeler.document.dirty { " · Not saved" } else { "" };
    let subtitle = format!("{}{dirty}", modeler.document.name);
    let bar = ui.app_bar(AppBar::new(Icon::Edit, TITLE).subtitle(&subtitle));

    let m = &*modeler;
    let file = [
        item(m, Command::New, Some(Icon::Document)),
        item(m, Command::Open, Some(Icon::OpenFile)),
        item(m, Command::Save, Some(Icon::Save)),
        item(m, Command::SaveAs, Some(Icon::SaveAs)),
        MenuItem::separator(),
        item(m, Command::ExportObj, Some(Icon::Export)),
    ];
    let edit = [
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
    let primitives: Vec<MenuItem> = Primitive::ALL.iter().map(|primitive| item(m, Command::Add(*primitive), None)).collect();
    let mesh = [
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
        MenuItem::submenu("Add", &primitives).icon(Icon::Add),
    ];
    let paint = [
        item(m, Command::AssignPaint, Some(Icon::Brush)),
        item(m, Command::PaintTool, None),
        MenuItem::separator(),
        item(m, Command::AddPaint, Some(Icon::Add)),
        item(m, Command::RemovePaint, Some(Icon::Remove)),
    ];
    let viewpoints: Vec<MenuItem> =
        [Viewpoint::Front, Viewpoint::Back, Viewpoint::Right, Viewpoint::Left, Viewpoint::Top, Viewpoint::Bottom].iter().map(|viewpoint| item(m, Command::View(*viewpoint), None)).collect();
    let levels: Vec<MenuItem> = (0..=MAXIMUM_LEVELS).map(|levels| item(m, Command::Level(levels), None)).collect();
    let view = [
        item(m, Command::FrameAll, Some(Icon::Home)),
        item(m, Command::FrameSelected, Some(Icon::Search)),
        MenuItem::submenu("Look from", &viewpoints).icon(Icon::Camera),
        MenuItem::separator(),
        MenuItem::submenu("Subdivision", &levels).icon(Icon::Tiles),
        item(m, Command::ToggleShader, None),
        item(m, Command::ToggleSmooth, None),
        item(m, Command::ToggleXray, None),
        item(m, Command::ToggleGrid, None),
        MenuItem::separator(),
        item(m, Command::ToggleTheme, Some(Icon::Moon)),
        item(m, Command::Fullscreen, Some(Icon::FullScreen)),
    ];
    let menus: [(&str, &[MenuItem]); 5] = [("File", &file), ("Edit", &edit), ("Mesh", &mesh), ("Paint", &paint), ("View", &view)];
    let menu = MenuBar::new(&menus);
    let width = menu.width(ui);
    let chosen = ui.add("Menu", Rect::new(bar.x, bar.y, width, bar.height), menu).map(str::to_string);
    if let Some(label) = chosen
        && let Some(command) = Command::from_label(&label)
    {
        modeler.run(ui, command);
    }
}

fn status_bar(modeler: &Modeler, ui: &mut Ui) {
    let message = modeler.editor.message.as_ref().map(|message| (Severity::Warning, message.clone())).or_else(|| modeler.message.clone());
    let amount;
    let mut items = Vec::new();
    if let Some(operation) = &modeler.editor.operation {
        amount = format!("{} · {} · {}", operation.name, operation.axis.name(), operation.amount);
        items.extend([
            Status::Shortcut("X", "X axis"),
            Status::Shortcut("Y", "Y axis"),
            Status::Shortcut("Z", "Z axis"),
            Status::Shortcut("N", "Normal"),
            Status::Shortcut("Ctrl", "Snap"),
            Status::Shortcut("Enter", "Keep"),
            Status::Shortcut("Esc", "Cancel"),
            Status::Spacer,
            Status::Text(&amount),
        ]);
    } else {
        items.extend([
            Status::Shortcut("1", "Vertices"),
            Status::Shortcut("2", "Edges"),
            Status::Shortcut("3", "Faces"),
            Status::Shortcut("G", "Move"),
            Status::Shortcut("R", "Rotate"),
            Status::Shortcut("S", "Scale"),
            Status::Shortcut("E", "Extrude"),
            Status::Shortcut("Ctrl+R", "Loop cut"),
            Status::Shortcut("Alt", "Click a loop"),
            Status::Shortcut("Middle", "Orbit"),
            Status::Shortcut("Shift+Middle", "Pan"),
            Status::Spacer,
        ]);
        if let Some((severity, text)) = &message {
            items.push(Status::Indicator(*severity, text));
        }
    }
    ui.status_bar(&items);
}

fn left_column(modeler: &mut Modeler, ui: &mut Ui, mut column: Rect) {
    let bottom = select_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = edit_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = add_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    model_card(modeler, ui, column);
}

// A button for a command, with its shortcut in its tooltip.
fn command_button(modeler: &mut Modeler, ui: &mut Ui, rect: Rect, command: Command, label: &str, icon: Option<Icon>) {
    let mut button = Button::new(label);
    if let Some(icon) = icon {
        button = button.icon(icon);
    }
    let enabled = ui.enabled();
    ui.set_enabled(enabled && modeler.can(command));
    let response = ui.add(command.label(), rect, button);
    ui.set_enabled(enabled);
    let tip = match command.shortcut() {
        Some(shortcut) => format!("{} ({shortcut})", command.label()),
        None => command.label().to_string(),
    };
    ui.tooltip(&response, &tip);
    if response.clicked {
        modeler.run(ui, command);
    }
}

fn select_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) -> f32 {
    let open = ui.begin_card(rect, Card::new("Select"));
    let mut form = Form::new(open.area.content);
    let modes: Vec<&str> = Mode::ALL.iter().map(|mode| match mode {
        Mode::Vertices => "Vertices",
        Mode::Edges => "Edges",
        Mode::Faces => "Faces",
    }).collect();
    let mut mode = Mode::ALL.iter().position(|mode| *mode == modeler.editor.mode).unwrap_or(2);
    let row = form.control(ui, "Selection mode");
    if ui.add("Mode", row, Segmented::new(&mut mode, &modes)).changed {
        modeler.run(ui, [Command::VertexMode, Command::EdgeMode, Command::FaceMode][mode]);
    }
    let tools = [(Icon::Edit, "Select"), (Icon::Brush, "Paint"), (Icon::Cut, "Loop cut")];
    let mut tool = Tool::ALL.iter().position(|tool| *tool == modeler.editor.tool).unwrap_or(0);
    let row = form.control(ui, "Tool");
    if ui.add("Tool", row, Segmented::new(&mut tool, &tools)).changed {
        modeler.run(ui, [Command::SelectTool, Command::PaintTool, Command::LoopCutTool][tool]);
    }
    let [all, none, invert] = form.columns::<3>(CONTROL_HEIGHT);
    command_button(modeler, ui, all, Command::SelectAll, "All", None);
    command_button(modeler, ui, none, Command::SelectNone, "None", None);
    command_button(modeler, ui, invert, Command::InvertSelection, "Invert", None);
    let bottom = form.bottom();
    ui.end_card(open, bottom).bottom()
}

fn edit_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) -> f32 {
    let open = ui.begin_card(rect, Card::new("Edit"));
    let mut form = Form::new(open.area.content).gap(SPACE_S);
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
            command_button(modeler, ui, rect, command, label, icon);
        }
    }
    form.space(SPACE_S);
    let rect = form.take(Slider::<f64>::height(true));
    ui.add("Sharpness", rect, Slider::new(&mut modeler.crease, 0.0, 10.0).step(0.25).header("Crease sharpness").show_value(2));
    let [crease, clear] = form.columns::<2>(CONTROL_HEIGHT);
    command_button(modeler, ui, crease, Command::Crease, "Crease", None);
    command_button(modeler, ui, clear, Command::ClearCrease, "Clear crease", None);
    let bottom = form.bottom();
    ui.end_card(open, bottom).bottom()
}

fn add_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) -> f32 {
    let open = ui.begin_card(rect, Card::new("Add"));
    let mut form = Form::new(open.area.content);
    let mut row = form.control(ui, "Primitive");
    let button = row.cut_right(96.0, SPACE_S);
    let names: Vec<&str> = Primitive::ALL.iter().map(|primitive| Command::Add(*primitive).label()).collect();
    ui.add("Primitive", row, ComboBox::new(&mut modeler.primitive, &names));
    let primitive = Primitive::ALL[modeler.primitive.min(Primitive::ALL.len() - 1)];
    let response = ui.add("AddPrimitive", button, Button::new("Add").icon(Icon::Add));
    ui.tooltip(&response, "Adds it at the origin, in the paint picked, and selects it");
    if response.clicked {
        modeler.run(ui, Command::Add(primitive));
    }
    let bottom = form.bottom();
    ui.end_card(open, bottom).bottom()
}

fn model_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Model"));
    let mut area = card.content;
    let document = &modeler.document;
    let mesh = &document.mesh;
    let selection = &document.selection;
    let surface_faces = modeler.surface.as_ref().map_or(mesh.faces.len(), |surface| surface.mesh.faces.len());
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
        ui.field(row, label, FIELD_LABEL_WIDTH, &value);
    }
}

// The view: a card whose content the modeler's render fills with the mesh,
// with what's under way over its top left, and the box being selected.
fn view_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) {
    let subtitle = format!(
        "{} mode · {} tool · {} shader{}{}",
        modeler.editor.mode.name(),
        modeler.editor.tool.name(),
        modeler.shader.name(),
        if modeler.levels > 0 { format!(" · Subdivided {}×", modeler.levels) } else { String::new() },
        if modeler.xray { " · X-ray" } else { "" },
    );
    let area = ui.card_with_view(rect, Card::new("View").subtitle(&subtitle));
    modeler.view = area.content;
    let response = ui.add("View", area.content, View::new());
    interaction::view(modeler, ui, &response);
    context_menu(modeler, ui, &response);

    // The box being selected.
    if let Some((a, b)) = modeler.editor.selecting_box {
        let accent = ui.theme().accent;
        let box_rect = Rect::from_corners(a, b);
        ui.painter().shape(box_rect, 0.0_f32, accent.fade(0.12), Some(Border::new(accent, 1.0)));
    }
    // What's under way, at the view's top left.
    let label = match &modeler.editor.operation {
        Some(operation) => Some(format!("{} · {} · {}", operation.name, operation.axis.name(), operation.amount)),
        None => match modeler.editor.tool {
            Tool::LoopCut => Some("Click an edge to cut the loop across it".to_string()),
            Tool::Paint => Some(format!("Painting with {}", modeler.document.paints[modeler.editor.active_paint].name)),
            Tool::Select => None,
        },
    };
    if let Some(label) = label {
        let color = ui.theme().text_secondary;
        let at = area.content.pad(user_interface::Insets::all(SPACE_M));
        ui.text(Rect::new(at.x, at.y, at.width, MONO.line_height), &label, MONO, color);
    }
}

fn context_menu(modeler: &mut Modeler, ui: &mut Ui, response: &Response) {
    let m = &*modeler;
    let items = [
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
    ];
    let chosen = ui.context_menu("ViewMenu", response, &items).map(str::to_string);
    if let Some(label) = chosen
        && let Some(command) = Command::from_label(&label)
    {
        modeler.run(ui, command);
    }
}

fn right_column(modeler: &mut Modeler, ui: &mut Ui, mut column: Rect) {
    let bottom = shading_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    paints_card(modeler, ui, column);
}

fn shading_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) -> f32 {
    let open = ui.begin_card(rect, Card::new("Shading"));
    let mut form = Form::new(open.area.content);
    let mut shader = if modeler.shader == Shader::SimplePaint { 0 } else { 1 };
    let row = form.control(ui, "Shader");
    if ui.add("Shader", row, Segmented::new(&mut shader, &["SimplePaint", "Flat"])).changed {
        modeler.shader = if shader == 0 { Shader::SimplePaint } else { Shader::Flat };
    }
    let rect = form.take(Slider::<f64>::height(true));
    let mut levels = modeler.levels;
    if ui.add("Subdivision", rect, Slider::new(&mut levels, 0.0, MAXIMUM_LEVELS as f64).step(1.0).header("Catmull-Clark subdivision levels").show_value(0)).changed {
        modeler.run(ui, Command::Level(levels));
    }
    for (name, label, command) in [
        ("Smooth", "Smooth shading", Command::ToggleSmooth),
        ("XRay", "X-ray", Command::ToggleXray),
        ("Grid", "Grid", Command::ToggleGrid),
    ] {
        let mut value = modeler.checked(command).unwrap_or(false);
        let row = form.take(CONTROL_HEIGHT);
        if ui.add(name, row, ToggleSwitch::new(&mut value, label)).changed {
            modeler.run(ui, command);
        }
    }
    let bottom = form.bottom();
    ui.end_card(open, bottom).bottom()
}

fn paints_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) {
    let card = ui.card(rect, Card::new("Paints").subtitle("Each in SimplePaint's settings"));
    let mut form = Form::new(card.content);

    // The paints, of which the one picked is what Paint and new primitives
    // use.
    let list = form.take(ROW_HEIGHT * PAINT_ROWS);
    let counts: Vec<usize> = (0..modeler.document.paints.len()).map(|paint| modeler.document.mesh.faces.iter().filter(|face| face.material as usize == paint).count()).collect();
    let paints: Vec<(String, user_interface::Color)> = modeler.document.paints.iter().map(|paint| (paint.name.clone(), paint.color())).collect();
    let mut picked = Some(modeler.editor.active_paint);
    ui.add(
        "Paints",
        list,
        ListView::new(paints.len(), ROW_HEIGHT, |ui, index, rect, _| {
            let (name, color) = &paints[index];
            let swatch = Rect::new(rect.x, rect.center().y - SWATCH / 2.0, SWATCH, SWATCH);
            ui.swatch(swatch, *color);
            let primary = ui.theme().text_primary;
            let secondary = ui.theme().text_secondary;
            let mut text = Rect::new(rect.x + SWATCH + SPACE_M, rect.y, rect.width - SWATCH - SPACE_M, rect.height);
            let count = if counts[index] == 1 { "1 face".to_string() } else { format!("{} faces", counts[index]) };
            let count_rect = text.cut_right(96.0, SPACE_S);
            ui.text(text, name, BODY, primary);
            ui.text_aligned(count_rect, &count, CAPTION, secondary, user_interface::Align::End);
        })
        .selection(&mut picked),
    );
    if let Some(index) = picked {
        modeler.editor.active_paint = index;
    }
    let [add, remove, assign] = form.columns::<3>(CONTROL_HEIGHT);
    command_button(modeler, ui, add, Command::AddPaint, "Add", Some(Icon::Add));
    command_button(modeler, ui, remove, Command::RemovePaint, "Remove", Some(Icon::Remove));
    command_button(modeler, ui, assign, Command::AssignPaint, "Paint", Some(Icon::Brush));

    // The paint picked: its color, beside its name and its color's field.
    let index = modeler.editor.active_paint;
    let mut row = form.take(ColorPicker::height(PICKER_WIDTH));
    let picker = row.cut_left(PICKER_WIDTH, SPACE_M);
    let mut color = modeler.document.paints[index].color();
    if ui.add("Paint/Color", picker, ColorPicker::new(&mut color)).changed {
        modeler.begin_paint_edit();
        modeler.document.paints[index].set_color(color);
        modeler.document.touch();
    }
    let mut side = Form::new(row);
    let name_row = side.control(ui, "Name");
    let mut name = modeler.document.paints[index].name.clone();
    if ui.add("Paint/Name", name_row, TextBox::new(&mut name).max_length(32)).response.changed {
        modeler.begin_paint_edit();
        modeler.document.paints[index].name = name;
        modeler.document.touch();
    }
    let hex_row = side.control(ui, "Base color");
    if ui.add("Paint/Hex", hex_row, ColorField::new(&mut color)).changed {
        modeler.begin_paint_edit();
        modeler.document.paints[index].set_color(color);
        modeler.document.touch();
    }

    // SimplePaint's other settings.
    let mut parameters = modeler.document.paints[index].parameters;
    let mut changed = false;
    let sliders: [(&str, &str, &mut f64, f64, f64, usize, &str); 5] = [
        ("Paint/Brightness", "Brightness", &mut parameters.brightness, MARGIN, INTERIOR_MAXIMUM, 2, ""),
        ("Paint/Shift", "Shift", &mut parameters.shift, 0.0, INTERIOR_MAXIMUM, 2, ""),
        ("Paint/Rotation", "Rotation", &mut parameters.rotation_degrees, 0.0, 359.0, 0, "°"),
        ("Paint/Dark", "Dark point", &mut parameters.dark_point, 0.0, INTERIOR_MAXIMUM, 2, ""),
        ("Paint/Light", "Light point", &mut parameters.light_point, MARGIN, 1.0, 2, ""),
    ];
    for (name, header, value, minimum, maximum, decimals, suffix) in sliders {
        let rect = form.take(Slider::<f64>::height(true));
        let step = if decimals == 0 { 1.0 } else { 0.01 };
        changed |= ui.add(name, rect, Slider::new(value, minimum, maximum).step(step).header(header).show_value(decimals).suffix(suffix)).changed;
    }
    if changed {
        modeler.begin_paint_edit();
        modeler.document.paints[index].parameters = parameters;
        modeler.document.touch();
    }
}

// The dialog that's open, if any.
fn prompt(modeler: &mut Modeler, ui: &mut Ui) {
    let Some(prompt) = modeler.prompt else {
        return;
    };
    match prompt {
        Prompt::Unsaved(pending) => {
            let message = format!("{} has changes that aren't saved.", modeler.document.name);
            let dialog = Dialog::new("Save your changes?", &message).primary("Save").secondary("Don't save").close("Cancel");
            match ui.dialog(prompt.name(), dialog) {
                Some(DialogResult::Primary) => {
                    if modeler.document.name == crate::document::UNTITLED {
                        modeler.prompt_save_as(Some(pending));
                    } else if modeler.save() {
                        finish(modeler, ui, pending);
                    } else {
                        modeler.prompt = None;
                    }
                }
                Some(DialogResult::Secondary) => finish(modeler, ui, pending),
                Some(DialogResult::Close) => modeler.prompt = None,
                None => {}
            }
        }
        Prompt::SaveAs => {
            let dialog = Dialog::new("Save as", "Models are saved in the program's Models folder.").primary("Save").close("Cancel");
            let error = modeler.prompt_error.clone();
            let draft = &mut modeler.name_draft;
            let height = BODY.line_height + SPACE_XS + CONTROL_HEIGHT + if error.is_some() { SPACE_S + CAPTION.line_height * 2.0 } else { 0.0 };
            let result = ui.dialog_with(prompt.name(), dialog, height, |ui, rect| {
                let mut form = Form::new(rect).gap(SPACE_S);
                let row = form.control(ui, "Name");
                ui.add("SaveAs/Name", row, TextBox::new(draft).max_length(64).placeholder("Name the model"));
                if let Some(error) = &error {
                    let color = ui.theme().critical;
                    let rest = form.rest();
                    ui.paragraph(rest, error, CAPTION, color);
                }
            });
            match result {
                Some(DialogResult::Primary) => match modeler.save_as() {
                    Ok(pending) => {
                        modeler.prompt_error = None;
                        if let Some(pending) = pending {
                            finish(modeler, ui, pending);
                        }
                    }
                    Err(error) => {
                        modeler.prompt_error = Some(error);
                        modeler.prompt = Some(Prompt::SaveAs);
                    }
                },
                Some(_) => {
                    modeler.prompt = None;
                    modeler.prompt_error = None;
                }
                None => {}
            }
        }
        Prompt::Open => {
            let names = modeler.saved_names.clone();
            let message = if names.is_empty() { "There are no models in the program's Models folder yet." } else { "Pick a model from the program's Models folder." };
            let mut dialog = Dialog::new("Open a model", message).close("Cancel");
            if !names.is_empty() {
                dialog = dialog.primary("Open");
            }
            let height = if names.is_empty() { 0.0 } else { ROW_HEIGHT * OPEN_ROWS };
            let choice = &mut modeler.open_choice;
            let mut activated = false;
            let result = ui.dialog_with(prompt.name(), dialog, height, |ui, rect| {
                if names.is_empty() {
                    return;
                }
                let response = ui.add(
                    "OpenModel/List",
                    rect,
                    ListView::new(names.len(), ROW_HEIGHT, |ui, index, rect, _| {
                        let primary = ui.theme().text_primary;
                        ui.text(rect, &names[index], BODY, primary);
                    })
                    .selection(choice),
                );
                activated = response.activated.is_some();
            });
            match result {
                Some(DialogResult::Primary) => modeler.open_chosen(),
                Some(_) => modeler.prompt = None,
                None if activated => modeler.open_chosen(),
                None => {}
            }
        }
    }
}

// Does what was pending once the changes are saved or not.
fn finish(modeler: &mut Modeler, ui: &mut Ui, pending: Pending) {
    if pending == Pending::Close {
        modeler.prompt = None;
        ui.close();
    } else {
        modeler.proceed(pending);
    }
}

// The size of the paint list's swatches, for tests.
pub const SWATCH_SIZE: Size = Size::new(SWATCH, SWATCH);
