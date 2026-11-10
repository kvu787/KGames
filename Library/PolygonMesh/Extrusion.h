// Extruding faces and edges, and insetting faces, like Blender's. The new
// vertices start where the ones that they come from are, so a modeler moves
// the selection that these return, as Blender does after E.
//
// Mesh.h declares them, and says what each does. Each takes any range of
// faces or edges, keeps those that are in the mesh, each once, and does the
// edit with them, in Extrusion.cpp.

#pragma once

#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Selection.h"

namespace polygon_mesh {

template <RangeOf<size_t> Faces> Selection Mesh::extrude_faces(Faces&& faces) {
    return extrude_faces_impl(face_set(faces));
}

template <RangeOf<Edge> Edges> Selection Mesh::extrude_edges(Edges&& edges) {
    return extrude_edges_impl(edge_set(edges));
}

template <RangeOf<size_t> Faces> Selection Mesh::inset_region(Faces&& faces, float thickness) {
    return inset_region_impl(face_set(faces), thickness);
}

template <RangeOf<size_t> Faces> Selection Mesh::inset_individual(Faces&& faces, float thickness) {
    return inset_individual_impl(face_set(faces), thickness);
}

} // namespace polygon_mesh
