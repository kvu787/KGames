// A scene's terrain: a grid of hexels, hexagonal cells on the ground, each
// painted a color of the terrain's palette, or not painted, where the
// scene's background shows. It's flat, under the track, like the rest of the
// ground.
//
// Cells have axial coordinates, (q, r). With pointy tops, a cell's middle is
// at X = size * sqrt(3) * (q + r / 2) and Z = size * 3 / 2 * r, where `size`
// is the distance from a cell's middle to its corners. With flat tops, X and
// Z swap roles.
//
// In a scene's file, the terrain's cells are rows of characters, one a cell,
// from the cell at `Origin` on: a palette index as a digit or a letter, 0 to
// 9, then A to Z, then a to z, or "." for a cell that isn't painted.
//
//     "Terrain": {
//         "Size": 2,
//         "Orientation": "PointyTop",
//         "Palette": ["#17B856", "#2ACB6B"],
//         "Origin": [-85, -46],
//         "Rows": ["..0..1", "..11"]
//     }

use std::collections::BTreeMap;

use json::Value;

use crate::files::{self, Color};

// How far a cell can be from cell [0, 0], along each axis: a dozen times
// past the scene that there is. A terrain is saved as rows of characters,
// from its first cell to its last, so two cells at the ends of what an i32
// holds would be rows that no disk holds, and the cells around a cell at an
// end, as far as a brush or a fill reaches, would have coordinates past it.
// So a point farther away is in the farthest cell, a brush that reaches past
// it leaves those cells unpainted, and a file with cells past it is wrong.
pub const FARTHEST_CELL: i32 = 1024;

pub const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
// The most colors a palette has, one a character.
pub const PALETTE_SIZE: usize = 62;
const SQRT_3: f64 = 1.732_050_807_568_877_2;

#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    // From a cell's middle to its corners, in units.
    pub size: f64,
    // Whether cells have flat tops and bottoms along Z, or points.
    pub flat_top: bool,
    pub palette: Vec<Color>,
    // Each painted cell's palette index, by its axial coordinates.
    pub cells: BTreeMap<(i32, i32), u8>,
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain {
            size: 2.0,
            flat_top: false,
            palette: vec![[0x17, 0xB8, 0x56], [0xD8, 0xC4, 0x8A], [0x4A, 0x9B, 0xE0]],
            cells: BTreeMap::new(),
        }
    }
}

impl Terrain {
    // A cell's middle on the ground.
    pub fn center(&self, (q, r): (i32, i32)) -> [f64; 2] {
        let (q, r) = (q as f64, r as f64);
        let along = self.size * SQRT_3 * (q + r / 2.0);
        let across = self.size * 1.5 * r;
        if self.flat_top { [across, along] } else { [along, across] }
    }

    // The cell that a point on the ground is in.
    pub fn cell_at(&self, [x, z]: [f64; 2]) -> (i32, i32) {
        let (along, across) = if self.flat_top { (z, x) } else { (x, z) };
        let r = across / (self.size * 1.5);
        let q = along / (self.size * SQRT_3) - r / 2.0;
        round_axial(q, r)
    }

    // A cell's six corners, in order around it.
    pub fn corners(&self, cell: (i32, i32)) -> [[f64; 2]; 6] {
        let [x, z] = self.center(cell);
        let start = if self.flat_top { 0.0 } else { 30.0_f64 };
        std::array::from_fn(|index| {
            let angle = (start + 60.0 * index as f64).to_radians();
            [x + self.size * angle.cos(), z + self.size * angle.sin()]
        })
    }

    pub fn get(&self, cell: (i32, i32)) -> Option<u8> {
        self.cells.get(&cell).copied()
    }

    // Paints a cell with a palette index, or clears it with None. A cell
    // past the farthest stays unpainted.
    pub fn set(&mut self, cell: (i32, i32), color: Option<u8>) {
        let within = |coordinate: i32| (-FARTHEST_CELL..=FARTHEST_CELL).contains(&coordinate);
        match color {
            Some(_) if !(within(cell.0) && within(cell.1)) => {}
            Some(color) => {
                self.cells.insert(cell, color);
            }
            None => {
                self.cells.remove(&cell);
            }
        }
    }

    // Every cell connected to `start` through cells of its color, or
    // unpainted ones, which can go on forever, so only those within
    // `reach` cells of it count.
    pub fn region(&self, start: (i32, i32), reach: i32) -> Vec<(i32, i32)> {
        let color = self.get(start);
        let mut found = vec![start];
        let mut seen = std::collections::HashSet::from([start]);
        let mut index = 0;
        while index < found.len() {
            let cell = found[index];
            index += 1;
            for neighbor in neighbors(cell) {
                if distance(neighbor, start) <= reach && self.get(neighbor) == color && seen.insert(neighbor) {
                    found.push(neighbor);
                }
            }
        }
        found
    }

    // Takes a color out of the palette, and with it the cells painted with
    // it. The cells of the colors after it keep their colors.
    pub fn remove_color(&mut self, index: u8) {
        if (index as usize) >= self.palette.len() {
            return;
        }
        self.palette.remove(index as usize);
        self.cells.retain(|_, color| *color != index);
        for color in self.cells.values_mut() {
            if *color > index {
                *color -= 1;
            }
        }
    }

