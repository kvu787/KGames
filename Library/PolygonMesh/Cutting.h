// Cutting faces into more: subdividing them where they are, and loop cuts,
// like Blender's Subdivide and Loop Cut. A face next to the cut faces gets
// the new vertices on the edges that it shares with them, so that no edge
// ends partway along another, as Blender does with Create N-Gons.
//
// Mesh.h declares them, and says what each does. subdivide_faces takes any
// range of faces, keeps those that are in the mesh, each once, and
// subdivides them, in Cutting.cpp, which has loop_cut too.

#pragma once

#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Selection.h"

namespace polygon_mesh {

template <RangeOf<size_t> Faces> Selection Mesh::subdivide_faces(Faces&& faces) {
    return subdivide_faces_impl(face_set(faces));
}

} // namespace polygon_mesh
