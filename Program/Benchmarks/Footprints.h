// What the programs and the libraries hold in memory, and what they churn
// through a frame or an edit: each program's heap after it starts and per
// frame, the text cache, what the interface keeps for widgets that are gone,
// the meshes and the files, and how much GPU memory a program's drawing
// takes.
//
// The counts are of the C++ heap, from Allocator.cpp's operators.
// DirectWrite's layouts and Direct3D's resources are in their own heaps, so
// the text benchmarks also give the working set they add, and the GPU's
// memory comes from the device.
//
// Only `libraries` is here so far. `frames`, `caches` and `video` measure
// the programs and Library/UserInterface, and come with them.

#pragma once

#include "Benchmarks/Harness.h"
#include "Benchmarks/Memory.h"

namespace benchmarks::footprints {

// The libraries' data: a mesh and what's built from it, a model, and a
// scene.
memory::Report libraries(Budget budget);

} // namespace benchmarks::footprints
