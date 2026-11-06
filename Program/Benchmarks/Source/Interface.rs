// The programs' frames, as Library/UserInterface runs them: what a frame's
// update costs the CPU, without a window, through Headless, steady and with
// the pointer moving; what recording the frame's drawing costs the CPU; and
// what drawing it costs the GPU, timed with timestamp queries, into a target
// the size of the maximized window's client area. And the parts of a frame
// that every program pays for: text layout and the glyph atlas, the input
// reader, and the PerfBar's bookkeeping.

use std::path::PathBuf;

use graphics::{AdapterChoice, Gpu, GpuTimer, Offscreen};
use user_interface::game_input::{GameInputKeyState, GameInputKindKeyboard, GameInputKindMouse, GameInputMouseState};
use user_interface::input::reader::{InputReader, WindowState};
use user_interface::performance::{Frame, Performance};
use user_interface::tokens::{BODY, CAPTION, MAXIMIZED_HEIGHT, WINDOW_WIDTH};
use user_interface::{Event, FocusChange, Font, Fonts, Headless, Point, Program, Reading, Startup, TextStyle, Weight, clock};

use crate::harness::{Budget, Measurement, measure, measure_timed};

// A program with its frames ready to time, made fresh for each benchmark.
pub trait Subject {
    type P: Program;
    fn name(&self) -> &'static str;
    fn make(&self) -> Headless<Self::P>;
}

pub struct Showcase;
pub struct SimplePaint;
pub struct Modeler {
    pub levels: u32,
}
pub struct GameInput;
pub struct Racing;
pub struct Sphere;
pub struct Car;

// Where programs that would save files would save them, which none of these
// benchmarks make them do.
pub fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join("KGamesBenchmarks").join(name)
}

impl Subject for Showcase {
    type P = ui_showcase::showcase::Showcase;
    fn name(&self) -> &'static str {
        "UiShowcase"
    }
    fn make(&self) -> Headless<Self::P> {
        Headless::new(ui_showcase::showcase::Showcase::default())
    }
}

impl Subject for SimplePaint {
    type P = simple_paint_demo::demo::Demo;
    fn name(&self) -> &'static str {
        "SimplePaintDemo"
    }
    fn make(&self) -> Headless<Self::P> {
        Headless::new(simple_paint_demo::demo::Demo::new(simple_paint::Parameters::default(), scratch("Paint.json")))
    }
}

impl Subject for Modeler {
    type P = mesh_modeler::modeler::Modeler;
    fn name(&self) -> &'static str {
        if self.levels == 0 { "MeshModeler" } else { "MeshModeler/subdivided" }
    }
    fn make(&self) -> Headless<Self::P> {
        let mut app = Headless::new(mesh_modeler::modeler::Modeler::new(scratch("Models")));
        app.program.levels = self.levels;
        app.frames(2);
        app
    }
}

impl Subject for GameInput {
    type P = game_input_demo::demo::Demo;
    fn name(&self) -> &'static str {
        "GameInputDemo"
    }
    fn make(&self) -> Headless<Self::P> {
        let mut demo = game_input_demo::demo::Demo::scripted(0.1);
        for (index, name) in ["Xbox Controller", "DualSense"].iter().enumerate() {
            let step = format!(r#"{{ "Connect": {{ "Id": {}, "Name": "{name}", "Input": ["Gamepad"] }} }}"#, index + 1);
            demo.script(&json::parse(&step).unwrap()).unwrap();
        }
        let mut app = Headless::new(demo);
        app.frames(2);
        app
    }
}

impl Subject for Racing {
    type P = veehiicuul::program::Veehiicuul;
    fn name(&self) -> &'static str {
        "Veehiicuul"
    }
    fn make(&self) -> Headless<Self::P> {
        let mut app = Headless::start(Startup::for_tests("{}", &scratch("Veehiicuul")), veehiicuul::program::Veehiicuul::start);
        app.frames(2);
        app
    }
}

impl Subject for Sphere {
    type P = minimal_scene_test::moving_sphere::MovingSphere;
    fn name(&self) -> &'static str {
        "MinimalSceneTest"
    }
    fn make(&self) -> Headless<Self::P> {
        Headless::new(minimal_scene_test::moving_sphere::MovingSphere::new(minimal_scene_test::settings::SceneSettings::default()))
    }
}

impl Subject for Car {
    type P = spinning_car_test::spinning_car::SpinningCar;
    fn name(&self) -> &'static str {
        "SpinningCarTest"
    }
    fn make(&self) -> Headless<Self::P> {
        Headless::new(spinning_car_test::spinning_car::SpinningCar::new(spinning_car_test::settings::SceneSettings::default()))
    }
}

