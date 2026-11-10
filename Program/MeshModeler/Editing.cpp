#include "MeshModeler/Editing.h"

#include "Base/Panic.h"
#include "MeshModeler/Angles.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <memory>
#include <ranges>
#include <set>

namespace mesh_modeler::editing {

using camera::Camera;
using document::Document;
using document::Snapshot;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Face;
using polygon_mesh::Hit;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using polygon_mesh::SelectMode;
using polygon_mesh::SubdivisionSurface;
using user_interface::Point;
using user_interface::Rect;

polygon_mesh::SelectMode select_mode(Mode mode) {
    switch (mode) {
    case Mode::Vertices:
        return SelectMode::Vertices;
    case Mode::Edges:
        return SelectMode::Edges;
    case Mode::Faces:
        return SelectMode::Faces;
    }
    base::unreachable();
}

std::string_view name(Mode mode) {
    switch (mode) {
    case Mode::Vertices:
        return "Vertex";
    case Mode::Edges:
        return "Edge";
    case Mode::Faces:
        return "Face";
    }
    base::unreachable();
}

std::string_view name(Tool tool) {
    switch (tool) {
    case Tool::Select:
        return "Select";
    case Tool::Paint:
        return "Paint";
    case Tool::LoopCut:
        return "Loop cut";
    }
    base::unreachable();
}

std::string_view name(Axis axis) {
    switch (axis) {
    case Axis::Free:
        return "Free";
    case Axis::X:
        return "X";
    case Axis::Y:
        return "Y";
    case Axis::Z:
        return "Z";
    case Axis::Normal:
        return "Normal";
    }
    base::unreachable();
}

std::string_view name(OperationKind kind) {
    switch (kind) {
    case OperationKind::Move:
        return "Move";
    case OperationKind::Rotate:
        return "Rotate";
    case OperationKind::Scale:
        return "Scale";
    case OperationKind::Inset:
        return "Inset";
    }
    base::unreachable();
}

Operation::Operation(OperationKind kind, std::string name, Point start, Point pivot_screen, Vector3 pivot, Vector3 normal, Snapshot restore, Mesh base,
                     std::vector<std::uint32_t> vertices, std::vector<size_t> faces) :
    kind(kind),
    axis(Axis::Free),
    name(std::move(name)),
    start(start),
    pivot_screen(pivot_screen),
    pivot(pivot),
    normal(normal),
    restore(std::move(restore)),
    base(std::move(base)),
    vertices(std::move(vertices)),
    faces(std::move(faces)) {}

std::optional<Hit> Drawn::hit(Vector3 origin, Vector3 direction) const {
    std::optional<Hit> found = triangles.ray_cast(origin, direction);
    if (found && surface != nullptr) {
        found->face = surface->base_faces[found->face];
    }
    return found;
}

std::string describe_value(const Element& element) {
    return base::match(
        element, [](const Element::Vertex& vertex) { return std::format("Vertex({})", vertex.vertex); },
        [](const Element::Edge& edge) { return std::format("Edge({})", polygon_mesh::describe_value(edge.edge)); },
        [](const Element::Face& face) { return std::format("Face({})", face.face); });
}

namespace {

std::optional<Vector3> axis_vector(Axis axis, Vector3 normal) {
    switch (axis) {
    case Axis::Free:
        return std::nullopt;
    case Axis::X:
        return Vector3{1.0f, 0.0f, 0.0f};
    case Axis::Y:
        return Vector3{0.0f, 1.0f, 0.0f};
    case Axis::Z:
        return Vector3{0.0f, 0.0f, 1.0f};
    case Axis::Normal:
        return normal;
    }
    base::unreachable();
}

float angle_of(Point point, Point center) {
    return std::atan2(center.y - point.y, point.x - center.x);
}

// The distance from a point to a segment, on screen.
std::pair<float, float> segment_distance(Point point, Point a, Point b) {
    Point along = b - a;
    float length_squared = along.x * along.x + along.y * along.y;
    float t = length_squared > 0.0f ? std::clamp(((point.x - a.x) * along.x + (point.y - a.y) * along.y) / length_squared, 0.0f, 1.0f) : 0.0f;
    return {point.distance(a.lerp(b, t)), t};
}

} // namespace

void Editor::reset() {
    Mode kept = mode;
    *this = Editor();
    mode = kept;
}

void Editor::set_mode(Document& document, Mode to) {
    if (mode == to) {
        return;
    }
    mode = to;
    document.selection = document.selection.in_mode(document.mesh, select_mode(to));
    document.touch();
}

std::optional<Element> Editor::pick(const Document& document, const Camera& camera, Rect view, Point pointer, bool see_through, Drawn drawn) const {
    const Mesh& mesh = document.mesh;
    std::pair<Vector3, Vector3> ray = camera.ray(pointer, view);
    Vector3 origin = ray.first;
    Vector3 direction = ray.second;
    std::optional<Hit> hit = drawn.hit(origin, direction);
    const SubdivisionSurface* surface = drawn.surface;
    auto hidden = [&](Vector3 point) -> bool {
        if (see_through || surface != nullptr) {
            return false;
        }
        float distance = math::dot(math::subtract(point, origin), direction);
        return hit && hit->distance < distance - camera.pixel_size(view) * 3.0f;
    };
    switch (mode) {
    case Mode::Vertices: {
        std::optional<std::pair<float, std::uint32_t>> best;
        for (std::uint32_t vertex = 0; vertex < static_cast<std::uint32_t>(mesh.positions.size()); vertex++) {
            Point screen = camera.to_screen(mesh.position(vertex), view).first;
            float distance = screen.distance(pointer);
            if (distance <= PickRadius && (!best || distance < best->first) && !hidden(mesh.position(vertex))) {
                best = std::pair(distance, vertex);
            }
        }
        if (!best) {
            return std::nullopt;
        }
        return Element::Vertex{best->second};
    }
    case Mode::Edges: {
        std::optional<std::pair<float, Edge>> best;
        std::shared_ptr<const std::set<Edge>> edges = document.edges();
        for (Edge edge : *edges) {
            auto [a, b] = edge.vertices();
            Point sa = camera.to_screen(mesh.position(a), view).first;
            Point sb = camera.to_screen(mesh.position(b), view).first;
            auto [distance, t] = segment_distance(pointer, sa, sb);
            Vector3 point = math::add(mesh.position(a), math::scale(math::subtract(mesh.position(b), mesh.position(a)), t));
            if (distance <= PickRadius && (!best || distance < best->first) && !hidden(point)) {
                best = std::pair(distance, edge);
            }
        }
        if (!best) {
            return std::nullopt;
        }
        return Element::Edge{best->second};
    }
    case Mode::Faces:
        if (!hit) {
            return std::nullopt;
        }
        return Element::Face{hit->face};
    }
    base::unreachable();
}

bool Editor::click(Document& document, std::optional<Element> element, bool extend) {
    Snapshot before = document.snapshot();
    const Mesh& mesh = document.mesh;
    Selection picked;
    if (element) {
        picked = base::match(
            *element, [&](const Element::Vertex& vertex) { return Selection::from_vertices(mesh, {vertex.vertex}); },
            [&](const Element::Edge& edge) { return Selection::from_edges(mesh, {edge.edge}); },
            [&](const Element::Face& face) { return Selection::from_faces(mesh, {face.face}); });
    }
    if (extend) {
        bool already = element && base::match(
                                      *element, [&](const Element::Vertex& vertex) { return document.selection.vertices.contains(vertex.vertex); },
                                      [&](const Element::Edge& edge) { return document.selection.edges.contains(edge.edge); },
                                      [&](const Element::Face& face) { return document.selection.faces.contains(face.face); });
        if (already) {
            document.selection = without(document, *element);
        } else {
            document.selection = document.selection.union_(picked, mesh, select_mode(mode));
        }
    } else {
        document.selection = std::move(picked);
    }
    return document.commit("Select", std::move(before));
}

Selection Editor::without(const Document& document, Element element) const {
    const Mesh& mesh = document.mesh;
    const Selection& selection = document.selection;
    return base::match(
        element,
        [&](const Element::Vertex& removed) {
            return Selection::from_vertices(mesh, selection.vertices | std::views::filter([&](std::uint32_t other) { return other != removed.vertex; }));
        },
        [&](const Element::Edge& removed) { return Selection::from_edges(mesh, selection.edges | std::views::filter([&](Edge other) { return other != removed.edge; })); },
        [&](const Element::Face& removed) { return Selection::from_faces(mesh, selection.faces | std::views::filter([&](size_t other) { return other != removed.face; })); });
}

bool Editor::select_loop(Document& document, Edge edge, bool extend) {
    Snapshot before = document.snapshot();
    const Mesh& mesh = document.mesh;
    Selection picked = mode == Mode::Faces ? Selection::face_loop(mesh, edge) : Selection::edge_loop(mesh, edge);
    if (extend) {
        document.selection = document.selection.union_(picked, mesh, select_mode(mode));
    } else {
        document.selection = std::move(picked);
    }
    return document.commit("Select loop", std::move(before));
}

std::optional<std::pair<Edge, float>> Editor::nearest_edge(const Document& document, const Camera& camera, Rect view, Point pointer) const {
    const Mesh& mesh = document.mesh;
    struct Nearest {
        float distance;
        Edge edge;
        float t;
    };
    std::optional<Nearest> best;
    std::shared_ptr<const std::set<Edge>> edges = document.edges();
    for (Edge edge : *edges) {
        auto [a, b] = edge.vertices();
        Point sa = camera.to_screen(mesh.position(a), view).first;
        Point sb = camera.to_screen(mesh.position(b), view).first;
        auto [distance, t] = segment_distance(pointer, sa, sb);
        if (distance <= PickRadius * 2.0f && (!best || distance < best->distance)) {
            best = Nearest{distance, edge, t};
        }
    }
    if (!best) {
        return std::nullopt;
    }
    return std::pair(best->edge, best->t);
}

void Editor::select_box(Document& document, const Camera& camera, Rect view, std::pair<Point, Point> corners, bool extend, bool subtract) {
    Snapshot before = document.snapshot();
    Rect area = Rect::from_corners(corners.first, corners.second);
    const Mesh& mesh = document.mesh;
    std::vector<std::uint32_t> inside;
    for (std::uint32_t vertex = 0; vertex < static_cast<std::uint32_t>(mesh.positions.size()); vertex++) {
        if (area.contains(camera.to_screen(mesh.position(vertex), view).first)) {
            inside.push_back(vertex);
        }
    }
    auto is_inside = [&](std::uint32_t vertex) {
        return std::ranges::find(inside, vertex) != inside.end();
    };
    Selection boxed;
    switch (mode) {
    case Mode::Vertices:
        boxed = Selection::from_vertices(mesh, inside);
        break;
    case Mode::Edges: {
        std::shared_ptr<const std::set<Edge>> edges = document.edges();
        boxed = Selection::from_edges(mesh, *edges | std::views::filter([&](Edge edge) { return std::ranges::all_of(edge.vertices(), is_inside); }));
        break;
    }
    case Mode::Faces:
        boxed = Selection::from_faces(mesh, std::views::iota(size_t{0}, mesh.faces.size()) |
                                                std::views::filter([&](size_t face) { return std::ranges::all_of(mesh.faces[face].vertices, is_inside); }));
        break;
    }
    SelectMode in_mode = select_mode(mode);
    if (subtract) {
        Selection current = document.selection.in_mode(mesh, in_mode);
        switch (mode) {
        case Mode::Vertices:
            document.selection = Selection::from_vertices(mesh, current.vertices | std::views::filter([&](std::uint32_t vertex) { return !boxed.vertices.contains(vertex); }));
            break;
        case Mode::Edges:
            document.selection = Selection::from_edges(mesh, current.edges | std::views::filter([&](Edge edge) { return !boxed.edges.contains(edge); }));
            break;
        case Mode::Faces:
            document.selection = Selection::from_faces(mesh, current.faces | std::views::filter([&](size_t face) { return !boxed.faces.contains(face); }));
            break;
        }
    } else if (extend) {
        document.selection = document.selection.union_(boxed, mesh, in_mode);
    } else {
        document.selection = std::move(boxed);
    }
    document.commit("Select box", std::move(before));
}

void Editor::press(Point point, std::optional<std::uint64_t> step) {
    press_ = std::pair(point, step);
    selecting_box = std::nullopt;
}

void Editor::drag(Document& document, Point point) {
    if (!press_) {
        return;
    }
    auto& [start, step] = *press_;
    if (!selecting_box) {
        if (start.distance(point) <= DragThreshold) {
            return;
        }
        if (std::optional<std::uint64_t> version = std::exchange(step, std::nullopt)) {
            document.withdraw(*version);
        }
    }
    selecting_box = std::pair(start, point);
}

std::optional<std::pair<Point, Point>> Editor::release() {
    press_ = std::nullopt;
    return std::exchange(selecting_box, std::nullopt);
}

bool Editor::pressing() const {
    return press_.has_value();
}

std::vector<std::uint32_t> Editor::moving_vertices(const Document& document) {
    return std::vector<std::uint32_t>(document.selection.vertices.begin(), document.selection.vertices.end());
}

Vector3 Editor::selection_normal(const Document& document) {
    const Mesh& mesh = document.mesh;
    std::vector<size_t> faces(document.selection.faces.begin(), document.selection.faces.end());
    if (faces.empty()) {
        for (size_t face = 0; face < mesh.faces.size(); face++) {
            if (std::ranges::any_of(mesh.faces[face].vertices, [&](std::uint32_t vertex) { return document.selection.vertices.contains(vertex); })) {
                faces.push_back(face);
            }
        }
    }
    Vector3 sum{0.0f, 0.0f, 0.0f};
    for (size_t face : faces) {
        sum = math::add(sum, mesh.face_area_vector(face));
    }
    if (math::length(sum) > 1e-9f) {
        return math::normalize(sum);
    }
    return {0.0f, 1.0f, 0.0f};
}

bool Editor::begin(Document& document, OperationKind kind, Point pointer, const Camera& camera, Rect view) {
    return begin_from(document, kind, pointer, camera, view, document.snapshot(), name(kind));
}

bool Editor::begin_from(Document& document, OperationKind kind, Point pointer, const Camera& camera, Rect view, Snapshot restore, std::string_view step) {
    std::vector<std::uint32_t> vertices = moving_vertices(document);
    if (vertices.empty() || (kind == OperationKind::Inset && document.selection.faces.empty())) {
        message = std::format("{} needs a selection{}.", name(kind), kind == OperationKind::Inset ? " of faces" : "");
        return false;
    }
    Vector3 pivot = document.mesh.center_of(vertices).value_or(Vector3{0.0f, 0.0f, 0.0f});
    Point pivot_screen = camera.to_screen(pivot, view).first;
    // A pointer on the pivot would scale and turn by nothing, so they
    // start a little away from it.
    Point start = pointer;
    if ((kind == OperationKind::Scale || kind == OperationKind::Rotate || kind == OperationKind::Inset) && start.distance(pivot_screen) < 20.0f) {
        start = Point(pivot_screen.x + 100.0f, pivot_screen.y);
    }
    operation = Operation(kind, std::string(step), start, pivot_screen, pivot, selection_normal(document), std::move(restore), document.mesh, std::move(vertices),
                          std::vector<size_t>(document.selection.faces.begin(), document.selection.faces.end()));
    message = std::nullopt;
    return true;
}

void Editor::constrain(Axis axis) {
    if (operation) {
        operation->axis = operation->axis == axis ? Axis::Free : axis;
    }
}

void Editor::update(Document& document, Point pointer, bool snap, const Camera& camera, Rect view) {
    if (!operation) {
        return;
    }
    Operation& edit = *operation;
    Mesh mesh = edit.base;
    const std::vector<std::uint32_t>& vertices = edit.vertices;
    auto snap_to = [snap](float value, float step) {
        return snap ? std::round(value / step) * step : value;
    };
    switch (edit.kind) {
    case OperationKind::Move: {
        Point delta = pointer - edit.start;
        Vector3 offset;
        if (std::optional<Vector3> axis = axis_vector(edit.axis, edit.normal)) {
            // How far along the axis, from how far the pointer
            // moved along the axis's line on screen.
            Point tip = camera.to_screen(math::add(edit.pivot, *axis), view).first;
            Point along = tip - edit.pivot_screen;
            float pixels = along.x * along.x + along.y * along.y;
            float amount = pixels > 1.0f ? (delta.x * along.x + delta.y * along.y) / pixels : 0.0f;
            offset = math::scale(*axis, snap_to(amount, 0.1f));
        } else {
            float size = camera.pixel_size(view);
            Vector3 world = math::add(math::scale(camera.right(), delta.x * size), math::scale(camera.up(), -delta.y * size));
            offset = {snap_to(world[0], 0.1f), snap_to(world[1], 0.1f), snap_to(world[2], 0.1f)};
        }
        mesh.translate(vertices, offset);
        edit.amount = std::format("{:.3f}, {:.3f}, {:.3f}", offset[0], offset[1], offset[2]);
        break;
    }
    case OperationKind::Rotate: {
        float turned = angle_of(pointer, edit.pivot_screen) - angle_of(edit.start, edit.pivot_screen);
        float angle = to_radians(snap_to(to_degrees(turned), 15.0f));
        Vector3 backward = camera.backward();
        // Counterclockwise on screen turns about the axis toward
        // the camera.
        Vector3 axis = backward;
        if (std::optional<Vector3> limited = axis_vector(edit.axis, edit.normal)) {
            axis = math::dot(*limited, backward) < 0.0f ? math::scale(*limited, -1.0f) : *limited;
        }
        mesh.rotate(vertices, edit.pivot, axis, angle);
        edit.amount = std::format("{:.1f}°", to_degrees(angle));
        break;
    }
    case OperationKind::Scale: {
        float initial = std::max(edit.start.distance(edit.pivot_screen), 1.0f);
        float factor = snap_to(pointer.distance(edit.pivot_screen) / initial, 0.1f);
        Vector3 factors{factor, factor, factor};
        switch (edit.axis) {
        case Axis::X:
            factors = {factor, 1.0f, 1.0f};
            break;
        case Axis::Y:
            factors = {1.0f, factor, 1.0f};
            break;
        case Axis::Z:
            factors = {1.0f, 1.0f, factor};
            break;
        case Axis::Free:
        case Axis::Normal:
            break;
        }
        mesh.scale(vertices, edit.pivot, factors);
        edit.amount = std::format("{:.3f}", factor);
        break;
    }
    case OperationKind::Inset: {
        float initial = edit.start.distance(edit.pivot_screen);
        float thickness = snap_to(std::max((initial - pointer.distance(edit.pivot_screen)) * camera.pixel_size(view), 0.0f), 0.05f);
        Selection selection = mesh.inset_region(edit.faces, thickness);
        document.selection = std::move(selection);
        edit.amount = std::format("{:.3f}", thickness);
        break;
    }
    }
    document.mesh = std::move(mesh);
    document.touch();
}

void Editor::confirm(Document& document) {
    if (operation) {
        Operation done = std::move(*operation);
        operation = std::nullopt;
        document.commit(done.name, std::move(done.restore));
    }
}

void Editor::cancel(Document& document) {
    if (operation) {
        Operation done = std::move(*operation);
        operation = std::nullopt;
        document.restore(std::move(done.restore));
    }
}

bool Editor::extrude(Document& document, Point pointer, const Camera& camera, Rect view) {
    Snapshot restore = document.snapshot();
    Selection selection;
    if (!document.selection.faces.empty()) {
        std::vector<size_t> faces(document.selection.faces.begin(), document.selection.faces.end());
        selection = document.mesh.extrude_faces(faces);
    } else if (!document.selection.edges.empty()) {
        std::vector<Edge> edges(document.selection.edges.begin(), document.selection.edges.end());
        selection = document.mesh.extrude_edges(edges);
    } else {
        message = "Extrude needs faces or edges selected.";
        return false;
    }
    document.selection = std::move(selection);
    document.touch();
    bool faces = !document.selection.faces.empty();
    bool began = begin_from(document, OperationKind::Move, pointer, camera, view, std::move(restore), "Extrude");
    if (began && faces) {
        constrain(Axis::Normal);
    }
    return began;
}

bool Editor::inset(Document& document, Point pointer, const Camera& camera, Rect view) {
    return begin(document, OperationKind::Inset, pointer, camera, view);
}

void Editor::delete_(Document& document) {
    Mode in_mode = mode;
    apply(document, "Delete", [in_mode](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.is_empty()) {
            return std::unexpected("Delete needs a selection.");
        }
        switch (in_mode) {
        case Mode::Vertices:
            return mesh.delete_vertices(selection.vertices);
        case Mode::Edges:
            return mesh.delete_edges(selection.edges);
        case Mode::Faces:
            return mesh.delete_faces(selection.faces);
        }
        base::unreachable();
    });
}

