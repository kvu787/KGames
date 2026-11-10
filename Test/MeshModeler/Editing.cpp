// What input does to MeshModeler's model, through the cards' and the view's
// real layout, which user_interface::Headless builds without a window or a
// GPU: picking as a press goes down and box selection, modal moves and
// extrusions, loop cuts, painting, creases, subdivision, undo, and where the
// cards go at both supported window heights.

#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Math/Math.h"
#include "MeshModeler/MeshModeler.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"
#include "Json/Json.h"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <format>
#include <initializer_list>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <system_error>
#include <tuple>
#include <utility>
#include <vector>

using math::Vector3;
using mesh_modeler::document::Document;
using mesh_modeler::document::Snapshot;
using mesh_modeler::editing::Element;
using mesh_modeler::modeler::Command;
using mesh_modeler::modeler::Message;
using mesh_modeler::modeler::Modeler;
using mesh_modeler::modeler::PaintEdit;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Rect;
using user_interface::ThemeKind;

namespace tokens = user_interface::tokens;

namespace {

std::filesystem::path folder_for(std::string_view test) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "MeshModelerEditingTests" / test;
    std::error_code ignored;
    std::filesystem::remove_all(folder, ignored);
    std::filesystem::create_directories(folder);
    return folder;
}

Headless<Modeler> modeler(std::string_view test) {
    Headless<Modeler> app{Modeler(folder_for(test))};
    app.frames(2);
    return app;
}

// Where a point of the world is in the window.
Point screen(const Headless<Modeler>& app, Vector3 point) {
    return app.program.camera.to_screen(point, app.program.view_).first;
}

// The vertex at a position of the starting cube, whose corners are at 1 and
// -1.
std::uint32_t vertex_at(const Headless<Modeler>& app, Vector3 position) {
    const Mesh& mesh = app.program.document.mesh;
    for (std::uint32_t vertex = 0; vertex < static_cast<std::uint32_t>(mesh.positions.size()); vertex++) {
        if (mesh.position(vertex) == position) {
            return vertex;
        }
    }
    base::panic("no vertex is there");
}

json::Value state_at(const Headless<Modeler>& app, std::initializer_list<std::string_view> path) {
    json::Value value = app.state();
    for (std::string_view part : path) {
        const json::Value* member = value.get(part);
        if (member == nullptr) {
            base::panic(std::format("no {} in the state", part));
        }
        json::Value next = *member;
        value = std::move(next);
    }
    return value;
}

double number(const Headless<Modeler>& app, std::initializer_list<std::string_view> path) {
    json::Value value = state_at(app, path);
    if (const json::Value::Number* found = base::as<json::Value::Number>(value)) {
        return found->value;
    }
    base::panic(std::format("{} is {}", testing::describe(path), value.to_json()));
}

std::optional<std::string> text(const Headless<Modeler>& app, std::string_view name) {
    json::Value value = state_at(app, {name});
    if (const json::Value::String* found = base::as<json::Value::String>(value)) {
        return found->value;
    }
    return std::nullopt;
}

bool flag(const Headless<Modeler>& app, std::string_view name) {
    return state_at(app, {name}) == json::Value(true);
}

// What the modal edit under way says of itself, like its Kind or Axis.
std::optional<std::string> operation(const Headless<Modeler>& app, std::string_view name) {
    json::Value value = state_at(app, {"Operation"});
    if (const json::Value* member = value.get(name)) {
        if (const json::Value::String* found = base::as<json::Value::String>(*member)) {
            return found->value;
        }
    }
    return std::nullopt;
}

// The name of a step of the history, or of none, as a test names what it
// expects of Undo and Redo.
std::optional<std::string_view> step(std::string_view name) {
    return name;
}

} // namespace

