// What the keys and the pointer do in the modeler, as in Blender's edit mode:
//
// - Keys run the commands whose shortcuts they are, unless a field with focus
//   takes them, a menu or a dialog is open, or a modal edit is under way.
// - In the view, the middle button orbits, with Shift pans, and the wheel
//   zooms at the pointer. The left button selects with the Select tool: a
//   click picks, Shift adds or takes away, Alt picks a loop, and a drag
//   selects a box, with Ctrl to take it away. With the Paint tool it paints
//   the faces it presses and drags over, and with the Loop cut tool it cuts
//   the loop under the pointer. The right button opens a menu of edits.
// - A modal edit, like moving, follows the pointer anywhere until a click or
//   Enter keeps it, or a right-click or Esc takes it back. X, Y and Z limit it
//   to an axis, N to the selection's normal, and Ctrl snaps it.

use user_interface::{Cursor, Key, MouseButton, Response, Shortcut, Ui, View};

use crate::editing::{Axis, Element, Tool};
use crate::modeler::{COMMANDS, Command, Modeler};

// Runs the commands whose shortcuts were pressed this frame, without
// repeats, unless something else has the keys.
pub fn keys(modeler: &mut Modeler, ui: &mut Ui) {
    if modeler.editor.operation.is_some() || modeler.prompt.is_some() || ui.any_popup() || ui.modal() {
        return;
    }
    let modifiers = ui.input().modifiers();
    let mut pressed = Vec::new();
    for (command, _, shortcuts) in COMMANDS {
        // The library toggles fullscreen with F11 itself.
        if *command == Command::Fullscreen {
            continue;
        }
        for text in *shortcuts {
            if let Some(shortcut) = Shortcut::parse(text)
                && shortcut.modifiers == modifiers
                && ui.key_pressed_once(shortcut.key)
            {
                pressed.push(*command);
            }
        }
    }
    for command in pressed {
        modeler.run(ui, command);
    }
    // Esc goes back to the Select tool.
    if ui.key_pressed_once(Key::ESCAPE) && modeler.editor.tool != Tool::Select {
        modeler.editor.tool = Tool::Select;
    }
}

// What the pointer does in the view, whose response this is.
pub fn view(modeler: &mut Modeler, ui: &mut Ui, response: &Response) {
    let view = modeler.view;
    let modifiers = ui.input().modifiers();
    let pointer = ui.pointer();

    // The camera.
    if response.held_button == Some(MouseButton::Middle) {
        if modifiers.shift {
            modeler.camera.pan(response.drag_delta, view);
        } else {
            modeler.camera.orbit(response.drag_delta);
        }
    }
    if response.wheel.y != 0.0
        && let Some(at) = response.pointer
    {
        modeler.camera.zoom(response.wheel.y, at, view);
    }
    if modeler.editor.operation.is_some() {
        return;
    }

    // What's under the pointer, for the tool.
    let inside = pointer.filter(|point| view.contains(*point) && (response.hovered || response.held));
    let see_through = modeler.xray;
    let surface = modeler.surface.as_ref();
    modeler.editor.hover = None;
    modeler.editor.loop_preview = None;
    if let Some(point) = inside {
        match modeler.editor.tool {
            Tool::Select => {
                if !modeler.editor.pressing() || modeler.editor.selecting_box.is_none() {
                    modeler.editor.hover = modeler.editor.pick(&modeler.document, &modeler.camera, view, point, see_through, surface);
                }
            }
            Tool::Paint => modeler.editor.hover = face_at(modeler, point, surface).map(Element::Face),
            Tool::LoopCut => modeler.editor.loop_preview = modeler.editor.nearest_edge(&modeler.document, &modeler.camera, view, point),
        }
    }

    match modeler.editor.tool {
        Tool::Select => {
            if response.pressed
                && let Some(point) = pointer
            {
                modeler.editor.press(point);
            }
            if response.held_button == Some(MouseButton::Left)
                && let Some(point) = pointer
            {
                modeler.editor.drag(point);
            }
            if response.released && modeler.editor.pressing() {
                let hover = modeler.editor.hover;
                match modeler.editor.release() {
                    Some(corners) => modeler.editor.select_box(&mut modeler.document, &modeler.camera, view, corners, modifiers.shift, modifiers.control),
                    None if modifiers.alt => {
                        if let Some(point) = pointer
                            && let Some((edge, _)) = modeler.editor.nearest_edge(&modeler.document, &modeler.camera, view, point)
                        {
                            modeler.editor.select_loop(&mut modeler.document, edge, modifiers.shift);
                        }
                    }
                    None => modeler.editor.click(&mut modeler.document, hover, modifiers.shift),
                }
            }
        }
        Tool::Paint => {
            if response.held_button == Some(MouseButton::Left) {
                let face = inside.and_then(|point| face_at(modeler, point, surface));
                modeler.editor.paint_at(&mut modeler.document, face);
            }
            if response.released {
                modeler.editor.end_paint(&mut modeler.document);
            }
        }
        Tool::LoopCut => {
            if response.clicked
                && let Some((edge, fraction)) = modeler.editor.loop_preview
            {
                modeler.editor.loop_cut(&mut modeler.document, edge, fraction);
            }
        }
    }
}

// The face of the mesh under the pointer, through its surface when it's
// subdivided.
fn face_at(modeler: &Modeler, point: user_interface::Point, surface: Option<&polygon_mesh::SubdivisionSurface>) -> Option<usize> {
    let (origin, direction) = modeler.camera.ray(point, modeler.view);
    let hit = match surface {
        Some(surface) => surface.ray_cast(origin, direction),
        None => modeler.document.mesh.ray_cast(origin, direction),
    };
    hit.map(|hit| hit.face)
}

// A modal edit: a target over the whole window takes the pointer from
// everything else while it's under way.
pub fn operation(modeler: &mut Modeler, ui: &mut Ui) {
    if modeler.editor.operation.is_none() {
        return;
    }
    let bounds = ui.bounds();
    let cover = ui.add("Operation", bounds, View::new().cursor(Cursor::Move));
    for (key, axis) in [(Key::X, Axis::X), (Key::Y, Axis::Y), (Key::Z, Axis::Z), (Key::N, Axis::Normal)] {
        if ui.key_pressed_once(key) {
            modeler.editor.constrain(axis);
        }
    }
    if let Some(pointer) = ui.pointer() {
        let snap = ui.input().modifiers().control;
        modeler.editor.update(&mut modeler.document, pointer, snap, &modeler.camera, modeler.view);
    }
    if cover.pressed || ui.key_pressed_once(Key::ENTER) {
        modeler.editor.confirm(&mut modeler.document);
    } else if cover.secondary_pressed || ui.key_pressed_once(Key::ESCAPE) {
        modeler.editor.cancel(&mut modeler.document);
    }
}
