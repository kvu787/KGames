// Lays out the modeler's interface with Library/UserInterface, and keeps it
// true to the modeler every frame: the app bar with its menu bar, the status
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
use user_interface::tokens::{BODY, CAPTION, CARD_GAP, CONTROL_HEIGHT, MONO, SPACE_M, SPACE_S, SPACE_XS};
use user_interface::{
    AppBar, Border, Button, Canvas, Card, ColorField, ColorPicker, ComboBox, Cursor, Dialog, DialogResult, Field, Form, Icon, ListRow,
    ListView, MenuBar, MenuItem, Rect, RowState, Segmented, Severity, Size, Slider, Status, Text, TextBox, ToggleSwitch, Ui, UiEventKind,
    View,
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

// The command buttons, by their nodes' names: the Select, Edit and Paints
// cards'. Each is named by its command's label.
const COMMAND_BUTTONS: &[(Command, &str, Option<Icon>)] = &[
    (Command::SelectAll, "All", None),
    (Command::SelectNone, "None", None),
    (Command::InvertSelection, "Invert", None),
    (Command::Move, "Move", Some(Icon::Move)),
    (Command::Rotate, "Rotate", Some(Icon::Rotate)),
    (Command::Scale, "Scale", None),
    (Command::Extrude, "Extrude", None),
    (Command::Inset, "Inset", None),
    (Command::LoopCutTool, "Loop cut", Some(Icon::Cut)),
    (Command::Fill, "Fill", Some(Icon::Fill)),
    (Command::Merge, "Merge", None),
    (Command::Subdivide, "Subdivide", Some(Icon::Tiles)),
    (Command::Delete, "Delete", Some(Icon::Delete)),
    (Command::FlipNormals, "Flip", None),
    (Command::RecalculateNormals, "Recalculate", None),
    (Command::Crease, "Crease", None),
    (Command::ClearCrease, "Clear crease", None),
    (Command::AddPaint, "Add", Some(Icon::Add)),
    (Command::RemovePaint, "Remove", Some(Icon::Remove)),
    (Command::AssignPaint, "Paint", Some(Icon::Brush)),
];

pub fn build(modeler: &mut Modeler, ui: &mut Ui) {
    app_bar(modeler, ui);
    ui.status_bar(Vec::new());
    let mut page = ui.page();
    let left = page.cut_left(LEFT_WIDTH, CARD_GAP);
    let right = page.cut_right(RIGHT_WIDTH, CARD_GAP);
    left_column(modeler, ui, left);
    view_card(modeler, ui, page);
    right_column(modeler, ui, right);
    // The modal edit's cover, over everything, which takes the pointer while
    // an operation is under way.
    ui.node("Operation", ui.bounds(), View::new().cursor(Cursor::Move));
    ui.set_visible("Operation", modeler.editor.operation.is_some());
    prompts(modeler, ui);
}

// What the menus, buttons, dialogs and fields did, then everything the cards
// show, true to the modeler.
pub fn update(modeler: &mut Modeler, ui: &mut Ui) {
    // A paint's edit ends once the pointer lets go and its name has no focus.
    if modeler.paint_edit.is_some() && !ui.input().any_button_down() && !ui.is_focused("Paint/Name") {
        modeler.end_paint_edit();
    }
    let paints = modeler.document.paints.len();
    modeler.editor.active_paint = modeler.editor.active_paint.min(paints - 1);

    interaction::keys(modeler, ui);
    events(modeler, ui);
    let view_response = ui.response("View");
    interaction::view(modeler, ui, &view_response);
    interaction::operation(modeler, ui);
    sync(modeler, ui);
}

fn events(modeler: &mut Modeler, ui: &mut Ui) {
    let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
    for event in events {
        match (event.source.as_str(), &event.kind) {
            ("Menu" | "ViewMenu", UiEventKind::Chose(label)) => {
                if let Some(command) = Command::from_label(label) {
                    modeler.run(ui, command);
                }
            }
            ("Mode", UiEventKind::Selected(index)) => {
                let command = [Command::VertexMode, Command::EdgeMode, Command::FaceMode][(*index).min(2)];
                modeler.run(ui, command);
            }
            ("Tool", UiEventKind::Selected(index)) => {
                let command = [Command::SelectTool, Command::PaintTool, Command::LoopCutTool][(*index).min(2)];
                modeler.run(ui, command);
            }
            ("Sharpness", UiEventKind::Changed(value)) => modeler.crease = *value as f32,
            ("Primitive", UiEventKind::Selected(index)) => modeler.primitive = *index,
            ("AddPrimitive", UiEventKind::Clicked) => {
                let primitive = Primitive::ALL[modeler.primitive.min(Primitive::ALL.len() - 1)];
                modeler.run(ui, Command::Add(primitive));
            }
            ("Shader", UiEventKind::Selected(index)) => {
                modeler.shader = if *index == 0 { Shader::SimplePaint } else { Shader::Flat };
            }
            ("Subdivision", UiEventKind::Changed(value)) => {
                let levels = *value as u32;
                modeler.run(ui, Command::Level(levels));
            }
            ("Smooth", UiEventKind::Toggled(_)) => modeler.run(ui, Command::ToggleSmooth),
            ("XRay", UiEventKind::Toggled(_)) => modeler.run(ui, Command::ToggleXray),
            ("Grid", UiEventKind::Toggled(_)) => modeler.run(ui, Command::ToggleGrid),
            ("Paints", UiEventKind::Selected(index)) => modeler.editor.active_paint = *index,
            ("Paint/Color", UiEventKind::ColorChanged(color)) => {
                modeler.begin_paint_edit();
                let index = modeler.editor.active_paint;
                modeler.document.paints[index].set_color(*color);
                modeler.document.touch();
            }
            ("Paint/Hex", UiEventKind::ColorChanged(color)) => {
                modeler.begin_paint_edit();
                let index = modeler.editor.active_paint;
                modeler.document.paints[index].set_color(*color);
                modeler.document.touch();
            }
            ("Paint/Name", UiEventKind::TextChanged(name)) => {
                modeler.begin_paint_edit();
                let index = modeler.editor.active_paint;
                modeler.document.paints[index].name = name.clone();
                modeler.document.touch();
            }
            (source, UiEventKind::Changed(value)) if source.starts_with("Paint/") => {
                modeler.begin_paint_edit();
                let index = modeler.editor.active_paint;
                let mut parameters = modeler.document.paints[index].parameters;
                match source {
                    "Paint/Brightness" => parameters.brightness = *value,
                    "Paint/Shift" => parameters.shift = *value,
                    "Paint/Rotation" => parameters.rotation_degrees = *value,
                    "Paint/Dark" => parameters.dark_point = *value,
                    "Paint/Light" => parameters.light_point = *value,
                    _ => {}
                }
                modeler.document.paints[index].parameters = parameters;
                modeler.document.touch();
            }
            ("Unsaved", UiEventKind::Dialog(result)) => {
                let Some(Prompt::Unsaved(pending)) = modeler.prompt else {
                    continue;
                };
                match result {
                    DialogResult::Primary => {
                        if modeler.document.name == crate::document::UNTITLED {
                            modeler.prompt_save_as(Some(pending));
                        } else if modeler.save() {
                            finish(modeler, ui, pending);
                        } else {
                            modeler.prompt = None;
                        }
                    }
                    DialogResult::Secondary => finish(modeler, ui, pending),
                    DialogResult::Close => modeler.prompt = None,
                }
            }
            ("SaveAs/Name", UiEventKind::TextChanged(text)) => modeler.name_draft = text.clone(),
            ("SaveAs", UiEventKind::Dialog(result)) => match result {
                DialogResult::Primary => match modeler.save_as() {
                    Ok(pending) => {
                        if modeler.prompt_error.take().is_some() {
                            ui.request_rebuild();
                        }
                        if let Some(pending) = pending {
                            finish(modeler, ui, pending);
                        }
                    }
                    Err(error) => {
                        if modeler.prompt_error.as_ref() != Some(&error) {
                            ui.request_rebuild();
                        }
                        modeler.prompt_error = Some(error);
                        modeler.prompt = Some(Prompt::SaveAs);
                    }
                },
                _ => {
                    modeler.prompt = None;
                    if modeler.prompt_error.take().is_some() {
                        ui.request_rebuild();
                    }
                }
            },
            ("OpenModel/List", UiEventKind::Selected(index)) => modeler.open_choice = Some(*index),
            ("OpenModel/List", UiEventKind::Activated(_)) => modeler.open_chosen(),
            ("OpenModel", UiEventKind::Dialog(result)) => match result {
                DialogResult::Primary => modeler.open_chosen(),
                _ => modeler.prompt = None,
            },
            (source, UiEventKind::Clicked) => {
                if let Some((command, _, _)) = COMMAND_BUTTONS.iter().find(|(command, _, _)| command.label() == source) {
                    modeler.run(ui, *command);
                }
            }
            _ => {}
        }
    }
}

fn item(modeler: &Modeler, vertical_sync: bool, command: Command, icon: Option<Icon>) -> MenuItem {
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
    if command == Command::VerticalSync {
        item = item.checked(vertical_sync);
    }
    item
}

// The menus, remade from the modeler's state: what can be done, and what's
// checked.
fn menus(modeler: &Modeler, vertical_sync: bool) -> Vec<(&'static str, Vec<MenuItem>)> {
    let m = modeler;
    let item = |command, icon| item(m, vertical_sync, command, icon);
    let file = vec![
        item(Command::New, Some(Icon::Document)),
        item(Command::Open, Some(Icon::OpenFile)),
        item(Command::Save, Some(Icon::Save)),
        item(Command::SaveAs, Some(Icon::SaveAs)),
        MenuItem::separator(),
        item(Command::ExportObj, Some(Icon::Export)),
    ];
    let edit = vec![
        item(Command::Undo, Some(Icon::Undo)),
        item(Command::Redo, Some(Icon::Redo)),
        MenuItem::separator(),
        item(Command::SelectAll, Some(Icon::SelectAll)),
        item(Command::SelectNone, None),
        item(Command::InvertSelection, None),
        MenuItem::separator(),
        item(Command::VertexMode, None),
        item(Command::EdgeMode, None),
        item(Command::FaceMode, None),
        MenuItem::separator(),
        item(Command::SelectTool, None),
        item(Command::PaintTool, Some(Icon::Brush)),
        item(Command::LoopCutTool, Some(Icon::Cut)),
    ];
    let primitives: Vec<MenuItem> = Primitive::ALL.iter().map(|primitive| item(Command::Add(*primitive), None)).collect();
    let mesh = vec![
        item(Command::Move, Some(Icon::Move)),
        item(Command::Rotate, Some(Icon::Rotate)),
        item(Command::Scale, None),
        MenuItem::separator(),
        item(Command::Extrude, None),
        item(Command::Inset, None),
        item(Command::Fill, Some(Icon::Fill)),
        item(Command::Merge, None),
        item(Command::Subdivide, Some(Icon::Tiles)),
        MenuItem::separator(),
        item(Command::Delete, Some(Icon::Delete)),
        item(Command::DeleteFaces, None),
        MenuItem::separator(),
        item(Command::FlipNormals, None),
        item(Command::RecalculateNormals, None),
        MenuItem::separator(),
        item(Command::Crease, None),
        item(Command::ClearCrease, None),
        MenuItem::separator(),
        MenuItem::submenu("Add", primitives).icon(Icon::Add),
    ];
    let paint = vec![
        item(Command::AssignPaint, Some(Icon::Brush)),
        item(Command::PaintTool, None),
        MenuItem::separator(),
        item(Command::AddPaint, Some(Icon::Add)),
        item(Command::RemovePaint, Some(Icon::Remove)),
    ];
    let viewpoints: Vec<MenuItem> = [Viewpoint::Front, Viewpoint::Back, Viewpoint::Right, Viewpoint::Left, Viewpoint::Top, Viewpoint::Bottom]
        .iter()
        .map(|viewpoint| item(Command::View(*viewpoint), None))
        .collect();
    let levels: Vec<MenuItem> = (0..=MAXIMUM_LEVELS).map(|levels| item(Command::Level(levels), None)).collect();
    let view = vec![
        item(Command::FrameAll, Some(Icon::Home)),
        item(Command::FrameSelected, Some(Icon::Search)),
        MenuItem::submenu("Look from", viewpoints).icon(Icon::Camera),
        MenuItem::separator(),
        MenuItem::submenu("Subdivision", levels).icon(Icon::Tiles),
        item(Command::ToggleShader, None),
        item(Command::ToggleSmooth, None),
        item(Command::ToggleXray, None),
        item(Command::ToggleGrid, None),
        MenuItem::separator(),
        item(Command::ToggleTheme, Some(Icon::Moon)),
        item(Command::VerticalSync, None),
        item(Command::Fullscreen, Some(Icon::FullScreen)),
    ];
    vec![("File", file), ("Edit", edit), ("Mesh", mesh), ("Paint", paint), ("View", view)]
}

// The context menu's items, over the view.
fn context_items(modeler: &Modeler, vertical_sync: bool) -> Vec<MenuItem> {
    let item = |command, icon| item(modeler, vertical_sync, command, icon);
    vec![
        item(Command::Move, Some(Icon::Move)),
        item(Command::Rotate, Some(Icon::Rotate)),
        item(Command::Scale, None),
        MenuItem::separator(),
        item(Command::Extrude, None),
        item(Command::Inset, None),
        item(Command::LoopCutTool, Some(Icon::Cut)),
        item(Command::Fill, Some(Icon::Fill)),
        item(Command::Merge, None),
        item(Command::Subdivide, Some(Icon::Tiles)),
        MenuItem::separator(),
        item(Command::Crease, None),
        item(Command::AssignPaint, Some(Icon::Brush)),
        MenuItem::separator(),
        item(Command::Delete, Some(Icon::Delete)),
    ]
}

fn app_bar(modeler: &Modeler, ui: &mut Ui) {
    let dirty = if modeler.document.dirty { " · Not saved" } else { "" };
    let subtitle = format!("{}{dirty}", modeler.document.name);
    let bar = ui.app_bar(AppBar::new(Icon::Edit, TITLE).subtitle(&subtitle));
    let vertical_sync = ui.vertical_sync();
    let menu = MenuBar::new(menus(modeler, vertical_sync).into_iter().collect());
    let width = menu.width(ui);
    ui.node("Menu", Rect::new(bar.x, bar.y, width, bar.height), menu);
}

fn sync(modeler: &Modeler, ui: &mut Ui) {
    // The app bar and the menus.
    let dirty = if modeler.document.dirty { " · Not saved" } else { "" };
    ui.get_mut::<AppBar>("AppBar").set_subtitle(&format!("{}{dirty}", modeler.document.name));
    let vertical_sync = ui.vertical_sync();
    for (title, items) in menus(modeler, vertical_sync) {
        ui.get_mut::<MenuBar>("Menu").set_menu(title, items);
    }
    ui.set_context_menu("View", "ViewMenu", context_items(modeler, vertical_sync));

    // The status bar.
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
            Status::text(&amount),
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
        if let Some((severity, text)) = &message {
            items.push(Status::indicator(*severity, text));
        }
    }
    ui.get_mut::<user_interface::StatusBar>("StatusBar").set_items(items);

    // The segments, sliders and switches.
    let mode = Mode::ALL.iter().position(|mode| *mode == modeler.editor.mode).unwrap_or(2);
    ui.get_mut::<Segmented>("Mode").set_selected(mode);
    let tool = Tool::ALL.iter().position(|tool| *tool == modeler.editor.tool).unwrap_or(0);
    ui.get_mut::<Segmented>("Tool").set_selected(tool);
    if !ui.response("Sharpness").held {
        ui.get_mut::<Slider>("Sharpness").set_value(modeler.crease);
    }
    ui.get_mut::<ComboBox>("Primitive").set_selected(modeler.primitive);
    ui.get_mut::<Segmented>("Shader").set_selected(if modeler.shader == Shader::SimplePaint { 0 } else { 1 });
    if !ui.response("Subdivision").held {
        ui.get_mut::<Slider>("Subdivision").set_value(modeler.levels as f64);
    }
    for (name, command) in [("Smooth", Command::ToggleSmooth), ("XRay", Command::ToggleXray), ("Grid", Command::ToggleGrid)] {
        let on = modeler.checked(command).unwrap_or(false);
        ui.get_mut::<ToggleSwitch>(name).set_on(on);
    }

    // The command buttons' enabledness.
    for (command, _, _) in COMMAND_BUTTONS {
        ui.set_enabled(command.label(), modeler.can(*command));
    }

    // The Model card's fields.
    let document = &modeler.document;
    let mesh = &document.mesh;
    let selection = &document.selection;
    let surface_faces = modeler.surface().map_or(mesh.faces.len(), |surface| surface.mesh.faces.len());
    let fields = [
        ("Model/Name", document.name.clone()),
        ("Model/Saved", if document.dirty { "No".to_string() } else { "Yes".to_string() }),
        ("Model/Vertices", format!("{} · {} selected", mesh.positions.len(), selection.vertices.len())),
        ("Model/Edges", format!("{} · {} selected", document.edges().len(), selection.edges.len())),
        ("Model/Faces", format!("{} · {} selected", mesh.faces.len(), selection.faces.len())),
        ("Model/Creased", mesh.creases.len().to_string()),
        ("Model/Surface", surface_faces.to_string()),
        ("Model/Undo", document.undo_name().unwrap_or("Nothing").to_string()),
        ("Model/Redo", document.redo_name().unwrap_or("Nothing").to_string()),
    ];
    for (name, value) in fields {
        ui.get_mut::<Field>(name).set_value(&value);
    }

    // The view's subtitle, overlay and modal cover.
    let subtitle = format!(
        "{} mode · {} tool · {} shader{}{}",
        modeler.editor.mode.name(),
        modeler.editor.tool.name(),
        modeler.shader.name(),
        if modeler.levels > 0 { format!(" · Subdivided {}×", modeler.levels) } else { String::new() },
        if modeler.xray { " · X-ray" } else { "" },
    );
    ui.get_mut::<Card>("ViewCard").set_subtitle(&subtitle);
    let label = match &modeler.editor.operation {
        Some(operation) => Some(format!("{} · {} · {}", operation.name, operation.axis.name(), operation.amount)),
        None => match modeler.editor.tool {
            Tool::LoopCut => Some("Click an edge to cut the loop across it".to_string()),
            Tool::Paint => Some(format!("Painting with {}", modeler.document.paints[modeler.editor.active_paint].name)),
            Tool::Select => None,
        },
    };
    ui.get_mut::<Canvas<ViewOverlay>>("ViewOverlay").data = ViewOverlay {
        selecting_box: modeler.editor.selecting_box,
        label,
    };
    ui.set_visible("Operation", modeler.editor.operation.is_some());

    // The paints.
    let counts: Vec<usize> = (0..modeler.document.paints.len())
        .map(|paint| modeler.document.mesh.faces.iter().filter(|face| face.material as usize == paint).count())
        .collect();
    let rows: Vec<ListRow> = modeler
        .document
        .paints
        .iter()
        .zip(&counts)
        .map(|(paint, count)| {
            let count = if *count == 1 { "1 face".to_string() } else { format!("{count} faces") };
            ListRow::new(&paint.name).swatch(paint.color()).trailing(&count)
        })
        .collect();
    let list = ui.get_mut::<ListView>("Paints");
    let changed = list.rows().len() != rows.len()
        || list.rows().iter().zip(&rows).any(|(a, b)| a.text != b.text || a.trailing != b.trailing || a.swatch != b.swatch);
    if changed {
        list.set_rows(rows);
    }
    list.set_selected(Some(modeler.editor.active_paint));

    // The paint picked.
    let index = modeler.editor.active_paint;
    let paint = &modeler.document.paints[index];
    let picker = ui.get_mut::<ColorPicker>("Paint/Color");
    if picker.color() != paint.color() {
        picker.set_color(paint.color());
    }
    if !ui.is_focused("Paint/Name") && ui.get::<TextBox>("Paint/Name").text() != paint.name {
        let name = paint.name.clone();
        ui.get_mut::<TextBox>("Paint/Name").set_text(&name);
    }
    let hex = ui.get_mut::<ColorField>("Paint/Hex");
    if hex.color() != paint.color() {
        hex.set_color(paint.color());
    }
    let parameters = paint.parameters;
    for (name, value) in [
        ("Paint/Brightness", parameters.brightness),
        ("Paint/Shift", parameters.shift),
        ("Paint/Rotation", parameters.rotation_degrees),
        ("Paint/Dark", parameters.dark_point),
        ("Paint/Light", parameters.light_point),
    ] {
        if !ui.response(name).held {
            ui.get_mut::<Slider>(name).set_value(value);
        }
    }

    // The dialogs. The Open dialog's buttons and height follow the saved
    // names, so it re-lays out when they change.
    let open_list_changed = {
        let list = ui.get_mut::<ListView>("OpenModel/List");
        let changed = list.rows().len() != modeler.saved_names.len() || list.rows().iter().zip(&modeler.saved_names).any(|(a, b)| a.text != *b);
        if changed {
            list.set_rows(modeler.saved_names.iter().map(|name| ListRow::new(name)).collect());
        }
        list.set_selected(modeler.open_choice);
        changed
    };
    if open_list_changed {
        ui.request_rebuild();
    }
    ui.get_mut::<Dialog>("Unsaved").set_message(&format!("{} has changes that aren't saved.", modeler.document.name));
    ui.set_visible("Unsaved", matches!(modeler.prompt, Some(Prompt::Unsaved(_))));
    ui.set_visible("SaveAs", modeler.prompt == Some(Prompt::SaveAs));
    ui.set_visible("OpenModel", modeler.prompt == Some(Prompt::Open));
    if modeler.prompt == Some(Prompt::SaveAs) && ui.get::<TextBox>("SaveAs/Name").text() != modeler.name_draft && !ui.is_focused("SaveAs/Name") {
        let draft = modeler.name_draft.clone();
        ui.get_mut::<TextBox>("SaveAs/Name").set_text(&draft);
    }
}

