// The mesh: positions, faces of any number of sides with a material each,
// and the sharpness of creased edges.
//
// Mesh has every operation of the library that's a mesh's own, as Rust's
// `impl Mesh` blocks gave it one in each module. Each is declared here, under
// the name of the file that defines it, like Editing.cpp. An operation that
// takes some of the mesh's vertices, edges or faces takes any range of them,
// so it's a template, which the file's header defines, like Editing.h:
// include PolygonMesh.h, which has them all.

#pragma once

#include "Math/Math.h"

#include <array>
#include <compare>
#include <concepts>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <functional>
#include <initializer_list>
#include <map>
#include <optional>
#include <ranges>
#include <set>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace polygon_mesh {

class Topology;
struct Hit;
struct Selection;
struct Shading;
struct SubdivisionSurface;
struct Triangles;

// Any sharpness of at least this is infinitely sharp, as in OpenSubdiv, where
// a crease loses 1 of sharpness each level, so 10 outlasts any level that a
// modeler would subdivide to.
inline constexpr float InfinitelySharp = 10.0f;

// Any range of T, or of what converts to one: a std::set, a std::vector, a
// std::array, a view like std::views::iota(0, 8), or a list in braces, like
// {3, 7}, which a parameter takes when its template's type defaults to
// std::initializer_list<T>. It's what `impl IntoIterator<Item = T>` is in
// Rust, for the vertices, edges or faces that an operation takes.
template <class Range, class T>
concept RangeOf = std::ranges::input_range<Range> && std::convertible_to<std::ranges::range_reference_t<Range>, T>;

// An undirected edge, by its two vertices, the smaller first, so that both
// faces next to an edge name it the same way.
class Edge {
public:
    Edge(std::uint32_t a, std::uint32_t b) : vertices_(a <= b ? std::array{a, b} : std::array{b, a}) {}

    // The smaller vertex, then the larger.
    std::array<std::uint32_t, 2> vertices() const {
        return vertices_;
    }

    bool contains(std::uint32_t vertex) const {
        return vertices_[0] == vertex || vertices_[1] == vertex;
    }

    // The end that isn't `vertex`.
    std::uint32_t other(std::uint32_t vertex) const {
        return vertices_[0] == vertex ? vertices_[1] : vertices_[0];
    }

    bool operator==(const Edge& edge) const {
        return key() == edge.key();
    }

    // By the smaller vertex, then by the larger.
    std::strong_ordering operator<=>(const Edge& edge) const {
        return key() <=> edge.key();
    }

private:
    friend struct std::hash<Edge>;

    // Both vertices as one number, the smaller in its upper half, which
    // orders edges as their vertices do, and compares in one step, where
    // MSVC compares two std::arrays an element at a time, many times slower.
    // Sets and maps of edges compare them all the time.
    std::uint64_t key() const {
        return (static_cast<std::uint64_t>(vertices_[0]) << 32) | vertices_[1];
    }

    std::array<std::uint32_t, 2> vertices_;
};

// How a failed check in a test shows an edge, like Edge([3, 7]).
std::string describe_value(Edge edge);

// A polygon, by its vertices, counterclockwise when seen from its front, the
// right-handed convention, and its material, which SimplePaint paints it with.
struct Face {
    std::vector<std::uint32_t> vertices;
    std::uint32_t material;

    Face(std::vector<std::uint32_t> vertices, std::uint32_t material) : vertices(std::move(vertices)), material(material) {}

    // Each side, from a vertex to the next, in the face's winding, as a
    // std::array of the two.
    auto sides() const {
        size_t count = vertices.size();
        return std::views::iota(size_t{0}, count) | std::views::transform([this, count](size_t index) { return std::array{vertices[index], vertices[(index + 1) % count]}; });
    }

    auto edges() const {
        return sides() | std::views::transform([](std::array<std::uint32_t, 2> side) { return Edge(side[0], side[1]); });
    }

    // Whether the face goes from a to b along one of its sides.
    bool has_side(std::uint32_t a, std::uint32_t b) const;

    // The same polygon wound the other way, from the same first vertex.
    void flip();

    bool operator==(const Face&) const = default;
};

std::string describe_value(const Face& face);

// A polygon mesh. Every face has at least 3 vertices, each in range and none
// twice, which `validate` checks and every operation keeps. A vertex that no
// face uses is a loose vertex. Edges are the faces' sides, so there are no
// edges without faces. `creases` has the sharpness of each creased edge,
// more than 0 and at most InfinitelySharp; an edge that isn't there is
// smooth. It copies, so that a modeler can undo by keeping copies.
struct Mesh {
    std::vector<math::Vector3> positions;
    std::vector<Face> faces;
    std::map<Edge, float> creases;

