// The tracks: reading Ribeye and what's wrong with other files, and the
// ground that a track's outlines make.

use std::path::{Path, PathBuf};

use meshes::car;
use simple_paint::validate_mesh;
use veehiicuul::collision::Footprint;
use veehiicuul::ground::{self, CURBS, GRASS, MATERIAL_COUNT, ROAD, START_LINE, triangulate};
use veehiicuul::track::Track;

fn tracks() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul/Tracks")
}

fn ribeye() -> Track {
    Track::load(&tracks().join("Ribeye.json")).unwrap_or_else(|problem| panic!("{problem}"))
}

#[test]
fn ribeye_is_a_road_with_two_islands_and_starts_on_its_line() {
    let track = ribeye();
    assert_eq!(track.name, "Ribeye");
    assert_eq!(track.road.len(), 512);
    assert_eq!(track.islands.iter().map(Vec::len).collect::<Vec<_>>(), [224, 64]);
    assert_eq!(track.edges.len(), 800);
    assert_eq!(track.start.position, [94.5437, -48.4802]);
    assert!((track.start.heading.to_degrees() + 104.267).abs() < 1e-9);
    assert_eq!((track.camera_height, track.follow_height), (150.0, 45.0));
    // A car at the start touches no edge.
    let footprint = Footprint::of_mesh(&car::VERTICES);
    assert!(!track.edges.touch(&footprint, track.start.position, track.start.heading));
    let all = Track::load_all(&tracks()).unwrap();
    assert_eq!(all.iter().map(|track| track.name.as_str()).collect::<Vec<_>>(), ["Ribeye"]);
}

// A track with all it needs, and an outline that's a square, for each
// test to change one part of.
const TRACK: &str = r##"{
    "Start": { "Position": [0, 0], "Heading": 90 },
    "StartLine": { "Position": [1, 0], "Width": 4, "Depth": 1 },
    "Camera": { "Center": [0, 0], "Height": 50 },
    "FollowHeight": 20,
    "Colors": { "Grass": "#00FF00", "Road": "#808080", "Curbs": ["#FF0000", "#FFFFFF"] },
    "CurbWidth": 1,
    "CurbLength": 5,
    "Road": [[-10, -10], [10, -10], [10, 10], [-10, 10]],
    "Islands": []
}"##;

#[test]
fn says_what_is_wrong_with_a_track() {
    let track = Track::parse("Square", TRACK).unwrap();
    assert_eq!(track.start.heading, std::f64::consts::FRAC_PI_2);
    assert_eq!(track.colors.road, [128.0 / 255.0; 3]);
    for (from, to, problem) in [
        ("\"Islands\": []", "\"Islands\": [], \"Hills\": 1", "Hills isn't part of a track. They're Start, StartLine, Camera, FollowHeight, Colors, CurbWidth, CurbLength, Road, Islands"),
        ("\"Road\": [[-10, -10], [10, -10], [10, 10], [-10, 10]],", "", "needs Road"),
        ("[[-10, -10], [10, -10], [10, 10], [-10, 10]]", "[[-10, -10], [10, -10]]", "Road needs at least 3 points, not 2"),
        ("[[-10, -10], [10, -10], [10, 10], [-10, 10]]", "[[-10, -10], [10, -10], [10, -10], [-10, 10]]", "Road[1] is the same point as the one after it"),
        ("[10, 10], [-10, 10]]", "[10, 10], [-10]]", "Road[3] must be a point, [X, Z], not a list of 1"),
        ("\"#808080\"", "\"gray\"", "Colors.Road must be a color like \"#1CDA61\", not \"gray\""),
        ("\"Height\": 50", "\"Height\": 0", "Camera.Height must be above 0, not 0"),
        ("\"Islands\": []", "\"Islands\": [[[0, 0], [1, 0]]]", "Islands[0] needs at least 3 points, not 2"),
    ] {
        assert_eq!(TRACK.matches(from).count(), 1, "{from}");
        match Track::parse("Square", &TRACK.replace(from, to)) {
            Ok(_) => panic!("{to} parsed"),
            Err(actual) => assert_eq!(actual, problem),
        }
    }
}

// Twice a triangle's signed area: below 0 if it's wound counterclockwise
// seen from above, where X is right and -Z up.
fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn area(outline: &[[f64; 2]]) -> f64 {
    (0..outline.len())
        .map(|index| {
            let [a, b] = [outline[index], outline[(index + 1) % outline.len()]];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .abs()
}

// Cutting a polygon into triangles covers it with one fewer than it has
// edges, less one, each wound counterclockwise seen from above, whichever
// way its outline goes.
fn check_triangles(outline: &[[f64; 2]]) {
    let triangles = triangulate(outline);
    assert_eq!(triangles.len(), outline.len() - 2);
    let mut covered = 0.0;
    for [a, b, c] in triangles {
        let turn = turn(outline[a], outline[b], outline[c]);
        assert!(turn < 0.0, "{:?} isn't counterclockwise", [a, b, c]);
        covered -= turn;
    }
    assert!((covered - area(outline)).abs() < 1e-6 * area(outline), "{covered} of {}", area(outline));
}

#[test]
fn triangles_cover_a_polygon_even_where_it_turns_in() {
    let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    check_triangles(&square);
    let mut reversed = square;
    reversed.reverse();
    check_triangles(&reversed);
    // An L, which turns in at one corner, and a comb, at several.
    check_triangles(&[[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]]);
    check_triangles(&[
        [0.0, 0.0],
        [5.0, 0.0],
        [5.0, 3.0],
        [4.0, 3.0],
        [4.0, 1.0],
        [3.0, 1.0],
        [3.0, 3.0],
        [2.0, 3.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 3.0],
        [0.0, 3.0],
    ]);
    let track = ribeye();
    check_triangles(&track.road);
    for island in &track.islands {
        check_triangles(island);
    }
}

// Ribeye's ground is the road, the islands' grass, an even number of curbs
// along each outline, and the start line's 25 by 3 squares, and it's what
// SimplePaint's pipeline takes.
#[test]
fn ribeye_s_ground_has_every_part_in_order() {
    let track = ribeye();
    let ground = ground::mesh(&track);
    validate_mesh(&ground.vertices, &ground.indices, MATERIAL_COUNT as u32).unwrap();
    let materials: Vec<u32> = ground
        .indices
        .chunks(3)
        .map(|triangle| ground.vertices[triangle[0] as usize].material_index)
        .collect();
    let road = 510;
    let islands = 222 + 62;
    let curbs = 800 * 2;
    let squares = 25 * 3 * 2;
    assert_eq!(materials.len(), road + islands + curbs + squares);
    assert!(materials[..road].iter().all(|&material| material == ROAD));
    assert!(materials[road..road + islands].iter().all(|&material| material == GRASS));
    let curb_materials = &materials[road + islands..road + islands + curbs];
    assert!(curb_materials.iter().all(|material| CURBS.contains(material)));
    // Each outline's curbs alternate, starting and ending on different
    // colors, so that they alternate all the way around.
    let mut start = 0;
    for outline in [512, 224, 64] {
        let edges = &curb_materials[start..start + outline * 2];
        let changes = edges.windows(2).filter(|pair| pair[0] != pair[1]).count();
        assert!(changes >= 2 && changes % 2 == 1, "{changes} changes along an outline of {outline}");
        start += outline * 2;
    }
    let squares = &materials[road + islands + curbs..];
    assert!(squares.iter().all(|material| START_LINE.contains(material)));
    assert_eq!(ground::colors(&track)[ROAD as usize], track.colors.road);
}