fn left_column(modeler: &Modeler, ui: &mut Ui, mut column: Rect) {
    let bottom = select_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = edit_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    let bottom = add_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    model_card(ui, column);
}

// A command's button, with its shortcut in its tooltip, named by the
// command's label.
fn command_button(modeler: &Modeler, ui: &mut Ui, rect: Rect, command: Command, label: &str, icon: Option<Icon>) {
    let mut button = Button::new(label);
    if let Some(icon) = icon {
        button = button.icon(icon);
    }
    ui.node(command.label(), rect, button);
    ui.set_enabled(command.label(), modeler.can(command));
    let tip = match command.shortcut() {
        Some(shortcut) => format!("{} ({shortcut})", command.label()),
        None => command.label().to_string(),
    };
    ui.set_tooltip(command.label(), &tip);
}

// A card whose height its content decides: the content is laid out in a form
// as tall as the column, and the card node, their parent, takes the height
// they used.
fn sized_card(ui: &mut Ui, name: &str, title: &str, column: Rect, content: impl FnOnce(&mut Ui, &mut Form)) -> f32 {
    let card = Card::new(title);
    let area = card.area(ui, column);
    let content_top = area.content.y;
    let mut bottom = content_top;
    ui.group(name, column, Card::new(title), |ui| {
        let mut form = Form::new(name, area.content);
        content(ui, &mut form);
        bottom = form.bottom();
    });
    let rect = Rect::new(column.x, column.y, column.width, card.height_around(bottom - content_top));
    ui.set_rect(name, rect);
    rect.bottom()
}

