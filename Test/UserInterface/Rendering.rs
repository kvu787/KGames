// Draws frames offscreen with the GPU, without a window, and checks their
// pixels: the theme's background, a card, an accent button's fill and label,
// a standard button's elevation border, and the page for an unsupported
// setup.

use user_interface::graphics::{AdapterChoice, Gpu, Image, write_png};
use user_interface::{Button, Card, Headless, Program, Rect, Theme, ThemeKind, Ui};

struct Page;

impl Program for Page {
    fn title(&self) -> String {
        "Page".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        let card = ui.card(Rect::new(40.0, 40.0, 400.0, 200.0), Card::new("A card"));
        let [accent, standard] = card.content.top_strip(32.0).columns::<2>(8.0);
        ui.add("Accent", accent, Button::new("Accent").accent());
        ui.add("Standard", standard, Button::new("Standard"));
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

fn save(image: &Image, name: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    write_png(&folder.join(format!("{name}.png")), image).unwrap();
}

#[test]
fn a_frame_draws_the_background_the_widgets_and_their_text() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Page, kind);
        let image = app.render(&mut gpu).unwrap();
        let theme = Theme::new(kind);
        save(&image, &format!("Page{}", kind.name()));
        assert_eq!((image.width, image.height), (2560, 1369));
        assert!(close(image.pixel(1000, 1000), theme.background.to_bytes()), "{:?}", image.pixel(1000, 1000));
        // The card is its fill over the background.
        assert!(close(image.pixel(400, 220), theme.card_opaque().to_bytes()), "{:?}", image.pixel(400, 220));
        // The accent button's fill, between its label and its edge.
        let button = app.target("Accent");
        let fill = image.pixel(button.x as u32 + 6, button.center().y as u32);
        assert!(close(fill, theme.accent.over(theme.card_opaque()).to_bytes()), "{fill:?}");
        // Its label is in text_on_accent: some pixel in its middle is.
        let mut found = false;
        for x in button.x as u32 + 40..button.right() as u32 - 40 {
            for y in button.y as u32 + 8..button.bottom() as u32 - 8 {
                found |= close(image.pixel(x, y), theme.text_on_accent.to_bytes());
            }
        }
        assert!(found, "the button's label didn't draw");
        // A standard button's elevation border is darker on one edge than
        // the other: the bottom in the light theme, the top in the dark one.
        let standard = app.target("Standard");
        let x = standard.center().x as u32 + 40;
        let top = image.pixel(x, standard.y as u32);
        let bottom = image.pixel(x, standard.bottom() as u32 - 1);
        let (stronger, weaker) = match kind {
            ThemeKind::Light => (bottom, top),
            ThemeKind::Dark => (top, bottom),
        };
        let brightness = |pixel: [u8; 4]| pixel[0] as i32 + pixel[1] as i32 + pixel[2] as i32;
        match kind {
            ThemeKind::Light => assert!(brightness(stronger) < brightness(weaker), "{top:?} {bottom:?}"),
            ThemeKind::Dark => assert!(brightness(stronger) > brightness(weaker), "{top:?} {bottom:?}"),
        }
    }
}

struct Unsupported;

impl Program for Unsupported {
    fn title(&self) -> String {
        "Unsupported".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.unsupported_setup(&[
            "The display is 1920 × 1080 pixels. It must be 2560 × 1440.".to_string(),
            "The window must be maximized, or borderless fullscreen with F11.".to_string(),
        ]);
    }
}

#[test]
fn an_unsupported_setup_says_what_is_wrong() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::with_size(Unsupported, ThemeKind::Light, user_interface::Size::new(1280.0, 720.0));
    let image = app.render(&mut gpu).unwrap();
    save(&image, "UnsupportedSetup");
    // The window's corner is the background, and the box's middle has the
    // critical background in it.
    let theme = Theme::light();
    assert!(close(image.pixel(5, 5), theme.background.to_bytes()));
    let critical = theme.critical_background.to_bytes();
    let found = (0..1280).step_by(8).any(|x| close(image.pixel(x, 360), critical));
    assert!(found);
}
