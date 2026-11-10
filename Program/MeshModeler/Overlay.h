// What the view draws over the mesh: the floor's grid and axes, the cage's
// edges and vertices, the selected faces' tint, the element under the
// pointer, and the loop that the loop cut tool would cut. Each is an
// OverlayItem for Viewport, in the theme's colors: the accent for what's
// selected, as Fluent marks selection, and its data colors for the axes and
// creases.

#pragma once

#include "Math/Math.h"
#include "MeshModeler/Document.h"
#include "MeshModeler/Editing.h"
#include "MeshModeler/Viewport.h"
#include "PolygonMesh/PolygonMesh.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstdint>
#include <span>
#include <vector>

namespace mesh_modeler::overlay {

// The grid: lines a unit apart, out to this many units from the origin.
inline constexpr std::int32_t GridReach = 10;

viewport::OverlayItem line(math::Vector3 a, math::Vector3 b, float width, user_interface::Color color);

viewport::OverlayItem point(math::Vector3 at, float size, user_interface::Color color);

viewport::OverlayItem triangle(math::Vector3 a, math::Vector3 b, math::Vector3 c, user_interface::Color color);

// What the overlay shows, besides the document and the editor.
struct Look {
    const user_interface::Theme& theme;
    // The surface that's drawn, when the mesh is subdivided, or null, and
    // the triangles drawn, of it or of the mesh.
    const polygon_mesh::SubdivisionSurface* surface;
    const polygon_mesh::Triangles& triangles;
    bool grid;
    // The loop cut's preview's edges.
    std::span<const std::array<math::Vector3, 2>> preview;
};

// The floor's grid on the XZ plane, with the X axis in red and Z in blue.
void grid(const user_interface::Theme& theme, std::vector<viewport::OverlayItem>& items);

// Builds the overlay: what the mesh hides into `tested`, and what shows
// through it into `through`, which is everything in X-ray.
void build(const document::Document& document, const editing::Editor& editor, const Look& look, bool xray, std::vector<viewport::OverlayItem>& tested,
           std::vector<viewport::OverlayItem>& through);

} // namespace mesh_modeler::overlay