fn select_card(modeler: &Modeler, ui: &mut Ui, column: Rect) -> f32 {
    sized_card(ui, "SelectCard", "Select", column, |ui, form| {
        let modes: Vec<&str> = Mode::ALL
            .iter()
            .map(|mode| match mode {
                Mode::Vertices => "Vertices",
                Mode::Edges => "Edges",
                Mode::Faces => "Faces",
            })
            .collect();
        let mode = Mode::ALL.iter().position(|mode| *mode == modeler.editor.mode).unwrap_or(2);
        let row = form.control(ui, "Selection mode");
        ui.node("Mode", row, Segmented::new(&modes, mode));
        let tools = [(Icon::Edit, "Select"), (Icon::Brush, "Paint"), (Icon::Cut, "Loop cut")];
        let tool = Tool::ALL.iter().position(|tool| *tool == modeler.editor.tool).unwrap_or(0);
        let row = form.control(ui, "Tool");
        ui.node("Tool", row, Segmented::new(&tools, tool));
        let [all, none, invert] = form.columns::<3>(CONTROL_HEIGHT);
        command_button(modeler, ui, all, Command::SelectAll, "All", None);
        command_button(modeler, ui, none, Command::SelectNone, "None", None);
        command_button(modeler, ui, invert, Command::InvertSelection, "Invert", None);
    })
}