// A frame's update with nothing changing, which is what most frames are:
// the layouts are cached, and the widgets draw what they drew.
fn steady<S: Subject>(subject: &S, budget: Budget) -> Measurement {
    let mut app = subject.make();
    let measurement = measure(&format!("ui/{}/frame/steady", subject.name()), "frame", budget, 1.0, || {
        app.frame();
    });
    measurement.with_note(format!("{} targets, {} instances", app.ui.targets().len(), app.ui.instance_count()))
}

// A frame's update with the pointer moving across the page, which changes
// what's hovered, and so what's drawn, and can show a tooltip.
fn moving<S: Subject>(subject: &S, budget: Budget) -> Measurement {
    let mut app = subject.make();
    let mut step = 0usize;
    measure(&format!("ui/{}/frame/moving", subject.name()), "frame", budget, 1.0, || {
        step += 1;
        let t = step as f32 * 0.017;
        let x = (t.sin() * 0.5 + 0.5) * WINDOW_WIDTH;
        let y = ((t * 0.7).cos() * 0.5 + 0.5) * MAXIMIZED_HEIGHT;
        app.send(&[Event::PointerMoved(Point::new(x, y))]);
    })
    // `send` runs two frames.
    .with_note("2 frames an iteration")
}

// The first frame: a program made from nothing, with every layout and
// glyph still to make.
fn first<S: Subject>(subject: &S, budget: Budget) -> Measurement {
    measure(&format!("ui/{}/start", subject.name()), "start", budget, 1.0, || {
        let app = subject.make();
        std::hint::black_box(app.frame_number());
    })
}

// What the GPU takes to draw the last frame into a target the size of the
// maximized window's client area, and what the CPU takes to record it.
fn draw<S: Subject>(subject: &S, gpu: &mut Gpu, budget: Budget) -> Vec<Measurement> {
    let mut app = subject.make();
    let mut target = Offscreen::new(gpu, WINDOW_WIDTH as u32, MAXIMIZED_HEIGHT as u32, [0.0; 4]).expect("an offscreen target");
    let mut timer = GpuTimer::new(gpu, 1).expect("a GPU timer");
    let background = app.ui.theme().background;
    let clear = [background.r, background.g, background.b, 1.0];
    let mut record_times = Vec::new();
    let gpu_time = measure_timed(&format!("gpu/{}/draw", subject.name()), "frame", budget, 1.0, || {
        let frame = target.begin().expect("a frame");
        unsafe { frame.clear(clear) };
        let span = timer.begin(&frame);
        let before = std::time::Instant::now();
        app.record(gpu, &frame).expect("the frame recorded");
        record_times.push(before.elapsed().as_secs_f64() * 1000.0);
        timer.end(&frame, span);
        timer.resolve(&frame);
        let fence = target.submit(gpu, frame).expect("the frame submitted");
        timer.read(gpu, fence).expect("the GPU's times")[0]
    });
    let note = format!("{} instances", app.ui.instance_count());
    let count = gpu_time.iterations;
    let record = Measurement::from_times(&format!("cpu/{}/record", subject.name()), "frame", &record_times[record_times.len() - count..], 1.0);
    vec![gpu_time.with_note(note.clone()), record.with_note(note)]
}

// Every program's CPU frame benchmarks.
pub fn frames(budget: Budget) -> Vec<Measurement> {
    let mut all = Vec::new();
    all.push(steady(&Showcase, budget));
    all.push(moving(&Showcase, budget));
    all.push(first(&Showcase, budget));
    all.push(steady(&SimplePaint, budget));
    all.push(steady(&Modeler { levels: 0 }, budget));
    all.push(steady(&Modeler { levels: 4 }, budget));
    all.push(moving(&Modeler { levels: 4 }, budget));
    all.push(steady(&GameInput, budget));
    all.push(steady(&Racing, budget));
    all.push(first(&Racing, budget));
    all.push(steady(&Sphere, budget));
    all.push(steady(&Car, budget));
    all
}

// Every program's GPU draw benchmarks, on the fastest GPU.
pub fn draws(budget: Budget) -> Vec<Measurement> {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).expect("a GPU");
    let mut all = Vec::new();
    all.extend(draw(&Showcase, &mut gpu, budget));
    all.extend(draw(&SimplePaint, &mut gpu, budget));
    all.extend(draw(&Modeler { levels: 4 }, &mut gpu, budget));
    all.extend(draw(&GameInput, &mut gpu, budget));
    all.extend(draw(&Racing, &mut gpu, budget));
    all.extend(draw(&Sphere, &mut gpu, budget));
    all.extend(draw(&Car, &mut gpu, budget));
    all
}

