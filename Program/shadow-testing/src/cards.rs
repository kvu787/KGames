// Describes the page from the program's state: a column of cards on the
// left, one for each way of drawing a shadow, with sliders for its settings,
// and the stage, the rest of the page, which the interface leaves empty for
// render to draw the blue box and its shadow in, in the middle. Programs
// support one client area, 2560 x 1440 in borderless fullscreen, so the cards
// have fixed places in pixels.

use user_interface::tokens::CARD_GAP;
use user_interface::{AppBar, Card, Form, Icon, Rect, Slider, Status, Tree, Ui};

use crate::linear::{MAX_LENGTH, Setting};
use crate::shadow_testing::{BOX_SIZE, Message, ShadowTesting, TITLE};

// The column of cards.
pub const COLUMN_WIDTH: f32 = 400.0;

pub fn view(program: &ShadowTesting, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    let subtitle = "A box's shadow, drawn the ways being tried";
    let status = status_bar(ui);
    let shell = tree.shell(ui, AppBar::new(Icon::Contrast, TITLE).subtitle(subtitle), false, Some(status));
    let mut stage = shell.page;
    let column = stage.cut_left(COLUMN_WIDTH, CARD_GAP);
    linear(program, ui, &mut tree, column);
    program.box_rect.set(Some(stage.centered(BOX_SIZE).round()));
    tree
}

fn status_bar(ui: &mut Ui) -> Vec<Status> {
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    vec![
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::Spacer,
        Status::text(environment.gpu.clone()),
        Status::text(display.trim().to_string()),
    ]
}

// The linear shadow's card, as tall as its content, at the top of the
// rectangle.
fn linear(program: &ShadowTesting, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let linear = program.linear;
    tree.sized_card(ui, None, rect, Card::new("Linear shadow"), |ui, card, area| {
        let mut form = Form::new(area.content);
        form.paragraph(
            ui,
            card,
            "Pure black around the box, as far as its length. Its opacity goes in a straight line from the start's at the box's edge to the end's at the length.",
        );
        let height = Slider::<Message>::height(true);
        let length = Slider::new(linear.length, 0.0, MAX_LENGTH as f64)
            .step(1.0)
            .header("Length")
            .show_value(0)
            .suffix(" px")
            .on_change(|value| Message::Linear(Setting::Length, value));
        card.add("Length", form.take(height), length);
        let start = Slider::new(linear.start, 0.0, 1.0)
            .step(0.01)
            .header("Start opacity")
            .show_value(2)
            .on_change(|value| Message::Linear(Setting::Start, value));
        card.add("StartOpacity", form.take(height), start);
        let end = Slider::new(linear.end, 0.0, 1.0)
            .step(0.01)
            .header("End opacity")
            .show_value(2)
            .on_change(|value| Message::Linear(Setting::End, value));
        card.add("EndOpacity", form.take(height), end);
        form.bottom()
    });
}
