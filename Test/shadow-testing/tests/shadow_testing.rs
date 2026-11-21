// Tests ShadowTesting without a window, for every way of drawing a shadow:
// what each slider does, on the frame of the press, in the tiles' real
// layout, that the tiles are alike and every card fits in its tile, and,
// drawn offscreen with the GPU, that every pixel around each box is what the
// way's formula says, in both themes, and that each way's shadow fades out
// before the edge of its stage at its strongest settings.

use shadow_testing::approaches::{Approach, KINDS, Kind, box_distance, coverage};
use shadow_testing::shadow_testing::{BOX_COLOR, BOX_SIZE, Message, ShadowTesting};
use user_interface::graphics::{AdapterChoice, Gpu, Image, write_png};
use user_interface::json::Value;
use user_interface::tokens::WINDOW_HEIGHT;
use user_interface::{Color, Headless, Key, Point, Rect, Theme, ThemeKind, slider_rail};

fn program() -> Headless<ShadowTesting> {
    let mut app = Headless::new(ShadowTesting::default());
    app.frame();
    app
}

fn save(image: &Image, name: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    write_png(&folder.join(format!("{name}.png")), image).unwrap();
}

// Whether two colors' red, green and blue are within 2 of each other.
fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

fn slider(kind: Kind, setting: &str) -> String {
    format!("{}/{setting}", kind.name())
}

// Where a slider's thumb goes for a fraction of its rail.
fn on_rail(app: &Headless<ShadowTesting>, name: &str, fraction: f32) -> Point {
    let rect = app.target(name);
    let (left, right) = slider_rail(rect);
    Point::new(left + (right - left) * fraction, rect.center().y)
}

// Settings for checking pixels, unlike the defaults, with a shadow whose
// end doesn't fade to nothing.
fn test_values(kind: Kind) -> Vec<f32> {
    match kind {
        Kind::Linear => vec![40.0, 0.6, 0.2],
        Kind::Curved => vec![40.0, 0.6, 1.7],
        Kind::Gaussian => vec![20.0, 6.0, 10.0, 4.0, 0.5],
        Kind::Offset => vec![10.0, -7.0, 0.4],
        // An angle whose edges never pass through a sample exactly, so that
        // the GPU's rounding can't put a sample on the other side of one.
        Kind::Long => vec![70.0, 30.0, 0.6, 0.2],
        Kind::Fluent => vec![64.0],
    }
}

fn set_all(app: &mut Headless<ShadowTesting>, kind: Kind, values: &[f32]) {
    for (index, value) in values.iter().enumerate() {
        app.send_message(Message::Set(kind, index, *value as f64));
    }
}

#[test]
fn every_widget_fits_in_the_window_in_both_themes() {
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(ShadowTesting::default(), theme);
        app.frame();
        for target in app.ui.targets().iter().filter(|target| target.listed) {
            let rect = target.rect;
            assert!(rect.x >= 0.0 && rect.y >= 0.0 && rect.right() <= 2560.0 && rect.bottom() <= WINDOW_HEIGHT, "{} is at {rect:?}", target.name);
        }
    }
}

// Every way's stage is the same size, with the same box in its middle on
// whole pixels, and every slider is in its way's tile, left of its stage.
#[test]
fn the_tiles_are_alike_and_each_way_s_sliders_are_in_its_tile() {
    let app = program();
    let first = app.program.tile(KINDS[0]);
    for kind in KINDS {
        let tile = app.program.tile(kind);
        assert_eq!(tile.boxed, tile.boxed.round(), "{}", kind.name());
        assert_eq!(tile.boxed.size(), BOX_SIZE);
        assert!((tile.stage.width - first.stage.width).abs() <= 1.0 && (tile.stage.height - first.stage.height).abs() <= 1.0, "{}", kind.name());
        assert!((tile.boxed.center().x - tile.stage.center().x).abs() <= 0.5 && (tile.boxed.center().y - tile.stage.center().y).abs() <= 0.5);
        for setting in kind.settings() {
            let name = slider(kind, setting.name);
            let rect = app.target(&name);
            assert!(rect.x >= tile.area.x && rect.right() <= tile.stage.x && rect.y >= tile.area.y && rect.bottom() <= tile.area.bottom(), "{name} is at {rect:?}, out of {tile:?}");
        }
        for target in app.ui.targets() {
            assert!(!tile.stage.intersects(target.rect), "{} at {:?} is on {}'s stage", target.name, target.rect, kind.name());
        }
    }
}