void Editor::delete_only_faces(Document& document) {
    apply(document, "Delete faces", [](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.faces.empty()) {
            return std::unexpected("Delete faces needs faces selected.");
        }
        return mesh.delete_only_faces(selection.faces);
    });
}

void Editor::fill(Document& document) {
    Mode in_mode = mode;
    apply(document, "Fill", [in_mode](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (in_mode == Mode::Edges && !selection.edges.empty()) {
            return mesh.fill(selection.edges);
        }
        return mesh.make_face(selection.vertices);
    });
}

void Editor::merge(Document& document) {
    apply(document, "Merge at center", [](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.vertices.size() < 2) {
            return std::unexpected("Merge needs two vertices or more selected.");
        }
        return mesh.merge_at_center(selection.vertices);
    });
}

void Editor::subdivide(Document& document) {
    apply(document, "Subdivide", [](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.faces.empty()) {
            return std::unexpected("Subdivide needs faces selected.");
        }
        return mesh.subdivide_faces(selection.faces);
    });
}

void Editor::flip(Document& document) {
    apply(document, "Flip normals", [](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.faces.empty()) {
            return std::unexpected("Flip needs faces selected.");
        }
        mesh.flip_faces(selection.faces);
        return selection;
    });
}

void Editor::recalculate_normals(Document& document) {
    apply(document, "Recalculate normals", [](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        std::vector<size_t> faces;
        if (selection.faces.empty()) {
            faces.reserve(mesh.faces.size());
            for (size_t face = 0; face < mesh.faces.size(); face++) {
                faces.push_back(face);
            }
        } else {
            faces.assign(selection.faces.begin(), selection.faces.end());
        }
        mesh.recalculate_normals_outside(faces);
        return selection;
    });
}

