#include "MeshModeler/Overlay.h"

#include "Base/Sum.h"

#include <algorithm>
#include <memory>
#include <optional>
#include <set>
#include <utility>

namespace mesh_modeler::overlay {

using document::Document;
using editing::Editor;
using editing::Element;
using editing::Mode;
using editing::Tool;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::InfinitelySharp;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using polygon_mesh::Triangles;
using user_interface::Color;
using user_interface::Theme;
using viewport::KindLine;
using viewport::KindPoint;
using viewport::KindTriangle;
using viewport::OverlayItem;

namespace {

// Widths of lines and sizes of dots, in pixels.
constexpr float GridWidth = 1.0f;
constexpr float AxisWidth = 2.0f;
constexpr float EdgeWidth = 1.5f;
constexpr float SelectedEdgeWidth = 2.5f;
constexpr float HoverWidth = 3.0f;
constexpr float VertexSize = 5.0f;
constexpr float SelectedVertexSize = 7.0f;
constexpr float HoverVertexSize = 9.0f;

std::optional<std::array<Vector3, 2>> mesh_edge(const Document& document, Edge edge) {
    auto [a, b] = edge.vertices();
    std::uint32_t count = static_cast<std::uint32_t>(document.mesh.positions.size());
    if (a < count && b < count) {
        return std::array{document.mesh.position(a), document.mesh.position(b)};
    }
    return std::nullopt;
}

void push_edge(const std::optional<std::array<Vector3, 2>>& edge, float width, Color color, std::vector<OverlayItem>& items) {
    if (edge) {
        items.push_back(line((*edge)[0], (*edge)[1], width, color));
    }
}

} // namespace

OverlayItem line(Vector3 a, Vector3 b, float width, Color color) {
    return OverlayItem{
        .a = a,
        .width = width,
        .b = b,
        .color = color.pack_premultiplied(),
        .c = {0.0f, 0.0f, 0.0f},
        .kind = KindLine,
    };
}

OverlayItem point(Vector3 at, float size, Color color) {
    return OverlayItem{
        .a = at,
        .width = size,
        .b = at,
        .color = color.pack_premultiplied(),
        .c = at,
        .kind = KindPoint,
    };
}

OverlayItem triangle(Vector3 a, Vector3 b, Vector3 c, Color color) {
    return OverlayItem{
        .a = a,
        .width = 0.0f,
        .b = b,
        .color = color.pack_premultiplied(),
        .c = c,
        .kind = KindTriangle,
    };
}

void grid(const Theme& theme, std::vector<OverlayItem>& items) {
    float reach = static_cast<float>(GridReach);
    Color minor = theme.divider;
    Color major = theme.control_strong_stroke.fade(0.35f);
    for (std::int32_t step = -GridReach; step <= GridReach; step++) {
        float at = static_cast<float>(step);
        Color color = step % 5 == 0 ? major : minor;
        if (step != 0) {
            items.push_back(line({at, 0.0f, -reach}, {at, 0.0f, reach}, GridWidth, color));
            items.push_back(line({-reach, 0.0f, at}, {reach, 0.0f, at}, GridWidth, color));
        }
    }
    items.push_back(line({-reach, 0.0f, 0.0f}, {reach, 0.0f, 0.0f}, AxisWidth, theme.data.red));
    items.push_back(line({0.0f, 0.0f, -reach}, {0.0f, 0.0f, reach}, AxisWidth, theme.data.blue));
}

void build(const Document& document, const Editor& editor, const Look& look, bool xray, std::vector<OverlayItem>& tested, std::vector<OverlayItem>& through) {
    tested.clear();
    through.clear();
    const Theme& theme = look.theme;
    if (look.grid) {
        grid(theme, tested);
    }
    std::vector<OverlayItem> cage_items;
    std::vector<OverlayItem>& items = cage_items;
    const Mesh& mesh = document.mesh;
    const Selection& selection = document.selection;
    Color accent = theme.accent;

    // The selected faces' tint, on the triangles drawn.
    Color tint = accent.fade(xray ? 0.18f : 0.28f);
    std::optional<size_t> hover_face;
    if (editor.hover && !editor.operation) {
        if (const Element::Face* face = base::as<Element::Face>(*editor.hover)) {
            hover_face = face->face;
        }
    }
    Color hover_tint = accent.fade(0.12f);
    if (!selection.faces.empty() || hover_face) {
        const Triangles& triangles = look.triangles;
        for (size_t index = 0; index < triangles.indices.size() / 3; index++) {
            const std::uint32_t* corners = &triangles.indices[index * 3];
            size_t face = triangles.faces[index];
            size_t base_face = look.surface != nullptr ? look.surface->base_faces[face] : face;
            Color color;
            if (selection.faces.contains(base_face)) {
                color = tint;
            } else if (hover_face == base_face) {
                color = hover_tint;
            } else {
                continue;
            }
            Vector3 a = triangles.vertices[corners[0]].position;
            Vector3 b = triangles.vertices[corners[1]].position;
            Vector3 c = triangles.vertices[corners[2]].position;
            items.push_back(triangle(a, b, c, color));
        }
    }

    // The cage's edges: creased ones in pink, selected ones in the accent.
    Color cage = look.surface != nullptr ? theme.text_secondary.fade(0.8f) : theme.text_primary.fade(0.7f);
    Color crease = theme.data.pink;
    std::vector<std::pair<Vector3, Vector3>> selected_edges;
    std::shared_ptr<const std::set<Edge>> edges = document.edges();
    for (Edge edge : *edges) {
        auto [a, b] = edge.vertices();
        Vector3 pa = mesh.position(a);
        Vector3 pb = mesh.position(b);
        if (selection.edges.contains(edge)) {
            selected_edges.emplace_back(pa, pb);
            continue;
        }
        float sharpness = mesh.crease(edge);
        if (sharpness > 0.0f) {
            float width = EdgeWidth + std::min(sharpness, InfinitelySharp) / InfinitelySharp;
            items.push_back(line(pa, pb, width, crease));
        } else {
            items.push_back(line(pa, pb, EdgeWidth, cage));
        }
    }
    for (const auto& [a, b] : selected_edges) {
        items.push_back(line(a, b, SelectedEdgeWidth, accent));
    }

    // Vertices, in vertex mode.
    if (editor.mode == Mode::Vertices) {
        Color dot = theme.text_primary;
        for (std::uint32_t vertex = 0; vertex < static_cast<std::uint32_t>(mesh.positions.size()); vertex++) {
            if (!selection.vertices.contains(vertex)) {
                items.push_back(point(mesh.position(vertex), VertexSize, dot));
            }
        }
        for (std::uint32_t vertex : selection.vertices) {
            if (vertex < mesh.positions.size()) {
                items.push_back(point(mesh.position(vertex), SelectedVertexSize, accent));
            }
        }
    }

    // What the pointer is on, over the rest.
    Color hover = accent.fade(0.65f);
    if (editor.hover && !editor.operation) {
        if (const Element::Vertex* vertex = base::as<Element::Vertex>(*editor.hover)) {
            if (vertex->vertex < mesh.positions.size()) {
                items.push_back(point(mesh.position(vertex->vertex), HoverVertexSize, hover));
            }
        } else if (const Element::Edge* edge = base::as<Element::Edge>(*editor.hover)) {
            push_edge(mesh_edge(document, edge->edge), HoverWidth, hover, items);
        }
    }

    if (xray) {
        through.insert(through.end(), cage_items.begin(), cage_items.end());
    } else {
        tested.insert(tested.end(), cage_items.begin(), cage_items.end());
    }

    // The loop cut's preview shows through the mesh, as it goes around it.
    if (editor.tool == Tool::LoopCut) {
        Color color = theme.data.orange;
        for (const std::array<Vector3, 2>& edge : look.preview) {
            through.push_back(line(edge[0], edge[1], SelectedEdgeWidth, color));
        }
    }
}

} // namespace mesh_modeler::overlay