    std::uint32_t add_vertex(math::Vector3 position) {
        positions.push_back(position);
        return static_cast<std::uint32_t>(positions.size() - 1);
    }

    size_t add_face(std::vector<std::uint32_t> vertices, std::uint32_t material) {
        faces.emplace_back(std::move(vertices), material);
        return faces.size() - 1;
    }

    size_t vertex_count() const {
        return positions.size();
    }

    size_t face_count() const {
        return faces.size();
    }

    math::Vector3 position(std::uint32_t vertex) const {
        return positions[vertex];
    }

    Topology topology() const;

    // Each edge once, in order.
    std::set<Edge> edges() const;

    // An edge's sharpness, 0 if it isn't creased.
    float crease(Edge edge) const;

    // Sets an edge's sharpness, limited to 0 to InfinitelySharp, and NaN to
    // 0. Setting 0 removes the crease.
    void set_crease(Edge edge, float sharpness);

    // How many materials SimplePaint needs to paint the mesh: one more than
    // the largest index, and at least 1.
    std::uint32_t material_count() const;

    // The average of a face's vertices.
    math::Vector3 face_center(size_t face) const;

    // A face's area times its unit normal, by Newell's method, which works for
    // any polygon, even one that isn't flat or convex. Measuring from the
    // center keeps it precise far from the origin.
    math::Vector3 face_area_vector(size_t face) const;

    // The unit normal that the face's winding points to, or zero for a face
    // without area.
    math::Vector3 face_normal(size_t face) const;

    float face_area(size_t face) const;

    // Checks what every operation expects: finite positions, faces of at
    // least 3 vertices in range with none twice, and creases between two
    // vertices in range with sharpness in (0, InfinitelySharp].
    std::expected<void, std::string> validate() const;

    // Drops the creases of edges that no face has any more.
    void prune_creases();

