// The hexel terrain: where cells are, which cell a point is in, brushes and
// fills, its palette, and its rows of characters in a scene's file.

use json::Value;
use veehiicuul::terrain::{FARTHEST_CELL, Terrain, disk, distance, neighbors};

fn terrain(flat_top: bool) -> Terrain {
    Terrain {
        flat_top,
        ..Terrain::default()
    }
}

// A cell's middle is in it, and so is each point a little inside each of its
// corners, with pointy tops or flat ones.
#[test]
fn a_point_is_in_the_cell_whose_middle_is_nearest() {
    for flat_top in [false, true] {
        let terrain = terrain(flat_top);
        for cell in [(0, 0), (3, -2), (-7, 11), (12, 5)] {
            let center = terrain.center(cell);
            assert_eq!(terrain.cell_at(center), cell);
            for corner in terrain.corners(cell) {
                let inside = [center[0] + (corner[0] - center[0]) * 0.95, center[1] + (corner[1] - center[1]) * 0.95];
                assert_eq!(terrain.cell_at(inside), cell, "{flat_top} {cell:?}");
            }
            // Neighbors are as far apart as a cell is wide.
            for neighbor in neighbors(cell) {
                let other = terrain.center(neighbor);
                let apart = (other[0] - center[0]).hypot(other[1] - center[1]);
                assert!((apart - terrain.size * 3.0_f64.sqrt()).abs() < 1e-9);
            }
        }
    }
    // Pointy tops: a corner straight up and down along Z.
    let corners = terrain(false).corners((0, 0));
    assert!(corners.iter().any(|corner| corner[0].abs() < 1e-9 && (corner[1] - 2.0).abs() < 1e-9));
}

#[test]
fn a_brush_is_the_cells_within_its_radius() {
    for (radius, count) in [(0, 1), (1, 7), (2, 19), (3, 37)] {
        let cells = disk((4, -1), radius);
        assert_eq!(cells.len(), count);
        assert!(cells.iter().all(|&cell| distance(cell, (4, -1)) <= radius));
    }
}

// A fill takes the cells joined to the first through cells of its color,
// and stops at cells of other colors, or its reach.
#[test]
fn a_fill_stops_at_other_colors_and_its_reach() {
    let mut terrain = terrain(false);
    for cell in disk((0, 0), 3) {
        if distance(cell, (0, 0)) == 3 {
            terrain.set(cell, Some(2));
        }
    }
    let inside = terrain.region((0, 0), 60);
    assert_eq!(inside.len(), 19);
    let ring = terrain.region((3, 0), 60);
    assert_eq!(ring.len(), 18);
    // Outside the ring, unpainted cells go on, as far as the reach.
    assert_eq!(terrain.region((10, 0), 2).len(), 19);
}

#[test]
fn removing_a_color_takes_it_off_its_cells() {
    let mut terrain = terrain(false);
    terrain.set((0, 0), Some(0));
    terrain.set((1, 0), Some(1));
    terrain.set((2, 0), Some(2));
    terrain.remove_color(1);
    assert_eq!(terrain.palette.len(), 2);
    assert_eq!(terrain.get((0, 0)), Some(0));
    assert_eq!(terrain.get((1, 0)), None);
    assert_eq!(terrain.get((2, 0)), Some(1), "the colors after it keep their colors");
}

