// The window's tree of retained controls. Program::build makes it: nodes
// with names, fixed-pixel rectangles and controls, nested in groups. It
// lives until the client area changes or the program asks to rebuild, and
// the program talks to it through setters, typed access and events.
//
// Each frame walks the visible tree twice: dispatch, where every node
// registers its target and handles its input, and draw, after the program's
// update, where every node paints.

use std::any::Any;
use std::collections::HashMap;

use crate::core::control::{AutoTarget, Control, NodeContext, UiEvent, UiEventKind};
use crate::core::ui::{Id, Response, Ui, id};
use crate::drawing::geometry::{Color, Rect};
use crate::widgets::dialog::DialogResult;

pub(crate) struct Node {
    pub name: String,
    pub id: Id,
    // Taken out while the control runs, so that it can borrow the Ui.
    pub control: Option<Box<dyn Control>>,
    pub rect: Rect,
    pub visible: bool,
    pub enabled: bool,
    // Whether it and its parts may take keyboard focus, like a toolbar's
    // segments that shouldn't.
    pub focusable: bool,
    pub children: Vec<usize>,
    // What the input did to it in this frame's dispatch.
    pub response: Response,
    // The frame that registered its target, so a node shown after dispatch
    // still gets one.
    pub interacted: u64,
}

pub(crate) struct Tree {
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
    pub index: HashMap<Id, usize>,
    // The group being built into, while building.
    stack: Vec<usize>,
    building: bool,
    // What new nodes' focusable starts as: false for a program whose keys
    // must stay its own, which turns it back on for its text fields.
    focusable_default: bool,
}

impl Default for Tree {
    fn default() -> Self {
        Tree {
            nodes: Vec::new(),
            roots: Vec::new(),
            index: HashMap::new(),
            stack: Vec::new(),
            building: false,
            focusable_default: true,
        }
    }
}

impl Ui {
    // Starts a build: empties the tree, keeping the old nodes so that new
    // ones with the same names adopt their state, and calls `build`.
    pub(crate) fn build_tree(&mut self, build: impl FnOnce(&mut Ui)) {
        let old = std::mem::take(&mut self.tree);
        self.previous_nodes = old.nodes.into_iter().map(|node| (node.id, node.control)).collect();
        self.tree.building = true;
        self.tree.focusable_default = true;
        self.rebuild_requested = false;
        build(self);
        self.tree.building = false;
        self.tree.stack.clear();
        self.previous_nodes.clear();
    }

    pub fn request_rebuild(&mut self) {
        self.rebuild_requested = true;
    }

    pub(crate) fn rebuild_requested(&self) -> bool {
        self.rebuild_requested
    }

    pub(crate) fn tree_is_empty(&self) -> bool {
        self.tree.nodes.is_empty()
    }

    // Adds a control under the name, in the rectangle, to the group being
    // built, and returns the rectangle. Only in Program::build.
    pub fn node(&mut self, name: &str, rect: Rect, control: impl Control) -> Rect {
        self.insert(name, rect, Box::new(control));
        rect
    }

    // Adds a control whose children `children` builds, like a card with what
    // it holds or a group that shows and hides together.
    pub fn group(&mut self, name: &str, rect: Rect, control: impl Control, children: impl FnOnce(&mut Ui)) -> Rect {
        let index = self.insert(name, rect, Box::new(control));
        self.tree.stack.push(index);
        children(self);
        self.tree.stack.pop();
        rect
    }

    fn insert(&mut self, name: &str, rect: Rect, mut control: Box<dyn Control>) -> usize {
        assert!(self.tree.building, "nodes are made in Program::build, not during a frame");
        let node_id = id(name);
        assert!(!self.tree.index.contains_key(&node_id), "there's already a node named {name}");
        // A rebuilt node keeps what the user put in its predecessor, like a
        // text box's text.
        if let Some(previous) = self.previous_nodes.remove(&node_id).flatten()
            && (*previous).type_id() == (*control).type_id()
        {
            control.adopt(previous);
        }
        let index = self.tree.nodes.len();
        self.tree.nodes.push(Node {
            name: name.to_string(),
            id: node_id,
            control: Some(control),
            rect,
            visible: true,
            enabled: true,
            focusable: self.tree.focusable_default,
            children: Vec::new(),
            response: Response::default(),
            interacted: 0,
        });
        match self.tree.stack.last() {
            Some(parent) => {
                let parent = *parent;
                self.tree.nodes[parent].children.push(index);
            }
            None => self.tree.roots.push(index),
        }
        self.tree.index.insert(node_id, index);
        index
    }