    pub fn parse(value: &Value) -> Result<Terrain, String> {
        files::only(value, &["Size", "Orientation", "Palette", "Origin", "Rows"], "the terrain")?;
        let flat_top = match files::member(value, "Orientation", "Terrain.")? {
            Value::String(text) if text.eq_ignore_ascii_case("PointyTop") => false,
            Value::String(text) if text.eq_ignore_ascii_case("FlatTop") => true,
            other => {
                return Err(format!(
                    "Terrain.Orientation must be \"PointyTop\" or \"FlatTop\", not {}",
                    other.describe()
                ));
            }
        };
        let palette = match files::member(value, "Palette", "Terrain.")? {
            Value::Array(items) if items.len() <= PALETTE_SIZE => items
                .iter()
                .enumerate()
                .map(|(index, item)| files::color(item, &format!("Terrain.Palette[{index}]")))
                .collect::<Result<Vec<_>, _>>()?,
            other => {
                return Err(format!(
                    "Terrain.Palette must be a list of {PALETTE_SIZE} colors at most, not {}",
                    other.describe()
                ));
            }
        };
        let origin = files::point(files::member(value, "Origin", "Terrain.")?, "Terrain.Origin")?;
        if origin.iter().any(|value| value.fract() != 0.0) {
            return Err("Terrain.Origin must be a cell's coordinates, two whole numbers".to_string());
        }
        if origin.iter().any(|value| value.abs() > FARTHEST_CELL as f64) {
            return Err(format!("Terrain.Origin must be within {FARTHEST_CELL} cells of [0, 0], not [{}, {}]", origin[0], origin[1]));
        }
        let first = [origin[0] as i64, origin[1] as i64];
        let mut cells = BTreeMap::new();
        let Value::Array(rows) = files::member(value, "Rows", "Terrain.")? else {
            return Err("Terrain.Rows must be a list of rows of characters".to_string());
        };
        for (row_index, row) in rows.iter().enumerate() {
            let Value::String(row) = row else {
                return Err(format!("Terrain.Rows[{row_index}] must be text, not {}", row.describe()));
            };
            for (column, character) in row.bytes().enumerate() {
                if character == b'.' {
                    continue;
                }
                let color = ALPHABET
                    .iter()
                    .position(|&letter| letter == character)
                    .filter(|&color| color < palette.len())
                    .ok_or_else(|| {
                        format!(
                            "Terrain.Rows[{row_index}] has {:?}, which isn't one of the palette's {} colors",
                            character as char,
                            palette.len()
                        )
                    })?;
                let (q, r) = (first[0] + column as i64, first[1] + row_index as i64);
                if q > FARTHEST_CELL as i64 || r > FARTHEST_CELL as i64 {
                    return Err(format!("Terrain.Rows[{row_index}] goes past {FARTHEST_CELL} cells from [0, 0]"));
                }
                cells.insert((q as i32, r as i32), color as u8);
            }
        }
        Ok(Terrain {
            size: files::positive(files::member(value, "Size", "Terrain.")?, "Terrain.Size")?,
            flat_top,
            palette,
            cells,
        })
    }

    pub fn to_value(&self) -> Value {
        let (mut q0, mut r0, mut q1, mut r1) = (0, 0, -1, -1);
        if let Some(((q, r), _)) = self.cells.iter().next() {
            (q0, r0, q1, r1) = (*q, *r, *q, *r);
        }
        for &(q, r) in self.cells.keys() {
            (q0, r0, q1, r1) = (q0.min(q), r0.min(r), q1.max(q), r1.max(r));
        }
        let rows: Vec<Value> = (r0..=r1)
            .map(|r| {
                let row: String = (q0..=q1)
                    .map(|q| match self.get((q, r)) {
                        Some(color) => ALPHABET[color as usize] as char,
                        None => '.',
                    })
                    .collect();
                row.trim_end_matches('.').to_string().into()
            })
            .collect();
        json::object([
            ("Size", self.size.into()),
            ("Orientation", if self.flat_top { "FlatTop" } else { "PointyTop" }.into()),
            ("Palette", Value::Array(self.palette.iter().map(|&color| files::color_text(color).into()).collect())),
            ("Origin", Value::Array(vec![(q0 as f64).into(), (r0 as f64).into()])),
            ("Rows", Value::Array(rows)),
        ])
    }
}

// The cell nearest fractional axial coordinates, by rounding them as the
// cube coordinates (q, r, -q - r), and no farther than the farthest cell,
// which a point far away, or any point of a terrain of tiny cells, is in.
fn round_axial(q: f64, r: f64) -> (i32, i32) {
    let s = -q - r;
    let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
    let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    let within = |coordinate: f64| (coordinate as i32).clamp(-FARTHEST_CELL, FARTHEST_CELL);
    (within(rq), within(rr))
}

pub fn neighbors((q, r): (i32, i32)) -> [(i32, i32); 6] {
    [(q + 1, r), (q - 1, r), (q, r + 1), (q, r - 1), (q + 1, r - 1), (q - 1, r + 1)]
}

// How many steps between two cells.
pub fn distance((q0, r0): (i32, i32), (q1, r1): (i32, i32)) -> i32 {
    let (dq, dr) = (q1 - q0, r1 - r0);
    (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
}

// Every cell within `radius` steps of `center`.
pub fn disk(center: (i32, i32), radius: i32) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    for dq in -radius..=radius {
        for dr in (-radius).max(-dq - radius)..=radius.min(-dq + radius) {
            cells.push((center.0 + dq, center.1 + dr));
        }
    }
    cells
}