fn edit_card(modeler: &Modeler, ui: &mut Ui, column: Rect) -> f32 {
    sized_card(ui, "EditCard", "Edit", column, |ui, form| {
        form.set_gap(SPACE_S);
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
        let rect = form.take(Slider::height(true));
        ui.node("Sharpness", rect, Slider::new(modeler.crease, 0.0, 10.0).step(0.25).header("Crease sharpness").show_value(2));
        let [crease, clear] = form.columns::<2>(CONTROL_HEIGHT);
        command_button(modeler, ui, crease, Command::Crease, "Crease", None);
        command_button(modeler, ui, clear, Command::ClearCrease, "Clear crease", None);
    })
}

fn add_card(modeler: &Modeler, ui: &mut Ui, column: Rect) -> f32 {
    sized_card(ui, "AddCard", "Add", column, |ui, form| {
        let mut row = form.control(ui, "Primitive");
        let button = row.cut_right(96.0, SPACE_S);
        let names: Vec<&str> = Primitive::ALL.iter().map(|primitive| Command::Add(*primitive).label()).collect();
        ui.node("Primitive", row, ComboBox::new(&names, modeler.primitive));
        ui.node("AddPrimitive", button, Button::new("Add").icon(Icon::Add));
        ui.set_tooltip("AddPrimitive", "Adds it at the origin, in the paint picked, and selects it");
    })
}

