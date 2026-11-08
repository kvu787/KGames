// Laying a card's content out in fixed pixels, top to bottom: a Form gives
// rows for controls with their headers above them, as XAML's Header puts
// them, or labels beside them, and headings, captions and gaps between them,
// adding the text to the tree as it goes. Its `bottom` is where the content
// ends, for a card that sizes itself.
//
// And a 3D view's area, which takes every mouse button and the wheel without
// taking keyboard focus, so that the program's keys keep working while it's
// used, and sends what the pointer does in it.

use crate::core::interaction::{Cursor, Interaction, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Tree, Widget};
use crate::design::controls::TEXT_BOX_HEADER_GAP;
use crate::design::tokens::{BODY, BODY_STRONG, CAPTION, CARD_PADDING, CONTROL_HEIGHT, FORM_ROW_GAP, SPACE_S, SPACE_XS};
use crate::drawing::geometry::{Point, Rect};
use crate::input::devices::{Modifiers, MouseButton};
use crate::widgets::text::{Divider, Text};
use crate::widgets::widget::{Reaction, react};

pub struct Form {
    area: Rect,
    // Labels go beside controls, in a column this wide, or above them.
    label_width: Option<f32>,
    gap: f32,
    used: f32,
    // Whether everything taken so far fit in the area.
    fits: bool,
}

impl Form {
    // Headers above the controls.
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            label_width: None,
            gap: FORM_ROW_GAP,
            used: area.y,
            fits: true,
        }
    }

    // Labels beside the controls, in a column `label_width` wide.
    pub fn beside(area: Rect, label_width: f32) -> Self {
        Self {
            label_width: Some(label_width),
            ..Self::new(area)
        }
    }

    // The gap between rows.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    // A rectangle `height` tall for anything, and the gap after it.
    pub fn take(&mut self, height: f32) -> Rect {
        self.fits &= height <= self.area.height;
        let rect = self.area.cut_top(height, self.gap);
        self.used = rect.bottom();
        rect
    }

    // A control's row with its label, which goes in the tree: above the
    // control, or beside it. Returns the control's rectangle, `height` tall.
    pub fn row<M: 'static>(&mut self, tree: &mut Tree<M>, label: &str, height: f32) -> Rect {
        match self.label_width {
            Some(width) => {
                let row = self.take(height.max(CONTROL_HEIGHT));
                tree.draw(Rect::new(row.x, row.y, width - SPACE_S, CONTROL_HEIGHT.min(row.height)), Text::label(label));
                Rect::new(row.x + width, row.y, row.width - width, height)
            }
            None => {
                if !label.is_empty() {
                    let header = self.area.cut_top(BODY.line_height, TEXT_BOX_HEADER_GAP);
                    tree.draw(header, Text::label(label));
                }
                self.take(height)
            }
        }
    }

    // A control's row of the standard height.
    pub fn control<M: 'static>(&mut self, tree: &mut Tree<M>, label: &str) -> Rect {
        self.row(tree, label, CONTROL_HEIGHT)
    }

    // A heading in Body Strong, with a divider under it.
    pub fn heading<M: 'static>(&mut self, tree: &mut Tree<M>, text: &str) {
        let row = self.area.cut_top(BODY_STRONG.line_height + SPACE_XS + 1.0, self.gap);
        tree.draw(row.top_strip(BODY_STRONG.line_height), Text::heading(text));
        tree.draw(row.bottom_strip(1.0), Divider);
        self.used = row.bottom();
    }

    // A caption, wrapped, in the secondary color.
    pub fn caption<M: 'static>(&mut self, ui: &mut Ui, tree: &mut Tree<M>, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, CAPTION);
        let rect = self.take(height);
        tree.draw(rect, Text::caption(text).wrap());
    }

    // Body text, wrapped.
    pub fn paragraph<M: 'static>(&mut self, ui: &mut Ui, tree: &mut Tree<M>, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, BODY);
        let rect = self.take(height);
        tree.draw(rect, Text::paragraph(text));
    }

    // N columns of equal width, `height` tall, like a row of buttons.
    pub fn columns<const N: usize>(&mut self, height: f32) -> [Rect; N] {
        self.take(height).columns::<N>(SPACE_S)
    }

    // N controls side by side, each with its header above it, as `row` puts
    // one, in columns of equal width with a gap of a card's padding between
    // them. Returns the controls' rectangles, `height` tall.
    pub fn headed_columns<M: 'static, const N: usize>(&mut self, tree: &mut Tree<M>, labels: [&str; N], height: f32) -> [Rect; N] {
        let headed = labels.iter().any(|label| !label.is_empty());
        let header_height = if headed { BODY.line_height + TEXT_BOX_HEADER_GAP } else { 0.0 };
        let area = self.take(header_height + height);
        let columns = area.columns::<N>(CARD_PADDING);
        for (column, label) in columns.iter().zip(labels) {
            if !label.is_empty() {
                tree.draw(Rect::new(column.x, column.y, column.width, BODY.line_height), Text::label(label));
            }
        }
        columns.map(|column| Rect::new(column.x, column.y + header_height, column.width, height))
    }

    pub fn space(&mut self, height: f32) {
        self.area.cut_top(height, 0.0);
    }

    // What's left.
    pub fn rest(&self) -> Rect {
        self.area
    }

    // Where the content ends.
    pub fn bottom(&self) -> f32 {
        self.used
    }

    // Whether everything taken fit in the area, for content that may not.
    pub fn fits(&self) -> bool {
        self.fits
    }

    pub fn width(&self) -> f32 {
        self.area.width
    }
}

