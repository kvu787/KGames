// How the modeler edits the mesh, as Blender's edit mode does: vertices,
// edges or faces are selected, by a click, a drag of a box, or a loop; and the
// selection is moved, turned and scaled by the pointer, along an axis or
// freely, or extruded, inset, cut, merged, filled, deleted, creased or
// painted. Each edit is a step of the document's history.
//
// Moving, turning, scaling and insetting are modal, as in Blender: they
// follow the pointer from where they began until a click or Enter keeps them,
// or a right-click or Esc takes them back.

#pragma once

#include "Base/Sum.h"
#include "Math/Math.h"
#include "MeshModeler/Camera.h"
#include "MeshModeler/Document.h"
#include "PolygonMesh/PolygonMesh.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace mesh_modeler::editing {

// How near the pointer must be to a vertex or an edge to pick it.
inline constexpr float PickRadius = 12.0f;
// How far the pointer moves before a press on nothing becomes a box.
inline constexpr float DragThreshold = 4.0f;

enum class Mode {
    Vertices,
    Edges,
    Faces,
};

inline constexpr std::array<Mode, 3> AllModes{Mode::Vertices, Mode::Edges, Mode::Faces};

polygon_mesh::SelectMode select_mode(Mode mode);

std::string_view name(Mode mode);

enum class Tool {
    Select,
    // Paints the faces that the pointer presses or drags over.
    Paint,
    // Cuts a loop across the ring of quads under the pointer.
    LoopCut,
};

inline constexpr std::array<Tool, 3> AllTools{Tool::Select, Tool::Paint, Tool::LoopCut};

std::string_view name(Tool tool);

// What a modal edit is limited to.
enum class Axis {
    Free,
    X,
    Y,
    Z,
    // Along the selection's normal, as an extrusion moves.
    Normal,
};

std::string_view name(Axis axis);

enum class OperationKind {
    Move,
    Rotate,
    Scale,
    Inset,
};

std::string_view name(OperationKind kind);

class Editor;

// A modal edit under way.
class Operation {
public:
    OperationKind kind;
    Axis axis;
    // What its history step is called, like "Extrude" for a move that
    // follows an extrusion.
    std::string name;

private:
    // The editor begins it, and applies it.
    friend class Editor;

    Operation(OperationKind kind, std::string name, user_interface::Point start, user_interface::Point pivot_screen, math::Vector3 pivot, math::Vector3 normal,
              document::Snapshot restore, polygon_mesh::Mesh base, std::vector<std::uint32_t> vertices, std::vector<size_t> faces);

    // Where the pointer was as it began, and where the pivot is on screen.
    user_interface::Point start;
    user_interface::Point pivot_screen;
    math::Vector3 pivot;
    math::Vector3 normal;
    // What a cancel goes back to, and what the edit applies to.
    document::Snapshot restore;
    polygon_mesh::Mesh base;
    std::vector<std::uint32_t> vertices;
    std::vector<size_t> faces;

public:
    // What it's done, for the status bar: like "0.25" or "45°".
    std::string amount;
};

// What the view draws, which a ray from the pointer goes through: the
// surface, when the mesh is subdivided, and the triangles, of it or of the
// mesh.
struct Drawn {
    // Null when the mesh isn't subdivided.
    const polygon_mesh::SubdivisionSurface* surface;
    const polygon_mesh::Triangles& triangles;

    // The nearest face that the ray hits, as one of the mesh's, through the
    // surface's when the mesh is subdivided.
    std::optional<polygon_mesh::Hit> hit(math::Vector3 origin, math::Vector3 direction) const;
};

// A vertex, an edge or a face under the pointer.
struct Element {
    struct Vertex {
        std::uint32_t vertex;
        bool operator==(const Vertex&) const = default;
    };
    struct Edge {
        polygon_mesh::Edge edge;
        bool operator==(const Edge&) const = default;
    };
    struct Face {
        size_t face;
        bool operator==(const Face&) const = default;
    };
    std::variant<Vertex, Edge, Face> value;

