// WinUI's menus: MenuFlyout, which opens from a button, from a right-click as
// a context menu, or from a MenuBar's items; its items with their icons,
// checks and keyboard accelerators; its separators; and its submenus, which
// open beside their item, each raised 8 more than the menu it's from. A menu
// shows whole on the frame that opens it, and goes on the frame that closes
// it.
//
// Items are targets named "<menu>/<label>", and each carries the message
// that choosing it sends. The arrows move a highlight, Right opens a submenu
// and Left closes it, Enter chooses, and Esc closes the top menu. The
// functions here are what every owner of a menu shares: a button, a menu
// bar's item, and an element with a context menu.

use crate::core::interaction::{ControlState, Id, Interaction, Part, Sense, id};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{
    MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING, MENU_FLYOUT_MIN_WIDTH, MENU_FLYOUT_PADDING, MENU_ITEM_ACCELERATOR_GAP, MENU_ITEM_CHECK_GLYPH,
    MENU_ITEM_CHEVRON_SIZE, MENU_ITEM_HEIGHT, MENU_ITEM_ICON_SIZE, MENU_ITEM_MARGIN, MENU_ITEM_PADDING, MENU_ITEM_PLACEHOLDER,
    MENU_SEPARATOR_HEIGHT,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, ELEVATION_FLYOUT, ELEVATION_SUBMENU, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_S, elevation};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border, Layer};
use crate::drawing::text::Fonts;
use crate::input::devices::Key;

pub struct MenuItem<M> {
    label: String,
    icon: Option<Icon>,
    // Its keyboard accelerator's text, like "Ctrl+S".
    accelerator: Option<String>,
    enabled: bool,
    // A ToggleMenuFlyoutItem's check.
    checked: Option<bool>,
    // What choosing it sends.
    message: Option<M>,
    submenu: Option<Vec<MenuItem<M>>>,
    separator: bool,
}

