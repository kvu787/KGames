// The tracks: the curves that make their edges, Ribeye's shape, what's
// wrong with a track, reading one, and the ground that a track and a
// terrain make.

use std::path::Path;

use json::Value;
use simple_paint::validate_mesh;
use veehiicuul::collision::Footprint;
use veehiicuul::curve::{self, SAMPLES_PER_SPAN};
use veehiicuul::ground::{self, BARRIERS, COLOR_COUNT, ROAD, START_LINE, area, triangulate, with_holes};
use veehiicuul::scene::Scene;
use veehiicuul::terrain::Terrain;
use veehiicuul::track::{Pose, Track, inside};

fn ribeye() -> Scene {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul/Scenes/Ribeye.json");
    Scene::load(&path).unwrap_or_else(|problem| panic!("{problem}"))
}

fn near(a: [f64; 2], b: [f64; 2], tolerance: f64) -> bool {
    curve::distance(a, b) < tolerance
}

// A curve is smooth, near its control points, and the same all the way
// around: a square's is a rounded square inside it, and a circle's control
// points make a curve of about the circle's radius.
#[test]
fn a_curve_goes_around_near_its_control_points() {
    let square = [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]];
    let outline = curve::sample(&square);
    assert_eq!(outline.len(), 4 * SAMPLES_PER_SPAN);
    // At a control point's parameter, the curve is a sixth of the way to
    // each neighbor from it, and two thirds of it.
    assert!(near(curve::point(&square, 1.0), [5.0, 1.0], 1e-12));
    assert!(near(curve::point(&square, 5.0), [5.0, 1.0], 1e-12), "it goes around");
    // Inside its control points, though past where it is at their
    // parameters, between them.
    for point in &outline {
        assert!((0.0..=6.0).contains(&point[0]) && (0.0..=6.0).contains(&point[1]));
    }
    assert!(outline.iter().any(|point| point[0] > 5.5));
    for point in curve::sample(&curve::circle([10.0, -5.0], 20.0, 12)) {
        let radius = curve::distance(point, [10.0, -5.0]);
        assert!((radius - 20.0).abs() < 0.05, "{radius}");
    }
}

// A new control point goes after the one that starts the span nearest it,
// so that it bends the curve where it's put.
#[test]
fn a_new_control_point_goes_into_the_nearest_span() {
    let square = [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]];
    // The span from parameter 1 to 2 is pulled most by points 1 and 2,
    // along X = 5.
    assert_eq!(curve::insertion(&square, [7.0, 3.0]), 2);
    assert_eq!(curve::insertion(&square, [3.0, -1.0]), 1);
}

// Ribeye's curves make a road with two islands, whose shape the car at the
// start doesn't touch, with the start line from edge to edge, and nothing
// wrong.
#[test]
fn ribeye_s_curves_make_its_road_and_start_line() {
    let scene = ribeye();
    let track = &scene.track;
    let counts: Vec<usize> = track.curves().map(Vec::len).collect();
    assert_eq!(counts, [64, 28, 16]);
    let shape = track.shape();
    assert_eq!(shape.road.len(), 64 * SAMPLES_PER_SPAN);
    assert_eq!(shape.edges.len(), (64 + 28 + 16) * SAMPLES_PER_SPAN);
    assert!(shape.problems.is_empty(), "{:?}", shape.problems);
    let footprint = Footprint::from_sides(1.5, 1.5, 2.9925, 3.0);
    assert!(!shape.edges.touch(&footprint, track.start.position, track.start.radians()));
    let line = shape.start_line;
    assert!((12.0..13.5).contains(&line.width), "{}", line.width);
    assert!(inside(&shape.road, line.center) && !inside(&shape.islands[0], line.center));
}

#[test]
fn says_what_is_wrong_with_a_track_s_shape() {
    let mut track = Track::round();
    assert!(track.shape().problems.is_empty(), "{:?}", track.shape().problems);
    track.start.position = [0.0, 0.0];
    assert_eq!(track.shape().problems, ["The start isn't on the road"]);
    track.start.position = [32.5, 0.0];
    track.islands.push(curve::circle([200.0, 0.0], 5.0, 6));
    assert_eq!(track.shape().problems, ["Island 2 is outside the road"]);
    track.islands.pop();
    // An island as big as the road crosses its edge.
    track.islands[0] = curve::circle([20.0, 0.0], 30.0, 10);
    let problems = track.shape().problems;
    assert!(problems.contains(&"The road's edge crosses island 1's".to_string()), "{problems:?}");
    // An island inside an island.
    track.islands[0] = curve::circle([20.0, 0.0], 6.0, 8);
    track.islands.push(curve::circle([20.0, 0.0], 2.0, 6));
    let problems = track.shape().problems;
    assert!(problems.contains(&"Island 2 is inside island 1".to_string()), "{problems:?}");
    track.islands.pop();
    // A curve that crosses itself, like a figure eight.
    track.islands[0] = vec![[-5.0, -5.0], [5.0, 5.0], [5.0, -5.0], [-5.0, 5.0]];
    let problems = track.shape().problems;
    assert!(problems.contains(&"Island 1's edge crosses itself".to_string()), "{problems:?}");
}