TEST(a_click_in_the_view_selects_the_face_under_it) {
    Headless<Modeler> app = modeler("ClickFace");
    Point center = app.program.view_.center();
    app.click_point(center);
    const Selection& selection = app.program.document.selection;
    CHECK_EQ(selection.faces.size(), 1);
    // The camera looks at the origin from +Z's side, so the face in the
    // middle of the view is the cube's +Z face.
    size_t face = *selection.faces.begin();
    Vector3 normal = app.program.document.mesh.face_normal(face);
    CHECK(normal[2] > 0.99f, "{}", testing::describe(normal));
    CHECK_EQ(selection.vertices.size(), 4);
    CHECK_EQ(app.program.document.undo_name(), step("Select"));

    // A click on nothing selects nothing.
    Point corner = app.program.view_.origin() + Point(20.0f, 20.0f);
    app.click_point(corner);
    CHECK(app.program.document.selection.is_empty());
}

TEST(a_press_in_the_view_selects_as_it_goes_down) {
    Headless<Modeler> app = modeler("PressFace");
    app.press_point(app.program.view_.center());
    // The press's own frame has the selection, and the step for Undo.
    CHECK_EQ(app.program.document.selection.faces.size(), 1);
    CHECK_EQ(app.program.document.undo_name(), step("Select"));
    app.release();
    CHECK_EQ(app.program.document.selection.faces.size(), 1);
    CHECK_EQ(app.program.document.undo_steps(), 1);
}

TEST(a_press_that_becomes_a_box_takes_its_selection_back_and_is_one_step_with_it) {
    Headless<Modeler> app = modeler("PressThenBox");
    app.key(Key::Digit1);
    // Everything selected, with a step to redo.
    app.key(Key::A).key_with(Modifiers::Alt, Key::A).key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.document.selection.vertices.size(), 8);
    size_t steps = app.program.document.undo_steps();
    Point a = screen(app, {1.0f, 1.0f, 1.0f});
    app.press_point(a);
    CHECK_EQ(app.program.document.selection.vertices.size(), 1);
    CHECK_EQ(std::pair(app.program.document.undo_steps(), app.program.document.redo_name()), std::pair(steps + 1, std::optional<std::string_view>()));
    // Past the threshold, the drag is a box, and the press's step is gone,
    // as if it was never made: the selection and Redo are back.
    app.move_point(a + Point(10.0f, 10.0f));
    CHECK(app.program.editor.selecting_box.has_value());
    CHECK_EQ(app.program.document.selection.vertices.size(), 8);
    CHECK_EQ(std::pair(app.program.document.undo_steps(), app.program.document.redo_name()), std::pair(steps, step("Select none")));
    // The box, which is too small for all eight, selects as it's let go
    // of, as one step, which Undo takes back to before the press.
    app.release();
    CHECK(app.program.document.selection.vertices.size() <= 1);
    CHECK_EQ(std::pair(app.program.document.undo_steps(), app.program.document.undo_name()), std::pair(steps + 1, step("Select box")));
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.document.selection.vertices.size(), 8);
}

TEST(withdrawing_a_step_puts_back_the_oldest_that_it_pushed_out_of_a_full_history) {
    auto make_step = [](Document& document, std::uint32_t index) {
        Snapshot before = document.snapshot();
        document.selection = Selection::from_vertices(document.mesh, {index % 8});
        CHECK(document.commit(std::format("Step {}", index), std::move(before)));
    };
    Document document;
    // 200 steps fill the history, and the next pushes the first out.
    for (std::uint32_t index = 0; index < 200; index++) {
        make_step(document, index);
    }
    CHECK_EQ(document.undo_steps(), 200);
    make_step(document, 200);
    CHECK_EQ(document.undo_steps(), 200);
    // Withdrawn, it's as if it was never made, the first step included.
    std::uint64_t version = document.version;
    CHECK(document.withdraw(version));
    CHECK_EQ(std::tuple(document.undo_steps(), document.undo_name(), document.redo_name()), std::tuple(size_t{200}, step("Step 199"), std::optional<std::string_view>()));
    for (int index = 0; index < 200; index++) {
        CHECK(document.undo().has_value());
    }
    CHECK(document.selection.is_empty());
    // A step can't be withdrawn once something else changed the document.
    make_step(document, 1);
    document.touch();
    version = document.version;
    CHECK(!document.withdraw(version - 1));
    CHECK(!document.withdraw(version), "the touch isn't a step");
}