fn model_card(ui: &mut Ui, rect: Rect) {
    ui.card("ModelCard", rect, Card::new("Model"), |ui, card| {
        let mut area = card.content;
        let fields = [
            ("Model/Name", "Name"),
            ("Model/Saved", "Saved"),
            ("Model/Vertices", "Vertices"),
            ("Model/Edges", "Edges"),
            ("Model/Faces", "Faces"),
            ("Model/Creased", "Creased edges"),
            ("Model/Surface", "Surface faces"),
            ("Model/Undo", "Undo"),
            ("Model/Redo", "Redo"),
        ];
        for (name, label) in fields {
            if area.height < BODY.line_height {
                break;
            }
            let row = area.cut_top(BODY.line_height, SPACE_XS);
            ui.node(name, row, Field::new(label, FIELD_LABEL_WIDTH, ""));
        }
    });
}

// What's drawn over the view: the box being selected, and what's under way
// at the top left.
#[derive(Clone, Default)]
struct ViewOverlay {
    selecting_box: Option<(user_interface::Point, user_interface::Point)>,
    label: Option<String>,
}

fn draw_view_overlay(ui: &mut Ui, content: Rect, data: &ViewOverlay) {
    if let Some((a, b)) = data.selecting_box {
        let accent = ui.theme().accent;
        let box_rect = Rect::from_corners(a, b);
        ui.painter().shape(box_rect, 0.0_f32, accent.fade(0.12), Some(Border::new(accent, 1.0)));
    }
    if let Some(label) = &data.label {
        let color = ui.theme().text_secondary;
        let at = content.pad(user_interface::Insets::all(SPACE_M));
        ui.text(Rect::new(at.x, at.y, at.width, MONO.line_height), label, MONO, color);
    }
}