// Each slider sets its setting on the frame of the press, to the step
// nearest where on the rail it is, and a drag past its end sets the most it
// allows.
#[test]
fn a_press_on_any_slider_sets_its_setting_and_a_drag_takes_it_to_its_end() {
    let mut app = program();
    for kind in KINDS {
        for (index, setting) in kind.settings().iter().enumerate() {
            let name = slider(kind, setting.name);
            let at = on_rail(&app, &name, 0.25);
            app.press_point(at);
            let asked = setting.minimum + (setting.maximum - setting.minimum) * 0.25;
            let stepped = setting.minimum + ((asked - setting.minimum) / setting.step).round() * setting.step;
            let value = app.program.approach(kind).values[index];
            assert!((value - stepped).abs() < 1e-4, "{name} is {value}, not {stepped}");
            app.move_point(Point::new(2550.0, at.y)).release();
            assert_eq!(app.program.approach(kind).values[index], setting.maximum, "{name}");
        }
    }
}

#[test]
fn the_state_has_every_way_s_settings() {
    let mut app = program();
    app.send_message(Message::Set(Kind::Linear, 1, 0.3));
    let state = app.state();
    for kind in KINDS {
        let approach = state.get(kind.name()).unwrap_or_else(|| panic!("the state has no {}", kind.name()));
        for (setting, value) in kind.settings().iter().zip(&app.program.approach(kind).values) {
            let rounded = (*value as f64 * 10_000.0).round() / 10_000.0;
            assert_eq!(approach.get(setting.name), Some(&Value::Number(rounded)), "{}/{}", kind.name(), setting.name);
        }
    }
    assert_eq!(state.get("Linear").and_then(|linear| linear.get("StartOpacity")), Some(&Value::Number(0.3)));
}

// Ctrl+R sets every way's settings back to their defaults, on the frame of
// the press.
#[test]
fn ctrl_r_resets_every_way_on_its_frame() {
    let mut app = program();
    for kind in KINDS {
        set_all(&mut app, kind, &test_values(kind));
    }
    app.hold(Key::CONTROL).hold(Key::R);
    for kind in KINDS {
        assert_eq!(app.program.approach(kind), &Approach::new(kind), "{}", kind.name());
    }
}

// The linear shadow's formula itself: the start's opacity half a pixel out
// from the box, down in a straight line to the end's at the length, and
// none past it, round around the corners.
#[test]
fn the_linear_shadow_s_opacity_goes_from_the_start_to_the_end_in_a_straight_line() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let values = [40.0, 0.6, 0.2];
    let at = |distance: f32| opacity(Kind::Linear, &values, rect, 10.0 + distance, 5.0);
    assert!(near(at(0.5), 0.6 - 0.4 * 0.5 / 40.0));
    assert!(near(at(20.0), 0.4));
    assert!(near(at(39.5), 0.6 - 0.4 * 39.5 / 40.0));
    assert!(near(at(40.5), 0.0));
    assert!(near(opacity(Kind::Linear, &[0.0, 0.6, 0.2], rect, 10.5, 5.0), 0.0));
    // As far from a corner diagonally as straight out from a side.
    assert!(near(box_distance(rect, 13.0, 14.0), 5.0));
    assert!(near(box_distance(rect, 5.0, 13.0), 3.0));
    assert!(near(box_distance(rect, 5.0, 5.0), -5.0));
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

fn opacity(kind: Kind, values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
    kind.opacity(values, rect, x, y).expect("the program draws it")
}

// The curved shadow at a power of 1 is the linear one fading to nothing, and
// at a higher power it's fainter everywhere between the box and the length.
#[test]
fn the_curved_shadow_at_a_power_of_1_is_linear_and_higher_powers_fade_faster() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    for distance in [0.5, 7.0, 20.0, 39.5, 40.5, 60.0] {
        let x = 10.0 + distance;
        assert!(near(opacity(Kind::Curved, &[40.0, 0.6, 1.0], rect, x, 5.0), opacity(Kind::Linear, &[40.0, 0.6, 0.0], rect, x, 5.0)), "{distance}");
        if distance < 40.0 {
            assert!(opacity(Kind::Curved, &[40.0, 0.6, 2.0], rect, x, 5.0) < opacity(Kind::Curved, &[40.0, 0.6, 1.0], rect, x, 5.0) || distance < 1.0);
        }
    }
    assert!(near(opacity(Kind::Curved, &[40.0, 0.6, 2.0], rect, 30.0, 5.0), 0.6 * 0.25));
}

