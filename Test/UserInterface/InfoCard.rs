// The card that test programs float over their scenes: its name, where it
// goes, and F1, which shows and hides it.

use user_interface::tokens::PAGE_PADDING;
use user_interface::{Headless, InfoCard, Key, Program, Ui};

struct Test {
    shown: bool,
}

impl Program for Test {
    fn title(&self) -> String {
        "Test".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        if InfoCard::toggled(ui.input()) {
            self.shown = !self.shown;
        }
        if self.shown {
            ui.info_card(&InfoCard {
                title: "A test",
                description: "What it shows",
                rows: &[("Row", "Value")],
                keys: &[("S", "Something")],
            });
        }
    }
}

#[test]
fn the_card_floats_at_the_bottom_left_and_f1_hides_it() {
    let mut app = Headless::new(Test { shown: true });
    assert_eq!(app.label(InfoCard::NAME).as_deref(), Some("A test"));
    let card = app.target(InfoCard::NAME);
    assert_eq!((card.x, card.width), (PAGE_PADDING, InfoCard::WIDTH));
    assert_eq!(card.bottom(), app.size().height - PAGE_PADDING);
    app.key(Key::F1);
    assert!(!app.has_target(InfoCard::NAME));
    app.key(Key::F1);
    assert!(app.has_target(InfoCard::NAME));
}