TEST(shift_adds_to_the_selection_and_takes_away_from_it) {
    Headless<Modeler> app = modeler("ShiftClick");
    app.key(Key::Digit1);
    CHECK_EQ(text(app, "Mode"), "Vertex");
    Point a = screen(app, {1.0f, 1.0f, 1.0f});
    Point b = screen(app, {-1.0f, 1.0f, 1.0f});
    app.click_point(a);
    app.hold(Key::Shift).click_point(b).release_key(Key::Shift);
    CHECK_EQ(app.program.document.selection.vertices.size(), 2);
    app.hold(Key::Shift).click_point(a).release_key(Key::Shift);
    std::vector<std::uint32_t> selection(app.program.document.selection.vertices.begin(), app.program.document.selection.vertices.end());
    CHECK_EQ(selection, std::vector<std::uint32_t>{vertex_at(app, {-1.0f, 1.0f, 1.0f})});
}

TEST(a_drag_selects_a_box_and_hidden_vertices_are_not_picked) {
    Headless<Modeler> app = modeler("Box");
    app.key(Key::Digit1);
    Rect view = app.program.view_;
    app.drag(view.origin() + Point(10.0f, 10.0f), Point(view.right() - 10.0f, view.bottom() - 10.0f), 4);
    CHECK_EQ(app.program.document.selection.vertices.size(), 8);
    CHECK_EQ(app.program.document.undo_name(), step("Select box"));
    // Ctrl takes a box away.
    Point corner = screen(app, {1.0f, 1.0f, 1.0f});
    app.hold(Key::Control);
    app.drag(corner - Point(15.0f, 15.0f), corner + Point(15.0f, 15.0f), 2);
    app.release_key(Key::Control);
    CHECK_EQ(app.program.document.selection.vertices.size(), 7);

    // The corner at the back, which the cube hides, isn't picked by a click
    // on it, until X-ray shows it.
    app.key_with(Modifiers::Alt, Key::A);
    Point back = screen(app, {-1.0f, -1.0f, -1.0f});
    app.click_point(back);
    CHECK(!app.program.document.selection.vertices.contains(vertex_at(app, {-1.0f, -1.0f, -1.0f})));
    app.key_with(Modifiers::Alt, Key::Z);
    CHECK(flag(app, "XRay"));
    app.click_point(back);
    std::vector<std::uint32_t> picked(app.program.document.selection.vertices.begin(), app.program.document.selection.vertices.end());
    CHECK_EQ(picked, std::vector<std::uint32_t>{vertex_at(app, {-1.0f, -1.0f, -1.0f})});
}

TEST(keys_select_all_none_and_the_inverse_in_each_mode) {
    Headless<Modeler> app = modeler("SelectKeys");
    app.key(Key::A);
    CHECK_EQ(app.program.document.selection.faces.size(), 6);
    app.key_with(Modifiers::Alt, Key::A);
    CHECK(app.program.document.selection.is_empty());
    app.click_point(app.program.view_.center());
    app.key_with(Modifiers::Control, Key::I);
    CHECK_EQ(app.program.document.selection.faces.size(), 5);
    // Edge mode keeps the faces' edges, which are all of them, so all the
    // faces are selected, as in Blender.
    app.key(Key::Digit2);
    CHECK_EQ(app.program.document.selection.edges.size(), 12);
    CHECK_EQ(app.program.document.selection.faces.size(), 6);
}