impl<M: Clone + 'static> MenuItem<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            accelerator: None,
            enabled: true,
            checked: None,
            message: None,
            submenu: None,
            separator: false,
        }
    }

    pub fn submenu(label: impl Into<String>, items: Vec<MenuItem<M>>) -> Self {
        Self {
            submenu: Some(items),
            ..Self::new(label)
        }
    }

    pub fn separator() -> Self {
        Self {
            separator: true,
            ..Self::new("")
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn accelerator(mut self, text: impl Into<String>) -> Self {
        self.accelerator = Some(text.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    // What choosing it sends.
    pub fn on_choose(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }

    fn is_enabled(&self) -> bool {
        self.enabled && !self.separator
    }

    fn height(&self) -> f32 {
        if self.separator { MENU_SEPARATOR_HEIGHT } else { MENU_ITEM_HEIGHT + MENU_ITEM_MARGIN.vertical() }
    }
}

// Where an open menu is, and which item the keyboard highlights.
#[derive(Default, Clone)]
pub(crate) struct MenuState {
    origin: Point,
    highlighted: Option<usize>,
    // The submenu open from one of its items.
    open_submenu: Option<usize>,
}

// A menu's panel and columns, from its items and where it opens.
struct MenuLayout {
    panel: Rect,
    has_icons: bool,
    has_checks: bool,
}

fn layout<M: Clone + 'static>(fonts: &mut Fonts, bounds: Rect, origin: Point, items: &[MenuItem<M>]) -> MenuLayout {
    let has_icons = items.iter().any(|item| item.icon.is_some());
    let has_checks = items.iter().any(|item| item.checked.is_some());
    let indent = MENU_ITEM_PLACEHOLDER * (has_icons as u32 + has_checks as u32) as f32;
    let mut width: f32 = MENU_FLYOUT_MIN_WIDTH;
    for item in items {
        let text = if item.separator {
            0.0
        } else if item.submenu.is_some() {
            fonts.measure(&item.label, BODY).width + MENU_ITEM_ACCELERATOR_GAP + MENU_ITEM_CHEVRON_SIZE
        } else {
            fonts.measure(&item.label, BODY).width + item.accelerator.as_ref().map_or(0.0, |text| MENU_ITEM_ACCELERATOR_GAP + fonts.measure(text, CAPTION).width)
        };
        width = width.max(MENU_ITEM_MARGIN.horizontal() + MENU_ITEM_PADDING.horizontal() + indent + text);
    }
    let width = width.ceil();
    let height = MENU_FLYOUT_PADDING.vertical() + items.iter().map(MenuItem::height).sum::<f32>();
    // It opens down from its origin, or up if there's no room below.
    let mut y = origin.y;
    if y + height > bounds.bottom() - SPACE_S {
        y = (origin.y - height).max(bounds.y + SPACE_S);
    }
    let x = origin.x.clamp(bounds.x + SPACE_S, (bounds.right() - width - SPACE_S).max(bounds.x));
    MenuLayout {
        panel: Rect::new(x.round(), y.round(), width, height),
        has_icons,
        has_checks,
    }
}

// Each item's slot in the panel, from the top.
fn slots<M: Clone + 'static>(panel: Rect, items: &[MenuItem<M>]) -> Vec<Rect> {
    let mut top = panel.y + MENU_FLYOUT_PADDING.top;
    items
        .iter()
        .map(|item| {
            let slot = Rect::new(panel.x, top, panel.width, item.height());
            top += item.height();
            slot
        })
        .collect()
}

fn item_name<M>(menu: &str, item: &MenuItem<M>) -> String {
    format!("{menu}/{}", item.label)
}

fn child_name(item_name: &str) -> String {
    format!("{item_name}/Menu")
}

// Opens the menu named `name` that `owner` owns, at the point.
pub(crate) fn open_menu<M>(ctx: &mut Dispatch<M>, owner: Id, name: &str, origin: Point) {
    ctx.open_popup_of(owner, name);
    let state = ctx.memory_of::<MenuState>(id(name));
    state.origin = origin;
    state.highlighted = None;
    state.open_submenu = None;
}

// Registers an open menu's targets: its panel, its items, and its open
// submenu's, recursively.
pub(crate) fn parts_menu<M: Clone + 'static>(ctx: &mut Parts, name: &str, owner: Id, items: &[MenuItem<M>]) {
    if !ctx.popup_open_of(owner) {
        return;
    }
    let state = ctx.memory_of::<MenuState>(id(name)).clone();
    let bounds = ctx.bounds();
    let layout = layout(ctx.fonts(), bounds, state.origin, items);
    ctx.part_on(Layer::Popup, Part::Named(name.to_string()), name, layout.panel, Sense::NONE, "MenuFlyout", None);
    for (item, slot) in items.iter().zip(slots(layout.panel, items)) {
        if item.separator {
            continue;
        }
        let rect = slot.pad(MENU_ITEM_MARGIN);
        let item_name = item_name(name, item);
        ctx.part_on_enabled(Layer::Popup, Part::Named(item_name.clone()), &item_name, rect, Sense::CLICK, "MenuFlyoutItem", Some(item.label.clone()), item.enabled);
    }
    if let Some(index) = state.open_submenu
        && let Some(item) = items.get(index)
        && let Some(children) = &item.submenu
    {
        let item_name = item_name(name, item);
        parts_menu(ctx, &child_name(&item_name), id(&item_name), children);
    }
}

