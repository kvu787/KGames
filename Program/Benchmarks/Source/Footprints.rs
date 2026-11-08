// What the programs and the libraries hold in memory, and what they churn
// through a frame or an edit: each program's heap after it starts and per
// frame, the text cache, what the interface keeps for widgets that are gone,
// the meshes and the files, and how much GPU memory a program's drawing
// takes.
//
// The counts are of the Rust heap, from Memory.rs' allocator. DirectWrite's
// layouts and Direct3D's resources are in their own heaps, so the text
// benchmarks also give the working set they add, and the GPU's memory comes
// from the device.

use std::path::Path;

use graphics::{AdapterChoice, Gpu, Offscreen};
use polygon_mesh::{Mesh, Shading, primitives};
use user_interface::tokens::{BODY, MAXIMIZED_HEIGHT, WINDOW_WIDTH};
use user_interface::{Draw, Event, Font, Fonts, Headless, Point, Program, Rect, TextStyle, Tree, Ui, Weight, Widget};

use crate::harness::Budget;
use crate::interface::{Car, GameInput, Modeler, Racing, Showcase, SimplePaint, Sphere, Subject};
use crate::memory::{self, Report, Usage, Video};

// How many frames or runs a benchmark counts at most. A memory benchmark
// needs only enough runs for the per-run numbers to be whole; it isn't
// looking for a minimum the way a timing benchmark is.
const FRAMES: usize = 200;
const RUNS: usize = 3;

// Veehiicuul's folder, whose models and scenes are the inputs, as in
// Libraries.rs.
fn veehiicuul() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../Veehiicuul"))
}

// A torus of 1,152 quads, like a model a modeler would subdivide.
fn torus() -> Mesh {
    primitives::torus(1.0, 0.3, 48, 24)
}

// What a program holds once it has started and drawn its first frame, and
// what each steady frame allocates and frees.
fn program<S: Subject>(subject: &S, budget: Budget) -> Vec<Usage> {
    let (start, mut app) = memory::held(&format!("memory/{}/start", subject.name()), "program", || subject.make());
    let note = format!("{} targets, {} instances", app.ui.targets().len(), app.ui.instance_count());
    let steady = memory::measure(&format!("memory/{}/frame/steady", subject.name()), "frame", budget, FRAMES, || {
        app.frame();
    });
    vec![start.with_note(note.clone()), steady.with_note(note)]
}

// Frames with the pointer crossing the page, which changes what's hovered,
// lays out new text, and shows tooltips.
fn moving<S: Subject>(subject: &S, budget: Budget) -> Usage {
    let mut app = subject.make();
    let mut step = 0usize;
    memory::measure(&format!("memory/{}/frame/moving", subject.name()), "frame", budget, FRAMES, || {
        step += 1;
        let t = step as f32 * 0.017;
        let x = (t.sin() * 0.5 + 0.5) * WINDOW_WIDTH;
        let y = ((t * 0.7).cos() * 0.5 + 0.5) * MAXIMIZED_HEIGHT;
        app.send(&[Event::PointerMoved(Point::new(x, y))]);
    })
    .with_note("2 frames an iteration")
}

// Frames in a row, counted only after the program has run for long enough
// to fill its caches and cross the text cache's purge, every 600 frames.
// What's kept here is what a program that has
// been up for a while still grows by a frame: a leak, if it isn't nothing.
fn long(budget: Budget) -> Usage {
    let (warmup, frames) = if budget.iterations <= 1 { (2, 2) } else { (800, 1500) };
    let mut app = Headless::new(ui_showcase::showcase::Showcase::default());
    app.frames(warmup);
    memory::start();
    app.frames(frames);
    let counts = memory::stop();
    Usage::from_counts("memory/UiShowcase/frames/long", "frame", counts, frames as f64)
        .with_note(format!("{frames} frames after the first {warmup}"))
}

// A program whose widgets have a new name every frame, as a program's would
// if it named them after what they show, like a row's text. The interface
// keeps widgets' state by name, so this is what it never lets go of.
struct Churner {
    frame: u32,
    // Widgets a frame.
    widgets: u32,
}

// A widget that keeps a number between frames, like a scroll offset.
struct Remember {
    value: f32,
}

impl Widget<()> for Remember {
    fn kind(&self) -> &'static str {
        "Remember"
    }

    fn draw(&self, ctx: &mut Draw) {
        *ctx.memory::<f32>() = self.value;
    }
}

impl Program for Churner {
    type Message = ();

    fn title(&self) -> String {
        "Churner".to_string()
    }

    fn input(&self, _ui: &Ui) -> Vec<()> {
        // One message a frame, to count the frame.
        vec![()]
    }

    fn update(&mut self, _message: (), _ui: &mut user_interface::Context<()>) {
        self.frame += 1;
    }

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        for index in 0..self.widgets {
            let name = format!("Row {} {index}", self.frame);
            tree.add(&name, Rect::new(index as f32, 0.0, 1.0, 1.0), Remember { value: index as f32 });
        }
        tree
    }
}