TEST(g_moves_the_selection_along_an_axis_and_esc_takes_it_back) {
    Headless<Modeler> app = modeler("Move");
    app.key(Key::A);
    std::vector<Vector3> before = app.program.document.mesh.positions;
    Point start = app.program.view_.center() + Point(-200.0f, 100.0f);
    app.move_point(start);
    app.key(Key::G);
    app.key(Key::X);
    app.move_point(start + Point(120.0f, 40.0f));
    CHECK_EQ(operation(app, "Kind"), "Move");
    CHECK_EQ(operation(app, "Axis"), "X");
    std::vector<Vector3> moved = app.program.document.mesh.positions;
    for (size_t index = 0; index < std::min(before.size(), moved.size()); index++) {
        Vector3 old = before[index];
        Vector3 now = moved[index];
        CHECK(now[0] > old[0] + 0.1f, "{} {}", testing::describe(old), testing::describe(now));
        CHECK_EQ(std::pair(now[1], now[2]), std::pair(old[1], old[2]));
    }
    // Esc takes it back, without a step of history.
    size_t steps = app.program.document.undo_steps();
    app.key(Key::Escape);
    CHECK_EQ(app.program.document.mesh.positions, before);
    CHECK_EQ(app.program.document.undo_steps(), steps);
    CHECK(!app.program.editor.operation.has_value());

    // A click keeps it, as a step that Ctrl+Z takes back and Ctrl+Y redoes.
    app.key(Key::G);
    app.key(Key::Y);
    app.move_point(start + Point(0.0f, -80.0f));
    app.click_point(start + Point(0.0f, -80.0f));
    std::vector<Vector3> raised = app.program.document.mesh.positions;
    bool all_raised = true;
    for (size_t index = 0; index < std::min(raised.size(), before.size()); index++) {
        all_raised = all_raised && raised[index][1] > before[index][1] && raised[index][0] == before[index][0];
    }
    CHECK(all_raised);
    CHECK_EQ(app.program.document.undo_name(), step("Move"));
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.document.mesh.positions, before);
    app.key_with(Modifiers::Control, Key::Y);
    CHECK_EQ(app.program.document.mesh.positions, raised);
    // The click that kept it didn't select anything.
    CHECK_EQ(app.program.document.selection.faces.size(), 6);
}

TEST(ctrl_snaps_a_rotation_to_15_degrees) {
    Headless<Modeler> app = modeler("Rotate");
    app.key(Key::A);
    Point center = app.program.view_.center();
    app.move_point(center + Point(200.0f, 0.0f));
    app.key(Key::R);
    app.key(Key::Y);
    app.hold(Key::Control);
    // About 40 degrees counterclockwise on screen snaps to 45.
    float angle = mesh_modeler::to_radians(40.0f);
    app.move_point(center + Point(200.0f * std::cos(angle), -200.0f * std::sin(angle)));
    CHECK_EQ(operation(app, "Amount"), "45.0°");
    app.key(Key::Enter);
    app.release_key(Key::Control);
    CHECK_EQ(app.program.document.undo_name(), step("Rotate"));
    // Turned about Y, the corners stay as far from the axis and as high.
    for (Vector3 position : app.program.document.mesh.positions) {
        float radius = std::sqrt(position[0] * position[0] + position[2] * position[2]);
        CHECK(std::abs(radius - std::sqrt(2.0f)) < 1e-4f, "{}", testing::describe(position));
        CHECK(std::abs(std::abs(position[1]) - 1.0f) < 1e-5f);
    }
}

TEST(e_extrudes_the_selected_face_along_its_normal) {
    Headless<Modeler> app = modeler("Extrude");
    Point center = app.program.view_.center();
    app.click_point(center);
    size_t face = *app.program.document.selection.faces.begin();
    CHECK(app.program.document.mesh.face_normal(face)[2] > 0.99f);
    app.key(Key::E);
    CHECK_EQ(operation(app, "Axis"), "Normal");
    // Out along +Z, which is toward the pointer's lower left on screen.
    Point tip = screen(app, {0.0f, 0.0f, 2.0f});
    Point foot = screen(app, {0.0f, 0.0f, 1.0f});
    app.move_point(center + (tip - foot));
    app.click_point(center + (tip - foot));
    const Mesh& mesh = app.program.document.mesh;
    CHECK_EQ(std::pair(mesh.positions.size(), mesh.faces.size()), (std::pair<size_t, size_t>(12, 10)));
    CHECK(mesh.validate().has_value());
    std::vector<size_t> selected(app.program.document.selection.faces.begin(), app.program.document.selection.faces.end());
    CHECK_EQ(selected.size(), 1);
    Vector3 moved = mesh.face_center(selected[0]);
    CHECK(std::abs(moved[2] - 2.0f) < 0.05f, "{}", testing::describe(moved));
    CHECK_EQ(app.program.document.undo_name(), step("Extrude"));
    // Undo takes back the extrusion with its move, in one step.
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.document.mesh.faces.size(), 6);
}