    // The faces that are in the mesh, each once.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> std::set<size_t> face_set(Faces&& items) const {
        std::set<size_t> set;
        for (auto&& item : items) {
            size_t face = static_cast<size_t>(item);
            if (face < faces.size()) {
                set.insert(face);
            }
        }
        return set;
    }

    // The vertices that are in the mesh, each once.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> std::set<std::uint32_t> vertex_set(Vertices&& items) const {
        std::set<std::uint32_t> set;
        for (auto&& item : items) {
            std::uint32_t vertex = static_cast<std::uint32_t>(item);
            if (vertex < positions.size()) {
                set.insert(vertex);
            }
        }
        return set;
    }

    // The edges that some face has, each once.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> std::set<Edge> edge_set(Edges&& items) const {
        std::set<Edge> existing = edges();
        std::set<Edge> set;
        for (auto&& item : items) {
            Edge edge = item;
            if (existing.contains(edge)) {
                set.insert(edge);
            }
        }
        return set;
    }

    // Editing.h and Editing.cpp: a modeler's edits. One that changes which
    // faces there are returns the selection that results.

    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> void translate(Vertices&& vertices, math::Vector3 offset);

    // Rotates by `angle` radians about `axis` through `pivot`, counterclockwise
    // when seen from where the axis points, like Math's rotations.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> void rotate(Vertices&& vertices, math::Vector3 pivot, math::Vector3 axis, float angle);

    // Scales by each of `factors` along X, Y and Z, about `pivot`. A negative
    // factor mirrors the vertices, which turns their faces inside out until
    // they're flipped.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> void scale(Vertices&& vertices, math::Vector3 pivot, math::Vector3 factors);

    // Transforms by `matrix` about `pivot`: the pivot goes where the matrix
    // takes the origin, and the rest goes with it.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> void transform(Vertices&& vertices, math::Vector3 pivot, const math::Matrix& matrix);

    // Deletes the vertices and every face that uses one, like Blender's
    // Delete Vertices. The faces' other vertices stay, loose if no face is
    // left that uses them.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> Selection delete_vertices(Vertices&& vertices);

    // Deletes the faces that have one of the edges, and the edges' vertices
    // that no face is left using, like Blender's Delete Edges.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> Selection delete_edges(Edges&& edges);

    // Deletes the faces, and their vertices that no face is left using, like
    // Blender's Delete Faces.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection delete_faces(Faces&& faces);

    // Deletes the faces and nothing else, like Blender's Only Faces. Their
    // vertices stay, loose if no face is left that uses them. A mesh has no
    // edges without faces, so the edges that only these faces had go too.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection delete_only_faces(Faces&& faces);

    // Removes the vertices that no face uses, and returns each old vertex's
    // new number, or nothing for one that's gone.
    std::vector<std::optional<std::uint32_t>> remove_unused_vertices();

    // Makes a face of the vertices, like Blender's F. If their boundary edges
    // go around them in one loop, the face follows it, wound against the
    // faces next to it, filling the hole. Otherwise it goes around their
    // middle, in the plane that they spread across most, wound the way that
    // agrees with most of the faces that it shares an edge with, or else
    // facing away from the rest of the mesh. Its material is a neighbor's.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> std::expected<Selection, std::string> make_face(Vertices&& vertices);

    // Fills each hole that the boundary edges go all the way around with a
    // face, wound against the faces around it, like F on a loop of edges in
    // Blender. Edges that aren't on the boundary are left out.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> std::expected<Selection, std::string> fill(Edges&& edges);

    // Merges the vertices into one at their average, which keeps the
    // smallest of their numbers before the rest are renumbered. Faces lose
    // the sides that collapse, and faces left with fewer than 3 vertices go.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> Selection merge_at_center(Vertices&& vertices);

    // Merges each of the vertices into the first, by number, within
    // `distance` of it, like Blender's Merge by Distance: the first stays
    // where it is. Returns the selected vertices that are left.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> Selection merge_by_distance(Vertices&& vertices, float distance);

    // Replaces each vertex with its representative in faces and creases,
    // removes the vertices that were replaced, and returns each old vertex's
    // new number. Faces lose the sides that collapse, split where they come
    // to touch themselves, and go when they're left with fewer than 3
    // vertices, or with the same vertices as a face before them.
    std::vector<std::optional<std::uint32_t>> weld(std::span<const std::uint32_t> representative);

    // Removes the faces, and the creases of edges that no face has any more.
    void remove_faces(const std::set<size_t>& removed);

    // Removes the vertices, and any face or crease that uses one, renumbers
    // the rest in order, and returns each old vertex's new number.
    std::vector<std::optional<std::uint32_t>> remove_vertices(const std::set<std::uint32_t>& removed);

    // Reverses the faces' winding, so they face the other way.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> void flip_faces(Faces&& faces);

    // Winds the faces consistently, and each surface that they make so that
    // it faces out, like Blender's Recalculate Outside. Faces that share an
    // edge with only each other agree on their winding, and then a closed
    // surface faces out when it encloses a positive volume, and an open one
    // when its face farthest from its middle faces away from it.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> void recalculate_normals_outside(Faces&& faces);

    // Sets the edges' sharpness, from 0, smooth, to InfinitelySharp.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> void crease_edges(Edges&& edges, float sharpness);

    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> void assign_material(Faces&& faces, std::uint32_t material);

    // Extrusion.h and Extrusion.cpp: extruding and insetting.

    // Extrudes the faces as one region, like Blender's Extrude Region: the
    // region moves onto new vertices, with a quad along each edge of its
    // boundary between the old vertices and the new, of the material of the
    // face that it comes from. Vertices in the middle of the region, which
    // nothing else uses, move with it instead of being copied. A region that
    // shares no edge with other faces leaves a copy of itself behind, flipped,
    // so that a lone face becomes a closed box. Returns the region, at the
    // same face numbers, with its new vertices.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection extrude_faces(Faces&& faces);

    // Extrudes the edges, like Blender does in edge mode: each edge gets a quad
    // from it to a copy of it, wound to agree with the edge's first face, and
    // of that face's material, so that extruding a boundary continues its
    // surface. Edges that share a vertex share its copy. Returns the copies.
    template <RangeOf<Edge> Edges = std::initializer_list<Edge>> Selection extrude_edges(Edges&& edges);

    // Insets the faces as one region, like Blender's Inset Faces: each vertex
    // on the region's boundary gets a copy `thickness` inside it, in the
    // planes of its faces, and the region shrinks onto the copies, with a quad
    // of the face's material along each edge of the boundary, between the
    // old vertices and the new. Where the boundary turns, the copy goes
    // farther, so that it's `thickness` from both edges. Returns the region,
    // at the same face numbers.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection inset_region(Faces&& faces, float thickness);

    // Insets each face on its own, like Blender's Inset Faces with
    // Individual: a smaller copy of the face, `thickness` inside each of its
    // edges, in its plane, and a quad between each of its edges and the
    // copy's. Returns the copies, at the faces' numbers.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection inset_individual(Faces&& faces, float thickness);

    // Cutting.h and Cutting.cpp: subdividing faces, and loop cuts.

    // Splits each face in place, without smoothing: a triangle into 4 at its
    // edges' midpoints, and any other face into a quad at each corner, from
    // the corner to its edges' midpoints and the face's center. Both halves
    // of a creased edge keep its sharpness, and the pieces their face's
    // material. Returns the pieces, the first of each at its face's number.
    template <RangeOf<size_t> Faces = std::initializer_list<size_t>> Selection subdivide_faces(Faces&& faces);

    // Cuts a loop through the edge ring of `edge`, like Blender's Loop Cut:
    // a new vertex on each edge of the ring, `fraction` of the way along it
    // from `edge`'s first vertex, and the same way along the others, which
    // run beside it. Each quad that the ring crosses splits in two, and the
    // faces at its ends get the new vertices. Returns the new loop of edges.
    std::expected<Selection, std::string> loop_cut(Edge edge, float fraction);

    // Subdivision.cpp: Catmull-Clark subdivision.

    // The mesh subdivided `levels` times. Every face becomes a quad at each
    // corner, so after one level all faces are quads, of their face's
    // material. The base mesh's vertices keep their numbers at every level,
    // then come a vertex for each edge, in the topology's order, then one for
    // each face. Loose vertices stay where they are.
    SubdivisionSurface catmull_clark(std::uint32_t levels) const;

    // Triangulation.cpp: triangles for SimplePaint.

    Triangles triangulate(Shading shading) const;

    // A face's triangles, as places in its list of vertices, by ear clipping
    // in the plane of its normal, so that concave faces work, wound like the
    // face: n - 2 of them for a face of n vertices. A face without a normal,
    // or one that crosses itself, is cut as well as it can be.
    std::vector<std::array<size_t, 3>> face_triangles(size_t face) const;

    // Picking.h and Picking.cpp: rays, and where things are.

    // The nearest face that the ray from `origin` along `direction` hits,
    // from the front or the back, through the face's triangles, so that a
    // concave face is hit only where it is. Nothing if it hits none, or if
    // `direction` is zero.
    std::optional<Hit> ray_cast(math::Vector3 origin, math::Vector3 direction) const;

    // The vertex nearest the ray, of those within `radius` of it and in front
    // of `origin`, the nearer to `origin` of two as near. A modeler checks
    // whether a face hides it.
    std::optional<std::uint32_t> nearest_vertex(math::Vector3 origin, math::Vector3 direction, float radius) const;

    // The edge nearest the ray, of those within `radius` of it, in front of
    // `origin`, the nearer to `origin` of two as near.
    std::optional<Edge> nearest_edge(math::Vector3 origin, math::Vector3 direction, float radius) const;

    // The smallest box around every vertex, as its least and greatest
    // corners, or nothing for a mesh without vertices.
    std::optional<std::array<math::Vector3, 2>> bounds() const;

    // The average of the vertices' positions, or nothing for none.
    template <RangeOf<std::uint32_t> Vertices = std::initializer_list<std::uint32_t>> std::optional<math::Vector3> center_of(Vertices&& vertices) const;

    // Files.cpp: JSON, and Wavefront OBJ. Files.h describes the JSON.

    std::string to_json() const;

    // Reads a mesh from to_json's format, or says what's wrong with it.
    static std::expected<Mesh, std::string> from_json(std::string_view text);

    std::expected<void, std::string> save(const std::filesystem::path& path) const;

    static std::expected<Mesh, std::string> load(const std::filesystem::path& path);

    // The mesh as Wavefront OBJ: its positions, and its faces, each after a
    // usemtl of its material, Material0 for material 0, wherever the material
    // changes. OBJ has no creases, and counts vertices from 1.
    std::string to_obj() const;

    bool operator==(const Mesh&) const = default;

