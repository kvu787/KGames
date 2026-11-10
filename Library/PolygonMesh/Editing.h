// A modeler's edits of a mesh, like Blender's in edit mode: moving vertices,
// deleting, removing loose vertices, making faces, merging, flipping and
// recalculating normals, creasing, and assigning materials. Extrusion.h and
// Cutting.h have the rest. An edit that changes which faces there are
// returns the selection that results, as Blender leaves it, and the others
// leave the selection as it was. Elements that aren't in the mesh are left
// out of an edit, and so is an edit with nothing to do.
//
// Edits that add vertices and faces add them at the end, and keep the
// numbers of the faces that they change in place, like an extruded region's.
// Edits that remove vertices or faces, deleting and merging, renumber the
// rest in order, without gaps.
//
// Mesh.h declares the edits, and says what each does. Here are the ones that
// take any range of vertices, edges or faces: each keeps those that are in
// the mesh, each once, and does the edit with them, in Editing.cpp.

#pragma once

#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Selection.h"

#include <cstdint>
#include <expected>
#include <string>

namespace polygon_mesh {

template <RangeOf<std::uint32_t> Vertices> void Mesh::translate(Vertices&& vertices, math::Vector3 offset) {
    translate_impl(vertex_set(vertices), offset);
}

template <RangeOf<std::uint32_t> Vertices> void Mesh::rotate(Vertices&& vertices, math::Vector3 pivot, math::Vector3 axis, float angle) {
    rotate_impl(vertex_set(vertices), pivot, axis, angle);
}

template <RangeOf<std::uint32_t> Vertices> void Mesh::scale(Vertices&& vertices, math::Vector3 pivot, math::Vector3 factors) {
    scale_impl(vertex_set(vertices), pivot, factors);
}

template <RangeOf<std::uint32_t> Vertices> void Mesh::transform(Vertices&& vertices, math::Vector3 pivot, const math::Matrix& matrix) {
    transform_impl(vertex_set(vertices), pivot, matrix);
}

template <RangeOf<std::uint32_t> Vertices> Selection Mesh::delete_vertices(Vertices&& vertices) {
    return delete_vertices_impl(vertex_set(vertices));
}

template <RangeOf<Edge> Edges> Selection Mesh::delete_edges(Edges&& edges) {
    return delete_edges_impl(edge_set(edges));
}

template <RangeOf<size_t> Faces> Selection Mesh::delete_faces(Faces&& faces) {
    return delete_faces_impl(face_set(faces));
}

template <RangeOf<size_t> Faces> Selection Mesh::delete_only_faces(Faces&& faces) {
    return delete_only_faces_impl(face_set(faces));
}

template <RangeOf<std::uint32_t> Vertices> std::expected<Selection, std::string> Mesh::make_face(Vertices&& vertices) {
    return make_face_impl(vertex_set(vertices));
}

template <RangeOf<Edge> Edges> std::expected<Selection, std::string> Mesh::fill(Edges&& edges) {
    return fill_impl(edge_set(edges));
}

template <RangeOf<std::uint32_t> Vertices> Selection Mesh::merge_at_center(Vertices&& vertices) {
    return merge_at_center_impl(vertex_set(vertices));
}

template <RangeOf<std::uint32_t> Vertices> Selection Mesh::merge_by_distance(Vertices&& vertices, float distance) {
    return merge_by_distance_impl(vertex_set(vertices), distance);
}

template <RangeOf<size_t> Faces> void Mesh::flip_faces(Faces&& faces) {
    flip_faces_impl(face_set(faces));
}

template <RangeOf<size_t> Faces> void Mesh::recalculate_normals_outside(Faces&& faces) {
    recalculate_normals_outside_impl(face_set(faces));
}

template <RangeOf<Edge> Edges> void Mesh::crease_edges(Edges&& edges, float sharpness) {
    crease_edges_impl(edge_set(edges), sharpness);
}

template <RangeOf<size_t> Faces> void Mesh::assign_material(Faces&& faces, std::uint32_t material) {
    assign_material_impl(face_set(faces), material);
}

} // namespace polygon_mesh