TEST(a_right_click_cancels_an_extrusion_whole) {
    Headless<Modeler> app = modeler("CancelExtrude");
    app.click_point(app.program.view_.center());
    Mesh before = app.program.document.mesh;
    app.key(Key::E);
    app.move_point(app.program.view_.center() + Point(-60.0f, 60.0f));
    CHECK_EQ(app.program.document.mesh.faces.size(), 10);
    Point pointer = app.pointer();
    app.press_button(pointer, MouseButton::Right).release_button(MouseButton::Right);
    CHECK_EQ(app.program.document.mesh, before);
    CHECK(!app.program.editor.operation.has_value());
    // The right-click that cancelled didn't open the view's menu.
    CHECK(!text(app, "Prompt").has_value());
}

TEST(i_insets_the_selected_faces) {
    Headless<Modeler> app = modeler("Inset");
    Point center = app.program.view_.center();
    app.click_point(center);
    app.move_point(center + Point(150.0f, 0.0f));
    app.key(Key::I);
    app.move_point(center + Point(100.0f, 0.0f));
    app.key(Key::Enter);
    const Mesh& mesh = app.program.document.mesh;
    CHECK_EQ(mesh.faces.size(), 10);
    CHECK(mesh.validate().has_value());
    CHECK_EQ(app.program.document.undo_name(), step("Inset"));
    size_t face = *app.program.document.selection.faces.begin();
    CHECK(mesh.face_area(face) < 4.0f && mesh.face_area(face) > 0.5f, "{}", mesh.face_area(face));
}

TEST(ctrl_r_cuts_the_loop_under_the_pointer) {
    Headless<Modeler> app = modeler("LoopCut");
    app.key_with(Modifiers::Control, Key::R);
    CHECK_EQ(text(app, "Tool"), "Loop cut");
    // The vertical edge at the front right: its middle.
    Point middle = screen(app, {1.0f, 0.0f, 1.0f});
    app.move_point(middle);
    CHECK_EQ(number(app, {"LoopPreviewEdges"}), 4.0);
    // It cuts as the button goes down.
    app.press_point(middle);
    const Mesh& mesh = app.program.document.mesh;
    CHECK_EQ(std::pair(mesh.positions.size(), mesh.faces.size()), (std::pair<size_t, size_t>(12, 10)));
    CHECK(mesh.validate().has_value());
    // The cut goes around the cube's middle, and is selected, as edges.
    CHECK_EQ(app.program.document.selection.edges.size(), 4);
    CHECK(std::ranges::all_of(app.program.document.selection.vertices, [&](std::uint32_t vertex) { return std::abs(mesh.position(vertex)[1]) < 1e-5f; }));
    CHECK_EQ(text(app, "Mode"), "Edge");
    app.release();
    // Esc goes back to the Select tool.
    app.key(Key::Escape);
    CHECK_EQ(text(app, "Tool"), "Select");
}

TEST(alt_click_selects_an_edge_loop) {
    Headless<Modeler> app = modeler("AltLoop");
    app.key_with(Modifiers::Control, Key::R);
    app.click_point(screen(app, {1.0f, 0.0f, 1.0f}));
    app.key(Key::Escape);
    app.key(Key::Digit2);
    app.key_with(Modifiers::Alt, Key::A);
    // Alt and a click on a horizontal edge of the middle loop selects it.
    Point point = screen(app, {1.0f, 0.0f, 0.0f});
    app.hold(Key::Alt).click_point(point).release_key(Key::Alt);
    const Selection& selection = app.program.document.selection;
    CHECK_EQ(selection.edges.size(), 4, "{}", testing::describe(selection.edges));
    CHECK_EQ(app.program.document.undo_name(), step("Select loop"));
}