// Does what the input on an open menu means: the keys move its highlight
// and choose, the pointer opens submenus, and a press chooses, which sends
// the item's message and closes every menu.
pub(crate) fn interact_menu<M: Clone + 'static>(interaction: &Interaction, ctx: &mut Dispatch<M>, name: &str, owner: Id, items: &[MenuItem<M>], level: u32) {
    if !ctx.popup_open_of(owner) {
        return;
    }
    let menu_id = id(name);
    let mut state = std::mem::take(ctx.memory_of::<MenuState>(menu_id));
    let on_top = ctx.popup_on_top_of(owner);
    let selectable: Vec<usize> = (0..items.len()).filter(|index| items[*index].is_enabled()).collect();
    // The item the keyboard chose, if any: its message, which an item may
    // not have, in which case choosing it only closes the menus.
    let mut chosen: Option<Option<M>> = None;
    let mut open_child: Option<usize> = state.open_submenu;
    if on_top && !selectable.is_empty() {
        let position = state.highlighted.and_then(|highlighted| selectable.iter().position(|index| *index == highlighted));
        let (down, up) = (ctx.input().key_pressed(Key::DOWN), ctx.input().key_pressed(Key::UP));
        if down {
            state.highlighted = Some(selectable[position.map_or(0, |at| (at + 1) % selectable.len())]);
        }
        if up {
            state.highlighted = Some(selectable[position.map_or(selectable.len() - 1, |at| (at + selectable.len() - 1) % selectable.len())]);
        }
        if down || up {
            ctx.set_focus_visible(true);
        }
        if let Some(highlighted) = state.highlighted {
            let item = &items[highlighted];
            let opens = item.submenu.is_some();
            let input = ctx.input();
            if (input.key_pressed_once(Key::ENTER) || input.key_pressed_once(Key::SPACE)) && !opens {
                chosen = Some(item.message.clone());
            } else if (input.key_pressed(Key::RIGHT) || input.key_pressed_once(Key::ENTER)) && opens {
                open_child = Some(highlighted);
            }
        }
        if level > 0 && ctx.input().key_pressed(Key::LEFT) {
            ctx.close_popup_of(owner);
        }
    }
    let mut close_all = false;
    for (index, item) in items.iter().enumerate() {
        if item.separator {
            continue;
        }
        let part = Part::Named(item_name(name, item));
        let Some(hit) = interaction.hit(&part) else {
            continue;
        };
        if hit.hovered {
            state.highlighted = Some(index);
            if item.submenu.is_some() {
                open_child = Some(index);
            } else if open_child.is_some() {
                open_child = None;
            }
        }
        if hit.activated && item.is_enabled() {
            match item.submenu {
                Some(_) => open_child = Some(index),
                None => {
                    if let Some(message) = &item.message {
                        ctx.send(message.clone());
                    }
                    close_all = true;
                }
            }
        }
    }
    if let Some(message) = chosen {
        if let Some(message) = message {
            ctx.send(message);
        }
        close_all = true;
    }
    state.open_submenu = open_child;
    *ctx.memory_of::<MenuState>(menu_id) = state;

    // The open submenu, beside its item, and what's chosen from it.
    if let Some(index) = open_child
        && let Some(item) = items.get(index)
        && let Some(children) = &item.submenu
    {
        let item_name = item_name(name, item);
        let item_id = id(&item_name);
        let child = child_name(&item_name);
        if !ctx.popup_open_of(item_id) {
            let bounds = ctx.bounds();
            let origin = ctx.memory_of::<MenuState>(menu_id).origin;
            let layout = layout(ctx.fonts(), bounds, origin, items);
            let slot = slots(layout.panel, items)[index];
            let rect = slot.pad(MENU_ITEM_MARGIN);
            ctx.open_popup_of(item_id, &child);
            let state = ctx.memory_of::<MenuState>(id(&child));
            state.origin = Point::new(rect.right() + MENU_ITEM_MARGIN.right, rect.y - MENU_FLYOUT_PADDING.top - MENU_ITEM_MARGIN.top);
            state.highlighted = None;
            state.open_submenu = None;
        }
        interact_menu(interaction, ctx, &child, item_id, children, level + 1);
    } else {
        // Any submenu that was open from this menu closes.
        for item in items.iter().filter(|item| item.submenu.is_some()) {
            ctx.close_popup_of(id(&item_name(name, item)));
        }
    }
    if close_all {
        ctx.close_popups();
    }
}