// Text through DirectWrite: a layout that the cache has, one that it
// doesn't, and glyphs that the atlas doesn't have yet.
pub fn text(budget: Budget) -> Vec<Measurement> {
    let mut all = Vec::new();
    let mut fonts = Fonts::new().expect("DirectWrite");
    let labels: Vec<String> = (0..64).map(|index| format!("Label {index}")).collect();
    let mut index = 0usize;
    all.push(measure("text/layout/cached", "measure", budget, 64.0, || {
        fonts.begin_frame();
        for label in &labels {
            std::hint::black_box(fonts.measure(label, BODY));
        }
    }));
    all.push(measure("text/layout/new", "layout", budget, 1.0, || {
        index += 1;
        fonts.begin_frame();
        std::hint::black_box(fonts.measure(&format!("Never seen before {index}"), BODY));
    }));
    let paragraph = "The quick brown fox jumps over the lazy dog, and the five boxing wizards jump quickly, while pack my box with five dozen liquor jugs.";
    all.push(measure("text/layout/wrapped-new", "layout", budget, 1.0, || {
        index += 1;
        fonts.begin_frame();
        std::hint::black_box(fonts.measure_wrapped(&format!("{paragraph} {index}"), BODY, 400.0));
    }));
    let mut quads = 0usize;
    all.push(measure("text/quads/cached", "quad", budget, 64.0 * 8.0, || {
        fonts.begin_frame();
        quads = 0;
        for label in &labels {
            fonts.quads(label, BODY, None, Point::new(10.5, 20.0), &mut |_| quads += 1);
        }
    }));
    let mut size = 0u32;
    all.push(measure("text/rasterize/new", "glyph", budget, 10.0, || {
        size += 1;
        fonts.begin_frame();
        // Ten digits at a size no glyph has been rasterized at yet.
        let style = TextStyle::new(Font::Text, Weight::Regular, 8.0 + size as f32 * 0.125, 20.0);
        fonts.quads("0123456789", style, None, Point::ZERO, &mut |_| {});
    }));
    all.push(measure("text/layout/caption-cached", "measure", budget, 64.0, || {
        fonts.begin_frame();
        for label in &labels {
            std::hint::black_box(fonts.measure(label, CAPTION));
        }
    }));
    all
}

// The input reader with a frame's readings from a 1000 Hz mouse at 60
// frames a second, and a keyboard that holds a key.
pub fn input(budget: Budget) -> Vec<Measurement> {
    let mut reader = InputReader::with_repeat(true, 0.5, 30.0);
    let now = clock::now();
    let window = WindowState {
        cursor: Point::new(100.0, 50.0),
        inside: true,
        focus_changes: vec![FocusChange { time: now, gained: true }],
    };
    reader.read(&[], &[], &window, now);
    let window = WindowState { focus_changes: Vec::new(), ..window };
    let tick = clock::frequency() / 1000;
    let mut time = now;
    let mut readings = Vec::new();
    for index in 0..17 {
        time += tick;
        readings.push((
            time,
            GameInputKindMouse,
            Reading {
                device: Some(2),
                kinds: GameInputKindMouse,
                mouse: GameInputMouseState {
                    positionX: index,
                    positionY: index * 2,
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
    }
    readings.push((
        time,
        GameInputKindKeyboard,
        Reading {
            device: Some(1),
            kinds: GameInputKindKeyboard,
            keys: vec![GameInputKeyState {
                scanCode: 0x1E,
                codePoint: 0,
                virtualKey: 0x41,
                isDeadKey: false,
            }],
            ..Default::default()
        },
    ));
    vec![measure("input/reader/frame", "frame", budget, 1.0, || {
        time += tick;
        std::hint::black_box(reader.read(&readings, &[], &window, time));
    })
    .with_note("17 mouse readings and 1 keyboard reading")]
}

// The PerfBar's bookkeeping: a frame presented, its present known, and the
// stats refreshed, at 240 frames a second.
pub fn perf_bar(budget: Budget) -> Vec<Measurement> {
    let mut performance = Performance::given();
    let frequency = clock::frequency() as f64;
    let mut frame = 0u64;
    let base = clock::now();
    vec![measure("perfbar/frame", "frame", budget, 1.0, || {
        frame += 1;
        let seconds = frame as f64 / 240.0;
        let present = base + (seconds * frequency) as i64;
        performance.frame_started(seconds);
        performance.presented(Frame {
            input: Some(present - (0.002 * frequency) as i64),
            read: present - (0.001 * frequency) as i64,
            present,
        });
        performance.known(vec![present_timing::Present {
            start: present + 5,
            thread: 1,
            mode: present_timing::PresentMode::IndependentFlip,
            tearing: true,
            screen: Some(present + (0.004 * frequency) as i64),
            flip_delay: 0,
        }]);
        performance.update(seconds);
    })]
}