TEST(the_paint_tool_paints_the_faces_it_drags_over) {
    Headless<Modeler> app = modeler("PaintTool");
    app.click("Paints/1");
    CHECK_EQ(number(app, {"ActivePaint"}), 1.0);
    app.key(Key::P);
    Point front = screen(app, {0.0f, 0.0f, 1.0f});
    Point right = screen(app, {1.0f, 0.0f, 0.0f});
    app.drag(front, right, 6);
    std::vector<size_t> painted;
    for (size_t face = 0; face < 6; face++) {
        if (app.program.document.mesh.faces[face].material == 1) {
            painted.push_back(face);
        }
    }
    CHECK_EQ(painted.size(), 2, "{}", testing::describe(painted));
    CHECK_EQ(app.program.document.undo_name(), step("Paint"));
    // The stroke is one step.
    app.key_with(Modifiers::Control, Key::Z);
    CHECK(std::ranges::all_of(app.program.document.mesh.faces, [](const polygon_mesh::Face& face) { return face.material == 0; }));
}

TEST(the_paint_button_paints_the_selected_faces_with_the_paint_picked) {
    Headless<Modeler> app = modeler("PaintButton");
    app.key(Key::A);
    CHECK(!app.is_enabled("Remove paint") || app.program.document.paints.size() > 1);
    app.click("Paints/3");
    app.click("Paint selected faces");
    CHECK(std::ranges::all_of(app.program.document.mesh.faces, [](const polygon_mesh::Face& face) { return face.material == 3; }));
    // Removing the paint repaints its faces with the first.
    app.click("Remove paint");
    CHECK_EQ(app.program.document.paints.size(), 3);
    CHECK(std::ranges::all_of(app.program.document.mesh.faces, [](const polygon_mesh::Face& face) { return face.material == 0; }));
    // Adding one picks it.
    app.click("Add paint");
    CHECK_EQ(app.program.document.paints.size(), 4);
    CHECK_EQ(app.program.editor.active_paint, 3);
}

// An Undo or a Redo that takes the paint picked away picks one that's there
// as it does, not at the frame's end: the next message of its frame may
// paint with the paint picked. The test is the two messages in one frame.
TEST(an_undo_or_a_redo_that_takes_the_paint_picked_away_picks_one_that_s_there_for_its_frame_s_next_message) {
    Headless<Modeler> app = modeler("UndoThenPaint");
    auto paints_are_there = [&] {
        size_t count = app.program.document.paints.size();
        return std::ranges::all_of(app.program.document.mesh.faces, [&](const polygon_mesh::Face& face) { return face.material < count; });
    };
    app.key(Key::A);
    app.click("Add paint");
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{5}, size_t{4}));
    // Undo takes the paint that was added away, and the faces are painted
    // with the last that's left.
    app.queue_message(Message::Command{Command::Undo{}});
    app.queue_message(Message::Command{Command::AssignPaint{}});
    app.frame();
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{4}, size_t{3}));
    CHECK(paints_are_there());
    CHECK(std::ranges::all_of(app.program.document.mesh.faces, [](const polygon_mesh::Face& face) { return face.material == 3; }));

    // Redo, of the last paint's removal, with that paint picked again.
    app.click("Remove paint");
    app.key_with(Modifiers::Control, Key::Z);
    app.click("Paints/3");
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{4}, size_t{3}));
    app.queue_message(Message::Command{Command::Redo{}});
    app.queue_message(Message::Command{Command::AssignPaint{}});
    app.frame();
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{3}, size_t{2}));
    CHECK(paints_are_there());
}

