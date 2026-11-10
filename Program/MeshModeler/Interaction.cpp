#include "MeshModeler/Interaction.h"

#include "Base/Sum.h"
#include "MeshModeler/Editing.h"
#include "PolygonMesh/PolygonMesh.h"
#include "UserInterface/Input/Devices.h"

#include <initializer_list>
#include <optional>
#include <string_view>
#include <utility>

namespace mesh_modeler::interaction {

using editing::Axis;
using editing::Drawn;
using editing::Element;
using editing::Tool;
using modeler::Command;
using modeler::CommandEntry;
using modeler::Commands;
using modeler::Message;
using modeler::Modeler;
using polygon_mesh::Edge;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Shortcut;
using user_interface::Ui;
using user_interface::ViewInput;

namespace {

// The face of the mesh under the pointer, through what the view draws.
std::optional<size_t> face_at(const Modeler& modeler, Point point, Drawn drawn) {
    std::pair<math::Vector3, math::Vector3> ray = modeler.camera.ray(point, modeler.view_);
    std::optional<polygon_mesh::Hit> hit = drawn.hit(ray.first, ray.second);
    if (!hit) {
        return std::nullopt;
    }
    return hit->face;
}

} // namespace

std::vector<Message> messages(const Modeler& modeler, const Ui& ui) {
    std::vector<Message> sent;
    // A paint's edit ends once the pointer lets go and its name has no focus.
    if (modeler.paint_edit && !ui.input().any_button_down() && !ui.is_focused("Paint/Name")) {
        sent.push_back(Message::EndPaintEdit{});
    }
    if (modeler.editor.operation) {
        for (auto [key, axis] : {std::pair(Key::X, Axis::X), std::pair(Key::Y, Axis::Y), std::pair(Key::Z, Axis::Z), std::pair(Key::N, Axis::Normal)}) {
            if (ui.key_pressed_once(key)) {
                sent.push_back(Message::Constrain{axis});
            }
        }
        if (ui.key_pressed_once(Key::Enter)) {
            sent.push_back(Message::ConfirmOperation{});
        }
        if (ui.key_pressed_once(Key::Escape)) {
            sent.push_back(Message::CancelOperation{});
        }
    } else if (!modeler.prompt && !ui.any_popup() && !ui.modal()) {
        Modifiers modifiers = ui.input().modifiers();
        for (const CommandEntry& entry : Commands) {
            // The library switches the theme with F9, vertical sync with F10,
            // and fullscreen with F11, itself.
            if (base::is<Command::ToggleTheme>(entry.command) || base::is<Command::VerticalSync>(entry.command) || base::is<Command::Fullscreen>(entry.command)) {
                continue;
            }
            for (std::string_view text : entry.shortcuts) {
                std::optional<Shortcut> shortcut = Shortcut::parse(text);
                if (shortcut && shortcut->modifiers == modifiers && ui.key_pressed_once(shortcut->key_)) {
                    sent.push_back(Message::Command{entry.command});
                }
            }
        }
        // Esc goes back to the Select tool.
        if (ui.key_pressed_once(Key::Escape) && modeler.editor.tool != Tool::Select) {
            sent.push_back(Message::Command{Command::SelectTool{}});
        }
    }
    sent.push_back(Message::Prepare{});
    return sent;
}

void view(Modeler& modeler, ViewInput input) {
    Rect area = modeler.view_;
    Modifiers modifiers = input.modifiers;
    std::optional<Point> pointer = input.pointer;

    // The camera.
    if (input.holding(MouseButton::Middle)) {
        if (modifiers.shift) {
            modeler.camera.pan(input.drag_delta, area);
        } else {
            modeler.camera.orbit(input.drag_delta);
        }
    }
    if (input.wheel.y != 0.0f && pointer) {
        modeler.camera.zoom(input.wheel.y, *pointer, area);
    }
    if (modeler.editor.operation) {
        return;
    }

    // What's under the pointer, for the tool.
    std::optional<Point> inside;
    if (pointer && area.contains(*pointer) && (input.hovered || input.held)) {
        inside = pointer;
    }
    bool see_through = modeler.xray;
    Drawn drawn{
        .surface = modeler.surface(),
        .triangles = modeler.scene.triangles,
    };
    modeler.editor.hover = std::nullopt;
    modeler.editor.loop_preview = std::nullopt;
    if (inside) {
        Point point = *inside;
        switch (modeler.editor.tool) {
        case Tool::Select:
            if (!modeler.editor.pressing() || !modeler.editor.selecting_box) {
                modeler.editor.hover = modeler.editor.pick(modeler.document, modeler.camera, area, point, see_through, drawn);
            }
            break;
        case Tool::Paint:
            if (std::optional<size_t> face = face_at(modeler, point, drawn)) {
                modeler.editor.hover = Element::Face{*face};
            }
            break;
        case Tool::LoopCut:
            modeler.editor.loop_preview = modeler.editor.nearest_edge(modeler.document, modeler.camera, area, point);
            break;
        }
    }

    switch (modeler.editor.tool) {
    case Tool::Select:
        // A click selects as it goes down: with Alt, the loop through the
        // nearest edge, and otherwise what's under the pointer. A drag
        // from it takes that back, and selects a box when it's let go of.
        if (input.pressed && pointer) {
            Point point = *pointer;
            bool stepped = false;
            if (modifiers.alt) {
                if (std::optional<std::pair<Edge, float>> nearest = modeler.editor.nearest_edge(modeler.document, modeler.camera, area, point)) {
                    stepped = modeler.editor.select_loop(modeler.document, nearest->first, modifiers.shift);
                }
            } else {
                std::optional<Element> hover = modeler.editor.hover;
                stepped = modeler.editor.click(modeler.document, hover, modifiers.shift);
            }
            modeler.editor.press(point, stepped ? std::optional(modeler.document.version) : std::nullopt);
        }
        if (input.holding(MouseButton::Left) && pointer) {
            modeler.editor.drag(modeler.document, *pointer);
        }
        if (input.released) {
            if (std::optional<std::pair<Point, Point>> corners = modeler.editor.release()) {
                modeler.editor.select_box(modeler.document, modeler.camera, area, *corners, modifiers.shift, modifiers.control);
            }
        }
        break;
    case Tool::Paint:
        if (input.holding(MouseButton::Left)) {
            std::optional<size_t> face;
            if (inside) {
                face = face_at(modeler, *inside, drawn);
            }
            modeler.editor.paint_at(modeler.document, face);
        }
        if (input.released) {
            modeler.editor.end_paint(modeler.document);
        }
        break;
    case Tool::LoopCut:
        if (input.pressed && modeler.editor.loop_preview) {
            Edge edge = modeler.editor.loop_preview->first;
            float fraction = modeler.editor.loop_preview->second;
            modeler.editor.loop_cut(modeler.document, edge, fraction);
        }
        break;
    }
}

void operation(Modeler& modeler, ViewInput input) {
    if (!modeler.editor.operation) {
        return;
    }
    if (input.pointer) {
        bool snap = input.modifiers.control;
        modeler.editor.update(modeler.document, *input.pointer, snap, modeler.camera, modeler.view_);
    }
    if (input.pressed) {
        modeler.editor.confirm(modeler.document);
    } else if (input.secondary_pressed) {
        modeler.editor.cancel(modeler.document);
    }
}

} // namespace mesh_modeler::interaction
