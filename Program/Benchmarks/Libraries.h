// The libraries' CPU work: Catmull-Clark subdivision, triangulation,
// topology and ray casts of Library/PolygonMesh, Library/Gltf reading the
// game's models, Library/Json with a scene file, Library/Math's matrices,
// and Library/PresentTiming's tracker following presents through Windows'
// events.

#pragma once

#include "Benchmarks/Harness.h"

#include <vector>

namespace benchmarks::libraries {

std::vector<Measurement> meshes(Budget budget);

std::vector<Measurement> models(Budget budget);

std::vector<Measurement> json(Budget budget);

std::vector<Measurement> math(Budget budget);

std::vector<Measurement> present_timing(Budget budget);

} // namespace benchmarks::libraries
