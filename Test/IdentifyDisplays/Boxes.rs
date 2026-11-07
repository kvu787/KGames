// Draws IdentifyDisplays' box for each display offscreen, without putting
// windows on the displays, and checks it: its size fits its text, and its
// accent bar and identifier draw.

use identify_displays::Identity;
use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::{Fonts, Headless, Program, Size, Theme, ThemeKind, Ui};

struct Shown(Identity);

impl Program for Shown {
    fn title(&self) -> String {
        "Display identifier".to_string()
    }

    fn build(&mut self, _ui: &mut Ui) {}

    fn update(&mut self, ui: &mut Ui) {
        self.0.draw(ui);
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
