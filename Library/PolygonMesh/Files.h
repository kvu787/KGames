// Meshes in files: JSON, which keeps all of a mesh and reads well, with each
// position, face and crease on its own line, and Wavefront OBJ, for other
// programs, which keeps the faces and their materials:
//
//     {
//         "Positions": [
//             [-1, -1, -1],
//             [1, -1, -1],
//             ...
//         ],
//         "Faces": [
//             [0, 3, 2, 1],
//             ...
//         ],
//         "Materials": [0, 0, 0, 0, 0, 0],
//         "Creases": [
//             [2, 3, 2.5]
//         ]
//     }
//
// Faces are lists of positions' numbers, from 0, counterclockwise when seen
// from the front. Materials has one material for each face, and Creases an
// edge's two vertices and its sharpness for each creased edge. Materials and
// Creases can be left out, for faces of material 0 and no creases.
//
// Mesh.h declares Mesh's to_json, from_json, save, load and to_obj, which
// Files.cpp defines.

#pragma once

#include "PolygonMesh/Mesh.h"
