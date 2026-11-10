#include "Veehiicuul/Ground.h"

#include "Base/Numbers.h"
#include "SimplePaint/Geometry.h"
#include "Veehiicuul/Files.h"

#include <algorithm>
#include <cmath>
#include <compare>
#include <limits>
#include <optional>
#include <utility>

namespace veehiicuul::ground {

using files::Color;
using files::srgb;
using meshes::Mesh;
using simple_paint::Vertex;
using terrain::Terrain;
using track::Shape;
using track::Track;

namespace {

// The start line's rows of squares, along the road.
constexpr size_t StartLineRows = 3;
constexpr Color Black{0, 0, 0};
constexpr Color White{255, 255, 255};

Mesh empty() {
    return Mesh();
}

Vertex vertex(std::array<double, 2> point, std::uint32_t material_index) {
    return Vertex{
        .position = {static_cast<float>(point[0]), 0.0f, static_cast<float>(point[1])},
        .normal = {0.0f, 1.0f, 0.0f},
        .material_index = material_index,
    };
}

// Twice the signed area of the triangle a, b, c, along X and Z: below 0 if
// they go counterclockwise seen from above, where X is right and -Z up.
double turn(std::array<double, 2> a, std::array<double, 2> b, std::array<double, 2> c) {
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
}

// A quadrilateral, as two triangles wound counterclockwise seen from above,
// like every triangle of the meshes, the right-handed convention for faces
// that face up.
void quad(Mesh& mesh, const std::array<std::array<double, 2>, 4>& corners, std::uint32_t material) {
    std::uint32_t first = static_cast<std::uint32_t>(mesh.vertices.size());
    for (std::array<double, 2> corner : corners) {
        mesh.vertices.push_back(vertex(corner, material));
    }
    for (std::array<size_t, 3> triangle : {std::array<size_t, 3>{0, 1, 2}, std::array<size_t, 3>{0, 2, 3}}) {
        size_t a = triangle[0];
        size_t b = triangle[1];
        size_t c = triangle[2];
        bool clockwise = turn(corners[a], corners[b], corners[c]) > 0.0;
        if (clockwise) {
            std::swap(b, c);
        }
        for (size_t corner : {a, b, c}) {
            mesh.indices.push_back(first + static_cast<std::uint32_t>(corner));
        }
    }
}

// A point of the polygon that `point`, inside it, can see: along a line
// toward +X to the nearest edge, then the end of that edge farther along X,
// or a point inside the triangle between them nearest that line.
std::optional<size_t> visible_point(std::span<const std::array<double, 2>> polygon, std::array<double, 2> point) {
    double x = point[0];
    double z = point[1];
    std::optional<std::pair<double, size_t>> nearest;
    for (size_t index = 0; index < polygon.size(); index++) {
        std::array<double, 2> a = polygon[index];
        std::array<double, 2> b = polygon[(index + 1) % polygon.size()];
        if ((a[1] > z) == (b[1] > z)) {
            continue;
        }
        double crossing = a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
        if (crossing >= x && (!nearest || crossing < nearest->first)) {
            nearest = std::pair(crossing, index);
        }
    }
    if (!nearest) {
        return std::nullopt;
    }
    auto [crossing, edge] = *nearest;
    size_t next = (edge + 1) % polygon.size();
    size_t candidate = polygon[edge][0] > polygon[next][0] ? edge : next;
    std::array<double, 2> hit{crossing, z};
    std::array<std::array<double, 2>, 3> triangle{point, hit, polygon[candidate]};
    // A flat triangle, when the line meets the edge at its end, has nothing
    // in it.
    double doubled_area = turn(triangle[0], triangle[1], triangle[2]);
    // 1 or -1, by its sign, as f64::signum gives.
    double sign = std::isnan(doubled_area) ? doubled_area : std::copysign(1.0, doubled_area);
    auto inside = [&](std::array<double, 2> p) {
        if (!(std::abs(doubled_area) > 1e-12)) {
            return false;
        }
        for (size_t corner = 0; corner < 3; corner++) {
            if (!(sign * turn(triangle[corner], triangle[(corner + 1) % 3], p) >= 0.0)) {
                return false;
            }
        }
        return true;
    };
    // The point in the triangle at the smallest angle from the line.
    size_t best = candidate;
    double best_angle = std::numeric_limits<double>::infinity();
    for (size_t index = 0; index < polygon.size(); index++) {
        std::array<double, 2> p = polygon[index];
        if (index == candidate || p == polygon[candidate] || !inside(p)) {
            continue;
        }
        double angle = std::abs(std::atan2(p[1] - z, p[0] - x));
        if (angle < best_angle) {
            best = index;
            best_angle = angle;
        }
    }
    // A point that an earlier cut made twice is the one whose corner the
    // cut goes into, so that the outline doesn't cross itself.
    size_t count = polygon.size();
    for (size_t index = 0; index < count; index++) {
        if (polygon[index] != polygon[best]) {
            continue;
        }
        std::array<double, 2> before = polygon[(index + count - 1) % count];
        std::array<double, 2> corner = polygon[index];
        std::array<double, 2> after = polygon[(index + 1) % count];
        // The outline goes counterclockwise seen from above, so its inside
        // is where turns are below 0.
        bool into = turn(before, corner, after) < 0.0 ? turn(before, corner, point) < 0.0 && turn(corner, after, point) < 0.0
                                                      : turn(before, corner, point) < 0.0 || turn(corner, after, point) < 0.0;
        if (into) {
            return index;
        }
    }
    return best;
}

// The road, with its islands cut out.
void road(Mesh& mesh, std::span<const std::array<double, 2>> outline, std::span<const std::vector<std::array<double, 2>>> islands) {
    std::vector<std::array<double, 2>> polygon = with_holes(outline, islands);
    std::uint32_t first = static_cast<std::uint32_t>(mesh.vertices.size());
    mesh.vertices.reserve(mesh.vertices.size() + polygon.size());
    for (std::array<double, 2> point : polygon) {
        mesh.vertices.push_back(vertex(point, Road));
    }
    for (std::array<size_t, 3> triangle : triangulate(polygon)) {
        for (size_t corner : triangle) {
            mesh.indices.push_back(first + static_cast<std::uint32_t>(corner));
        }
    }
}

// Barriers along an outline, `width` wide, beside it on the side away from
// the road: outside the road's outline, and inside an island's. They
// alternate between two colors, about `length` long each, an even number of
// them, so that they alternate all the way around.
void barriers(Mesh& mesh, std::span<const std::array<double, 2>> outline, bool outside, double width, double length) {
    size_t count = outline.size();
    auto point = [&](size_t index) {
        return outline[index % count];
    };
    // Each edge's normal, of length 1, away from the road.
    double sign = (area(outline) > 0.0) == outside ? 1.0 : -1.0;
    std::vector<std::array<double, 2>> normals;
    normals.reserve(count);
    for (size_t index = 0; index < count; index++) {
        std::array<double, 2> a = point(index);
        std::array<double, 2> b = point(index + 1);
        double x = b[0] - a[0];
        double z = b[1] - a[1];
        double edge_length = std::max(std::hypot(x, z), 1e-12);
        normals.push_back({sign * z / edge_length, -sign * x / edge_length});
    }
    // Where each point's barrier ends: along the mitered normal of its two
    // edges, so that the barrier is as wide all the way around.
    std::vector<std::array<double, 2>> offsets;
    offsets.reserve(count);
    for (size_t index = 0; index < count; index++) {
        std::array<double, 2> before = normals[(index + count - 1) % count];
        std::array<double, 2> after = normals[index];
        std::array<double, 2> sum{before[0] + after[0], before[1] + after[1]};
        double sum_length = std::hypot(sum[0], sum[1]);
        if (sum_length < 1e-9) {
            offsets.push_back({after[0] * width, after[1] * width});
            continue;
        }
        std::array<double, 2> miter{sum[0] / sum_length, sum[1] / sum_length};
        double scale = width / std::max(miter[0] * after[0] + miter[1] * after[1], 0.25);
        offsets.push_back({miter[0] * scale, miter[1] * scale});
    }
    std::vector<double> lengths;
    lengths.reserve(count);
    for (size_t index = 0; index < count; index++) {
        std::array<double, 2> a = point(index);
        std::array<double, 2> b = point(index + 1);
        lengths.push_back(std::hypot(b[0] - a[0], b[1] - a[1]));
    }
    // Added up from -0, as Rust's sum of floats is.
    double perimeter = -0.0;
    for (double edge_length : lengths) {
        perimeter += edge_length;
    }
    // At least 2, even if an outline with no length and barriers with none
    // give no number.
    double pieces = std::fmax(2.0 * std::round(perimeter / (2.0 * length)), 2.0);
    double along = 0.0;
    for (size_t index = 0; index < count; index++) {
        // The barrier that the middle of the edge is in.
        size_t piece = base::saturating_cast<size_t>((along + lengths[index] / 2.0) / perimeter * pieces);
        along += lengths[index];
        std::array<double, 2> a = point(index);
        std::array<double, 2> b = point(index + 1);
        std::array<double, 2> from = offsets[index];
        std::array<double, 2> to = offsets[(index + 1) % count];
        std::array<std::array<double, 2>, 4> corners{a, {a[0] + from[0], a[1] + from[1]}, {b[0] + to[0], b[1] + to[1]}, b};
        quad(mesh, corners, Barriers[piece % 2]);
    }
}

// The start line: squares of black and white across the road, at a right
// angle to the start's heading.
void start_line(Mesh& mesh, const Track& track, const Shape& shape) {
    std::array<double, 2> ahead = track.start.ahead();
    double sin = ahead[0];
    double cos = ahead[1];
    std::array<double, 2> across{cos, -sin};
    track::StartLine line = shape.start_line;
    double square = line.depth / static_cast<double>(StartLineRows);
    size_t columns = std::max(base::saturating_cast<size_t>(std::round(line.width / square)), size_t{1});
    std::array<double, 2> size{line.width / static_cast<double>(columns), square};
    auto place = [&](double column, double row) -> std::array<double, 2> {
        double x = column * size[0] - line.width / 2.0;
        double z = row * size[1] - line.depth / 2.0;
        return {
            line.center[0] + across[0] * x + ahead[0] * z,
            line.center[1] + across[1] * x + ahead[1] * z,
        };
    };
    for (size_t row = 0; row < StartLineRows; row++) {
        for (size_t column = 0; column < columns; column++) {
            double x = static_cast<double>(column);
            double z = static_cast<double>(row);
            std::array<std::array<double, 2>, 4> corners{place(x, z), place(x + 1.0, z), place(x + 1.0, z + 1.0), place(x, z + 1.0)};
            quad(mesh, corners, StartLine[(row + column) % 2]);
        }
    }
}

} // namespace

std::array<std::array<float, 3>, ColorCount> colors(const Terrain& terrain, const Track& track) {
    std::array<std::array<float, 3>, ColorCount> all{};
    for (size_t index = 0; index < std::min(terrain.palette.size(), terrain::PaletteSize); index++) {
        all[index] = srgb(terrain.palette[index]);
    }
    all[Road] = srgb(track.road_color);
    all[Barriers[0]] = srgb(track.barrier_colors[0]);
    all[Barriers[1]] = srgb(track.barrier_colors[1]);
    all[StartLine[0]] = srgb(Black);
    all[StartLine[1]] = srgb(White);
    return all;
}

Mesh terrain_mesh(const Terrain& terrain) {
    Mesh mesh = empty();
    for (const auto& [cell, color] : terrain.cells) {
        std::array<std::array<double, 2>, 6> corners = terrain.corners(cell);
        std::uint32_t first = static_cast<std::uint32_t>(mesh.vertices.size());
        for (std::array<double, 2> corner : corners) {
            mesh.vertices.push_back(vertex(corner, static_cast<std::uint32_t>(color)));
        }
        // A fan from the first corner, wound counterclockwise seen from
        // above.
        for (std::array<size_t, 2> pair : {std::array<size_t, 2>{1, 2}, std::array<size_t, 2>{2, 3}, std::array<size_t, 2>{3, 4}, std::array<size_t, 2>{4, 5}}) {
            size_t b = pair[0];
            size_t c = pair[1];
            bool clockwise = turn(corners[0], corners[b], corners[c]) > 0.0;
            if (clockwise) {
                std::swap(b, c);
            }
            for (std::uint32_t corner : {first, first + static_cast<std::uint32_t>(b), first + static_cast<std::uint32_t>(c)}) {
                mesh.indices.push_back(corner);
            }
        }
    }
    return mesh;
}

Mesh track_mesh(const Track& track, const Shape& shape) {
    Mesh mesh = empty();
    road(mesh, shape.road, shape.islands);
    barriers(mesh, shape.road, true, track.barrier_width, track.barrier_length);
    for (const std::vector<std::array<double, 2>>& island : shape.islands) {
        barriers(mesh, island, false, track.barrier_width, track.barrier_length);
    }
    start_line(mesh, track, shape);
    return mesh;
}

double area(std::span<const std::array<double, 2>> outline) {
    // Added up from -0, as Rust's sum of floats is.
    double doubled = -0.0;
    for (size_t index = 0; index < outline.size(); index++) {
        std::array<double, 2> a = outline[index];
        std::array<double, 2> b = outline[(index + 1) % outline.size()];
        doubled += a[0] * b[1] - b[0] * a[1];
    }
    return doubled;
}

std::vector<std::array<double, 2>> with_holes(std::span<const std::array<double, 2>> outline, std::span<const std::vector<std::array<double, 2>>> holes) {
    // The outline counterclockwise seen from above, and the holes the other
    // way, so that the cuts join them into one outline that goes one way.
    std::vector<std::array<double, 2>> polygon(outline.begin(), outline.end());
    if (area(polygon) > 0.0) {
        std::ranges::reverse(polygon);
    }
    std::vector<std::vector<std::array<double, 2>>> cut;
    for (const std::vector<std::array<double, 2>>& hole : holes) {
        if (hole.size() < 3) {
            continue;
        }
        std::vector<std::array<double, 2>> turned = hole;
        if (area(turned) < 0.0) {
            std::ranges::reverse(turned);
        }
        cut.push_back(std::move(turned));
    }
    // The last of the points farthest along X, in the order of every double,
    // as max_by with total_cmp gives.
    auto rightmost = [](const std::vector<std::array<double, 2>>& hole) {
        size_t best = 0;
        for (size_t index = 1; index < hole.size(); index++) {
            if (std::strong_order(hole[best][0], hole[index][0]) <= 0) {
                best = index;
            }
        }
        return best;
    };
    // Rightmost first, so that each cut only crosses the outline.
    std::ranges::stable_sort(cut, [&](const std::vector<std::array<double, 2>>& a, const std::vector<std::array<double, 2>>& b) {
        return std::strong_order(b[rightmost(b)][0], a[rightmost(a)][0]) < 0;
    });
    for (const std::vector<std::array<double, 2>>& hole : cut) {
        size_t m = rightmost(hole);
        std::array<double, 2> point = hole[m];
        std::optional<size_t> visible = visible_point(polygon, point);
        if (!visible) {
            continue;
        }
        std::vector<std::array<double, 2>> joined;
        joined.reserve(polygon.size() + hole.size() + 2);
        joined.insert(joined.end(), polygon.begin(), polygon.begin() + static_cast<std::ptrdiff_t>(*visible) + 1);
        joined.insert(joined.end(), hole.begin() + static_cast<std::ptrdiff_t>(m), hole.end());
        joined.insert(joined.end(), hole.begin(), hole.begin() + static_cast<std::ptrdiff_t>(m) + 1);
        joined.push_back(polygon[*visible]);
        joined.insert(joined.end(), polygon.begin() + static_cast<std::ptrdiff_t>(*visible) + 1, polygon.end());
        polygon = std::move(joined);
    }
    return polygon;
}

std::vector<std::array<size_t, 3>> triangulate(std::span<const std::array<double, 2>> outline) {
    std::vector<size_t> corners(outline.size());
    for (size_t index = 0; index < corners.size(); index++) {
        corners[index] = index;
    }
    // Counterclockwise, so that corners that turn toward the inside turn
    // below 0.
    if (area(outline) > 0.0) {
        std::ranges::reverse(corners);
    }
    std::vector<std::array<size_t, 3>> triangles;
    triangles.reserve(base::saturating_sub(outline.size(), size_t{2}));
    size_t index = 0;
    // How many corners in a row weren't ears, to stop at a polygon that has
    // none, which only one whose edges cross can be.
    size_t misses = 0;
    while (corners.size() > 3) {
        size_t count = corners.size();
        size_t previous = corners[(index + count - 1) % count];
        size_t corner = corners[index];
        size_t next = corners[(index + 1) % count];
        std::array<double, 2> a = outline[previous];
        std::array<double, 2> b = outline[corner];
        std::array<double, 2> c = outline[next];
        bool ear = turn(a, b, c) < 0.0 && std::ranges::all_of(
                                              corners,
                                              [&](size_t other) {
                                                  std::array<double, 2> point = outline[other];
                                                  return other == previous || other == corner || other == next || point == a || point == b || point == c ||
                                                         !(turn(a, b, point) <= 0.0 && turn(b, c, point) <= 0.0 && turn(c, a, point) <= 0.0);
                                              });
        if (ear || misses > count) {
            triangles.push_back({previous, corner, next});
            corners.erase(corners.begin() + static_cast<std::ptrdiff_t>(index));
            index = (index + count - 2) % (count - 1);
            misses = 0;
        } else {
            index = (index + 1) % count;
            misses += 1;
        }
    }
    if (corners.size() == 3) {
        triangles.push_back({corners[0], corners[1], corners[2]});
    }
    return triangles;
}

} // namespace veehiicuul::ground
