// The geometry of a polygon mesh modeler, with no interface or GPU, so that
// programs and tests use it alike. README.md describes it.
//
// - Mesh.h: positions, faces with materials, and creased edges.
// - Topology.h: edges and their faces, boundaries, valences, loops and rings.
// - Selection.h: selected vertices, edges and faces, and modes.
// - Primitives.h: a cube, a plane, a cylinder, a cone, a UV sphere and a torus.
// - Editing.h: moving, deleting, making faces, merging, normals, creases and
//   materials.
// - Extrusion.h: extruding faces and edges, and insetting faces.
// - Cutting.h: subdividing faces, and loop cuts.
// - Subdivision.h: Catmull-Clark subdivision surfaces, as OpenSubdiv makes them.
// - Triangulation.h: triangles for SimplePaint, with flat or smooth normals.
// - Picking.h: ray casts, the nearest vertex and edge, bounds and centers.
// - Files.h: JSON, and Wavefront OBJ.

#pragma once

#include "PolygonMesh/Cutting.h"
#include "PolygonMesh/Editing.h"
#include "PolygonMesh/Extrusion.h"
#include "PolygonMesh/Files.h"
#include "PolygonMesh/Mesh.h"
#include "PolygonMesh/Picking.h"
#include "PolygonMesh/Primitives.h"
#include "PolygonMesh/Selection.h"
#include "PolygonMesh/Subdivision.h"
#include "PolygonMesh/Topology.h"
#include "PolygonMesh/Triangulation.h"