#[test]
fn reads_a_track_and_writes_it_back() {
    let track = Track::round();
    let value = track.to_value();
    let read = Track::parse(&value).unwrap();
    assert_eq!(read.road.len(), track.road.len());
    assert_eq!(read.start, Pose { position: [32.5, 0.0], heading: 180.0 });
    assert_eq!(read.to_value(), value);
    let with = |name: &str, member: Value| {
        let Value::Object(mut members) = value.clone() else { unreachable!() };
        members.retain(|(other, _)| other != name);
        members.push((name.to_string(), member));
        Track::parse(&Value::Object(members))
    };
    assert_eq!(with("Road", json::parse("[[0, 0], [1, 0]]").unwrap()).err().unwrap(), "Track.Road needs at least 3 points, not 2");
    assert_eq!(with("BarrierWidth", 0.0.into()).err().unwrap(), "Track.BarrierWidth must be above 0, not 0");
    assert_eq!(
        with("Hills", 1.0.into()).err().unwrap(),
        "Hills isn't part of the track. They're Start, Colors, BarrierWidth, BarrierLength, Road, Islands"
    );
}

// Twice a triangle's signed area: below 0 if it's wound counterclockwise
// seen from above, where X is right and -Z up.
fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

// The triangles that cover a polygon, which they cover once: their areas
// add up to its.
fn covered(outline: &[[f64; 2]]) -> f64 {
    let mut covered = 0.0;
    for [a, b, c] in triangulate(outline) {
        let turn = turn(outline[a], outline[b], outline[c]);
        assert!(turn <= 1e-9, "{:?} isn't counterclockwise", [a, b, c]);
        covered -= turn / 2.0;
    }
    covered
}

#[test]
fn triangles_cover_a_polygon_even_where_it_turns_in() {
    for outline in [
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        vec![[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]],
    ] {
        assert_eq!(triangulate(&outline).len(), outline.len() - 2);
        assert!((covered(&outline) - area(&outline).abs() / 2.0).abs() < 1e-9);
    }
}

// The road is its outline with its islands cut out: its triangles cover the
// outline's area less the islands'.
#[test]
fn the_road_has_its_islands_cut_out() {
    let square = |center: [f64; 2], size: f64| {
        vec![
            [center[0] - size, center[1] - size],
            [center[0] + size, center[1] - size],
            [center[0] + size, center[1] + size],
            [center[0] - size, center[1] + size],
        ]
    };
    let outline = square([0.0, 0.0], 10.0);
    let holes = vec![square([-4.0, 0.0], 2.0), square([5.0, 3.0], 1.0)];
    let joined = with_holes(&outline, &holes);
    assert_eq!(joined.len(), 4 + 2 * 6);
    assert!((covered(&joined) - (400.0 - 16.0 - 4.0)).abs() < 1e-6, "{}", covered(&joined));
    let shape = ribeye().track.shape();
    let road = with_holes(&shape.road, &shape.islands);
    let expected = (area(&shape.road).abs() - shape.islands.iter().map(|island| area(island).abs()).sum::<f64>()) / 2.0;
    assert!((covered(&road) - expected).abs() < 1e-3 * expected, "{} of {expected}", covered(&road));
}

// The track's mesh is the road, then an even number of barriers along each
// outline, then the start line, and SimplePaint's pipeline takes it.
#[test]
fn the_track_s_mesh_has_every_part_in_order() {
    let scene = ribeye();
    let shape = scene.track.shape();
    let mesh = ground::track_mesh(&scene.track, &shape);
    validate_mesh(&mesh.vertices, &mesh.indices, COLOR_COUNT as u32).unwrap();
    let materials: Vec<u32> = mesh.indices.chunks(3).map(|triangle| mesh.vertices[triangle[0] as usize].material_index).collect();
    let road = materials.iter().take_while(|&&material| material == ROAD).count();
    // A polygon with two holes has two more triangles than corners less 2.
    let corners = shape.road.len() + shape.islands.iter().map(Vec::len).sum::<usize>();
    assert_eq!(road, corners + 2 * 2 - 2);
    let barriers = shape.edges.len() * 2;
    assert!(materials[road..road + barriers].iter().all(|material| BARRIERS.contains(material)));
    assert!(materials[road + barriers..].iter().all(|material| START_LINE.contains(material)));
    assert_eq!(ground::colors(&scene.terrain, &scene.track)[ROAD as usize], [0x81 as f32 / 255.0, 0x85 as f32 / 255.0, 0x8B as f32 / 255.0]);
}

// Each painted hexel is four triangles of its palette's color.
#[test]
fn the_terrain_s_mesh_is_its_painted_hexels() {
    let mut terrain = Terrain::default();
    terrain.set((0, 0), Some(1));
    terrain.set((3, -2), Some(2));
    let mesh = ground::terrain_mesh(&terrain);
    assert_eq!((mesh.vertices.len(), mesh.indices.len()), (12, 24));
    assert_eq!(mesh.vertices[0].material_index, 1);
    validate_mesh(&mesh.vertices, &mesh.indices, COLOR_COUNT as u32).unwrap();
    assert_eq!(ground::colors(&terrain, &Track::round())[2], terrain.palette[2].map(|channel| channel as f32 / 255.0));
}
