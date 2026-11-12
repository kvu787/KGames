// Draws IdentifyDisplays' box for each display offscreen, without putting
// windows on the displays, and checks it: its size fits its text, its accent
// bar and identifier draw, and its last line says whether a box has focus.

use identify_displays::{FOCUSED_HINT, Identity, UNFOCUSED_HINT};
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::tokens::{BODY, SPACE_XXXL};
use user_interface::{Context, Fonts, Headless, Program, Size, Theme, ThemeKind, Tree, Ui};

struct Shown(Identity);

impl Program for Shown {
    type Message = ();

    fn title(&self) -> String {
        "Display identifier".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        tree.draw(ui.bounds(), self.0.clone());
        tree
    }
}

#[test]
fn each_display_gets_a_box_that_fits_its_identifier() {
    let displays = displays::enumerate().unwrap();
    assert!(!displays.is_empty());
    let mut fonts = Fonts::new().unwrap();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    for display in &displays {
        let identity = Identity::new(&[display]);
        let (width, height) = identity.size(&mut fonts);
        let scale = display.dpi as f32 / 96.0;
        assert!(width >= 360.0 * scale && height > 200.0 * scale, "{width} x {height}");
        let mut app = Headless::with_size(Shown(identity), ThemeKind::Light, Size::new(width, height));
        let image = app.render(&mut gpu).unwrap();
        write_png(&folder.join(format!("IdentifyDisplays{}.png", display.identifier)), &image).unwrap();
        let theme = Theme::light();
        let accent = theme.accent.to_bytes();
        let top = image.pixel(image.width / 2, 1);
        assert!(top.iter().zip(accent).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2), "{top:?}");
        // The identifier's dark strokes are somewhere in its line.
        let line = (4.0 * scale + 32.0 * scale) as u32;
        let dark = (line..line + (132.0 * scale) as u32)
            .flat_map(|y| (0..image.width).map(move |x| (x, y)))
            .any(|(x, y)| image.pixel(x, y)[0] < 64);
        assert!(dark, "the identifier didn't draw");
    }
}

// The last line says whether one of the boxes has focus, with a status icon:
// a click or Esc closes the boxes only then. Each display's box fits the
// longer of the two, as an overlay's window keeps its size.
#[test]
fn each_box_says_whether_a_click_or_esc_closes_them() {
    let displays = displays::enumerate().unwrap();
    let mut fonts = Fonts::new().unwrap();
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    let theme = Theme::light();
    for display in &displays {
        let scale = display.dpi as f32 / 96.0;
        let identity = Identity::new(&[display]);
        let (width, height) = identity.size(&mut fonts);
        let padding = (SPACE_XXXL * scale).round();
        let line = (BODY.line_height * scale).round();
        for hint in [FOCUSED_HINT, UNFOCUSED_HINT] {
            let hint_width = identity.hint_width(hint, &mut fonts);
            assert!(hint_width <= width - padding * 2.0, "{hint}: {hint_width} doesn't fit {width}");
        }
        for (focused, shown, hidden) in [(true, theme.success, theme.caution), (false, theme.caution, theme.success)] {
            let mut app = Headless::with_size(Shown(identity.clone().focused(focused)), ThemeKind::Light, Size::new(width, height));
            let image = app.render(&mut gpu).unwrap();
            let name = if focused { "Focused" } else { "NotFocused" };
            write_png(&folder.join(format!("IdentifyDisplays{}{name}.png", display.identifier)), &image).unwrap();
            let last = ((height - padding - line) as u32)..((height - padding) as u32);
            let has = |color: user_interface::Color| {
                let color = color.to_bytes();
                last.clone()
                    .flat_map(|y| (0..image.width).map(move |x| (x, y)))
                    .any(|(x, y)| image.pixel(x, y).iter().zip(color).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2))
            };
            assert!(has(shown), "{name}: the status icon's color isn't on the last line");
            assert!(!has(hidden), "{name}: the other status's color is on the last line");
        }
    }
}
