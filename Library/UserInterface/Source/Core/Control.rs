// What a retained control is: an object in the window's tree that owns its
// state, reacts to the input that lands on it, and draws itself. A program
// makes one per control in Program::build, with a name and a fixed-pixel
// rectangle, and from then on talks to it through events and setters.
//
// A frame runs in three phases: every visible control handles its input and
// emits events; the program's update reads the events and changes the tree;
// and every visible control draws. So a click's effect shows the same frame.

use std::any::Any;

use crate::core::ui::{Response, Sense, Ui};
use crate::drawing::geometry::{Color, Rect};
use crate::widgets::dialog::DialogResult;

// How a node registers itself as a target for the pointer, scripts and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AutoTarget {
    // A target under its name, which scripts see.
    #[default]
    Listed,
    // Covers what's under it without being seen, like a card.
    Unlisted,
    // The control registers its own targets in handle, like a menu bar's
    // items or a dialog's buttons.
    None,
}

// What a control knows of its node while it handles input or draws.
#[derive(Clone, Debug, Default)]
pub struct NodeContext {
    // The node's name, unique in the window.
    pub name: String,
    pub rect: Rect,
    // What the input did to the node this frame, from the dispatch phase.
    pub response: Response,
}

impl NodeContext {
    // Emits an event from this node.
    pub fn emit(&self, ui: &mut Ui, kind: UiEventKind) {
        ui.emit(&self.name, kind);
    }
}

// A control in the tree. It owns its state: a slider its value, a text box
// its text. It reacts to input in `handle`, which may emit events, and paints
// in `draw`. Geometry that `handle` computes and `draw` needs is kept in the
// control between the two.
pub trait Control: Any {
    // Like "Button" or "Slider", as WinUI names its controls.
    fn kind(&self) -> &'static str;

    fn sense(&self) -> Sense {
        Sense::NONE
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::Listed
    }

    // The part of the node's rectangle that is its target, like a slider's
    // track row under its header.
    fn target_rect(&self, rect: Rect) -> Rect {
        rect
    }

    // What it shows, like a button's text, for scripts and tests.
    fn label(&self) -> Option<String> {
        None
    }

    // Whether the node's children show, like an expander's content while
    // it's open, or a dialog's while it is.
    fn children_visible(&self) -> bool {
        true
    }

    // The layer the node's children go on, like a dialog's content on the
    // dialog layer. None leaves them where the node is.
    fn child_layer(&self) -> Option<crate::drawing::painter::Layer> {
        None
    }

    // Takes what the user put into this node's predecessor when the tree is
    // rebuilt, like a text box's text. `previous` is the same type as self.
    fn adopt(&mut self, _previous: Box<dyn Control>) {}

    // Reacts to the frame's input on it: mutates its own state and emits
    // events. ctx.response is what the input did to its target.
    fn handle(&mut self, _ui: &mut Ui, _ctx: &NodeContext) {}

    // Paints it. State is already settled: handle ran, then the program's
    // update.
    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext);
}

// The predecessor as its concrete type, for Control::adopt:
// `if let Some(previous) = adopted::<TextBox>(previous) { self.text = previous.text; }`
pub fn adopted<T: Control>(previous: Box<dyn Control>) -> Option<Box<T>> {
    (previous as Box<dyn Any>).downcast().ok()
}

// What the input did to a control, told to the program. `source` is the
// node's name.
#[derive(Clone, Debug, PartialEq)]
pub struct UiEvent {
    pub source: String,
    pub kind: UiEventKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventKind {
    // A button-like control was chosen, with a click or Space or Enter.
    Clicked,
    DoubleClicked,
    // Right-clicked, which asks for a context menu.
    SecondaryClicked,
    // A check box, switch or toggle button, with its new state.
    Toggled(bool),
    // A slider's or number box's value moved.
    Changed(f64),
    // A choice among indexed options: a combo box, radio group, segmented
    // control, selector bar, or a list's selection.
    Selected(usize),
    // A list's row was opened, with a double-click or Enter.
    Activated(usize),
    // A text box's text, as it changes, and when it's submitted with Enter.
    TextChanged(String),
    Submitted(String),
    ColorChanged(Color),
    // The label of the item chosen from a menu.
    Chose(String),
    // A dialog's button was chosen.
    Dialog(DialogResult),
    // An info bar's close button.
    Closed,
    // An expander opened or closed.
    Expanded(bool),
}