// The same, with an edit of the paint picked after the Undo: it edits a
// paint that's there, where it would write past the paints.
TEST(an_edit_of_the_paint_picked_after_an_undo_in_its_frame_edits_a_paint_that_s_there) {
    Headless<Modeler> app = modeler("UndoThenRename");
    app.click("Add paint");
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{5}, size_t{4}));
    app.queue_message(Message::Command{Command::Undo{}});
    app.queue_message(Message::Paint{PaintEdit::Name{"Renamed"}});
    app.frame();
    CHECK_EQ(std::pair(app.program.document.paints.size(), app.program.editor.active_paint), std::pair(size_t{4}, size_t{3}));
    CHECK_EQ(app.program.document.paints[3].name, "Renamed");
}

TEST(a_paint_s_color_and_settings_change_in_one_step_each) {
    Headless<Modeler> app = modeler("PaintSettings");
    size_t steps = app.program.document.undo_steps();
    Rect rail = app.target("Paint/Shift");
    app.drag(Point(rail.x + 20.0f, rail.bottom() - 10.0f), Point(rail.center().x, rail.bottom() - 10.0f), 4);
    app.frames(2);
    CHECK(app.program.document.paints[0].parameters.shift > 0.1);
    CHECK_EQ(app.program.document.undo_steps(), steps + 1);
    CHECK_EQ(app.program.document.undo_name(), step("Edit paint"));
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.document.paints[0].parameters.shift, 0.0);
}

TEST(shift_e_creases_the_selected_edges_and_the_surface_keeps_them) {
    Headless<Modeler> app = modeler("Crease");
    app.key(Key::Digit2);
    app.click_point(screen(app, {1.0f, 0.0f, 1.0f}));
    CHECK_EQ(app.program.document.selection.edges.size(), 1);
    Edge edge = *app.program.document.selection.edges.begin();
    Edge expected(vertex_at(app, {1.0f, 1.0f, 1.0f}), vertex_at(app, {1.0f, -1.0f, 1.0f}));
    CHECK_EQ(edge, expected);
    app.key_with(Modifiers::Shift, Key::E);
    CHECK_EQ(app.program.document.mesh.crease(edge), 1.0f);
    app.key_with(Modifiers::Alt, Key::E);
    CHECK_EQ(app.program.document.mesh.crease(edge), 0.0f);
}

TEST(ctrl_and_a_digit_subdivide_the_view_s_surface) {
    Headless<Modeler> app = modeler("Subdivision");
    app.key_with(Modifiers::Control, Key::Digit2);
    CHECK_EQ(number(app, {"Subdivision"}), 2.0);
    CHECK_EQ(number(app, {"SurfaceFaces"}), 96.0);
    // Faces are still picked on the surface, as the cage's.
    app.click_point(app.program.view_.center());
    CHECK_EQ(app.program.document.selection.faces.size(), 1);
    // Subdivision is how the view shows the model, not a change to it.
    CHECK_EQ(app.program.document.mesh.faces.size(), 6);
    app.key_with(Modifiers::Control, Key::Digit0);
    CHECK_EQ(number(app, {"SurfaceFaces"}), 6.0);
}

TEST(delete_and_merge_change_the_mesh_and_say_what_s_missing) {
    Headless<Modeler> app = modeler("Delete");
    // Nothing selected: Delete says why.
    app.key(Key::X);
    CHECK(text(app, "EditorMessage").value().contains("selection"));
    app.click_point(app.program.view_.center());
    app.key(Key::X);
    CHECK_EQ(app.program.document.mesh.faces.size(), 5);
    CHECK(!text(app, "EditorMessage").has_value());
    // Merging the open side's corners closes it to a point.
    app.key(Key::Digit1);
    app.key(Key::A);
    app.key(Key::M);
    CHECK_EQ(app.program.document.mesh.positions.size(), 1);
}

TEST(the_middle_button_orbits_and_the_wheel_zooms) {
    Headless<Modeler> app = modeler("Camera");
    Point center = app.program.view_.center();
    float yaw = app.program.camera.yaw;
    app.drag_button(center, center + Point(100.0f, 0.0f), 4, MouseButton::Middle);
    CHECK(app.program.camera.yaw < yaw);
    // The middle button doesn't select.
    CHECK(app.program.document.selection.is_empty());
    float height = app.program.camera.height;
    app.scroll_point(center, 2.0f);
    CHECK(app.program.camera.height < height);
    app.key(Key::Home);
    CHECK(app.program.camera.height > 3.0f);
}

