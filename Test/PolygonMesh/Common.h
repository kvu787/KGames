// What several of PolygonMesh's tests check meshes with.

#pragma once

#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <array>
#include <cstddef>
#include <expected>
#include <source_location>
#include <string>
#include <string_view>
#include <vector>

namespace common {

// The volume that the faces enclose, positive when they face out, from
// tetrahedra between the origin and a fan of each face's triangles.
float signed_volume(const polygon_mesh::Mesh& mesh);

void assert_close(float actual, float expected, float tolerance, std::string_view what);

void assert_near(std::array<float, 3> actual, std::array<float, 3> expected, float tolerance, std::string_view what);

// Checks the mesh's invariants and that its faces agree on which way is out.
void assert_valid(const polygon_mesh::Mesh& mesh, std::string_view what);

// How many faces have each number of sides, from 3.
std::vector<size_t> sides_histogram(const polygon_mesh::Mesh& mesh);

// The value of a result that must have one, or the test fails with what went
// wrong: unwrap.
template <class T> T unwrap(std::expected<T, std::string> result, std::source_location where = std::source_location::current()) {
    if (!result) {
        testing::fail("it went wrong: " + result.error(), where);
    }
    return std::move(*result);
}

inline void unwrap(const std::expected<void, std::string>& result, std::source_location where = std::source_location::current()) {
    if (!result) {
        testing::fail("it went wrong: " + result.error(), where);
    }
}

// What went wrong, of a result that must be that something did: unwrap_err.
template <class T> std::string unwrap_err(const std::expected<T, std::string>& result, std::source_location where = std::source_location::current()) {
    if (result) {
        testing::fail("it went well, where it should have gone wrong", where);
    }
    return result.error();
}

} // namespace common