// The view: a card whose content the modeler's render fills with the mesh.
fn view_card(modeler: &mut Modeler, ui: &mut Ui, rect: Rect) {
    let area = ui.card("ViewCard", rect, Card::new("View").subtitle("").with_view(), |ui, card| {
        ui.node("View", card.content, View::new());
        ui.node("ViewOverlay", card.content, Canvas::new(ViewOverlay::default(), draw_view_overlay));
    });
    modeler.view = area.content;
}

fn right_column(modeler: &Modeler, ui: &mut Ui, mut column: Rect) {
    let bottom = shading_card(modeler, ui, column);
    column.cut_top(bottom - column.y, CARD_GAP);
    paints_card(modeler, ui, column);
}

fn shading_card(modeler: &Modeler, ui: &mut Ui, column: Rect) -> f32 {
    sized_card(ui, "ShadingCard", "Shading", column, |ui, form| {
        let shader = if modeler.shader == Shader::SimplePaint { 0 } else { 1 };
        let row = form.control(ui, "Shader");
        ui.node("Shader", row, Segmented::new(&["SimplePaint", "Flat"], shader));
        let rect = form.take(Slider::height(true));
        ui.node(
            "Subdivision",
            rect,
            Slider::new(modeler.levels as f64, 0.0, MAXIMUM_LEVELS as f64).step(1.0).header("Catmull-Clark subdivision levels").show_value(0),
        );
        for (name, label, command) in [
            ("Smooth", "Smooth shading", Command::ToggleSmooth),
            ("XRay", "X-ray", Command::ToggleXray),
            ("Grid", "Grid", Command::ToggleGrid),
        ] {
            let value = modeler.checked(command).unwrap_or(false);
            let row = form.take(CONTROL_HEIGHT);
            ui.node(name, row, ToggleSwitch::new(value, label));
        }
    })
}