// Draws an open menu, raised for its level of nesting, and its open
// submenu.
pub(crate) fn draw_menu<M: Clone + 'static>(ctx: &mut Draw, name: &str, owner: Id, items: &[MenuItem<M>], level: u32) {
    if !ctx.popup_open_of(owner) {
        return;
    }
    let state = ctx.memory_of::<MenuState>(id(name)).clone();
    let bounds = ctx.bounds();
    let layout = layout(ctx.fonts(), bounds, state.origin, items);
    let panel = layout.panel;
    let focus_visible = ctx.focus_visible();
    let look = ctx.look().clone();
    let theme = ctx.theme().clone();
    let mut painter = ctx.painter();
    let layer_before = painter.layer();
    painter.set_layer(Layer::Popup);
    painter.push_clip(bounds);
    painter.shadow(panel, RADIUS_OVERLAY, elevation(ELEVATION_FLYOUT.z + (ELEVATION_SUBMENU.z - ELEVATION_FLYOUT.z) * level as f32));
    painter.shape(panel, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
    for (index, (item, slot)) in items.iter().zip(slots(panel, items)).enumerate() {
        if item.separator {
            let line = Rect::new(panel.x + 1.0, slot.y + 1.0, panel.width - 2.0, 1.0);
            painter.fill(line, theme.divider);
            continue;
        }
        let rect = slot.pad(MENU_ITEM_MARGIN);
        let part = Part::Named(item_name(name, item));
        let state_now = if item.enabled { look.part_state(&part) } else { ControlState::Disabled };
        let submenu_open = state.open_submenu == Some(index);
        let highlighted = state.highlighted == Some(index) && focus_visible && !look.hovered(&part);
        let background = match state_now {
            ControlState::Pressed => theme.subtle_fill_tertiary,
            ControlState::PointerOver => theme.subtle_fill_secondary,
            _ if submenu_open || highlighted => theme.subtle_fill_secondary,
            _ => theme.subtle_fill_transparent,
        };
        let foreground = if state_now == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let secondary = match state_now {
            ControlState::Disabled => theme.text_disabled,
            ControlState::Pressed => theme.text_tertiary,
            _ => theme.text_secondary,
        };
        let content = rect.pad(MENU_ITEM_PADDING);
        painter.rounded(rect, RADIUS_CONTROL, background);
        let mut x = content.x;
        if layout.has_checks {
            if item.checked == Some(true) {
                painter.icon(Point::new(x + MENU_ITEM_CHECK_GLYPH / 2.0, rect.center().y), Icon::CheckMark, MENU_ITEM_CHECK_GLYPH, foreground);
            }
            x += MENU_ITEM_PLACEHOLDER;
        }
        if layout.has_icons {
            if let Some(icon) = item.icon {
                painter.icon(Point::new(x + MENU_ITEM_ICON_SIZE / 2.0, rect.center().y), icon, MENU_ITEM_ICON_SIZE, foreground);
            }
            x += MENU_ITEM_PLACEHOLDER;
        }
        painter.text_in(Rect::new(x, rect.y, content.right() - x, rect.height), &item.label, BODY, foreground, Align::Start);
        if item.submenu.is_some() {
            let center = Point::new(content.right() - MENU_ITEM_CHEVRON_SIZE / 2.0, rect.center().y);
            painter.icon(center, Icon::ChevronRight, MENU_ITEM_CHEVRON_SIZE, secondary);
        } else if let Some(text) = &item.accelerator {
            painter.text_in(content, text, CAPTION, secondary, Align::End);
        }
    }
    painter.pop_clip();
    painter.set_layer(layer_before);
    if let Some(index) = state.open_submenu
        && let Some(item) = items.get(index)
        && let Some(children) = &item.submenu
    {
        let item_name = item_name(name, item);
        draw_menu(ctx, &child_name(&item_name), id(&item_name), children, level + 1);
    }
}

// A context menu on an element: named "<element>/ContextMenu", opened at
// the pointer by a right press, on the frame the button goes down.
fn context_menu_name(element: &str) -> String {
    format!("{element}/ContextMenu")
}

pub(crate) fn parts_context_menu<M: Clone + 'static>(ctx: &mut Parts, items: &[MenuItem<M>]) {
    let name = context_menu_name(ctx.name());
    let owner = ctx.id();
    parts_menu(ctx, &name, owner, items);
}

pub(crate) fn interact_context_menu<M: Clone + 'static>(interaction: &Interaction, ctx: &mut Dispatch<M>, items: &[MenuItem<M>]) {
    let name = context_menu_name(ctx.name());
    let owner = ctx.id();
    let hit = interaction.whole();
    if hit.secondary_pressed
        && let Some(pointer) = ctx.pointer()
    {
        open_menu(ctx, owner, &name, pointer);
    }
    interact_menu(interaction, ctx, &name, owner, items, 0);
}

pub(crate) fn draw_context_menu<M: Clone + 'static>(ctx: &mut Draw, items: &[MenuItem<M>]) {
    let name = context_menu_name(ctx.name());
    let owner = ctx.id();
    draw_menu(ctx, &name, owner, items, 0);
}

// WinUI's MenuBar: menus' names along a bar, each opening its menu below it.
// While one is open, the pointer moving onto another opens that one. Its
// items are targets named "<name>/<title>", and their menus
// "<name>/<title>/Menu".
pub struct MenuBar<M> {
    menus: Vec<(String, Vec<MenuItem<M>>)>,
}

