// Tests ShadowTesting without a window, for every way of drawing an
// omnidirectional shadow: what each slider does, on the frame of the press,
// in the tiles' real layout, that the tiles are alike and every card fits in
// its tile, that every way's shadow is the same on every side, and, drawn
// offscreen with the GPU, that every pixel around each box is what the way's
// formula says, in both themes, mirrored across the box, and that each way's
// shadow fades out before the edge of its stage at its strongest settings.

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

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
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

// Settings for checking pixels, unlike the defaults, with a linear shadow
// whose end doesn't fade to nothing.
fn test_values(kind: Kind) -> Vec<f32> {
    match kind {
        Kind::Linear => vec![40.0, 0.6, 0.2],
        Kind::Curved => vec![40.0, 0.6, 1.7],
        Kind::Smoothstep => vec![40.0, 0.6],
        Kind::Exponential => vec![10.0, 0.6],
        Kind::Bell => vec![14.0, 0.6],
        Kind::Gaussian | Kind::RoundGaussian | Kind::BSpline | Kind::LinearLight => vec![20.0, 4.0, 0.5],
        Kind::Layered => vec![4.0, 40.0, 0.3],
    }
}

// Whether the way's shadow goes by the distance from the box alone, so that
// it's as dark diagonally out from a corner as straight out from a side.
fn round(kind: Kind) -> bool {
    !matches!(kind, Kind::Gaussian | Kind::BSpline | Kind::Layered | Kind::LinearLight)
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
// whole pixels, and every slider and Reset button is inside its way's card,
// inside its padding, so that everything that the card says fits in it.
#[test]
fn the_tiles_are_alike_and_each_way_s_controls_fit_in_its_card() {
    use user_interface::tokens::CARD_PADDING;
    let app = program();
    let first = app.program.tile(KINDS[0]);
    for kind in KINDS {
        let tile = app.program.tile(kind);
        assert_eq!(tile.boxed, tile.boxed.round(), "{}", kind.name());
        assert_eq!(tile.boxed.size(), BOX_SIZE);
        assert!((tile.stage.width - first.stage.width).abs() <= 1.0 && (tile.stage.height - first.stage.height).abs() <= 1.0, "{}", kind.name());
        assert!((tile.boxed.center().x - tile.stage.center().x).abs() <= 0.5 && (tile.boxed.center().y - tile.stage.center().y).abs() <= 0.5);
        let inside = tile.card.inset(CARD_PADDING);
        for name in kind.settings().iter().map(|setting| setting.name).chain(["Reset"]) {
            let name = slider(kind, name);
            let rect = app.target(&name);
            assert!(rect.x >= inside.x && rect.right() <= inside.right() && rect.y >= inside.y && rect.bottom() <= inside.bottom(), "{name} is at {rect:?}, out of its card's {inside:?}");
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

// Each way's Reset button is off while the way is at its defaults, on from
// the frame of a press that changes a setting, and sets that way's settings,
// and only that way's, back to their defaults on the frame of its press.
#[test]
fn each_way_s_reset_button_resets_only_that_way_on_its_frame() {
    let mut app = program();
    for kind in KINDS {
        assert!(!app.is_enabled(&slider(kind, "Reset")), "{}'s Reset is on at its defaults", kind.name());
    }
    for kind in KINDS {
        let first = slider(kind, kind.settings()[0].name);
        let at = on_rail(&app, &first, 0.9);
        app.press_point(at);
        assert_ne!(app.program.approach(kind), &Approach::new(kind), "{}", kind.name());
        assert!(app.is_enabled(&slider(kind, "Reset")), "{}'s Reset is off after a change", kind.name());
        app.release();
    }
    for (index, kind) in KINDS.iter().enumerate() {
        app.press(&slider(*kind, "Reset"));
        assert_eq!(app.program.approach(*kind), &Approach::new(*kind), "{}", kind.name());
        assert!(!app.is_enabled(&slider(*kind, "Reset")), "{}'s Reset is on after it reset", kind.name());
        for later in &KINDS[index + 1..] {
            assert_ne!(app.program.approach(*later), &Approach::new(*later), "{}'s Reset reset {}", kind.name(), later.name());
        }
        app.release();
    }
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

// Every way's shadow is the same on every side: as far out from the middle
// of any of a square box's four sides, it's as dark. The box is square
// because a blur wider than a long box is a little lighter off its short
// sides, with less of the box near them, which is still the same in every
// direction. All but the blurred boxes go by the distance from the box alone,
// so they're as dark diagonally out from a corner too.
#[test]
fn every_way_s_shadow_is_the_same_on_every_side() {
    let rect = Rect::new(0.0, 0.0, 120.0, 120.0);
    let center = rect.center();
    for kind in KINDS {
        let values = test_values(kind);
        for distance in [0.5, 3.0, 12.5, 25.0, 39.5, 60.0] {
            let right = kind.opacity(&values, rect, rect.right() + distance, center.y);
            let sides = [
                kind.opacity(&values, rect, rect.left() - distance, center.y),
                kind.opacity(&values, rect, center.x, rect.top() - distance),
                kind.opacity(&values, rect, center.x, rect.bottom() + distance),
            ];
            for side in sides {
                assert!(near(side, right), "{} at {distance}: {side} on one side and {right} on the right", kind.name());
            }
            if round(kind) {
                let diagonal = distance / std::f32::consts::SQRT_2;
                let corner = kind.opacity(&values, rect, rect.right() + diagonal, rect.bottom() + diagonal);
                assert!(near(corner, right), "{} at {distance}: {corner} off the corner and {right} on the right", kind.name());
            }
        }
    }
}

// Every way's defaults are matched, so that only the shape of their falloff
// differs: out from the middle of the box's side, each starts within a
// hundredth or two of the edge opacity at the first pixel, and is half that
// within a pixel of the half distance.
#[test]
fn every_way_s_defaults_start_as_dark_and_are_half_as_dark_as_far_out() {
    use shadow_testing::approaches::{DEFAULT_EDGE_OPACITY, DEFAULT_HALF_DISTANCE};
    let rect = Rect::new(0.0, 0.0, BOX_SIZE.width, BOX_SIZE.height);
    for kind in KINDS {
        let values = Approach::new(kind).values;
        let at = |distance: f32| kind.opacity(&values, rect, rect.right() + distance, rect.center().y);
        let edge = at(0.5);
        assert!((edge - DEFAULT_EDGE_OPACITY).abs() <= 0.02, "{} starts at {edge}", kind.name());
        let half = (0..10_000).map(|step| 0.5 + step as f32 * 0.01).find(|distance| at(*distance) <= DEFAULT_EDGE_OPACITY / 2.0).unwrap();
        assert!((half - DEFAULT_HALF_DISTANCE).abs() <= 1.0, "{} is half as dark {half} out", kind.name());
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

// The curved shadow at a power of 1 is the linear one fading to nothing, and
// at a higher power it's fainter everywhere between the box and the length.
#[test]
fn the_curved_shadow_at_a_power_of_1_is_linear_and_higher_powers_fade_faster() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let curved = |values: [f32; 3], x: f32| Kind::Curved.opacity(&values, rect, x, 5.0);
    for distance in [0.5, 7.0, 20.0, 39.5, 40.5, 60.0] {
        let x = 10.0 + distance;
        assert!(near(curved([40.0, 0.6, 1.0], x), Kind::Linear.opacity(&[40.0, 0.6, 0.0], rect, x, 5.0)), "{distance}");
        if (1.0..40.0).contains(&distance) {
            assert!(curved([40.0, 0.6, 2.0], x) < curved([40.0, 0.6, 1.0], x), "{distance}");
        }
    }
    assert!(near(curved([40.0, 0.6, 2.0], 30.0), 0.6 * 0.25));
}

// The smoothstep shadow: the opacity at the box's edge, half of it halfway
// to the length, none at the length, and an S curve, as far above half a
// quarter of the way out as it's below half three quarters of the way.
#[test]
fn the_smoothstep_shadow_fades_along_an_s_curve() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let at = |distance: f32| Kind::Smoothstep.opacity(&[40.0, 0.6], rect, 10.0 + distance, 5.0);
    assert!(near(at(0.0001), 0.6));
    assert!(near(at(20.0), 0.3));
    assert!(near(at(10.0) + at(30.0), 0.6));
    assert!(at(10.0) > 0.3 && at(10.0) < 0.6);
    assert!(near(at(40.5), 0.0));
}

// The exponential shadow: a decay length further out, it's a factor of e
// fainter, and its quad ends where it's the faintest that shows.
#[test]
fn the_exponential_shadow_falls_by_e_every_decay_length() {
    use shadow_testing::approaches::FAINTEST;
    use shadow_testing::exponential::reach;
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let values = [10.0, 0.6];
    let at = |distance: f32| Kind::Exponential.opacity(&values, rect, 10.0 + distance, 5.0);
    assert!(near(at(0.0), 0.6));
    for distance in [0.5, 4.0, 17.0] {
        assert!(near(at(distance + 10.0), at(distance) / std::f32::consts::E), "{distance}");
    }
    assert!(near(at(reach(&values)), FAINTEST));
}

// The bell shadow: a width out, it's e to the minus a half of its opacity,
// two widths out e to the minus two, and its quad ends where it's the
// faintest that shows.
#[test]
fn the_bell_shadow_falls_along_a_bell_curve_of_the_distance() {
    use shadow_testing::approaches::FAINTEST;
    use shadow_testing::bell::reach;
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let values = [14.0, 0.6];
    let at = |distance: f32| Kind::Bell.opacity(&values, rect, 10.0 + distance, 5.0);
    assert!(near(at(0.0), 0.6));
    assert!(near(at(14.0), 0.6 * (-0.5f32).exp()));
    assert!(near(at(28.0), 0.6 * (-2.0f32).exp()));
    assert!(near(at(reach(&values)), FAINTEST));
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
    let gaussian = |values: [f32; 3], rect: Rect, x: f32, y: f32| Kind::Gaussian.opacity(&values, rect, x, y);
    // A blur of 8 is a deviation of 4, without a spread.
    let values = [8.0, 0.0, 0.8];
    assert!(near(gaussian(values, rect, 400.0, 200.0), 0.4));
    assert!(near(gaussian(values, rect, 400.0, 400.0), 0.2));
    assert!(near(gaussian(values, rect, 200.0, 200.0), 0.8));
    // Grown by the spread on every side: its edges are 6 out.
    let spread = [8.0, 6.0, 0.8];
    assert!(near(gaussian(spread, rect, 406.0, 200.0), 0.4));
    assert!(near(gaussian(spread, rect, -6.0, 200.0), 0.4));
    assert!(near(gaussian(spread, rect, 200.0, -6.0), 0.4));
    // A spread that shrinks it to nothing leaves no shadow.
    let small = Rect::new(0.0, 0.0, 20.0, 20.0);
    assert_eq!(gaussian([8.0, -10.0, 0.8], small, 10.0, 10.0), 0.0);
}

// The round Gaussian is the Gaussian straight out from the middle of a long
// side, and as dark diagonally out from a corner, where the Gaussian is
// lighter.
#[test]
fn the_round_gaussian_is_the_gaussian_on_the_sides_and_round_at_the_corners() {
    let rect = Rect::new(0.0, 0.0, 400.0, 400.0);
    let values = [20.0, 4.0, 0.5];
    for distance in [0.5, 3.0, 10.0, 25.0] {
        let side = Kind::RoundGaussian.opacity(&values, rect, 400.0 + distance, 200.0);
        assert!(near(side, Kind::Gaussian.opacity(&values, rect, 400.0 + distance, 200.0)), "{distance}");
        let diagonal = distance / std::f32::consts::SQRT_2;
        let corner = Kind::RoundGaussian.opacity(&values, rect, 400.0 + diagonal, 400.0 + diagonal);
        assert!(near(corner, side), "{distance}");
        assert!(Kind::Gaussian.opacity(&values, rect, 400.0 + diagonal, 400.0 + diagonal) < corner, "{distance}");
    }
}

// The B-spline blur: its cumulative distribution goes from 0 to 1 between
// two knots either side, smoothly, and its shadow is within a hundredth of
// the Gaussian's everywhere along a side, but nothing past two knots.
#[test]
fn the_b_spline_blur_is_nearly_the_gaussian_but_ends_exactly() {
    use shadow_testing::b_spline::{knot, spline_below};
    assert_eq!(spline_below(-2.0), 0.0);
    assert_eq!(spline_below(0.0), 0.5);
    assert_eq!(spline_below(2.0), 1.0);
    assert_eq!(spline_below(5.0), 1.0);
    for t in [-1.5, -1.0, -0.3, 0.7, 1.0, 1.9] {
        assert!(near(spline_below(t) + spline_below(-t), 1.0), "{t}");
        // No steeper across a knot than the spline, whose peak is 2/3.
        assert!((spline_below(t + 1e-4) - spline_below(t - 1e-4)).abs() <= 2e-4 * 2.0 / 3.0 + 1e-6, "it jumps at {t}");
    }
    let rect = Rect::new(0.0, 0.0, 400.0, 400.0);
    let values = [20.0, 4.0, 0.5];
    let end = 404.0 + 2.0 * knot(&values);
    let mut distance = 0.5;
    while distance < end - 400.0 {
        let spline = Kind::BSpline.opacity(&values, rect, 400.0 + distance, 200.0);
        let gaussian = Kind::Gaussian.opacity(&values, rect, 400.0 + distance, 200.0);
        assert!((spline - gaussian).abs() < 0.01, "{distance}: {spline} and {gaussian}");
        distance += 0.5;
    }
    assert_eq!(Kind::BSpline.opacity(&values, rect, end + 0.01, 200.0), 0.0);
    assert!(Kind::Gaussian.opacity(&values, rect, end + 0.01, 200.0) > 0.0);
}

// Layered Gaussians: one layer is the Gaussian at the same blur and
// opacity, and more are Gaussians of half the blur each over each other,
// darker near the box, with the same long tail as the widest.
#[test]
fn layered_gaussians_are_gaussians_of_half_the_blur_over_each_other() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
    let layered = |values: [f32; 3], x: f32| Kind::Layered.opacity(&values, rect, x, 50.0);
    let gaussian = |blur: f32, opacity: f32, x: f32| Kind::Gaussian.opacity(&[blur, 0.0, opacity], rect, x, 50.0);
    for distance in [0.5, 4.0, 15.0, 40.0] {
        let x = 200.0 + distance;
        assert!(near(layered([1.0, 40.0, 0.3], x), gaussian(40.0, 0.3, x)), "{distance}");
        let through = (1.0 - gaussian(40.0, 0.3, x)) * (1.0 - gaussian(20.0, 0.3, x)) * (1.0 - gaussian(10.0, 0.3, x));
        assert!(near(layered([3.0, 40.0, 0.3], x), 1.0 - through), "{distance}");
        assert!(layered([3.0, 40.0, 0.3], x) >= layered([1.0, 40.0, 0.3], x), "{distance}");
    }
}

// What a pixel should be: the box's color over what it covers of the
// pixel, and the background, darkened by the way's shadow, over the rest, in
// sRGB, or in linear light for a way that blends in it.
fn expected(kind: Kind, values: &[f32], rect: Rect, background: Color, x: u32, y: u32) -> [u8; 4] {
    let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
    let inside = coverage(box_distance(rect, cx, cy));
    let shadow = kind.opacity(values, rect, cx, cy) * (1.0 - inside);
    let rest = 1.0 - inside - shadow;
    let mix = |boxed: f32, behind: f32| boxed * inside + behind * rest;
    if kind.linear_light() {
        let ([br, bg, bb], [r, g, b]) = (BOX_COLOR.to_linear(), background.to_linear());
        return Color::from_linear(mix(br, r), mix(bg, g), mix(bb, b)).to_bytes();
    }
    Color::rgb(mix(BOX_COLOR.r, background.r), mix(BOX_COLOR.g, background.g), mix(BOX_COLOR.b, background.b)).to_bytes()
}

// The Gaussian in linear light is the Gaussian, at the same settings and
// opacity, but where it darkens the background by an opacity in linear
// light, which leaves more of it than darkening it in sRGB.
#[test]
fn the_gaussian_in_linear_light_darkens_less_at_the_same_opacity() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
    let values = test_values(Kind::LinearLight);
    let background = Theme::new(ThemeKind::Light).background;
    for distance in [0.5, 4.0, 15.0] {
        let x = (200.0 + distance) as u32;
        assert_eq!(Kind::LinearLight.opacity(&values, rect, x as f32 + 0.5, 50.5), Kind::Gaussian.opacity(&values, rect, x as f32 + 0.5, 50.5));
        let linear = expected(Kind::LinearLight, &values, rect, background, x, 50);
        let srgb = expected(Kind::Gaussian, &values, rect, background, x, 50);
        assert!(linear[0] > srgb[0] && linear[1] > srgb[1] && linear[2] > srgb[2], "{distance}: {linear:?} isn't lighter than {srgb:?}");
    }
    // The box is the same blue either way.
    assert!(close(expected(Kind::LinearLight, &values, rect, background, 100, 50), BOX_COLOR.to_bytes()));
}

// Every pixel of the way's stage that its quad reaches, and a little past
// it, is what its formula says, and the same as the pixel mirrored across
// the box's middle, left to right and top to bottom.
fn check_every_pixel(app: &Headless<ShadowTesting>, image: &Image, kind: Kind, theme: ThemeKind) {
    let values = &app.program.approach(kind).values;
    let tile = app.program.tile(kind);
    let rect = tile.boxed;
    let background = Theme::new(theme).background;
    let area = kind.area(values, rect).expand(4.0).intersect(tile.stage);
    let (mirror_x, mirror_y) = ((rect.left() + rect.right()) as u32 - 1, (rect.top() + rect.bottom()) as u32 - 1);
    for y in area.y as u32..area.bottom() as u32 {
        for x in area.x as u32..area.right() as u32 {
            let want = expected(kind, values, rect, background, x, y);
            let got = image.pixel(x, y);
            assert!(close(got, want), "{} at {values:?} in the {} theme: ({x}, {y}) is {got:?}, not {want:?}", kind.name(), theme.name());
            let mirrored = image.pixel(mirror_x - x, mirror_y - y);
            assert!(close(got, mirrored), "{} at {values:?}: ({x}, {y}) is {got:?}, but mirrored across the box it's {mirrored:?}", kind.name());
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

// At each way's strongest settings, all at their maximum, its shadow fades
// out before the edge of its stage: every pixel along the edge is the
// background.
#[test]
fn every_way_s_strongest_shadow_fades_out_on_its_stage() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let background = Theme::new(ThemeKind::Light).background.to_bytes();
    let mut app = program();
    for kind in KINDS {
        let values: Vec<f32> = kind.settings().iter().map(|setting| setting.maximum).collect();
        set_all(&mut app, kind, &values);
    }
    let image = app.render(&mut gpu).unwrap();
    save(&image, "ShadowTestingStrongest");
    for kind in KINDS {
        let stage = app.program.tile(kind).stage.round();
        let (left, top, right, bottom) = (stage.x as u32, stage.y as u32, stage.right() as u32 - 1, stage.bottom() as u32 - 1);
        let edge = (left..=right).flat_map(|x| [(x, top), (x, bottom)]).chain((top..=bottom).flat_map(|y| [(left, y), (right, y)]));
        for (x, y) in edge {
            assert!(close(image.pixel(x, y), background), "{}'s shadow reaches ({x}, {y}), on its stage's edge: {:?}", kind.name(), image.pixel(x, y));
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
    assert_eq!(app.program.approach(Kind::Linear).value("Length"), 75.0);
    let after = app.render(&mut gpu).unwrap();
    assert_ne!(after.pixel(x, y), before.pixel(x, y), "the shadow isn't 75 pixels long on the press's frame");
    check_every_pixel(&app, &after, Kind::Linear, ThemeKind::Light);
}