// A paint's row: its swatch, its name, and how many faces wear it.
fn paint_row(ui: &mut Ui, _index: usize, rect: Rect, _state: RowState, row: &ListRow) {
    if let Some(color) = row.swatch {
        let swatch = Rect::new(rect.x, rect.center().y - SWATCH / 2.0, SWATCH, SWATCH);
        ui.swatch(swatch, color);
    }
    let primary = ui.theme().text_primary;
    let secondary = ui.theme().text_secondary;
    let mut text = Rect::new(rect.x + SWATCH + SPACE_M, rect.y, rect.width - SWATCH - SPACE_M, rect.height);
    let count_rect = text.cut_right(96.0, SPACE_S);
    ui.text(text, &row.text, BODY, primary);
    ui.text_aligned(count_rect, &row.trailing, CAPTION, secondary, user_interface::Align::End);
}

fn paints_card(modeler: &Modeler, ui: &mut Ui, rect: Rect) {
    ui.card("PaintsCard", rect, Card::new("Paints").subtitle("Each in SimplePaint's settings"), |ui, card| {
        let mut form = Form::new("Paints", card.content);
        // The paints, of which the one picked is what Paint and new
        // primitives use.
        let list = form.take(ROW_HEIGHT * PAINT_ROWS);
        let mut paints = ListView::new(Vec::new(), ROW_HEIGHT).selectable().painter(paint_row);
        paints.set_selected(Some(modeler.editor.active_paint));
        ui.node("Paints", list, paints);
        let [add, remove, assign] = form.columns::<3>(CONTROL_HEIGHT);
        command_button(modeler, ui, add, Command::AddPaint, "Add", Some(Icon::Add));
        command_button(modeler, ui, remove, Command::RemovePaint, "Remove", Some(Icon::Remove));
        command_button(modeler, ui, assign, Command::AssignPaint, "Paint", Some(Icon::Brush));

        // The paint picked: its color, beside its name and its color's field.
        let index = modeler.editor.active_paint;
        let paint = &modeler.document.paints[index];
        let mut row = form.take(ColorPicker::height(PICKER_WIDTH));
        let picker = row.cut_left(PICKER_WIDTH, SPACE_M);
        ui.node("Paint/Color", picker, ColorPicker::new(paint.color()));
        let mut side = Form::new("PaintSide", row);
        let name_row = side.control(ui, "Name");
        ui.node("Paint/Name", name_row, TextBox::new(&paint.name).max_length(32));
        let hex_row = side.control(ui, "Base color");
        ui.node("Paint/Hex", hex_row, ColorField::new(paint.color()));

        // SimplePaint's other settings.
        let parameters = paint.parameters;
        let sliders: [(&str, &str, f64, f64, f64, usize, &str); 5] = [
            ("Paint/Brightness", "Brightness", parameters.brightness, MARGIN, INTERIOR_MAXIMUM, 2, ""),
            ("Paint/Shift", "Shift", parameters.shift, 0.0, INTERIOR_MAXIMUM, 2, ""),
            ("Paint/Rotation", "Rotation", parameters.rotation_degrees, 0.0, 359.0, 0, "°"),
            ("Paint/Dark", "Dark point", parameters.dark_point, 0.0, INTERIOR_MAXIMUM, 2, ""),
            ("Paint/Light", "Light point", parameters.light_point, MARGIN, 1.0, 2, ""),
        ];
        for (name, header, value, minimum, maximum, decimals, suffix) in sliders {
            let rect = form.take(Slider::height(true));
            let step = if decimals == 0 { 1.0 } else { 0.01 };
            ui.node(name, rect, Slider::new(value, minimum, maximum).step(step).header(header).show_value(decimals).suffix(suffix));
        }
    });
}

