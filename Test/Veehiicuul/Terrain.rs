// The hexel terrain: where cells are, which cell a point is in, brushes and
// fills, its palette, and its rows of characters in a scene's file.

use json::Value;
use veehiicuul::terrain::{Terrain, disk, distance, neighbors};

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
