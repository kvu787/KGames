// The card that test programs float over their scenes: its name, where it
// goes, and F1, which shows and hides it.

use user_interface::tokens::PAGE_PADDING;
use user_interface::{Context, Headless, InfoCard, Key, Program, Tree, Ui};

struct Test {
    shown: bool,
}

impl Program for Test {
    type Message = ();

    fn title(&self) -> String {
        "Test".to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<()> {
        if InfoCard::toggled(ui.input()) { vec![()] } else { Vec::new() }
    }

    fn update(&mut self, _toggle: (), _ui: &mut Context<()>) {
        self.shown = !self.shown;
    }

    fn view(&self, ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        if self.shown {
            tree.info_card(ui, &InfoCard {
                title: "A test",
                description: "What it shows",
                rows: &[("Row", "Value")],
                keys: &[("S", "Something")],
            });
        }
        tree
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