void Editor::crease(Document& document, float sharpness) {
    apply(document, "Crease", [sharpness](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.edges.empty()) {
            return std::unexpected("Crease needs edges selected.");
        }
        mesh.crease_edges(selection.edges, sharpness);
        return selection;
    });
}

void Editor::assign_paint(Document& document) {
    std::uint32_t paint = static_cast<std::uint32_t>(active_paint);
    apply(document, "Paint", [paint](Mesh& mesh, const Selection& selection) -> std::expected<Selection, std::string> {
        if (selection.faces.empty()) {
            return std::unexpected("Paint needs faces selected.");
        }
        mesh.assign_material(selection.faces, paint);
        return selection;
    });
}

void Editor::paint_at(Document& document, std::optional<size_t> face) {
    if (!painting) {
        painting = document.snapshot();
    }
    if (face && *face < document.mesh.faces.size() && document.mesh.faces[*face].material != static_cast<std::uint32_t>(active_paint)) {
        document.mesh.assign_material({*face}, static_cast<std::uint32_t>(active_paint));
        document.touch();
    }
}

void Editor::end_paint(Document& document) {
    if (painting) {
        Snapshot before = std::move(*painting);
        painting = std::nullopt;
        document.commit("Paint", std::move(before));
    }
}

void Editor::loop_cut(Document& document, Edge edge, float fraction) {
    apply(document, "Loop cut", [edge, fraction](Mesh& mesh, const Selection&) { return mesh.loop_cut(edge, fraction); });
    if (!message) {
        // The cut's edges, in edge mode, are what Blender selects.
        mode = mode == Mode::Faces ? Mode::Edges : mode;
    }
}