// The Gaussian shadow: erf is within its approximation's bound of known
// values, the shadow is half its opacity at the blurred rectangle's edge,
// a quarter at its corner, and nearly all of it deep inside.
#[test]
fn the_gaussian_shadow_is_a_blurred_rectangle() {
    use shadow_testing::gaussian::erf;
    for (x, want) in [(0.0, 0.0), (0.5, 0.520_499_9), (1.0, 0.842_700_8), (2.0, 0.995_322_3), (-1.0, -0.842_700_8)] {
        assert!((erf(x) - want).abs() < 1e-6, "erf({x}) is {}, not {want}", erf(x));
    }
    let rect = Rect::new(0.0, 0.0, 400.0, 400.0);
    // A blur of 8 is a deviation of 4; no offset or spread.
    let values = [8.0, 0.0, 0.0, 0.0, 0.8];
    assert!(near(opacity(Kind::Gaussian, &values, rect, 400.0, 200.0), 0.4));
    assert!(near(opacity(Kind::Gaussian, &values, rect, 400.0, 400.0), 0.2));
    assert!(near(opacity(Kind::Gaussian, &values, rect, 200.0, 200.0), 0.8));
    // Moved by the offset and grown by the spread: its edge is 10 + 6 right.
    let moved = [8.0, 10.0, 0.0, 6.0, 0.8];
    assert!(near(opacity(Kind::Gaussian, &moved, rect, 416.0, 200.0), 0.4));
    // A spread that shrinks it to nothing leaves no shadow.
    let small = Rect::new(0.0, 0.0, 20.0, 20.0);
    assert_eq!(opacity(Kind::Gaussian, &[8.0, 0.0, 0.0, -10.0, 0.8], small, 10.0, 10.0), 0.0);
}

// The offset shadow is the box moved, with hard edges.
#[test]
fn the_offset_shadow_is_the_box_moved() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let values = [5.0, -3.0, 0.4];
    assert!(near(opacity(Kind::Offset, &values, rect, 14.5, 6.5), 0.4));
    assert!(near(opacity(Kind::Offset, &values, rect, 15.5, 6.5), 0.0));
    assert!(near(opacity(Kind::Offset, &values, rect, 14.5, 7.5), 0.0));
    assert!(near(opacity(Kind::Offset, &values, rect, 5.5, -2.5), 0.4));
}

// The long shadow goes along its angle, from the start's opacity at the box
// to the end's at its length, and nowhere else.
#[test]
fn the_long_shadow_goes_along_its_angle_from_the_start_to_the_end() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    // Straight down, 40 long: a point 20 below the box's bottom, whose
    // samples are 20 below it on average, is halfway.
    let down = [40.0, 90.0, 0.6, 0.2];
    assert!(near(opacity(Kind::Long, &down, rect, 5.5, 30.0), 0.4));
    assert!(near(opacity(Kind::Long, &down, rect, 12.5, 30.0), 0.0));
    assert!(near(opacity(Kind::Long, &down, rect, 5.5, 51.5), 0.0));
    assert!(near(opacity(Kind::Long, &down, rect, 5.5, -1.5), 0.0));
    // At 45 degrees, down and right, a pixel diagonally off the box's
    // bottom right corner is in it, and one straight right of the box isn't.
    let diagonal = [40.0, 45.0, 0.6, 0.6];
    assert!(near(opacity(Kind::Long, &diagonal, rect, 20.5, 20.5), 0.6));
    assert!(near(opacity(Kind::Long, &diagonal, rect, 30.5, 2.5), 0.0));
}

// What a pixel should be: the box's color over what it covers of the
// pixel, and the background, darkened by the way's shadow, over the rest.
fn expected(kind: Kind, values: &[f32], rect: Rect, background: Color, x: u32, y: u32) -> [u8; 4] {
    let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
    let inside = coverage(box_distance(rect, cx, cy));
    let shadow = opacity(kind, values, rect, cx, cy) * (1.0 - inside);
    let rest = 1.0 - inside - shadow;
    let mix = |boxed: f32, behind: f32| boxed * inside + behind * rest;
    Color::rgb(mix(BOX_COLOR.r, background.r), mix(BOX_COLOR.g, background.g), mix(BOX_COLOR.b, background.b)).to_bytes()
}

// Every pixel of the way's stage that its quad reaches, and a little past
// it, is what its formula says.
fn check_every_pixel(app: &Headless<ShadowTesting>, image: &Image, kind: Kind, theme: ThemeKind) {
    let values = &app.program.approach(kind).values;
    let tile = app.program.tile(kind);
    let background = Theme::new(theme).background;
    let area = kind.area(values, tile.boxed).expand(4.0).intersect(tile.stage);
    for y in area.y as u32..area.bottom() as u32 {
        for x in area.x as u32..area.right() as u32 {
            let want = expected(kind, values, tile.boxed, background, x, y);
            let got = image.pixel(x, y);
            assert!(close(got, want), "{} at {values:?} in the {} theme: ({x}, {y}) is {got:?}, not {want:?}", kind.name(), theme.name());
        }
    }
}

