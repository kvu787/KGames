// Describes the page from the program's state: a tile for each way of
// drawing a shadow, three to a row, each a card with sliders for the way's
// settings beside its stage, which the interface leaves empty for render to
// draw the blue box and its shadow on, in the middle. Every stage is the same
// size, with the same box, so the ways can be compared side by side.
// Programs support one client area, 2560 x 1440 in borderless fullscreen, so
// the tiles have fixed places in pixels.

use user_interface::tokens::CARD_GAP;
use user_interface::{AppBar, Card, Form, Icon, Rect, Slider, Status, Tree, Ui};

use crate::approaches::{KINDS, Kind};
use crate::shadow_testing::{BOX_SIZE, Message, ShadowTesting, TITLE, Tile};

// Tiles to a row, and rows.
pub const COLUMNS: usize = 3;
pub const ROWS: usize = 2;
// Each tile's card.
pub const CARD_WIDTH: f32 = 320.0;

pub fn view(program: &ShadowTesting, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    let subtitle = "A box's shadow, drawn each of the ways being tried, side by side";
    let status = status_bar(ui);
    let shell = tree.shell(ui, AppBar::new(Icon::Contrast, TITLE).subtitle(subtitle), false, Some(status));
    let tiles = layout(shell.page);
    for (kind, tile) in KINDS.iter().zip(tiles) {
        card(program, ui, &mut tree, *kind, tile);
    }
    program.tiles.set(Some(tiles));
    tree
}

// Each way's tile in the page, in KINDS' order, three to a row.
pub fn layout(page: Rect) -> [Tile; KINDS.len()] {
    let rows = page.rows::<ROWS>(CARD_GAP);
    std::array::from_fn(|index| {
        let area = rows[index / COLUMNS].columns::<COLUMNS>(CARD_GAP)[index % COLUMNS];
        let mut stage = area;
        stage.cut_left(CARD_WIDTH, CARD_GAP);
        Tile { area, stage, boxed: stage.centered(BOX_SIZE).round() }
    })
}

fn status_bar(ui: &mut Ui) -> Vec<Status> {
    let environment = ui.environment().clone();
    let display = format!("{} {}", environment.display, environment.display_name);
    vec![
        Status::shortcut("Ctrl+R", "Reset"),
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::Spacer,
        Status::text(environment.gpu.clone()),
        Status::text(display.trim().to_string()),
    ]
}

// A way's card, as tall as its content, at the top left of its tile: what
// it draws, and a slider for each of its settings, named after the way and
// the setting, like "Linear/Length".
fn card(program: &ShadowTesting, ui: &mut Ui, tree: &mut Tree<Message>, kind: Kind, tile: Tile) {
    let values = program.approach(kind).values.clone();
    let rect = tile.area.left_strip(CARD_WIDTH);
    tree.sized_card(ui, None, rect, Card::new(kind.title()), |ui, card, area| {
        let mut form = Form::new(area.content);
        form.paragraph(ui, card, kind.description());
        let height = Slider::<Message>::height(true);
        for (index, setting) in kind.settings().iter().enumerate() {
            let slider = Slider::new(values[index], setting.minimum as f64, setting.maximum as f64)
                .step(setting.step as f64)
                .header(setting.label)
                .show_value(setting.decimals)
                .suffix(setting.suffix)
                .on_change(move |value| Message::Set(kind, index, value));
            card.add(&format!("{}/{}", kind.name(), setting.name), form.take(height), slider);
        }
        form.bottom()
    });
}