// The cells are rows of characters from the origin, which read back the
// same.
#[test]
fn writes_its_cells_as_rows_and_reads_them_back() {
    let mut terrain = terrain(true);
    terrain.set((-2, 5), Some(0));
    terrain.set((1, 5), Some(2));
    terrain.set((0, 7), Some(1));
    let value = terrain.to_value();
    assert_eq!(value.get("Origin"), Some(&Value::Array(vec![(-2.0).into(), 5.0.into()])));
    assert_eq!(value.get("Rows"), Some(&json::parse(r#"["0..2", "", "..1"]"#).unwrap()));
    assert_eq!(value.get("Orientation"), Some(&"FlatTop".into()));
    assert_eq!(Terrain::parse(&value).unwrap(), terrain);
    let text = r##"{ "Size": 2, "Orientation": "PointyTop", "Palette": ["#000000"], "Origin": [0, 0], "Rows": ["01"] }"##;
    assert_eq!(
        Terrain::parse(&json::parse(text).unwrap()).err().unwrap(),
        "Terrain.Rows[0] has '1', which isn't one of the palette's 1 colors"
    );
}

// A cell is FARTHEST_CELL at most from cell [0, 0], along each axis: a point
// far away, or any point of a terrain of tiny cells, is in the farthest
// cell, a brush there paints no cell past it, and a file whose cells are
// past it is wrong. So the cells around any cell have coordinates that an
// i32 holds, and a terrain's rows are a few thousand characters at most.
#[test]
fn cells_are_no_farther_than_the_farthest_cell() {
    const FAR: i32 = FARTHEST_CELL;
    let mut tiny = terrain(false);
    tiny.size = 1e-300;
    let far = tiny.cell_at([1.0, 0.0]);
    assert_eq!(far, (FAR, 0));
    assert_eq!(tiny.cell_at([-1.0, 0.0]), (-FAR, 0));
    assert_eq!(tiny.cell_at([1.0, 1.0]), (FAR, FAR));
    assert_eq!(terrain(true).cell_at([-1e300, 1e300]), (FAR, -FAR));
    // A brush and a fill there are the cells that they are anywhere.
    for cell in [far, (-FAR, FAR)] {
        for neighbor in neighbors(cell) {
            assert_eq!(distance(neighbor, cell), 1);
        }
        let brush = disk(cell, 8);
        assert_eq!(brush.len(), 217);
        assert!(brush.iter().all(|under| distance(*under, cell) <= 8));
        let filled = tiny.region(cell, 60);
        assert_eq!(filled.len(), 10981);
        assert!(filled.iter().all(|under| distance(*under, cell) <= 60));
    }
    // Painting with a brush there leaves the cells past the farthest
    // unpainted, so that the terrain reads back from what it writes.
    let mut painted = terrain(false);
    for under in disk((FAR, -FAR), 2) {
        painted.set(under, Some(1));
    }
    assert_eq!(painted.cells.len(), 9);
    assert_eq!(painted.get((FAR - 2, -FAR + 2)), Some(1));
    assert_eq!(painted.get((FAR + 1, -FAR)), None);
    assert_eq!(Terrain::parse(&painted.to_value()).unwrap(), painted);
    // Erasing past it is nothing to do, and not a mistake.
    painted.set((FAR + 1, -FAR), None);
    assert_eq!(painted.cells.len(), 9);

    let problem = |origin: &str, rows: &str| {
        let text = format!(r##"{{ "Size": 2, "Orientation": "PointyTop", "Palette": ["#000000"], "Origin": {origin}, "Rows": {rows} }}"##);
        match Terrain::parse(&json::parse(&text).unwrap()) {
            Ok(_) => "It reads".to_string(),
            Err(error) => error,
        }
    };
    assert_eq!(problem("[2147483647, 2147483647]", r#"["00", "00"]"#), "Terrain.Origin must be within 1024 cells of [0, 0], not [2147483647, 2147483647]");
    assert_eq!(problem("[0, -1025]", r#"["0"]"#), "Terrain.Origin must be within 1024 cells of [0, 0], not [0, -1025]");
    assert_eq!(problem("[1e300, 0]", "[]"), format!("Terrain.Origin must be within 1024 cells of [0, 0], not [1{}, 0]", "0".repeat(300)));
    assert_eq!(problem("[1024, 0]", r#"["00"]"#), "Terrain.Rows[0] goes past 1024 cells from [0, 0]");
    assert_eq!(problem("[0, 1024]", r#"["0", "", "0"]"#), "Terrain.Rows[2] goes past 1024 cells from [0, 0]");
    // Up to it, they read.
    assert_eq!(problem("[1023, -1024]", r#"["00", ".."]"#), "It reads");
}

// Save writes every cell between a terrain's first and its last, so cells at
// both ends make the most that a terrain's rows can be, which is a file that
// reads back.
#[test]
fn a_terrain_with_cells_at_both_ends_writes_rows_that_read_back() {
    let mut wide = terrain(false);
    wide.set((-FARTHEST_CELL, -FARTHEST_CELL), Some(0));
    wide.set((FARTHEST_CELL, FARTHEST_CELL), Some(2));
    // Where the brush can't paint, there's nothing to write.
    wide.set((i32::MAX, i32::MAX), Some(1));
    wide.set((i32::MIN, i32::MIN), Some(1));
    let value = wide.to_value();
    let Some(Value::Array(rows)) = value.get("Rows") else { panic!("no rows") };
    let across = 2 * FARTHEST_CELL as usize + 1;
    assert_eq!(rows.len(), across);
    assert_eq!(rows[0], Value::String("0".to_string()));
    assert_eq!(rows[across - 1], Value::String(format!("{}2", ".".repeat(across - 1))));
    assert_eq!(Terrain::parse(&value).unwrap(), wide);
}