// Every pixel around every box, and in it, is what its way's formula says,
// in both themes, for every way that the program draws.
#[test]
fn every_pixel_around_every_box_is_what_its_way_s_formula_says() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for theme in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(ShadowTesting::default(), theme);
        for kind in KINDS {
            set_all(&mut app, kind, &test_values(kind));
        }
        let image = app.render(&mut gpu).unwrap();
        save(&image, &format!("ShadowTesting{}", theme.name()));
        for kind in KINDS.into_iter().filter(|kind| kind.shaders().is_some()) {
            check_every_pixel(&app, &image, kind, theme);
        }
    }
}

// Fluent's shadow, which the interface draws: the box is blue, the shadow
// is under it, darker than the background below the box than above it, as
// its directional shadow falls down, and stronger at a higher Z.
#[test]
fn fluent_s_shadow_falls_below_the_box_and_grows_with_z() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = program();
    let tile = app.program.tile(Kind::Fluent);
    let rect = tile.boxed;
    let darkness = |image: &Image, x: f32, y: f32| {
        let [r, g, b, _] = image.pixel(x as u32, y as u32);
        let background = Theme::new(ThemeKind::Light).background.to_bytes();
        (background[0] as i32 + background[1] as i32 + background[2] as i32) - (r as i32 + g as i32 + b as i32)
    };
    let low = app.render(&mut gpu).unwrap();
    assert!(close(low.pixel(rect.center().x as u32, rect.center().y as u32), BOX_COLOR.to_bytes()));
    let below = darkness(&low, rect.center().x, rect.bottom() + 6.0);
    let above = darkness(&low, rect.center().x, rect.top() - 6.0);
    assert!(below > above && below > 10, "below {below}, above {above}");
    assert_eq!(darkness(&low, tile.stage.x + 2.0, tile.stage.y + 2.0), 0);
    set_all(&mut app, Kind::Fluent, &[128.0]);
    let high = app.render(&mut gpu).unwrap();
    assert!(darkness(&high, rect.center().x, rect.bottom() + 40.0) > darkness(&low, rect.center().x, rect.bottom() + 40.0));
}

// At each way's strongest settings, all at their maximum, and again with
// any offsets at their minimum, its shadow fades out before the edge of its
// stage: every pixel along the edge is the background.
#[test]
fn every_way_s_strongest_shadow_fades_out_on_its_stage() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let background = Theme::new(ThemeKind::Light).background.to_bytes();
    for minimum_offsets in [false, true] {
        let mut app = program();
        for kind in KINDS {
            let values: Vec<f32> = kind
                .settings()
                .iter()
                .map(|setting| if minimum_offsets && setting.name.starts_with("Offset") { setting.minimum } else { setting.maximum })
                .collect();
            set_all(&mut app, kind, &values);
        }
        let image = app.render(&mut gpu).unwrap();
        save(&image, &format!("ShadowTestingStrongest{}", if minimum_offsets { "Up" } else { "Down" }));
        for kind in KINDS {
            let stage = app.program.tile(kind).stage.round();
            let (left, top, right, bottom) = (stage.x as u32, stage.y as u32, stage.right() as u32 - 1, stage.bottom() as u32 - 1);
            let edge = (left..=right).flat_map(|x| [(x, top), (x, bottom)]).chain((top..=bottom).flat_map(|y| [(left, y), (right, y)]));
            for (x, y) in edge {
                assert!(close(image.pixel(x, y), background), "{}'s shadow reaches ({x}, {y}), on its stage's edge: {:?}", kind.name(), image.pixel(x, y));
            }
        }
    }
}

// A press on the linear shadow's length draws the shadow at its new length
// on the press's own frame.
#[test]
fn a_press_on_a_slider_draws_the_new_shadow_on_its_frame() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = program();
    let before = app.render(&mut gpu).unwrap();
    let tile = app.program.tile(Kind::Linear);
    // 60 pixels right of the box, past the default length.
    let (x, y) = (tile.boxed.right() as u32 + 59, tile.boxed.center().y as u32);
    assert!(close(before.pixel(x, y), Theme::new(ThemeKind::Light).background.to_bytes()));
    let at = on_rail(&app, "Linear/Length", 0.75);
    app.press_point(at);
    assert_eq!(app.program.approach(Kind::Linear).value("Length"), 96.0);
    let after = app.render(&mut gpu).unwrap();
    assert_ne!(after.pixel(x, y), before.pixel(x, y), "the shadow isn't 96 pixels long on the press's frame");
    check_every_pixel(&app, &after, Kind::Linear, ThemeKind::Light);
}
