// Draws frames offscreen with the GPU, without a window, and checks their
// pixels: the theme's background, a button's fill, and its label.

use user_interface::graphics::{AdapterChoice, Gpu, write_png};
use user_interface::{Button, Headless, Program, Rect, Theme, ThemeKind, Ui};

struct Card;

impl Program for Card {
    fn title(&self) -> String {
        "Card".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        let card = ui.card(Rect::new(40.0, 40.0, 400.0, 200.0), "A card");
        ui.button("Accent", Rect::new(card.content.x, card.content.y, 160.0, 32.0), Button::new("Accent").accent());
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

#[test]
fn a_frame_draws_the_background_the_widgets_and_their_text() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Card, kind);
        let image = app.render(&mut gpu).unwrap();
        let theme = Theme::new(kind);
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
        std::fs::create_dir_all(&folder).unwrap();
        write_png(&folder.join(format!("Card{}.png", kind.name())), &image).unwrap();

        assert_eq!((image.width, image.height), (2560, 1369));
        let background = theme.background.to_bytes();
        assert!(close(image.pixel(1000, 1000), background), "{:?}", image.pixel(1000, 1000));
        // The accent button's fill, between its label and its edge.
        let button = app.target("Accent");
        let fill = image.pixel(button.x as u32 + 6, button.center().y as u32);
        assert!(close(fill, theme.accent.over(theme.background).to_bytes()), "{fill:?}");
        // Its label is in text_on_accent: some pixel in its middle is.
        let mut found = false;
        for x in button.x as u32 + 40..button.right() as u32 - 40 {
            for y in button.y as u32 + 8..button.bottom() as u32 - 8 {
                found |= close(image.pixel(x, y), theme.text_on_accent.to_bytes());
            }
        }
        assert!(found, "the button's label didn't draw");
    }
}