std::vector<std::array<Vector3, 2>> Editor::preview_loop(const Document& document, Edge edge, float fraction) {
    Mesh mesh = document.mesh;
    std::expected<Selection, std::string> selection = mesh.loop_cut(edge, fraction);
    if (!selection) {
        return {};
    }
    std::vector<std::array<Vector3, 2>> edges;
    edges.reserve(selection->edges.size());
    for (Edge cut : selection->edges) {
        auto [a, b] = cut.vertices();
        edges.push_back({mesh.position(a), mesh.position(b)});
    }
    return edges;
}

void Editor::add_primitive(Document& document, Primitive kind) {
    Snapshot before = document.snapshot();
    Mesh added = make(kind);
    std::uint32_t paint = static_cast<std::uint32_t>(active_paint);
    for (Face& face : added.faces) {
        face.material = paint;
    }
    std::uint32_t offset = static_cast<std::uint32_t>(document.mesh.positions.size());
    size_t first_face = document.mesh.faces.size();
    document.mesh.positions.insert(document.mesh.positions.end(), added.positions.begin(), added.positions.end());
    for (const Face& face : added.faces) {
        std::vector<std::uint32_t> vertices;
        vertices.reserve(face.vertices.size());
        for (std::uint32_t vertex : face.vertices) {
            vertices.push_back(vertex + offset);
        }
        document.mesh.add_face(std::move(vertices), face.material);
    }
    for (const auto& [edge, sharpness] : added.creases) {
        auto [a, b] = edge.vertices();
        document.mesh.set_crease(Edge(a + offset, b + offset), sharpness);
    }
    document.selection = Selection::from_faces(document.mesh, std::views::iota(first_face, document.mesh.faces.size()));
    document.commit(std::format("Add {}", name(kind)), std::move(before));
}

std::string_view name(Primitive primitive) {
    switch (primitive) {
    case Primitive::Cube:
        return "Cube";
    case Primitive::Plane:
        return "Plane";
    case Primitive::Cylinder:
        return "Cylinder";
    case Primitive::Cone:
        return "Cone";
    case Primitive::Sphere:
        return "UV sphere";
    case Primitive::Torus:
        return "Torus";
    }
    base::unreachable();
}

polygon_mesh::Mesh make(Primitive primitive) {
    switch (primitive) {
    case Primitive::Cube:
        return polygon_mesh::primitives::cube(2.0f);
    case Primitive::Plane:
        return polygon_mesh::primitives::plane({2.0f, 2.0f}, {1, 1});
    case Primitive::Cylinder:
        return polygon_mesh::primitives::cylinder(1.0f, 2.0f, 16, 1);
    case Primitive::Cone:
        return polygon_mesh::primitives::cone(1.0f, 0.0f, 2.0f, 16);
    case Primitive::Sphere:
        return polygon_mesh::primitives::uv_sphere(1.0f, 24, 12);
    case Primitive::Torus:
        return polygon_mesh::primitives::torus(1.0f, 0.35f, 24, 12);
    }
    base::unreachable();
}

} // namespace mesh_modeler::editing
