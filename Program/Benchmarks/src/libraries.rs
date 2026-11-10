// The libraries' CPU work: Catmull-Clark subdivision, triangulation,
// topology and ray casts of Library/PolygonMesh, Library/Gltf reading the
// game's models, Library/Json with a scene file, Library/Math's matrices,
// and Library/PresentTiming's tracker following presents through Windows'
// events.

use std::path::Path;

use polygon_mesh::{Mesh, Shading, primitives};
use present_timing::events::{Event, IN_FRAME, Kind, Runtime};
use present_timing::tracker::Tracker;

use crate::harness::{Budget, Measurement, measure};

// Veehiicuul's folder, whose models and scenes are the inputs.
fn veehiicuul() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../Veehiicuul"))
}

// A torus of 1152 quads, like a model a modeler would subdivide.
fn torus() -> Mesh {
    primitives::torus(1.0, 0.3, 48, 24)
}

pub fn meshes(budget: Budget) -> Vec<Measurement> {
    let mut all = Vec::new();
    let cube = primitives::cube(2.0);
    for levels in 1..=4 {
        let faces = cube.catmull_clark(levels).mesh.face_count();
        all.push(
            measure(&format!("mesh/catmull-clark/cube/{levels}"), "subdivide", budget, 1.0, || {
                std::hint::black_box(cube.catmull_clark(levels));
            })
            .with_note(format!("{faces} faces out")),
        );
    }
    let torus = torus();
    for levels in 1..=2 {
        let faces = torus.catmull_clark(levels).mesh.face_count();
        all.push(
            measure(&format!("mesh/catmull-clark/torus/{levels}"), "subdivide", budget, 1.0, || {
                std::hint::black_box(torus.catmull_clark(levels));
            })
            .with_note(format!("{faces} faces out")),
        );
    }
    let smooth = torus.catmull_clark(2).mesh;
    let faces = smooth.face_count();
    all.push(
        measure("mesh/triangulate/smooth", "triangulate", budget, 1.0, || {
            std::hint::black_box(smooth.triangulate(Shading::Smooth { angle: std::f32::consts::PI }));
        })
        .with_note(format!("{faces} quads")),
    );
    all.push(
        measure("mesh/triangulate/flat", "triangulate", budget, 1.0, || {
            std::hint::black_box(smooth.triangulate(Shading::Flat));
        })
        .with_note(format!("{faces} quads")),
    );
    all.push(
        measure("mesh/topology", "build", budget, 1.0, || {
            std::hint::black_box(smooth.topology());
        })
        .with_note(format!("{faces} quads")),
    );
    let triangles = smooth.triangulate(Shading::Flat);
    let count = triangles.indices.len() / 3;
    all.push(
        measure("mesh/ray-cast/triangles", "cast", budget, 1.0, || {
            std::hint::black_box(triangles.ray_cast([0.3, 5.0, 0.2], [0.0, -1.0, 0.0]));
        })
        .with_note(format!("{count} triangles")),
    );
    all.push(
        measure("mesh/ray-cast/mesh", "cast", budget, 1.0, || {
            std::hint::black_box(smooth.ray_cast([0.3, 5.0, 0.2], [0.0, -1.0, 0.0]));
        })
        .with_note(format!("{faces} quads")),
    );
    all.push(measure("mesh/primitive/uv-sphere", "make", budget, 1.0, || {
        std::hint::black_box(primitives::uv_sphere(1.0, 64, 32));
    }));
    all.push(
        measure("mesh/edges", "collect", budget, 1.0, || {
            std::hint::black_box(smooth.edges());
        })
        .with_note(format!("{faces} quads")),
    );
    all
}

pub fn models(budget: Budget) -> Vec<Measurement> {
    let mut all = Vec::new();
    for name in ["SlopeCar", "BigCone"] {
        let path = veehiicuul().join("Models").join(format!("{name}.glb"));
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("Couldn't read {}: {error}", path.display()));
        let triangles = gltf::parse(&bytes).expect("a model").indices.len() / 3;
        all.push(
            measure(&format!("gltf/parse/{name}"), "parse", budget, 1.0, || {
                std::hint::black_box(gltf::parse(&bytes).expect("a model"));
            })
            .with_note(format!("{} KB, {triangles} triangles", bytes.len() / 1024)),
        );
    }
    all
}