// The dialogs, hidden until a prompt opens them.
fn prompts(modeler: &Modeler, ui: &mut Ui) {
    let unsaved = Dialog::new("Save your changes?", "").primary("Save").secondary("Don't save").close("Cancel");
    ui.node("Unsaved", Rect::default(), unsaved);
    ui.set_visible("Unsaved", matches!(modeler.prompt, Some(Prompt::Unsaved(_))));

    let error = modeler.prompt_error.clone();
    let height = BODY.line_height + SPACE_XS + CONTROL_HEIGHT + if error.is_some() { SPACE_S + CAPTION.line_height * 2.0 } else { 0.0 };
    let save_as = Dialog::new("Save as", "Models are saved in the program's Models folder.").primary("Save").close("Cancel").content_height(height);
    let content = save_as.content_area(ui, height);
    ui.group("SaveAs", Rect::default(), save_as, |ui| {
        let mut form = Form::new("SaveAsForm", content).gap(SPACE_S);
        let row = form.control(ui, "Name");
        ui.node("SaveAs/Name", row, TextBox::new(&modeler.name_draft).max_length(64).placeholder("Name the model"));
        if let Some(error) = &error {
            ui.node("SaveAs/Error", form.rest(), Text::new(error, CAPTION).color(user_interface::TextColor::Custom(ui.theme().critical)).wrapped());
        }
    });
    ui.set_visible("SaveAs", modeler.prompt == Some(Prompt::SaveAs));

    let names = &modeler.saved_names;
    let message = if names.is_empty() { "There are no models in the program's Models folder yet." } else { "Pick a model from the program's Models folder." };
    let mut open = Dialog::new("Open a model", message).close("Cancel");
    if !names.is_empty() {
        open = open.primary("Open");
    }
    let height = if names.is_empty() { 0.0 } else { ROW_HEIGHT * OPEN_ROWS };
    let open = open.content_height(height);
    let content = open.content_area(ui, height);
    ui.group("OpenModel", Rect::default(), open, |ui| {
        let mut list = ListView::new(names.iter().map(|name| ListRow::new(name)).collect(), ROW_HEIGHT).selectable();
        list.set_selected(modeler.open_choice);
        ui.node("OpenModel/List", content, list);
    });
    ui.set_visible("OpenModel", modeler.prompt == Some(Prompt::Open));
}

// Does what was pending once the changes are saved or not.
fn finish(modeler: &mut Modeler, ui: &mut Ui, pending: Pending) {
    if pending == Pending::Close {
        modeler.prompt = None;
        ui.close();
    } else {
        modeler.proceed(pending);
        if pending == Pending::Open {
            // The Open dialog's list and buttons follow the saved names.
            ui.request_rebuild();
        }
    }
}

// The size of the paint list's swatches, for tests.
pub const SWATCH_SIZE: Size = Size::new(SWATCH, SWATCH);