fn churn(name: &str, budget: Budget) -> Usage {
    let frames = if budget.iterations <= 1 { 2 } else { 1500 };
    let widgets = 20;
    let mut app = Headless::new(Churner { frame: 0, widgets });
    app.frames(10);
    memory::start();
    app.frames(frames);
    let counts = memory::stop();
    Usage::from_counts(name, "frame", counts, frames as f64).with_note(format!("{widgets} new names a frame, {frames} frames"))
}

// Every program's heap: what it starts with, and what a frame churns.
pub fn frames(budget: Budget) -> Report {
    let mut all = Vec::new();
    all.extend(program(&Showcase, budget));
    all.push(moving(&Showcase, budget));
    all.extend(program(&SimplePaint, budget));
    all.extend(program(&Modeler { levels: 0 }, budget));
    all.extend(program(&Modeler { levels: 4 }, budget));
    all.push(moving(&Modeler { levels: 4 }, budget));
    all.extend(program(&GameInput, budget));
    all.extend(program(&Racing, budget));
    all.extend(program(&Sphere, budget));
    all.extend(program(&Car, budget));
    all.push(long(budget));
    Report::of(all)
}

// The caches that every program's frames fill: the text layouts and their
// glyphs, and what the interface keeps for widgets by name.
pub fn caches(budget: Budget) -> Report {
    let mut all = Vec::new();
    let (fonts, mut fonts_cache) = memory::held("memory/text/fonts", "cache", || Fonts::new().expect("DirectWrite couldn't start"));
    all.push(fonts.with_note("DirectWrite's factory and the system font collection"));

    // Each text in each style is a layout of its own in the cache.
    let count = if budget.iterations <= 1 { 4 } else { 2000 };
    let labels: Vec<String> = (0..count).map(|index| format!("Label number {index}")).collect();
    // DirectWrite's layouts are in its own heap, which the allocator here
    // can't see, so what a layout costs there is the slope of the process's
    // private commit: the commit over the last three quarters of the
    // layouts, which leaves out the font files that the first ones load.
    let quarter = (count / 4).max(1);
    memory::start();
    for label in &labels[..quarter] {
        std::hint::black_box(fonts_cache.measure(label, BODY));
    }
    let first = memory::process_memory();
    for label in &labels[quarter..] {
        std::hint::black_box(fonts_cache.measure(label, BODY));
    }
    let counts = memory::stop();
    let committed = memory::process_memory().private_commit.saturating_sub(first.private_commit);
    all.push(
        Usage::from_counts("memory/text/layouts", "layout", counts, count as f64)
            .with_note(format!("{} B a layout of private commit, mostly DirectWrite's own", committed / (count - quarter).max(1) as u64)),
    );

    // Glyphs that the atlas doesn't have are rasterized into it, and their
    // pixels wait on the heap until the renderer copies them in. A frame
    // that's drawn takes them every frame; these are never taken.
    let sizes = if budget.iterations <= 1 { 2 } else { 40 };
    let mut glyphs = 0usize;
    memory::start();
    for size in 0..sizes {
        let style = TextStyle::new(Font::Text, Weight::Regular, 8.0 + size as f32 * 0.125, 20.0);
        fonts_cache.quads("0123456789", style, None, Point::ZERO, &mut |_| glyphs += 1);
    }
    let counts = memory::stop();
    all.push(
        Usage::from_counts("memory/text/glyphs", "glyph", counts, glyphs as f64)
            .with_note(format!("{glyphs} glyphs at {sizes} sizes, with each size's layout, their pixels waiting for the renderer")),
    );

    // Layouts unused for 600 frames go, which `begin_frame` does on the
    // 600th, so what's kept here is what the purge gave back.
    let purges = if budget.iterations <= 1 { 1 } else { 601 };
    memory::start();
    for _ in 0..purges {
        fonts_cache.begin_frame();
    }
    let counts = memory::stop();
    all.push(
        Usage::from_counts("memory/text/purge", "purge", counts, 1.0)
            .with_note(format!("{count} layouts let go of after 600 frames")),
    );
    drop(fonts_cache);

    all.push(churn("memory/ui/memory/new-names", budget));
    Report::of(all)
}