    fn node_index(&self, name: &str) -> usize {
        match self.tree.index.get(&id(name)) {
            Some(index) => *index,
            None => panic!("there's no node named {name}"),
        }
    }

    // The control of the node with the name, which must be a T.
    pub fn get<T: Control>(&self, name: &str) -> &T {
        let node = &self.tree.nodes[self.node_index(name)];
        let control = node.control.as_deref().unwrap_or_else(|| panic!("{name} can't be read while it runs"));
        match (control as &dyn Any).downcast_ref::<T>() {
            Some(control) => control,
            None => panic!("{name} is a {}, not a {}", control.kind(), std::any::type_name::<T>()),
        }
    }

    pub fn get_mut<T: Control>(&mut self, name: &str) -> &mut T {
        let index = self.node_index(name);
        let node = &mut self.tree.nodes[index];
        let kind = node.control.as_deref().map(Control::kind);
        match node.control.as_deref_mut().and_then(|control| (control as &mut dyn Any).downcast_mut::<T>()) {
            Some(control) => control,
            None => match kind {
                Some(kind) => panic!("{name} is a {kind}, not a {}", std::any::type_name::<T>()),
                None => panic!("{name} can't be changed while it runs"),
            },
        }
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.tree.index.contains_key(&id(name))
    }

    // Shows or hides the node and everything under it.
    pub fn set_visible(&mut self, name: &str, visible: bool) {
        let index = self.node_index(name);
        self.tree.nodes[index].visible = visible;
    }

    pub fn visible(&self, name: &str) -> bool {
        self.tree.nodes[self.node_index(name)].visible
    }