pub fn json(budget: Budget) -> Vec<Measurement> {
    let mut all = Vec::new();
    let path = veehiicuul().join("Scenes").join("Ribeye.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("Couldn't read {}: {error}", path.display()));
    let value = json::parse(&text).expect("a scene");
    all.push(
        measure("json/parse/scene", "parse", budget, 1.0, || {
            std::hint::black_box(json::parse(&text).expect("a scene"));
        })
        .with_note(format!("{} KB", text.len() / 1024)),
    );
    all.push(
        measure("json/write/scene", "write", budget, 1.0, || {
            std::hint::black_box(value.to_json());
        })
        .with_note(format!("{} KB", text.len() / 1024)),
    );
    all
}

pub fn math(budget: Budget) -> Vec<Measurement> {
    use math::Matrix;
    let mut all = Vec::new();
    let a = Matrix::rotation_y(0.3) * Matrix::translation(1.0, 2.0, 3.0);
    let b = Matrix::scaling(2.0, 2.0, 2.0) * Matrix::rotation_x(0.1);
    all.push(measure("math/matrix/multiply", "multiply", budget, 1000.0, || {
        let mut m = a;
        for _ in 0..1000 {
            m = std::hint::black_box(m * b);
        }
        std::hint::black_box(m);
    }));
    let points: Vec<[f32; 3]> = (0..10_000).map(|index| [index as f32, (index % 7) as f32, (index % 13) as f32]).collect();
    all.push(
        measure("math/matrix/transform-points", "point", budget, 10_000.0, || {
            for point in &points {
                std::hint::black_box(a.transform_point(*point));
            }
        })
        .with_note("10000 points"),
    );
    all
}

const PROCESS: u32 = 100;
const THREAD: u32 = 7;
const SWAP_CHAIN: u64 = 0xABC;
const WINDOW: u64 = 0x99;
const ALLOW_TEARING: u32 = 0x200;
const FLIP_WAIT_COMPLETE: u32 = 11;

fn event(time: i64, process: u32, thread: u32, kind: Kind) -> Event {
    Event { time, process, thread, kind }
}

// The events of one present that the display took at once, as the tracker's
// tests make them: the Present call, its token and queue packet, the token
// going in frame, the flip, and the display's interrupt.
fn torn(tracker: &mut Tracker, start: i64, sequence: u32) {
    let key = (1, sequence as u64, 1);
    for (offset, kind) in [
        (0, Kind::PresentStart { runtime: Runtime::Dxgi, swap_chain: SWAP_CHAIN, flags: ALLOW_TEARING, sync_interval: 0 }),
        (1, Kind::TokenCompositionSurfaceObject { key }),
        (2, Kind::PresentHistoryStart { token: 1000 + sequence as u64, model: 3, token_data: 0 }),
        (3, Kind::QueuePacketStart { packet_type: 0, submit_sequence: sequence, context: 0x5, present: true }),
        (4, Kind::PresentInfo { window: WINDOW }),
        (5, Kind::PresentStop { runtime: Runtime::Dxgi, result: 0 }),
    ] {
        tracker.handle(&event(start + offset, PROCESS, THREAD, kind));
    }
    tracker.handle(&event(start + 6, 4, 4, Kind::TokenStateChanged { key, state: IN_FRAME, independent_flip: true }));
    tracker.handle(&event(start + 10, 0, 0, Kind::MmioFlipMultiPlaneOverlay { submit_sequence: sequence, status: Some(FLIP_WAIT_COMPLETE) }));
    tracker.handle(&event(
        start + 20,
        0,
        0,
        Kind::SyncDpcMultiPlane {
            planes: vec![0x1000],
            flip_sequences: vec![(sequence as u64) << 32],
        },
    ));
}

pub fn present_timing(budget: Budget) -> Vec<Measurement> {
    let mut tracker = Tracker::new(PROCESS);
    let mut sequence = 0u32;
    vec![measure("present-timing/tracker/present", "present", budget, 1.0, || {
        sequence += 1;
        torn(&mut tracker, sequence as i64 * 100, sequence);
        std::hint::black_box(tracker.take());
    })
    .with_note("9 events a present")]
}