    template <base::AlternativeOf<Element> T> Element(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Element&) const = default;
};

// How a failed check in a test shows an element.
std::string describe_value(const Element& element);

enum class Primitive {
    Cube,
    Plane,
    Cylinder,
    Cone,
    Sphere,
    Torus,
};

inline constexpr std::array<Primitive, 6> AllPrimitives{Primitive::Cube, Primitive::Plane, Primitive::Cylinder, Primitive::Cone, Primitive::Sphere, Primitive::Torus};

std::string_view name(Primitive primitive);

polygon_mesh::Mesh make(Primitive primitive);

class Editor {
public:
    Mode mode = Mode::Faces;
    Tool tool = Tool::Select;
    std::optional<Operation> operation;
    std::optional<Element> hover;

private:
    // A press on the view, which selected as it went down, and may become a
    // box: where it was, and the document's version after the step that its
    // selection made, if it made one. And the box's corners.
    std::optional<std::pair<user_interface::Point, std::optional<std::uint64_t>>> press_;

public:
    std::optional<std::pair<user_interface::Point, user_interface::Point>> selecting_box;
    // The loop that the loop cut tool would cut: the edge and the fraction.
    std::optional<std::pair<polygon_mesh::Edge, float>> loop_preview;

private:
    // A paint stroke under way, from the document as it was.
    std::optional<document::Snapshot> painting;

public:
    size_t active_paint = 0;
    // What an edit said last, like why a face couldn't be made.
    std::optional<std::string> message;

    Editor() = default;
    Editor(const Editor&) = delete;
    Editor& operator=(const Editor&) = delete;
    Editor(Editor&&) = default;
    Editor& operator=(Editor&&) = default;

    // Forgets what's under way and the paint picked, for another document,
    // keeping the selection mode.
    void reset();

    // Switches the selection mode, converting the selection as Blender does.
    void set_mode(document::Document& document, Mode to);

    // The element under the pointer, in the selection mode: the nearest
    // vertex or edge within PickRadius pixels, or the face hit, that the
    // mesh doesn't hide, unless in X-ray or over a subdivided surface.
    std::optional<Element> pick(const document::Document& document, const camera::Camera& camera, user_interface::Rect view, user_interface::Point pointer, bool see_through,
                                Drawn drawn) const;

    // A click on the element, or on nothing: picks it, adds it to the
    // selection or takes it away with Shift, or selects nothing. Returns
    // whether the selection changed, which is a step.
    bool click(document::Document& document, std::optional<Element> element, bool extend);

    // Selects the loop through an edge: an edge loop, or in face mode, the
    // loop of faces across it. Returns whether the selection changed.
    bool select_loop(document::Document& document, polygon_mesh::Edge edge, bool extend);

    // The edge nearest the pointer, whatever the mode, for loops.
    std::optional<std::pair<polygon_mesh::Edge, float>> nearest_edge(const document::Document& document, const camera::Camera& camera, user_interface::Rect view,
                                                                     user_interface::Point pointer) const;

    // Selects what's in the box on screen: vertices inside it, and edges and
    // faces whose vertices all are. Shift adds to the selection, and Ctrl
    // takes away from it.
    void select_box(document::Document& document, const camera::Camera& camera, user_interface::Rect view, std::pair<user_interface::Point, user_interface::Point> corners,
                    bool extend, bool subtract);

    // A press, a drag and a release of the left button on the view with the
    // Select tool. A click selects as it goes down, and the press records
    // the document's version after the step that its selection made, if it
    // made one. A drag from it past DragThreshold becomes a box, which
    // takes that step back, so that the box selects from what was selected
    // before the press, as one step, when it's let go of.
    void press(user_interface::Point point, std::optional<std::uint64_t> step);

    void drag(document::Document& document, user_interface::Point point);