TEST(menus_run_commands) {
    Headless<Modeler> app = modeler("Menus");
    app.click("Menu/Mesh");
    app.move_to("Menu/Mesh/Menu/Add");
    app.click("Menu/Mesh/Menu/Add/Menu/Torus");
    CHECK_EQ(app.program.document.mesh.faces.size(), 6 + 24 * 12);
    CHECK_EQ(app.program.document.selection.faces.size(), 24 * 12);
    CHECK_EQ(app.program.document.undo_name(), step("Add Torus"));
    app.click("Menu/View");
    app.click("Menu/View/Menu/Flat shader");
    CHECK_EQ(text(app, "Shader"), "Flat");
}

TEST(f10_and_the_view_menu_switch_vertical_sync_once) {
    Headless<Modeler> app = modeler("VerticalSync");
    // The library switches it on F10, and the modeler's keys leave F10 to
    // it, so one press switches it once, from the on that it starts as.
    CHECK(app.vertical_sync());
    app.key(Key::F10);
    CHECK(!app.vertical_sync());
    app.click("Menu/View");
    app.click("Menu/View/Menu/VSync");
    CHECK(app.vertical_sync());
}

TEST(f9_and_the_view_menu_switch_the_theme_once) {
    Headless<Modeler> app = modeler("Theme");
    // The library switches it on F9, and the modeler's keys leave F9 to it,
    // so one press switches it once, from the light that it starts as.
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    app.key(Key::F9);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
    app.click("Menu/View");
    app.click("Menu/View/Menu/Dark theme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
}

TEST(the_cards_fit_at_both_window_heights) {
    for (auto [height, fullscreen] : {std::pair(tokens::MaximizedHeight, false), std::pair(tokens::FullscreenHeight, true)}) {
        Headless<Modeler> app = modeler(fullscreen ? "Fullscreen" : "Maximized");
        app.set_fullscreen(fullscreen).frames(2);
        CHECK_EQ(app.size().height, height);
        float bottom = height - tokens::StatusBarHeight - tokens::PagePadding;
        for (std::string_view name : {"Paint/Light", "Clear crease", "AddPrimitive", "Grid"}) {
            CHECK(app.target(name).bottom() <= bottom, "{} at {} is below {}", name, testing::describe(app.target(name)), bottom);
        }
        Rect view = app.target("View");
        CHECK(view.bottom() <= bottom && view.bottom() > bottom - 40.0f, "{}", testing::describe(view));
        CHECK(view.right() < tokens::WindowWidth - 400.0f);
    }
}

TEST(closing_with_changes_asks_first) {
    Headless<Modeler> app = modeler("Close");
    CHECK(app.program.may_close());
    app.key(Key::A);
    CHECK(!app.program.may_close());
    app.frames(2);
    CHECK_EQ(text(app, "Prompt"), "Unsaved");
    app.click("Unsaved/Close");
    CHECK(!app.closed());
    CHECK(!app.program.may_close());
    app.frames(2);
    app.click("Unsaved/Secondary");
    CHECK(app.closed());
}

// How long a frame takes with the pointer over a subdivided mesh, where the
// view picks what's under it: a benchmark, which only reports.
TEST(benchmark_hover_over_a_subdivided_mesh) {
    Headless<Modeler> app = modeler("HoverBenchmark");
    app.program.levels = 4;
    app.frames(2);
    CHECK(app.program.scene.triangles.indices.size() >= 6 * 256 * 2 * 3, "{} indices", app.program.scene.triangles.indices.size());
    app.move_to("View");
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    app.frames(30);
    double per_frame = std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count() * 1000.0 / 30.0;
    std::println(stderr, "A frame with the pointer over the subdivided cube takes {:.3f} ms to build", per_frame);
    CHECK(app.program.editor.hover.has_value() && base::is<Element::Face>(*app.program.editor.hover), "{}", testing::describe(app.program.editor.hover));
}
