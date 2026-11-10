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

#pragma once

#include "Benchmarks/Harness.h"
#include "Benchmarks/Memory.h"

namespace benchmarks::footprints {

// Every program's heap: what it starts with, and what a frame churns.
memory::Report frames(Budget budget);

// The caches that every program's frames fill: the text layouts and their
// glyphs, and what the interface keeps for widgets by name.
memory::Report caches(Budget budget);

// The libraries' data: a mesh and what's built from it, a model, and a
// scene.
memory::Report libraries(Budget budget);

// How much GPU memory a program's drawing takes: the frame's target, the
// glyph atlas, the upload buffer, the program's own meshes and depth buffer,
// and what the driver keeps for them. Each program's row is what it added
// over an idle device with the frame's target, and it goes again before the
// next one.
memory::Report video(Budget budget);

} // namespace benchmarks::footprints
