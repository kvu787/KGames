// Tests ShadowTesting without a window: what the sliders do, on the frame of
// the press, in the cards' real layout, that the box and its longest shadow
// fit on the stage, and, drawn offscreen with the GPU, that every pixel
// around the box is what the linear shadow's formula says, in both themes.

use shadow_testing::linear::{Linear, MAX_LENGTH, Setting, box_distance};
use shadow_testing::shadow_testing::{BOX_COLOR, BOX_SIZE, Message, ShadowTesting};
use user_interface::graphics::{AdapterChoice, Gpu, Image, write_png};
use user_interface::json::Value;
use user_interface::tokens::WINDOW_HEIGHT;
use user_interface::{Color, Headless, Point, Rect, Theme, ThemeKind, slider_rail};

fn program() -> Headless<ShadowTesting> {
    Headless::new(ShadowTesting::default())
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

// Where a slider's thumb goes for a fraction of its rail.
fn on_rail(app: &Headless<ShadowTesting>, name: &str, fraction: f32) -> Point {
    let rect = app.target(name);
    let (left, right) = slider_rail(rect);
    Point::new(left + (right - left) * fraction, rect.center().y)
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

// The box is on whole pixels, and its longest shadow is on the stage, clear
// of the cards and every widget.
#[test]
fn the_box_and_its_longest_shadow_fit_on_the_stage() {
    let mut app = program();
    app.frame();
    let rect = app.program.box_rect.get().expect("the view places the box");
    assert_eq!(rect, rect.round());
    assert_eq!(rect.size(), BOX_SIZE);
    let shadow = rect.expand(MAX_LENGTH + 1.0);
    for target in app.ui.targets() {
        assert!(!shadow.intersects(target.rect), "{} at {:?} is under the shadow at {shadow:?}", target.name, target.rect);
    }
    assert!(shadow.y > 120.0 && shadow.bottom() < WINDOW_HEIGHT - 60.0 && shadow.right() < 2560.0, "{shadow:?}");
}

// Each slider sets its setting on the frame of the press, where on the rail
// it is, and a drag past its end sets the most it allows.
#[test]
fn a_press_on_a_slider_sets_its_setting_and_a_drag_takes_it_to_its_end() {
    let mut app = program();
    let at = on_rail(&app, "Length", 0.25);
    app.press_point(at);
    assert_eq!(app.program.linear.length, 64.0);
    assert_eq!(app.label("Length").as_deref(), Some("64 px"));
    app.move_point(Point::new(1200.0, at.y)).release();
    assert_eq!(app.program.linear.length, MAX_LENGTH);

    let at = on_rail(&app, "StartOpacity", 0.75);
    app.press_point(at);
    assert_eq!(app.program.linear.start, 0.75);
    app.release();

    let at = on_rail(&app, "EndOpacity", 0.5);
    app.press_point(at);
    assert_eq!(app.program.linear.end, 0.5);
    app.move_point(Point::new(0.0, at.y)).release();
    assert_eq!(app.program.linear.end, 0.0);
}

#[test]
fn the_state_has_the_settings() {
    let mut app = program();
    app.send_message(Message::Linear(Setting::Start, 0.3));
    let state = app.state();
    let linear = state.get("Linear").expect("the state has the linear shadow");
    assert_eq!(linear.get("Length"), Some(&Value::Number(32.0)));
    assert_eq!(linear.get("StartOpacity"), Some(&Value::Number(0.3)));
    assert_eq!(linear.get("EndOpacity"), Some(&Value::Number(0.0)));
}

// The formula itself: the start's opacity half a pixel out from the box,
// down in a straight line to the end's at the length, and none past it or
// inside the box.
#[test]
fn the_linear_shadow_s_opacity_goes_from_the_start_to_the_end_in_a_straight_line() {
    let shadow = Linear { length: 40.0, start: 0.6, end: 0.2 };
    let near = |a: f32, b: f32| (a - b).abs() < 1e-6;
    assert!(near(shadow.opacity(-0.5), 0.0));
    assert!(near(shadow.opacity(0.5), 0.6 - 0.4 * 0.5 / 40.0));
    assert!(near(shadow.opacity(20.0), 0.4));
    assert!(near(shadow.opacity(39.5), 0.6 - 0.4 * 39.5 / 40.0));
    assert!(near(shadow.opacity(40.5), 0.0));
    let none = Linear { length: 0.0, ..shadow };
    assert!(near(none.opacity(0.5), 0.0));
    // Round around a corner: as far from the corner diagonally as straight
    // out from a side.
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    assert!(near(box_distance(rect, 13.0, 14.0), 5.0));
    assert!(near(box_distance(rect, 5.0, 13.0), 3.0));
    assert!(near(box_distance(rect, 5.0, 5.0), -5.0));
}

// What the pixel at the point should be: the box's color over what it
// covers of the pixel, and the background, darkened by the shadow, over the
// rest.
fn expected(shadow: &Linear, rect: Rect, background: Color, x: u32, y: u32) -> [u8; 4] {
    let distance = box_distance(rect, x as f32 + 0.5, y as f32 + 0.5);
    let inside = (0.5 - distance).clamp(0.0, 1.0);
    let rest = 1.0 - inside - shadow.opacity(distance);
    let mix = |boxed: f32, behind: f32| boxed * inside + behind * rest;
    Color::rgb(mix(BOX_COLOR.r, background.r), mix(BOX_COLOR.g, background.g), mix(BOX_COLOR.b, background.b)).to_bytes()
}

fn check_every_pixel_around_the_box(app: &Headless<ShadowTesting>, image: &Image, kind: ThemeKind) {
    let shadow = app.program.linear;
    let rect = app.program.box_rect.get().unwrap();
    let background = Theme::new(kind).background;
    let area = rect.expand(shadow.length + 4.0);
    for y in area.y as u32..area.bottom() as u32 {
        for x in area.x as u32..area.right() as u32 {
            let want = expected(&shadow, rect, background, x, y);
            let got = image.pixel(x, y);
            assert!(close(got, want), "with {shadow:?} in the {} theme, ({x}, {y}) is {got:?}, not {want:?}", kind.name());
        }
    }
}

// Every pixel around the box, and in it, is what the formula says: the
// box's blue, and the background darkened by the shadow's opacity at the
// pixel's center's distance from the box, round around the corners.
#[test]
fn every_pixel_around_the_box_is_what_the_linear_shadow_s_formula_says() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(ShadowTesting::default(), kind);
        app.send_message(Message::Linear(Setting::Length, 40.0));
        app.send_message(Message::Linear(Setting::Start, 0.6));
        app.send_message(Message::Linear(Setting::End, 0.2));
        let image = app.render(&mut gpu).unwrap();
        save(&image, &format!("ShadowTesting{}", kind.name()));
        check_every_pixel_around_the_box(&app, &image, kind);
    }
}

// A press on the length's slider draws the shadow at its new length on the
// press's own frame.
#[test]
fn a_press_on_a_slider_draws_the_new_shadow_on_its_frame() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = program();
    let before = app.render(&mut gpu).unwrap();
    let rect = app.program.box_rect.get().unwrap();
    // 100 pixels right of the box, past the first length.
    let (x, y) = (rect.right() as u32 + 99, rect.center().y as u32);
    assert!(close(before.pixel(x, y), Theme::new(ThemeKind::Light).background.to_bytes()));
    let at = on_rail(&app, "Length", 0.5);
    app.press_point(at);
    assert_eq!(app.program.linear.length, 128.0);
    let after = app.render(&mut gpu).unwrap();
    assert_ne!(after.pixel(x, y), before.pixel(x, y), "the shadow isn't 128 pixels long on the press's frame");
    check_every_pixel_around_the_box(&app, &after, ThemeKind::Light);
}
