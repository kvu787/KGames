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
            app.move_point(Point::new(2000.0, at.y)).release();
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
    let at = |distance: f32| Kind::Linear.opacity(&values, rect, 10.0 + distance, 5.0);
    let near = |a: f32, b: f32| (a - b).abs() < 1e-6;
    assert!(near(at(0.5), 0.6 - 0.4 * 0.5 / 40.0));
    assert!(near(at(20.0), 0.4));
    assert!(near(at(39.5), 0.6 - 0.4 * 39.5 / 40.0));
    assert!(near(at(40.5), 0.0));
    assert!(near(Kind::Linear.opacity(&[0.0, 0.6, 0.2], rect, 10.5, 5.0), 0.0));
    // As far from a corner diagonally as straight out from a side.
    assert!(near(box_distance(rect, 13.0, 14.0), 5.0));
    assert!(near(box_distance(rect, 5.0, 13.0), 3.0));
    assert!(near(box_distance(rect, 5.0, 5.0), -5.0));
}

// What a pixel should be: the box's color over what it covers of the
// pixel, and the background, darkened by the way's shadow, over the rest.
fn expected(kind: Kind, values: &[f32], rect: Rect, background: Color, x: u32, y: u32) -> [u8; 4] {
    let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
    let inside = coverage(box_distance(rect, cx, cy));
    let shadow = kind.opacity(values, rect, cx, cy) * (1.0 - inside);
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
// in both themes.
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
        for kind in KINDS {
            check_every_pixel(&app, &image, kind, theme);
        }
    }
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