// A surface that the pointer stops at, like a card's, without a widget:
// what's under it isn't under the pointer.
pub struct Cover;

impl<M: 'static> Widget<M> for Cover {
    fn kind(&self) -> &'static str {
        "Surface"
    }

    fn covers(&self) -> bool {
        true
    }
}

// An element that only groups its children, like to disable a run of them
// at once, with no target and nothing drawn of its own.
pub struct Group;

impl<M: 'static> Widget<M> for Group {
    fn kind(&self) -> &'static str {
        "Group"
    }
}

// What the pointer did in a 3D view this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewInput {
    // The pointer is on the view, and where it is, while it's on the view or
    // holds it.
    pub hovered: bool,
    pub pointer: Option<Point>,
    // Each button's press on the view this frame.
    pub pressed: bool,
    pub secondary_pressed: bool,
    pub middle_pressed: bool,
    // The view is held, by which button, and how far the pointer moved
    // while it was.
    pub held: bool,
    pub held_button: Option<MouseButton>,
    pub drag_delta: Point,
    // The left press ended, and ended on the view.
    pub released: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub secondary_clicked: bool,
    // Notches of the wheel turned over it.
    pub wheel: Point,
    pub modifiers: Modifiers,
}

impl ViewInput {
    // Whether the left button holds it down.
    pub fn pressing(&self) -> bool {
        self.held && self.held_button == Some(MouseButton::Left)
    }

    pub fn holding(&self, button: MouseButton) -> bool {
        self.held && self.held_button == Some(button)
    }
}

// A 3D view's area. It takes presses of every mouse button and the wheel,
// but never keyboard focus, and a press in it takes focus from whatever had
// it, so that the program's keys work while it's used.
pub struct View<M> {
    cursor: Option<Cursor>,
    sense: Sense,
    kind: &'static str,
    on_input: Option<Reaction<ViewInput, M>>,
}

impl<M: Clone + 'static> View<M> {
    pub fn new() -> Self {
        Self {
            cursor: None,
            sense: Sense::VIEW,
            kind: "View",
            on_input: None,
        }
    }

    // What it takes from the pointer, instead of every button and the
    // wheel, like an area that only takes clicks.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    // What it is to scripts, instead of "View".
    pub fn kind(mut self, kind: &'static str) -> Self {
        self.kind = kind;
        self
    }

    // The pointer's look over the view.
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    // What the pointer doing anything in the view sends, each frame that it
    // does.
    pub fn on_input(mut self, reaction: impl Fn(ViewInput) -> M + 'static) -> Self {
        self.on_input = Some(react(reaction));
        self
    }
}

impl<M: Clone + 'static> Default for View<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Clone + 'static> Widget<M> for View<M> {
    fn kind(&self) -> &'static str {
        self.kind
    }

    fn sense(&self) -> Option<Sense> {
        Some(self.sense)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let hit = interaction.whole();
        if hit.pressed || hit.secondary_pressed || hit.middle_pressed {
            ctx.clear_focus();
        }
        if let Some(cursor) = self.cursor
            && (hit.hovered || hit.held)
        {
            ctx.set_cursor(cursor);
        }
        if let Some(reaction) = &self.on_input {
            let input = ViewInput {
                hovered: hit.hovered,
                pointer: interaction.pointer,
                pressed: hit.pressed,
                secondary_pressed: hit.secondary_pressed,
                middle_pressed: hit.middle_pressed,
                held: hit.held,
                held_button: hit.held_button,
                drag_delta: interaction.drag_delta,
                released: hit.released,
                clicked: hit.clicked,
                double_clicked: hit.double_clicked,
                secondary_clicked: hit.secondary_clicked,
                wheel: hit.wheel,
                modifiers: ctx.modifiers(),
            };
            ctx.send(reaction(input));
        }
    }
}
