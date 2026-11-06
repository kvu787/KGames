// What the widgets share: their visual states, and the ring that shows
// keyboard focus.

use crate::geometry::{Color, Rect};
use crate::painter::{Corners, Stroke};
use crate::tokens::{FOCUS_OFFSET, RADIUS_CONTROL};
use crate::ui::{Response, Ui};

// A control's look follows its state, as WinUI's visual states do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Rest,
    Hover,
    Pressed,
    Disabled,
}

impl Visual {
    pub fn of(ui: &Ui, response: &Response) -> Visual {
        if !ui.enabled() {
            Visual::Disabled
        } else if response.held && response.hovered {
            Visual::Pressed
        } else if response.hovered {
            Visual::Hover
        } else {
            // Including a control whose press slid off it.
            Visual::Rest
        }
    }

    // One of four values, by state.
    pub fn pick<T: Copy>(self, rest: T, hover: T, pressed: T, disabled: T) -> T {
        match self {
            Visual::Rest => rest,
            Visual::Hover => hover,
            Visual::Pressed => pressed,
            Visual::Disabled => disabled,
        }
    }
}

// The two rings that show keyboard focus, FOCUS_OFFSET outside the control,
// if the keyboard moved focus to it.
pub fn focus_ring(ui: &mut Ui, response: &Response, rect: Rect, radius: f32) {
    if !response.focused || !ui.focus_visible() {
        return;
    }
    let outer_color = ui.theme().focus_outer;
    let inner_color = ui.theme().focus_inner;
    let mut painter = ui.painter();
    // The inner ring, 1 pixel, then the outer one, 2 pixels, around it.
    let inner = rect.expand(FOCUS_OFFSET - 2.0);
    painter.shape(
        inner,
        Corners::all(radius + FOCUS_OFFSET - 2.0),
        Color::TRANSPARENT,
        Some(Stroke::new(inner_color, 1.0)),
    );
    let outer = rect.expand(FOCUS_OFFSET);
    painter.shape(outer, Corners::all(radius + FOCUS_OFFSET), Color::TRANSPARENT, Some(Stroke::new(outer_color, 2.0)));
}

pub fn control_focus_ring(ui: &mut Ui, response: &Response) {
    focus_ring(ui, response, response.rect, RADIUS_CONTROL);
}