// The libraries' data: a mesh and what's built from it, a model, and a
// scene.
pub fn libraries(budget: Budget) -> Report {
    let mut all = Vec::new();
    let torus = torus();
    let (mesh, subdivided) = memory::held("memory/mesh/torus/2", "mesh", || torus.catmull_clark(2).mesh);
    let faces = subdivided.face_count();
    let vertices = subdivided.positions.len();
    let a_quad = |usage: Usage, what: &str| {
        let bytes = usage.kept as f64 / faces as f64;
        usage.with_note(format!("{faces} quads, {what}, {bytes:.0} B a quad"))
    };
    all.push(a_quad(mesh, &format!("{vertices} vertices")));
    all.push(
        memory::measure("memory/mesh/catmull-clark/torus/2", "subdivide", budget, RUNS, || {
            std::hint::black_box(torus.catmull_clark(2));
        })
        .with_note(format!("{} quads in, {faces} out", torus.face_count())),
    );
    let (topology, built) = memory::held("memory/mesh/topology", "topology", || subdivided.topology());
    let edges = built.edges().len();
    all.push(a_quad(topology, &format!("{edges} edges")));
    drop(built);
    all.push(
        memory::measure("memory/mesh/triangulate/smooth", "triangulate", budget, RUNS, || {
            std::hint::black_box(subdivided.triangulate(Shading::Smooth { angle: std::f32::consts::PI }));
        })
        .with_note(format!("{faces} quads, with the topology built again inside")),
    );
    let (triangles, drawn) = memory::held("memory/mesh/triangles", "triangles", || subdivided.triangulate(Shading::Flat));
    all.push(triangles.with_note(format!("{} triangles, {} vertices", drawn.indices.len() / 3, drawn.vertices.len())));
    drop(drawn);

    for name in ["SlopeCar", "BigCone"] {
        let path = veehiicuul().join("Models").join(format!("{name}.glb"));
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("Couldn't read {}: {error}", path.display()));
        let (usage, model) = memory::held(&format!("memory/gltf/{name}"), "model", || gltf::parse(&bytes).expect("a model"));
        all.push(usage.with_note(format!("{} KB on disk, {} triangles", bytes.len() / 1024, model.indices.len() / 3)));
        drop(model);
    }

    let path = veehiicuul().join("Scenes").join("Ribeye.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("Couldn't read {}: {error}", path.display()));
    let (usage, value) = memory::held("memory/json/scene", "parse", || json::parse(&text).expect("a scene"));
    all.push(usage.with_note(format!("{} KB of text", text.len() / 1024)));
    drop(value);
    all.push(
        memory::measure("memory/json/parse/scene", "parse", budget, RUNS, || {
            std::hint::black_box(json::parse(&text).expect("a scene"));
        })
        .with_note(format!("{} KB of text", text.len() / 1024)),
    );
    Report::of(all)
}

// How much GPU memory a program's drawing takes: the frame's target, the
// glyph atlas, the upload buffer, the program's own meshes and depth buffer,
// and what the driver keeps for them. Each program's row is what it added
// over an idle device with the frame's target, and it goes again before the
// next one.
pub fn video(_budget: Budget) -> Report {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).expect("a GPU");
    let mut video = Vec::new();
    let empty = gpu.video_memory().expect("the GPU's memory");
    video.push(Video {
        name: "memory/gpu/device".to_string(),
        local: empty[0].used,
        non_local: empty[1].used,
        note: format!("{} MiB of budget", empty[0].budget / (1024 * 1024)),
    });
    let mut target = Offscreen::new(&gpu, WINDOW_WIDTH as u32, MAXIMIZED_HEIGHT as u32, [0.0; 4]).expect("an offscreen target");
    let base = gpu.video_memory().expect("the GPU's memory");
    video.push(Video {
        name: "memory/gpu/target".to_string(),
        local: base[0].used - empty[0].used,
        non_local: base[1].used.saturating_sub(empty[1].used),
        note: format!("{} x {} target, over an idle device", WINDOW_WIDTH as u32, MAXIMIZED_HEIGHT as u32),
    });
    video.push(drawn(&Showcase, &mut gpu, &mut target));
    video.push(drawn(&SimplePaint, &mut gpu, &mut target));
    video.push(drawn(&Modeler { levels: 4 }, &mut gpu, &mut target));
    video.push(drawn(&GameInput, &mut gpu, &mut target));
    video.push(drawn(&Racing, &mut gpu, &mut target));
    video.push(drawn(&Sphere, &mut gpu, &mut target));
    video.push(drawn(&Car, &mut gpu, &mut target));
    Report { heap: Vec::new(), video }
}

// One program's GPU memory: what a frame of it, drawn into the target, adds
// to the device's usage, and what it leaves behind once it's gone.
fn drawn<S: Subject>(subject: &S, gpu: &mut Gpu, target: &mut Offscreen) -> Video {
    gpu.flush().expect("an idle GPU");
    let before = gpu.video_memory().expect("the GPU's memory");
    let mut app = subject.make();
    let frame = target.begin().expect("a frame");
    app.record(gpu, &frame).expect("the frame recorded");
    let fence = target.submit(gpu, frame).expect("the frame submitted");
    gpu.wait_for(fence).expect("the frame drawn");
    let after = gpu.video_memory().expect("the GPU's memory");
    drop(app);
    gpu.flush().expect("an idle GPU");
    let freed = after[0].used.saturating_sub(gpu.video_memory().expect("the GPU's memory")[0].used);
    Video {
        name: format!("memory/gpu/{}", subject.name()),
        local: after[0].used.saturating_sub(before[0].used),
        non_local: after[1].used.saturating_sub(before[1].used),
        note: format!("{:.1} MiB given back when it went", freed as f64 / (1024.0 * 1024.0)),
    }
}