    // Ends a press: returns the box, if it became one.
    std::optional<std::pair<user_interface::Point, user_interface::Point>> release();

    bool pressing() const;

    // Begins a modal edit of the selection from the pointer.
    bool begin(document::Document& document, OperationKind kind, user_interface::Point pointer, const camera::Camera& camera, user_interface::Rect view);

    // Limits the edit to an axis, or frees it if it's that axis already.
    void constrain(Axis axis);

    // Applies the edit for where the pointer is now; with `snap`, to whole
    // tenths, or 15 degrees.
    void update(document::Document& document, user_interface::Point pointer, bool snap, const camera::Camera& camera, user_interface::Rect view);

    // Keeps the edit, as one step of the history.
    void confirm(document::Document& document);

    // Takes the edit back, and whatever began it, like an extrusion.
    void cancel(document::Document& document);

    // Extrudes the selected faces, or else edges, and moves what it made
    // along their normal.
    bool extrude(document::Document& document, user_interface::Point pointer, const camera::Camera& camera, user_interface::Rect view);

    // Begins insetting the selected faces, as a region.
    bool inset(document::Document& document, user_interface::Point pointer, const camera::Camera& camera, user_interface::Rect view);

    // An instant edit of the selection, as one step of the history, which
    // says why if it can't be done. `edit` is called with the mesh, a
    // polygon_mesh::Mesh&, and the selection, a const polygon_mesh::Selection&,
    // and returns a std::expected<polygon_mesh::Selection, std::string>.
    template <class Edit> void apply(document::Document& document, std::string_view step, Edit&& edit) {
        document::Snapshot before = document.snapshot();
        polygon_mesh::Selection selection = document.selection;
        std::expected<polygon_mesh::Selection, std::string> edited = std::forward<Edit>(edit)(document.mesh, selection);
        if (edited) {
            document.selection = std::move(*edited);
            document.commit(step, std::move(before));
            message = std::nullopt;
        } else {
            document.restore(std::move(before));
            message = std::move(edited.error());
        }
    }

    void delete_(document::Document& document);

    void delete_only_faces(document::Document& document);

    // Makes a face of the selected vertices, or fills the loop of the
    // selected boundary edges.
    void fill(document::Document& document);

    void merge(document::Document& document);

    void subdivide(document::Document& document);

    void flip(document::Document& document);

    void recalculate_normals(document::Document& document);

    // Sets the selected edges' crease sharpness.
    void crease(document::Document& document, float sharpness);

    // Paints the selected faces with the active paint.
    void assign_paint(document::Document& document);

    // A stroke of the Paint tool: each face the pointer presses or drags
    // over takes the active paint; the whole stroke is one step.
    void paint_at(document::Document& document, std::optional<size_t> face);

    void end_paint(document::Document& document);

    // Cuts the loop through the edge, at the fraction along it from its
    // smaller vertex, and selects the new loop.
    void loop_cut(document::Document& document, polygon_mesh::Edge edge, float fraction);

    // The edges that a loop cut through the edge would make, to preview.
    static std::vector<std::array<math::Vector3, 2>> preview_loop(const document::Document& document, polygon_mesh::Edge edge, float fraction);

    // Adds a primitive to the mesh, around the origin, and selects it.
    void add_primitive(document::Document& document, Primitive kind);

private:
    // The selection less an element.
    polygon_mesh::Selection without(const document::Document& document, Element element) const;

    // The vertices that the selection moves.
    static std::vector<std::uint32_t> moving_vertices(const document::Document& document);

    // The selection's average normal: its faces', or else the faces' around
    // its vertices.
    static math::Vector3 selection_normal(const document::Document& document);

    bool begin_from(document::Document& document, OperationKind kind, user_interface::Point pointer, const camera::Camera& camera, user_interface::Rect view,
                    document::Snapshot restore, std::string_view step);
};

} // namespace mesh_modeler::editing
