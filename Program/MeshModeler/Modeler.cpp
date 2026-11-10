#include "MeshModeler/Modeler.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Vec.h"
#include "MeshModeler/Angles.h"
#include "MeshModeler/Cards.h"
#include "MeshModeler/Interaction.h"
#include "MeshModeler/Overlay.h"
#include "MeshModeler/Paint.h"
#include "SimplePaint/Orthographic.h"

#include <algorithm>
#include <cmath>
#include <format>

namespace mesh_modeler::modeler {

using camera::Camera;
using camera::Viewpoint;
using document::Document;
using document::Snapshot;
using document::Untitled;
using document::valid_name;
using editing::Axis;
using editing::Editor;
using editing::Element;
using editing::Mode;
using editing::OperationKind;
using editing::Primitive;
using editing::select_mode;
using editing::Tool;
using math::Vector3;
using paint::MaximumPaints;
using paint::next_color;
using paint::Paint;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using polygon_mesh::Shading;
using user_interface::Color;
using user_interface::Context;
using user_interface::DialogResult;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Severity;
using user_interface::Startup;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;
using viewport::OverlayItem;
using viewport::Shader;

namespace {

// The light of the flat shader, toward the camera's upper left, and how much
// of a paint's color is ambient.
constexpr std::array<float, 4> Light{-0.37f, 0.56f, 0.74f, 0.35f};

// Each command's label, which menus show and return, and its shortcuts, of
// which menus and tooltips show the first.
constexpr CommandEntry Table[] = {
    {Command::New{}, "New", {"Ctrl+N"}},
    {Command::Open{}, "Open", {"Ctrl+O"}},
    {Command::Save{}, "Save", {"Ctrl+S"}},
    {Command::SaveAs{}, "Save as", {"Ctrl+Shift+S"}},
    {Command::ExportObj{}, "Export OBJ", {"Ctrl+E"}},
    {Command::Undo{}, "Undo", {"Ctrl+Z"}},
    {Command::Redo{}, "Redo", {"Ctrl+Y", "Ctrl+Shift+Z"}},
    {Command::SelectAll{}, "Select all", {"A", "Ctrl+A"}},
    {Command::SelectNone{}, "Select none", {"Alt+A"}},
    {Command::InvertSelection{}, "Invert selection", {"Ctrl+I"}},
    {Command::VertexMode{}, "Vertices", {"1"}},
    {Command::EdgeMode{}, "Edges", {"2"}},
    {Command::FaceMode{}, "Faces", {"3"}},
    {Command::SelectTool{}, "Select tool", {"Q"}},
    {Command::PaintTool{}, "Paint tool", {"P"}},
    {Command::LoopCutTool{}, "Loop cut tool", {"Ctrl+R"}},
    {Command::Move{}, "Move", {"G"}},
    {Command::Rotate{}, "Rotate", {"R"}},
    {Command::Scale{}, "Scale", {"S"}},
    {Command::Extrude{}, "Extrude", {"E"}},
    {Command::Inset{}, "Inset", {"I"}},
    {Command::Fill{}, "Fill", {"F"}},
    {Command::Merge{}, "Merge at center", {"M"}},
    {Command::Subdivide{}, "Subdivide", {"Ctrl+D"}},
    {Command::Delete{}, "Delete", {"X", "Delete"}},
    {Command::DeleteFaces{}, "Delete only faces", {"Shift+X"}},
    {Command::FlipNormals{}, "Flip normals", {"Alt+N"}},
    {Command::RecalculateNormals{}, "Recalculate normals", {"Shift+N"}},
    {Command::Crease{}, "Crease edges", {"Shift+E"}},
    {Command::ClearCrease{}, "Clear crease", {"Alt+E"}},
    {Command::AssignPaint{}, "Paint selected faces", {"Ctrl+P"}},
    {Command::AddPaint{}, "Add paint", {}},
    {Command::RemovePaint{}, "Remove paint", {}},
    {Command::Add{Primitive::Cube}, "Cube", {}},
    {Command::Add{Primitive::Plane}, "Plane", {}},
    {Command::Add{Primitive::Cylinder}, "Cylinder", {}},
    {Command::Add{Primitive::Cone}, "Cone", {}},
    {Command::Add{Primitive::Sphere}, "UV sphere", {}},
    {Command::Add{Primitive::Torus}, "Torus", {}},
    {Command::FrameAll{}, "Frame all", {"Home"}},
    {Command::FrameSelected{}, "Frame selected", {"Period"}},
    {Command::View{Viewpoint::Front}, "Front", {"Numpad1"}},
    {Command::View{Viewpoint::Back}, "Back", {"Ctrl+Numpad1"}},
    {Command::View{Viewpoint::Right}, "Right", {"Numpad3"}},
    {Command::View{Viewpoint::Left}, "Left", {"Ctrl+Numpad3"}},
    {Command::View{Viewpoint::Top}, "Top", {"Numpad7"}},
    {Command::View{Viewpoint::Bottom}, "Bottom", {"Ctrl+Numpad7"}},
    {Command::Level{0}, "Off", {"Ctrl+0"}},
    {Command::Level{1}, "1 level", {"Ctrl+1"}},
    {Command::Level{2}, "2 levels", {"Ctrl+2"}},
    {Command::Level{3}, "3 levels", {"Ctrl+3"}},
    {Command::Level{4}, "4 levels", {"Ctrl+4"}},
    {Command::ToggleShader{}, "Flat shader", {"Shift+Z"}},
    {Command::ToggleSmooth{}, "Smooth shading", {"Shift+S"}},
    {Command::ToggleXray{}, "X-ray", {"Alt+Z"}},
    {Command::ToggleGrid{}, "Grid", {"Shift+G"}},
    {Command::ToggleTheme{}, "Dark theme", {"F9"}},
    {Command::VerticalSync{}, "VSync", {"F10"}},
    {Command::Fullscreen{}, "Fullscreen", {"F11"}},
};

std::pair<std::string, json::Value> member(std::string_view name, json::Value value) {
    return {std::string(name), std::move(value)};
}

json::Value object(std::vector<std::pair<std::string, json::Value>> members) {
    return json::Value::Object{std::move(members)};
}

} // namespace

const std::span<const CommandEntry> Commands = Table;

std::string_view Command::label() const {
    for (const CommandEntry& entry : Commands) {
        if (entry.command == *this) {
            return entry.label;
        }
    }
    return "";
}

std::optional<std::string_view> Command::shortcut() const {
    for (const CommandEntry& entry : Commands) {
        if (entry.command == *this) {
            if (entry.shortcuts.empty()) {
                return std::nullopt;
            }
            return *entry.shortcuts.begin();
        }
    }
    return std::nullopt;
}

std::optional<Command> Command::from_label(std::string_view label) {
    for (const CommandEntry& entry : Commands) {
        if (entry.label == label) {
            return entry.command;
        }
    }
    return std::nullopt;
}

std::string_view Prompt::name() const {
    return base::match(
        *this, [](const Prompt::SaveAs&) -> std::string_view { return "SaveAs"; }, [](const Prompt::Open&) -> std::string_view { return "OpenModel"; },
        [](const Prompt::Unsaved&) -> std::string_view { return "Unsaved"; });
}

Modeler::Modeler(std::filesystem::path folder) : folder(std::move(folder)) {}

Modeler Modeler::start(Startup& startup) {
    return Modeler(startup.data_folder / "Models");
}

const polygon_mesh::SubdivisionSurface* Modeler::surface() const {
    return surface_ ? &*surface_ : nullptr;
}

void Modeler::say(Severity severity, std::string text) {
    message = std::pair(severity, std::move(text));
}

void Modeler::run(Context<Message>& ui, Command command) {
    // A command ends a modal edit, keeping it, as a click would.
    if (editor.operation && !(base::is<Command::Undo>(command) || base::is<Command::Redo>(command))) {
        editor.confirm(document);
    }
    if (editor.operation) {
        editor.cancel(document);
    }
    // A held paint stroke commits as its own step before any command,
    // Undo and Redo included, so the history never interleaves.
    editor.end_paint(document);
    end_paint_edit();
    Point pointer = ui.input().pointer.value_or(view_.center());
    base::match(
        command, [&](const Command::New&) { ask_first(Pending::New); }, [&](const Command::Open&) { ask_first(Pending::Open); }, [&](const Command::Save&) { save(); },
        [&](const Command::SaveAs&) { prompt_save_as(std::nullopt); },
        [&](const Command::ExportObj&) {
            std::expected<std::filesystem::path, std::string> path = document.export_obj(folder);
            if (path) {
                say(Severity::Success, std::format("Exported {}", base::display(*path)));
            } else {
                say(Severity::Error, std::move(path.error()));
            }
        },
        [&](const Command::Undo&) {
            if (std::optional<std::string> step = document.undo()) {
                say(Severity::Informational, std::format("Undid {}", *step));
            } else {
                say(Severity::Informational, "There's nothing to undo");
            }
        },
        [&](const Command::Redo&) {
            if (std::optional<std::string> step = document.redo()) {
                say(Severity::Informational, std::format("Redid {}", *step));
            } else {
                say(Severity::Informational, "There's nothing to redo");
            }
        },
        [&](const Command::SelectAll&) {
            Snapshot before = document.snapshot();
            document.selection = Selection::all(document.mesh).in_mode(document.mesh, select_mode(editor.mode));
            document.commit("Select all", std::move(before));
        },
        [&](const Command::SelectNone&) {
            Snapshot before = document.snapshot();
            document.selection = Selection();
            document.commit("Select none", std::move(before));
        },
        [&](const Command::InvertSelection&) {
            Snapshot before = document.snapshot();
            document.selection = document.selection.inverted(document.mesh, select_mode(editor.mode));
            document.commit("Invert selection", std::move(before));
        },
        [&](const Command::VertexMode&) { editor.set_mode(document, Mode::Vertices); }, [&](const Command::EdgeMode&) { editor.set_mode(document, Mode::Edges); },
        [&](const Command::FaceMode&) { editor.set_mode(document, Mode::Faces); }, [&](const Command::SelectTool&) { editor.tool = Tool::Select; },
        [&](const Command::PaintTool&) { editor.tool = Tool::Paint; }, [&](const Command::LoopCutTool&) { editor.tool = Tool::LoopCut; },
        [&](const Command::Move&) { editor.begin(document, OperationKind::Move, pointer, camera, view_); },
        [&](const Command::Rotate&) { editor.begin(document, OperationKind::Rotate, pointer, camera, view_); },
        [&](const Command::Scale&) { editor.begin(document, OperationKind::Scale, pointer, camera, view_); },
        [&](const Command::Extrude&) { editor.extrude(document, pointer, camera, view_); }, [&](const Command::Inset&) { editor.inset(document, pointer, camera, view_); },
        [&](const Command::Fill&) { editor.fill(document); }, [&](const Command::Merge&) { editor.merge(document); },
        [&](const Command::Subdivide&) { editor.subdivide(document); }, [&](const Command::Delete&) { editor.delete_(document); },
        [&](const Command::DeleteFaces&) { editor.delete_only_faces(document); }, [&](const Command::FlipNormals&) { editor.flip(document); },
        [&](const Command::RecalculateNormals&) { editor.recalculate_normals(document); }, [&](const Command::Crease&) { editor.crease(document, crease); },
        [&](const Command::ClearCrease&) { editor.crease(document, 0.0f); }, [&](const Command::AssignPaint&) { editor.assign_paint(document); },
        [&](const Command::AddPaint&) {
            Snapshot before = document.snapshot();
            size_t count = document.paints.size();
            Paint added(std::format("Paint {}", count + 1), next_color(count));
            if (std::optional<size_t> index = document.add_paint(std::move(added))) {
                editor.active_paint = *index;
                document.commit("Add paint", std::move(before));
            } else {
                say(Severity::Warning, std::format("A model has at most {} paints", MaximumPaints));
            }
        },
        [&](const Command::RemovePaint&) {
            Snapshot before = document.snapshot();
            size_t index = editor.active_paint;
            if (document.remove_paint(index)) {
                editor.active_paint = std::min(index, document.paints.size() - 1);
                document.commit("Remove paint", std::move(before));
            } else {
                say(Severity::Warning, "A model needs a paint");
            }
        },
        [&](const Command::Add& add) { editor.add_primitive(document, add.primitive); },
        [&](const Command::FrameAll&) {
            if (std::optional<std::array<Vector3, 2>> bounds = document.mesh.bounds()) {
                camera.frame((*bounds)[0], (*bounds)[1]);
            }
        },
        [&](const Command::FrameSelected&) {
            const Mesh& mesh = document.mesh;
            std::vector<Vector3> points;
            for (std::uint32_t vertex : document.selection.vertices) {
                if (vertex < mesh.positions.size()) {
                    points.push_back(mesh.position(vertex));
                }
            }
            if (!points.empty()) {
                Vector3 minimum = points[0];
                Vector3 maximum = points[0];
                for (const Vector3& point : points) {
                    for (size_t axis = 0; axis < 3; axis++) {
                        minimum[axis] = std::fmin(minimum[axis], point[axis]);
                        maximum[axis] = std::fmax(maximum[axis], point[axis]);
                    }
                }
                camera.frame(minimum, maximum);
            } else {
                say(Severity::Informational, "Select something to frame it");
            }
        },
        [&](const Command::View& look) { camera.look_from(look.viewpoint); }, [&](const Command::Level& level) { levels = std::min(level.levels, MaximumLevels); },
        [&](const Command::ToggleShader&) { shader = shader == Shader::SimplePaint ? Shader::Flat : Shader::SimplePaint; }, [&](const Command::ToggleSmooth&) { smooth = !smooth; },
        [&](const Command::ToggleXray&) { xray = !xray; }, [&](const Command::ToggleGrid&) { grid = !grid; }, [&](const Command::ToggleTheme&) { ui.toggle_theme(); },
        [&](const Command::VerticalSync&) { ui.toggle_vertical_sync(); }, [&](const Command::Fullscreen&) { ui.toggle_fullscreen(); });
}

bool Modeler::can(Command command) const {
    const Selection& selection = document.selection;
    if (base::is<Command::Undo>(command)) {
        return document.undo_name().has_value();
    }
    if (base::is<Command::Redo>(command)) {
        return document.redo_name().has_value();
    }
    if (base::is<Command::Move>(command) || base::is<Command::Rotate>(command) || base::is<Command::Scale>(command) || base::is<Command::FrameSelected>(command)) {
        return !selection.vertices.empty();
    }
    if (base::is<Command::Extrude>(command)) {
        return !selection.faces.empty() || !selection.edges.empty();
    }
    if (base::is<Command::Inset>(command) || base::is<Command::Subdivide>(command) || base::is<Command::FlipNormals>(command) || base::is<Command::DeleteFaces>(command) ||
        base::is<Command::AssignPaint>(command)) {
        return !selection.faces.empty();
    }
    if (base::is<Command::Fill>(command)) {
        return selection.vertices.size() >= 2;
    }
    if (base::is<Command::Merge>(command)) {
        return selection.vertices.size() >= 2;
    }
    if (base::is<Command::Delete>(command) || base::is<Command::SelectNone>(command)) {
        return !selection.is_empty();
    }
    if (base::is<Command::Crease>(command) || base::is<Command::ClearCrease>(command)) {
        return !selection.edges.empty();
    }
    if (base::is<Command::AddPaint>(command)) {
        return document.paints.size() < MaximumPaints;
    }
    if (base::is<Command::RemovePaint>(command)) {
        return document.paints.size() > 1;
    }
    return true;
}

std::optional<bool> Modeler::checked(Command command) const {
    if (base::is<Command::ToggleShader>(command)) {
        return shader == Shader::Flat;
    }
    if (base::is<Command::ToggleSmooth>(command)) {
        return smooth;
    }
    if (base::is<Command::ToggleXray>(command)) {
        return xray;
    }
    if (base::is<Command::ToggleGrid>(command)) {
        return grid;
    }
    if (const Command::Level* level = base::as<Command::Level>(command)) {
        return levels == level->levels;
    }
    if (base::is<Command::VertexMode>(command)) {
        return editor.mode == Mode::Vertices;
    }
    if (base::is<Command::EdgeMode>(command)) {
        return editor.mode == Mode::Edges;
    }
    if (base::is<Command::FaceMode>(command)) {
        return editor.mode == Mode::Faces;
    }
    if (base::is<Command::SelectTool>(command)) {
        return editor.tool == Tool::Select;
    }
    if (base::is<Command::PaintTool>(command)) {
        return editor.tool == Tool::Paint;
    }
    if (base::is<Command::LoopCutTool>(command)) {
        return editor.tool == Tool::LoopCut;
    }
    return std::nullopt;
}

void Modeler::ask_first(Pending pending) {
    if (document.dirty) {
        prompt = Prompt::Unsaved{pending};
    } else {
        proceed(pending);
    }
}

void Modeler::proceed(Pending pending) {
    switch (pending) {
    case Pending::New:
        document = Document();
        editor.reset();
        camera = Camera();
        prompt = std::nullopt;
        say(Severity::Informational, "Started a new model");
        break;
    case Pending::Open: {
        saved_names = mesh_modeler::document::saved_names(folder);
        auto found = std::ranges::find(saved_names, document.name);
        if (found != saved_names.end()) {
            open_choice = static_cast<size_t>(found - saved_names.begin());
        } else if (!saved_names.empty()) {
            open_choice = 0;
        } else {
            open_choice = std::nullopt;
        }
        prompt = Prompt::Open{};
        break;
    }
    case Pending::Close:
        prompt = std::nullopt;
        break;
    }
}

void Modeler::prompt_save_as(std::optional<Pending> after) {
    name_draft = document.name;
    after_save = after;
    prompt = Prompt::SaveAs{};
}

bool Modeler::save() {
    if (document.name == Untitled) {
        prompt_save_as(std::nullopt);
        return false;
    }
    std::expected<std::filesystem::path, std::string> path = document.save(folder);
    if (path) {
        say(Severity::Success, std::format("Saved {}", base::display(*path)));
        return true;
    }
    say(Severity::Error, std::move(path.error()));
    return false;
}

std::expected<std::optional<Pending>, std::string> Modeler::save_as() {
    std::string name(base::trim(name_draft));
    if (!valid_name(name)) {
        return std::unexpected("A name can't be empty, end with a period, or have any of < > : \" / \\ | ? *.");
    }
    std::string previous = std::exchange(document.name, std::move(name));
    std::expected<std::filesystem::path, std::string> path = document.save(folder);
    if (path) {
        prompt = std::nullopt;
        say(Severity::Success, std::format("Saved {}", base::display(*path)));
        return std::exchange(after_save, std::nullopt);
    }
    document.name = std::move(previous);
    return std::unexpected(std::move(path.error()));
}

void Modeler::open_chosen() {
    if (!open_choice || *open_choice >= saved_names.size()) {
        return;
    }
    std::string name = saved_names[*open_choice];
    std::expected<Document, std::string> opened = Document::open(folder, name);
    if (opened) {
        document = std::move(*opened);
        editor.reset();
        if (std::optional<std::array<Vector3, 2>> bounds = document.mesh.bounds()) {
            camera.frame((*bounds)[0], (*bounds)[1]);
        }
        prompt = std::nullopt;
        say(Severity::Success, std::format("Opened {}", name));
    } else {
        say(Severity::Error, std::move(opened.error()));
    }
}

void Modeler::begin_paint_edit() {
    if (!paint_edit) {
        paint_edit = document.snapshot();
    }
}

void Modeler::end_paint_edit() {
    if (paint_edit) {
        Snapshot before = std::move(*paint_edit);
        paint_edit = std::nullopt;
        document.commit("Edit paint", std::move(before));
    }
}

void Modeler::prepare_scene(const Theme& theme) {
    Rect view = view_;
    std::tuple<std::uint64_t, std::uint32_t, bool> key(document.version, levels, smooth);
    if (built != key) {
        Shading shading = smooth ? Shading(Shading::Smooth{to_radians(SmoothAngleDegrees)}) : Shading(Shading::Flat{});
        if (levels > 0) {
            surface_ = document.mesh.catmull_clark(levels);
        } else {
            surface_ = std::nullopt;
        }
        scene.triangles = surface_ ? surface_->mesh.triangulate(shading) : document.mesh.triangulate(shading);
        scene.version += 1;
        built = key;
    }
    update_preview();
    Color background = theme.card_opaque();
    scene.view = view;
    scene.background = {background.r, background.g, background.b, 1.0f};
    scene.shader = shader;
    if (view.width >= 1.0f && view.height >= 1.0f) {
        scene.world_to_clip = camera.world_to_clip(view);
        if (std::expected<simple_paint::orthographic::ObjectTransforms, simple_paint::Error> transforms =
                simple_paint::orthographic::build_object_transforms(camera.view(), camera.projection(view))) {
            scene.transforms = *transforms;
        }
    }
    const std::vector<Paint>& paints = document.paints;
    for (size_t index = 0; index < MaximumPaints; index++) {
        const Paint& paint = index < paints.size() ? paints[index] : paints[0];
        scene.materials[index] = paint.material();
        scene.colors[index] = paint.linear_color();
    }
    scene.light = Light;
    overlay::Look look{
        .theme = theme,
        .surface = surface(),
        .triangles = scene.triangles,
        .grid = grid,
        .preview = preview,
    };
    std::vector<OverlayItem> tested = std::exchange(scene.overlay, {});
    std::vector<OverlayItem> through = std::exchange(scene.xray, {});
    overlay::build(document, editor, look, xray, tested, through);
    scene.overlay = std::move(tested);
    scene.xray = std::move(through);
}

void Modeler::update_preview() {
    if (!editor.loop_preview || editor.tool != Tool::LoopCut) {
        preview.clear();
        preview_key = std::nullopt;
        return;
    }
    Edge edge = editor.loop_preview->first;
    float fraction = editor.loop_preview->second;
    std::tuple<std::uint64_t, Edge, std::int32_t> key(document.version, edge, base::saturating_cast<std::int32_t>(std::round(fraction * 100.0f)));
    if (preview_key != key) {
        preview = Editor::preview_loop(document, edge, fraction);
        preview_key = key;
    }
}

json::Value Modeler::state() const {
    const Mesh& mesh = document.mesh;
    const Selection& selection = document.selection;
    json::Value operation;
    if (editor.operation) {
        operation = object(base::vec(member("Kind", mesh_modeler::editing::name(editor.operation->kind)), member("Axis", mesh_modeler::editing::name(editor.operation->axis)),
                                     member("Amount", std::string_view(editor.operation->amount))));
    }
    json::Value hover;
    if (editor.hover) {
        hover = base::match(
            *editor.hover, [](const Element::Vertex& vertex) { return json::Value(std::format("Vertex {}", vertex.vertex)); },
            [](const Element::Edge& edge) {
                auto [a, b] = edge.edge.vertices();
                return json::Value(std::format("Edge {}-{}", a, b));
            },
            [](const Element::Face& face) { return json::Value(std::format("Face {}", face.face)); });
    }
    std::vector<json::Value> materials;
    materials.reserve(mesh.faces.size());
    for (const polygon_mesh::Face& face : mesh.faces) {
        materials.emplace_back(face.material);
    }
    std::vector<json::Value> face_list;
    face_list.reserve(selection.faces.size());
    for (size_t face : selection.faces) {
        face_list.emplace_back(face);
    }
    std::vector<json::Value> paints;
    paints.reserve(document.paints.size());
    for (const Paint& paint : document.paints) {
        paints.emplace_back(std::string_view(paint.name));
    }
    return object(base::vec(
        member("Name", std::string_view(document.name)), member("Dirty", document.dirty), member("Mode", mesh_modeler::editing::name(editor.mode)),
        member("Tool", mesh_modeler::editing::name(editor.tool)), member("Operation", std::move(operation)), member("Hover", std::move(hover)),
        member("Mesh", object(base::vec(member("Vertices", mesh.positions.size()), member("Edges", document.edges()->size()), member("Faces", mesh.faces.size()),
                                        member("Creases", mesh.creases.size()), member("Valid", mesh.validate().has_value()),
                                        member("Materials", json::Value::Array{std::move(materials)})))),
        member("Selection", object(base::vec(member("Vertices", selection.vertices.size()), member("Edges", selection.edges.size()), member("Faces", selection.faces.size()),
                                             member("FaceList", json::Value::Array{std::move(face_list)})))),
        member("Paints", json::Value::Array{std::move(paints)}), member("ActivePaint", editor.active_paint), member("Shader", mesh_modeler::viewport::name(shader)),
        member("Subdivision", levels), member("SurfaceFaces", surface_ ? surface_->mesh.faces.size() : mesh.faces.size()), member("Smooth", smooth), member("XRay", xray),
        member("Undo", document.undo_name()), member("Redo", document.redo_name()), member("UndoSteps", document.undo_steps()),
        member("Prompt", prompt ? json::Value(prompt->name()) : json::Value()), member("Message", message ? json::Value(std::string_view(message->second)) : json::Value()),
        member("EditorMessage", editor.message ? json::Value(std::string_view(*editor.message)) : json::Value()), member("LoopPreviewEdges", preview.size()),
        member("Camera", object(base::vec(member("Yaw", to_degrees(camera.yaw)), member("Pitch", to_degrees(camera.pitch)), member("Height", camera.height))))));
}

void Modeler::finish(Context<Message>& ui, Pending pending) {
    if (pending == Pending::Close) {
        prompt = std::nullopt;
        ui.close();
    } else {
        proceed(pending);
    }
}

void Modeler::answer(Context<Message>& ui, Prompt answered, DialogResult result) {
    if (prompt != answered) {
        return;
    }
    base::match(
        answered,
        [&](const Prompt::Unsaved& unsaved) {
            Pending pending = unsaved.pending;
            switch (result) {
            case DialogResult::Primary:
                if (document.name == Untitled) {
                    prompt_save_as(pending);
                } else if (save()) {
                    finish(ui, pending);
                } else {
                    prompt = std::nullopt;
                }
                break;
            case DialogResult::Secondary:
                finish(ui, pending);
                break;
            case DialogResult::Close:
                prompt = std::nullopt;
                break;
            }
        },
        [&](const Prompt::SaveAs&) {
            if (result == DialogResult::Primary) {
                std::expected<std::optional<Pending>, std::string> saved = save_as();
                if (saved) {
                    prompt_error = std::nullopt;
                    if (std::optional<Pending> pending = *saved) {
                        finish(ui, *pending);
                    }
                } else {
                    prompt_error = std::move(saved.error());
                    prompt = Prompt::SaveAs{};
                }
            } else {
                prompt = std::nullopt;
                prompt_error = std::nullopt;
            }
        },
        [&](const Prompt::Open&) {
            if (result == DialogResult::Primary) {
                open_chosen();
            } else {
                prompt = std::nullopt;
            }
        });
}

void Modeler::edit_paint(PaintEdit edit) {
    begin_paint_edit();
    Paint& paint = document.paints[editor.active_paint];
    base::match(
        edit, [&](PaintEdit::Color& color) { paint.set_color(color.color); }, [&](PaintEdit::Name& name) { paint.name = std::move(name.name); },
        [&](PaintEdit::Brightness& brightness) { paint.parameters.brightness = brightness.value; }, [&](PaintEdit::Shift& shift) { paint.parameters.shift = shift.value; },
        [&](PaintEdit::Rotation& rotation) { paint.parameters.rotation_degrees = rotation.value; }, [&](PaintEdit::Dark& dark) { paint.parameters.dark_point = dark.value; },
        [&](PaintEdit::Light& light) { paint.parameters.light_point = light.value; });
    document.touch();
}

std::string Modeler::title() const {
    std::string_view dirty = document.dirty ? "*" : "";
    return std::format("{}{} - {}", document.name, dirty, Title);
}

std::vector<Message> Modeler::input(const Ui& ui) const {
    return interaction::messages(*this, ui);
}

void Modeler::update(Message sent, Context<Message>& ui) {
    base::match(
        sent, [&](Message::Command& command) { run(ui, command.command); }, [&](Message::Crease& sharpness) { crease = static_cast<float>(sharpness.value); },
        [&](Message::Primitive& picked) { primitive = picked.index; }, [&](Message::View& pointer) { interaction::view(*this, pointer.input); },
        [&](Message::Operation& pointer) { interaction::operation(*this, pointer.input); }, [&](Message::Constrain& constrain) { editor.constrain(constrain.axis); },
        [&](Message::ConfirmOperation&) { editor.confirm(document); }, [&](Message::CancelOperation&) { editor.cancel(document); },
        [&](Message::PickPaint& picked) { editor.active_paint = picked.index; }, [&](Message::Paint& paint) { edit_paint(std::move(paint.edit)); },
        [&](Message::EndPaintEdit&) { end_paint_edit(); }, [&](Message::NameDraft& draft) { name_draft = std::move(draft.name); },
        [&](Message::OpenChoice& choice) { open_choice = choice.index; }, [&](Message::OpenActivated&) { open_chosen(); },
        [&](Message::Unsaved& unsaved) {
            if (prompt && base::is<Prompt::Unsaved>(*prompt)) {
                answer(ui, *prompt, unsaved.result);
            }
        },
        [&](Message::SaveAsResult& save_as_result) { answer(ui, Prompt::SaveAs{}, save_as_result.result); },
        [&](Message::OpenResult& open_result) { answer(ui, Prompt::Open{}, open_result.result); },
        [&](Message::Prepare&) {
            editor.active_paint = std::min(editor.active_paint, document.paints.size() - 1);
            view_ = cards::view_content(ui.bounds());
            prepare_scene(ui.theme());
        });
}

Tree<Message> Modeler::view(Ui& ui) const {
    return cards::view(*this, ui);
}

void Modeler::prepare(graphics::Gpu& gpu) {
    viewport = mesh_modeler::viewport::Viewport(gpu);
}

void Modeler::render(graphics::Gpu& gpu, const graphics::Frame& frame) {
    if (viewport) {
        viewport->draw(gpu, frame, scene);
    }
}

bool Modeler::may_close() {
    if (document.dirty) {
        prompt = Prompt::Unsaved{Pending::Close};
        return false;
    }
    return true;
}

} // namespace mesh_modeler::modeler