impl<M: Clone + 'static> MenuBar<M> {
    pub fn new(menus: Vec<(impl Into<String>, Vec<MenuItem<M>>)>) -> Self {
        Self {
            menus: menus.into_iter().map(|(title, items)| (title.into(), items)).collect(),
        }
    }

    // The width its items take.
    pub fn width(&self, ui: &mut Ui) -> f32 {
        self.menus.iter().map(|(title, _)| ui.measure(title, BODY).width.ceil() + MENU_BAR_ITEM_PADDING.horizontal() + MENU_BAR_ITEM_MARGIN.horizontal()).sum()
    }

    // Each item's rectangle, from the bar's left.
    fn items(&self, fonts: &mut Fonts, rect: Rect) -> Vec<Rect> {
        let mut x = rect.x;
        self.menus
            .iter()
            .map(|(title, _)| {
                let width = fonts.measure(title, BODY).width.ceil() + MENU_BAR_ITEM_PADDING.horizontal();
                let slot = Rect::new(x, rect.y, width + MENU_BAR_ITEM_MARGIN.horizontal(), rect.height);
                x += slot.width;
                slot.pad(MENU_BAR_ITEM_MARGIN)
            })
            .collect()
    }

    fn item_name(name: &str, title: &str) -> String {
        format!("{name}/{title}")
    }
}

impl<M: Clone + 'static> Widget<M> for MenuBar<M> {
    fn kind(&self) -> &'static str {
        "MenuBar"
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        let rects = self.items(ctx.fonts(), rect);
        for ((title, items), item) in self.menus.iter().zip(rects) {
            let item_name = Self::item_name(ctx.name(), title);
            ctx.part(Part::Named(item_name.clone()), title, item, Sense::BUTTON, "MenuBarItem", Some(title.clone()));
            let menu_name = child_name(&item_name);
            parts_menu(ctx, &menu_name, id(&item_name), items);
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let rect = ctx.rect();
        let rects = self.items(ctx.fonts(), rect);
        let any_open = self.menus.iter().any(|(title, _)| ctx.popup_open_of(id(&Self::item_name(ctx.name(), title))));
        for ((title, items), item) in self.menus.iter().zip(rects) {
            let item_name = Self::item_name(ctx.name(), title);
            let item_id = id(&item_name);
            let menu_name = child_name(&item_name);
            let part = Part::Named(item_name.clone());
            let hit = interaction.hit(&part).cloned().unwrap_or_default();
            let open = ctx.popup_open_of(item_id);
            let mut activated = hit.activated;
            if any_open && !open && hit.hovered {
                // Moving across the bar while a menu is open opens this one.
                activated = true;
            }
            if interaction.focused_part.as_ref() == Some(&part) && interaction.focused && ctx.input().key_pressed(Key::DOWN) && !open {
                activated = true;
            }
            if activated {
                if open {
                    ctx.close_popup_of(item_id);
                } else {
                    open_menu(ctx, item_id, &menu_name, Point::new(item.x, item.bottom() + 4.0));
                }
            }
            interact_menu(interaction, ctx, &menu_name, item_id, items, 0);
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let rects = self.items(ctx.fonts(), rect);
        let look = ctx.look().clone();
        for ((title, items), item) in self.menus.iter().zip(rects) {
            let item_name = Self::item_name(ctx.name(), title);
            let item_id = id(&item_name);
            let part = Part::Named(item_name.clone());
            let open = ctx.popup_open_of(item_id);
            let state = look.part_state(&part);
            let theme = ctx.theme();
            let background = match state {
                ControlState::Pressed => theme.subtle_fill_tertiary,
                _ if open => theme.subtle_fill_tertiary,
                ControlState::PointerOver => theme.subtle_fill_secondary,
                _ => theme.subtle_fill_transparent,
            };
            let foreground = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
            let mut painter = ctx.painter();
            painter.rounded(item, RADIUS_CONTROL, background);
            painter.text_in(item, title, BODY, foreground, Align::Center);
            let focused = look.focused && look.focused_part.as_ref() == Some(&part);
            ctx.focus_visual_if(focused, item, RADIUS_CONTROL);
            draw_menu(ctx, &child_name(&item_name), item_id, items, 0);
        }
    }
}
