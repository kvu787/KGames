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
//   to an axis, and N to the selection's normal, unless it's an inset, which
//   has no axis. Ctrl snaps it.
//
// `messages` turns a step's keys into messages; `view` and `operation`
// apply the pointer's input, as update runs them; and `hover` finds what's
// under the pointer once the step's edits are done.

use user_interface::{Key, MouseButton, Shortcut, Ui, ViewInput};

use crate::editing::{Axis, Drawn, Element, Tool};
use crate::modeler::{COMMANDS, Command, Message, Modeler};

// A step's messages from the keys: the modal edit's, or the commands whose
// shortcuts were pressed, unless something else has the keys. Prepare goes
// last, after everything the step does.
pub fn messages(modeler: &Modeler, ui: &Ui) -> Vec<Message> {
    let mut messages = Vec::new();
    if modeler.editor.operation.is_some() {
        for (key, axis) in [(Key::X, Axis::X), (Key::Y, Axis::Y), (Key::Z, Axis::Z), (Key::N, Axis::Normal)] {
            if ui.key_pressed(key) {
                messages.push(Message::Constrain(axis));
            }
        }
        if ui.key_pressed(Key::ENTER) {
            messages.push(Message::ConfirmOperation);
        }
        if ui.key_pressed(Key::ESCAPE) {
            messages.push(Message::CancelOperation);
        }
    } else if modeler.prompt.is_none() && !ui.any_popup() && !ui.modal() {
        let modifiers = ui.input().modifiers();
        for (command, _, shortcuts) in COMMANDS {
            // The library switches the theme with F9 and vertical sync with
            // F10 itself.
            if matches!(command, Command::ToggleTheme | Command::VerticalSync) {
                continue;
            }
            for text in *shortcuts {
                if let Some(shortcut) = Shortcut::parse(text)
                    && shortcut.modifiers == modifiers
                    && ui.key_pressed(shortcut.key)
                {
                    messages.push(Message::Command(*command));
                }
            }
        }
        // Esc goes back to the Select tool.
        if ui.key_pressed(Key::ESCAPE) && modeler.editor.tool != Tool::Select {
            messages.push(Message::Command(Command::SelectTool));
        }
    }
    messages.push(Message::Prepare);
    messages
}

// What the pointer did in the view in a step.
pub fn view(modeler: &mut Modeler, input: ViewInput) {
    let view = modeler.view;
    let modifiers = input.modifiers;
    let pointer = input.pointer;

    // The camera.
    if input.holding(MouseButton::Middle) {
        if modifiers.shift {
            modeler.camera.pan(input.drag_delta, view);
        } else {
            modeler.camera.orbit(input.drag_delta);
        }
    }
    if input.wheel.y != 0.0
        && let Some(at) = pointer
    {
        modeler.camera.zoom(input.wheel.y, at, view);
    }
    // Where the pointer is over the view, which `hover` finds what's under
    // once the step's edits are done.
    let inside = pointer.filter(|point| view.contains(*point) && (input.hovered || input.held));
    modeler.editor.pointer = inside;
    if modeler.editor.operation.is_some() {
        return;
    }

    let see_through = modeler.xray;
    let drawn = Drawn {
        surface: modeler.surface.as_ref(),
        triangles: &modeler.scene.triangles,
    };
    match modeler.editor.tool {
        Tool::Select => {
            // A click selects as it goes down: with Alt, the loop through the
            // nearest edge, and otherwise what's under the pointer. A drag
            // from it takes that back, and selects a box when it's let go of.
            if input.pressed
                && let Some(point) = pointer
            {
                let stepped = if modifiers.alt {
                    match modeler.editor.nearest_edge(&modeler.document, &modeler.camera, view, point, see_through, drawn) {
                        Some((edge, _)) => modeler.editor.select_loop(&mut modeler.document, edge, modifiers.shift),
                        None => false,
                    }
                } else {
                    let element = inside.and_then(|point| modeler.editor.pick(&modeler.document, &modeler.camera, view, point, see_through, drawn));
                    modeler.editor.click(&mut modeler.document, element, modifiers.shift)
                };
                modeler.editor.press(point, stepped.then_some(modeler.document.version));
            }
            if input.holding(MouseButton::Left)
                && let Some(point) = pointer
            {
                modeler.editor.drag(&mut modeler.document, point);
            }
            if input.released
                && let Some(corners) = modeler.editor.release()
            {
                modeler.editor.select_box(&mut modeler.document, &modeler.camera, view, corners, modifiers.shift, modifiers.control);
            }
        }
        Tool::Paint => {
            if input.holding(MouseButton::Left) {
                let face = inside.and_then(|point| face_at(modeler, point, drawn));
                modeler.editor.paint_at(&mut modeler.document, face);
            }
            if input.released {
                modeler.editor.end_paint(&mut modeler.document);
            }
        }
        Tool::LoopCut => {
            if input.pressed
                && let Some((edge, fraction)) = inside.and_then(|point| modeler.editor.nearest_edge(&modeler.document, &modeler.camera, view, point, see_through, drawn))
            {
                modeler.editor.loop_cut(&mut modeler.document, edge, fraction);
            }
        }
    }
}

// What's under the pointer for the tool: the element that a click would
// pick, the face that the Paint tool would paint, or the loop that the Loop
// cut tool would cut, or nothing while a modal edit is under way or a box is
// being selected. Prepare finds it once the step's edits are done, in the
// mesh as they left it and with what the view draws of it, so that a frame
// shows and says what's under the pointer after an edit that changes what's
// there, like a Delete that renumbers the faces, or a loop cut through the
// edge that was under it.
pub fn hover(modeler: &mut Modeler) {
    modeler.editor.hover = None;
    modeler.editor.loop_preview = None;
    let Some(point) = modeler.editor.pointer.filter(|_| modeler.editor.operation.is_none()) else {
        return;
    };
    let view = modeler.view;
    let drawn = Drawn {
        surface: modeler.surface.as_ref(),
        triangles: &modeler.scene.triangles,
    };
    match modeler.editor.tool {
        Tool::Select => {
            if !modeler.editor.pressing() || modeler.editor.selecting_box.is_none() {
                modeler.editor.hover = modeler.editor.pick(&modeler.document, &modeler.camera, view, point, modeler.xray, drawn);
            }
        }
        Tool::Paint => modeler.editor.hover = face_at(modeler, point, drawn).map(Element::Face),
        Tool::LoopCut => modeler.editor.loop_preview = modeler.editor.nearest_edge(&modeler.document, &modeler.camera, view, point, modeler.xray, drawn),
    }
}

// The face of the mesh under the pointer, through what the view draws.
fn face_at(modeler: &Modeler, point: user_interface::Point, drawn: Drawn) -> Option<usize> {
    let (origin, direction) = modeler.camera.ray(point, modeler.view);
    drawn.hit(origin, direction).map(|hit| hit.face)
}

// A modal edit: what the pointer does on the cover over the whole window,
// which takes it from everything else while the edit is under way.
pub fn operation(modeler: &mut Modeler, input: ViewInput) {
    if modeler.editor.operation.is_none() {
        return;
    }
    if let Some(pointer) = input.pointer {
        let snap = input.modifiers.control;
        modeler.editor.update(&mut modeler.document, pointer, snap, &modeler.camera, modeler.view);
    }
    if input.pressed {
        modeler.editor.confirm(&mut modeler.document);
    } else if input.secondary_pressed {
        modeler.editor.cancel(&mut modeler.document);
    }
}
