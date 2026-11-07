// A canvas: a node whose contents the program paints itself, like a gamepad's
// sticks or a palette of theme colors. It owns the data its painter reads,
// which the program swaps with `ui.get_mut::<Canvas<T>>(name).data = ...`;
// the painter is a plain function, so it can't hold stale borrows of the
// program. It can also sense the pointer, like a 3D view's area, and the
// program reads what the input did with `ui.response(name)`.

use crate::core::control::{AutoTarget, Control, NodeContext, UiEventKind};
use crate::core::ui::{Cursor, Sense, Ui};
use crate::drawing::geometry::Rect;

pub struct Canvas<T: 'static> {
    pub data: T,
    paint: fn(&mut Ui, Rect, &T),
    sense: Sense,
    cursor: Option<Cursor>,
    listed: bool,
    kind: &'static str,
}

impl<T: 'static> Canvas<T> {
    pub fn new(data: T, paint: fn(&mut Ui, Rect, &T)) -> Self {
        Self {
            data,
            paint,
            sense: Sense::NONE,
            cursor: None,
            listed: false,
            kind: "Canvas",
        }
    }

    // Takes the pointer: presses, clicks, drags and the wheel, which the
    // program reads with ui.response(name).
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self.listed = true;
        self
    }

    // The pointer's look over it.
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    // What it is to scripts and tests, like "View".
    pub fn kind(mut self, kind: &'static str) -> Self {
        self.kind = kind;
        self.listed = true;
        self
    }
}

impl<T: 'static> Control for Canvas<T> {
    fn kind(&self) -> &'static str {
        self.kind
    }

    fn sense(&self) -> Sense {
        self.sense
    }

    fn auto_target(&self) -> AutoTarget {
        if self.listed { AutoTarget::Listed } else { AutoTarget::Unlisted }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let response = &ctx.response;
        if response.clicked {
            ctx.emit(ui, UiEventKind::Clicked);
        }
        if response.double_clicked {
            ctx.emit(ui, UiEventKind::DoubleClicked);
        }
        if response.secondary_clicked {
            ctx.emit(ui, UiEventKind::SecondaryClicked);
        }
        if let Some(cursor) = self.cursor
            && (response.hovered || response.held)
        {
            ui.set_cursor(cursor);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        (self.paint)(ui, ctx.rect, &self.data);
    }
}