private:
    // What the templates above do with their vertices, edges or faces that
    // are in the mesh, each once, which they collect with vertex_set,
    // edge_set and face_set. Editing.cpp, Extrusion.cpp and Cutting.cpp
    // define them.
    void translate_impl(const std::set<std::uint32_t>& vertices, math::Vector3 offset);
    void rotate_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, math::Vector3 axis, float angle);
    void scale_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, math::Vector3 factors);
    void transform_impl(const std::set<std::uint32_t>& vertices, math::Vector3 pivot, const math::Matrix& matrix);
    Selection delete_vertices_impl(const std::set<std::uint32_t>& removed);
    Selection delete_edges_impl(const std::set<Edge>& edges);
    Selection delete_faces_impl(const std::set<size_t>& selected);
    Selection delete_only_faces_impl(const std::set<size_t>& selected);
    std::expected<Selection, std::string> make_face_impl(const std::set<std::uint32_t>& vertices);
    std::expected<Selection, std::string> fill_impl(const std::set<Edge>& edges);
    Selection merge_at_center_impl(const std::set<std::uint32_t>& vertices);
    Selection merge_by_distance_impl(const std::set<std::uint32_t>& vertices, float distance);
    void flip_faces_impl(const std::set<size_t>& selected);
    void recalculate_normals_outside_impl(const std::set<size_t>& selected);
    void crease_edges_impl(const std::set<Edge>& edges, float sharpness);
    void assign_material_impl(const std::set<size_t>& selected, std::uint32_t material);
    Selection extrude_faces_impl(const std::set<size_t>& region);
    Selection extrude_edges_impl(const std::set<Edge>& edges);
    Selection inset_region_impl(const std::set<size_t>& region, float thickness);
    Selection inset_individual_impl(const std::set<size_t>& selected, float thickness);
    Selection subdivide_faces_impl(const std::set<size_t>& selected);

    // Editing.cpp.

    // The loop that the vertices' boundary edges make, if they make one loop
    // through all of them, in the order that fills it: against each edge's
    // face.
    std::optional<std::vector<std::uint32_t>> boundary_cycle(const Topology& topology, const std::set<std::uint32_t>& vertices) const;

    // A boundary edge's vertices in the order that a face filling its hole
    // goes along it, against its one face.
    std::array<std::uint32_t, 2> filling_side(const Topology& topology, Edge edge) const;

    std::vector<std::uint32_t> around_middle(const Topology& topology, const std::set<std::uint32_t>& vertices) const;

    // The material of a face that shares a side with the vertices' loop, or
    // else of one that uses one of them, or 0.
    std::uint32_t neighbor_material(const Topology& topology, std::span<const std::uint32_t> order) const;

    // Removes those of the vertices that no face uses.
    void remove_unused(const std::set<std::uint32_t>& candidates);

    // Whether a consistently wound surface faces in.
    bool faces_in(std::span<const size_t> surface) const;

    // Cutting.cpp.

    // Gives both halves of each creased edge that's cut its sharpness.
    void split_creases(const std::map<Edge, std::uint32_t>& cuts);

    // Triangulation.cpp.

    // The fan that each corner is in, by the number of one of its corners:
    // corners at a vertex share a fan when they're joined through smooth
    // edges.
    std::vector<size_t> smooth_fans(std::span<const math::Vector3> normals, std::span<const size_t> first_corner, size_t corners, float angle) const;

    // The angle of a face's corner, in radians, which weighs its normal in
    // the smooth normal of the corner's vertex, as Blender weighs it.
    float corner_angle(size_t face, size_t place) const;
};

std::string describe_value(const Mesh& mesh);

// The average of the positions, or nothing if there are none.
template <RangeOf<math::Vector3> Positions> std::optional<math::Vector3> average(Positions&& positions) {
    std::array<double, 3> sum{};
    size_t count = 0;
    for (math::Vector3 position : positions) {
        for (size_t axis = 0; axis < 3; axis++) {
            sum[axis] += static_cast<double>(position[axis]);
        }
        count += 1;
    }
    if (count == 0) {
        return std::nullopt;
    }
    double divisor = static_cast<double>(count);
    return math::Vector3{static_cast<float>(sum[0] / divisor), static_cast<float>(sum[1] / divisor), static_cast<float>(sum[2] / divisor)};
}

// From a at t = 0 to b at t = 1.
constexpr math::Vector3 lerp(math::Vector3 a, math::Vector3 b, float t) {
    return math::add(a, math::scale(math::subtract(b, a), t));
}

} // namespace polygon_mesh

// Edges as the keys of a std::unordered_map or set.
template <> struct std::hash<polygon_mesh::Edge> {
    size_t operator()(polygon_mesh::Edge edge) const noexcept {
        return std::hash<std::uint64_t>{}(edge.key());
    }
};