    // Enables or disables the node and everything under it.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        let index = self.node_index(name);
        self.tree.nodes[index].enabled = enabled;
    }

    // Whether the node and everything under it may take keyboard focus, for
    // controls like a toolbar's that shouldn't.
    pub fn set_focusable(&mut self, name: &str, focusable: bool) {
        let index = self.node_index(name);
        self.tree.nodes[index].focusable = focusable;
    }

    // What new nodes' focusable starts as, for the rest of the build: false
    // for a program whose keys must stay its own, which turns it back on for
    // its text fields with set_focusable.
    pub fn set_build_focusable(&mut self, focusable: bool) {
        self.tree.focusable_default = focusable;
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.tree.nodes[self.node_index(name)].enabled
    }

    // Moves the node, for the few that move without a rebuild.
    pub fn set_rect(&mut self, name: &str, rect: Rect) {
        let index = self.node_index(name);
        self.tree.nodes[index].rect = rect;
    }

    pub fn node_rect(&self, name: &str) -> Rect {
        self.tree.nodes[self.node_index(name)].rect
    }

    // What the input did to the node this frame.
    pub fn response(&self, name: &str) -> Response {
        self.tree.nodes[self.node_index(name)].response
    }

    // The dispatch phase: every visible node registers its target and
    // handles its input, parents before children, in build order.
    pub(crate) fn dispatch_tree(&mut self) {
        let roots = self.tree.roots.clone();
        for index in roots {
            self.run_node(index, true, true);
        }
    }

    // The draw phase: every visible node paints, in the same order.
    pub(crate) fn draw_tree(&mut self) {
        let roots = self.tree.roots.clone();
        for index in roots {
            self.run_node(index, true, false);
        }
    }

    fn run_node(&mut self, index: usize, enabled: bool, dispatch: bool) {
        let node = &mut self.tree.nodes[index];
        if !node.visible {
            return;
        }
        let enabled = enabled && node.enabled;
        // The node's own, not its parents': the build default covers every
        // node, and a field in an unfocusable card can turn it back on.
        let focusable = node.focusable;
        let mut control = node.control.take().expect("a control isn't in two phases at once");
        let name = node.name.clone();
        let rect = node.rect;
        let frame = self.frame_number();
        // The node's target, once a frame: in dispatch, or in draw for a node
        // that update showed.
        let was_enabled = self.enabled();
        let was_focusable = self.focusable();
        self.set_scope_enabled(enabled);
        self.set_scope_focusable(focusable);
        let needs_target = self.tree.nodes[index].interacted != frame;
        if needs_target {
            let target = control.target_rect(rect);
            let response = match control.auto_target() {
                AutoTarget::Listed => {
                    let response = self.interact_as(&name, target, control.sense(), control.kind(), enabled, true);
                    if let Some(label) = control.label() {
                        self.set_label(&name, &label);
                    }
                    response
                }
                AutoTarget::Unlisted => self.interact_as(&name, target, control.sense(), control.kind(), enabled, false),
                AutoTarget::None => Response {
                    id: id(&name),
                    rect,
                    enabled,
                    ..Response::default()
                },
            };
            let node = &mut self.tree.nodes[index];
            node.response = response;
            node.interacted = frame;
        }
        let ctx = NodeContext {
            name,
            rect,
            response: self.tree.nodes[index].response,
        };
        if dispatch {
            control.handle(self, &ctx);
            self.node_extras(&ctx);
        } else {
            // A node that update showed missed dispatch, so it handles here:
            // no input landed on it yet, but it registers its parts and
            // paints what handle paints, like a dialog's smoke, this frame.
            if needs_target {
                control.handle(self, &ctx);
                self.node_extras(&ctx);
            }
            control.draw(self, &ctx);
        }
        self.set_scope_enabled(was_enabled);
        self.set_scope_focusable(was_focusable);
        let children_visible = control.children_visible();
        let child_layer = control.child_layer();
        self.tree.nodes[index].control = Some(control);
        if children_visible {
            let children = self.tree.nodes[index].children.clone();
            let layer = self.layer();
            if let Some(child_layer) = child_layer {
                self.set_layer(child_layer);
            }
            for child in children {
                self.run_node(child, enabled, dispatch);
            }
            if child_layer.is_some() {
                self.set_layer(layer);
            }
        }
    }

    // The node's tooltip and context menu, if it has them.
    fn node_extras(&mut self, ctx: &NodeContext) {
        let node_id = id(&ctx.name);
        if let Some((tips, width)) = self.tooltips.get(&node_id).cloned() {
            let lines: Vec<crate::widgets::tool_tip::TooltipLine> = tips.iter().map(crate::widgets::tool_tip::Tip::line).collect();
            self.tooltip_lines_wide(&ctx.response, &lines, width);
        }
        if let Some((menu_name, items)) = self.context_menus.get(&node_id).cloned()
            && let Some(label) = self.context_menu(&menu_name, &ctx.response, &items)
        {
            self.emit(&menu_name, UiEventKind::Chose(label));
        }
    }

    // Gives the node a tooltip, which shows once the pointer rests on it.
    pub fn set_tooltip(&mut self, name: &str, text: &str) {
        self.set_tooltip_tips(name, vec![crate::widgets::tool_tip::Tip::Text(text.to_string())]);
    }

    // A tooltip with lines, columns and separators.
    pub fn set_tooltip_tips(&mut self, name: &str, tips: Vec<crate::widgets::tool_tip::Tip>) {
        self.tooltips.insert(id(name), (tips, crate::design::controls::TOOLTIP_MAX_WIDTH));
    }

    // A tooltip with more in it than WinUI's 320 pixels hold well, up to
    // `max_width` wide.
    pub fn set_tooltip_wide(&mut self, name: &str, tips: Vec<crate::widgets::tool_tip::Tip>, max_width: f32) {
        self.tooltips.insert(id(name), (tips, max_width));
    }

    pub fn clear_tooltip(&mut self, name: &str) {
        self.tooltips.remove(&id(name));
    }

    // Gives the node a context menu named `menu_name`, which opens when it's
    // right-clicked, and whose choices arrive as Chose events from
    // `menu_name`.
    pub fn set_context_menu(&mut self, name: &str, menu_name: &str, items: Vec<crate::widgets::menu::MenuItem>) {
        self.context_menus.insert(id(name), (menu_name.to_string(), items));
    }

    // Emits an event from the node with the name.
    pub fn emit(&mut self, source: &str, kind: UiEventKind) {
        self.events.push(UiEvent {
            source: source.to_string(),
            kind,
        });
    }

    // The frame's events, which dispatch emitted, for the program's update.
    pub fn events(&self) -> &[UiEvent] {
        &self.events
    }

    fn event(&self, source: &str, matches: impl Fn(&UiEventKind) -> bool) -> Option<&UiEventKind> {
        self.events.iter().find(|event| event.source == source && matches(&event.kind)).map(|event| &event.kind)
    }

    // Whether the control was clicked this frame.
    pub fn clicked(&self, name: &str) -> bool {
        self.event(name, |kind| *kind == UiEventKind::Clicked).is_some()
    }

    pub fn secondary_clicked(&self, name: &str) -> bool {
        self.event(name, |kind| *kind == UiEventKind::SecondaryClicked).is_some()
    }

    // The control's value, if it moved this frame.
    pub fn changed(&self, name: &str) -> Option<f64> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Changed(_))) {
            Some(UiEventKind::Changed(value)) => Some(*value),
            _ => None,
        }
    }

    // The toggle's new state, if it was switched this frame.
    pub fn toggled(&self, name: &str) -> Option<bool> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Toggled(_))) {
            Some(UiEventKind::Toggled(on)) => Some(*on),
            _ => None,
        }
    }

    // The index chosen this frame, from a combo box, radio group, segmented
    // control, selector bar or list.
    pub fn selected(&self, name: &str) -> Option<usize> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Selected(_))) {
            Some(UiEventKind::Selected(index)) => Some(*index),
            _ => None,
        }
    }

    // The row opened this frame, with a double-click or Enter.
    pub fn activated(&self, name: &str) -> Option<usize> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Activated(_))) {
            Some(UiEventKind::Activated(index)) => Some(*index),
            _ => None,
        }
    }

    // The text box's text, if it changed this frame.
    pub fn text_changed(&self, name: &str) -> Option<&str> {
        match self.event(name, |kind| matches!(kind, UiEventKind::TextChanged(_))) {
            Some(UiEventKind::TextChanged(text)) => Some(text),
            _ => None,
        }
    }

    // The text submitted with Enter this frame.
    pub fn submitted(&self, name: &str) -> Option<&str> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Submitted(_))) {
            Some(UiEventKind::Submitted(text)) => Some(text),
            _ => None,
        }
    }

    // The label of the item chosen from the menu this frame.
    pub fn chose(&self, name: &str) -> Option<&str> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Chose(_))) {
            Some(UiEventKind::Chose(label)) => Some(label),
            _ => None,
        }
    }

    // The dialog's button chosen this frame.
    pub fn dialog_result(&self, name: &str) -> Option<DialogResult> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Dialog(_))) {
            Some(UiEventKind::Dialog(result)) => Some(*result),
            _ => None,
        }
    }

    pub fn color_changed(&self, name: &str) -> Option<Color> {
        match self.event(name, |kind| matches!(kind, UiEventKind::ColorChanged(_))) {
            Some(UiEventKind::ColorChanged(color)) => Some(*color),
            _ => None,
        }
    }

    // Whether the info bar was closed this frame.
    pub fn closed(&self, name: &str) -> bool {
        self.event(name, |kind| *kind == UiEventKind::Closed).is_some()
    }

    // The expander's new state, if it was opened or closed this frame.
    pub fn expanded(&self, name: &str) -> Option<bool> {
        match self.event(name, |kind| matches!(kind, UiEventKind::Expanded(_))) {
            Some(UiEventKind::Expanded(open)) => Some(*open),
            _ => None,
        }
    }
}
