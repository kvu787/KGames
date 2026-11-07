// WinUI's menus: MenuFlyout, which opens from a button, from a right-click as
// a context menu, or from a MenuBar's items; its items with their icons,
// checks and keyboard accelerators; its separators; and its submenus, which
// open beside their item, each raised 8 more than the menu it's from.
//
// Items are targets named "<menu>/<label>", and what a menu returns is the
// label of the item chosen from it or its submenus, so labels in a menu are
// its own. The arrows move a highlight, Right opens a submenu and Left closes
// it, Enter chooses, and Esc closes the top menu.

use crate::core::interaction::{ControlState, Id, Response, Sense, id};
use crate::core::ui::Ui;
use crate::design::controls::{
    MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING, MENU_FLYOUT_MIN_WIDTH, MENU_FLYOUT_PADDING, MENU_ITEM_ACCELERATOR_GAP, MENU_ITEM_CHECK_GLYPH,
    MENU_ITEM_CHEVRON_SIZE, MENU_ITEM_HEIGHT, MENU_ITEM_ICON_SIZE, MENU_ITEM_MARGIN, MENU_ITEM_PADDING, MENU_ITEM_PLACEHOLDER,
    MENU_SEPARATOR_HEIGHT,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, DURATION_FASTER, ELEVATION_FLYOUT, ELEVATION_SUBMENU, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_S, elevation};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border, Layer};
use crate::input::devices::Key;
use crate::widgets::widget::Widget;

#[derive(Clone, Copy, Debug)]
pub enum MenuItem<'a> {
    Command {
        label: &'a str,
        icon: Option<Icon>,
        // Its keyboard accelerator's text, like "Ctrl+S".
        accelerator: Option<&'a str>,
        enabled: bool,
        // A ToggleMenuFlyoutItem's check.
        checked: Option<bool>,
    },
    Submenu {
        label: &'a str,
        icon: Option<Icon>,
        items: &'a [MenuItem<'a>],
        enabled: bool,
    },
    Separator,
}

impl<'a> MenuItem<'a> {
    pub const fn new(label: &'a str) -> Self {
        MenuItem::Command {
            label,
            icon: None,
            accelerator: None,
            enabled: true,
            checked: None,
        }
    }

    pub const fn submenu(label: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        MenuItem::Submenu {
            label,
            icon: None,
            items,
            enabled: true,
        }
    }

    pub const fn separator() -> Self {
        MenuItem::Separator
    }

    pub const fn icon(self, new_icon: Icon) -> Self {
        match self {
            MenuItem::Command { label, accelerator, enabled, checked, .. } => MenuItem::Command {
                label,
                icon: Some(new_icon),
                accelerator,
                enabled,
                checked,
            },
            MenuItem::Submenu { label, items, enabled, .. } => MenuItem::Submenu {
                label,
                icon: Some(new_icon),
                items,
                enabled,
            },
            MenuItem::Separator => MenuItem::Separator,
        }
    }

    pub const fn accelerator(self, text: &'a str) -> Self {
        match self {
            MenuItem::Command { label, icon, enabled, checked, .. } => MenuItem::Command {
                label,
                icon,
                accelerator: Some(text),
                enabled,
                checked,
            },
            other => other,
        }
    }

    pub const fn enabled(self, now: bool) -> Self {
        match self {
            MenuItem::Command { label, icon, accelerator, checked, .. } => MenuItem::Command {
                label,
                icon,
                accelerator,
                enabled: now,
                checked,
            },
            MenuItem::Submenu { label, icon, items, .. } => MenuItem::Submenu { label, icon, items, enabled: now },
            MenuItem::Separator => MenuItem::Separator,
        }
    }

    pub const fn checked(self, now: bool) -> Self {
        match self {
            MenuItem::Command { label, icon, accelerator, enabled, .. } => MenuItem::Command {
                label,
                icon,
                accelerator,
                enabled,
                checked: Some(now),
            },
            other => other,
        }
    }

    fn label(&self) -> &'a str {
        match self {
            MenuItem::Command { label, .. } | MenuItem::Submenu { label, .. } => label,
            MenuItem::Separator => "",
        }
    }

    fn is_enabled(&self) -> bool {
        match self {
            MenuItem::Command { enabled, .. } | MenuItem::Submenu { enabled, .. } => *enabled,
            MenuItem::Separator => false,
        }
    }

    fn height(&self) -> f32 {
        match self {
            MenuItem::Separator => MENU_SEPARATOR_HEIGHT,
            _ => MENU_ITEM_HEIGHT + MENU_ITEM_MARGIN.vertical(),
        }
    }
}

// Where an open menu is, and which item the keyboard highlights.
#[derive(Default)]
struct MenuState {
    origin: Point,
    highlighted: Option<usize>,
    // The submenu open from one of its items.
    open_submenu: Option<usize>,
}

impl Ui {
    // A menu that the button whose response this is opens and closes, below
    // it. Returns the label of the item chosen from it.
    pub fn flyout_menu<'a>(&mut self, name: &str, owner: &Response, items: &'a [MenuItem<'a>]) -> Option<&'a str> {
        if owner.clicked {
            if self.popup_open(owner.id) {
                self.close_popup(owner.id);
            } else {
                self.open_menu(owner.id, name, Point::new(owner.rect.x, owner.rect.bottom() + 4.0));
            }
        }
        if !self.popup_open(owner.id) {
            return None;
        }
        self.show_menu(name, owner.id, items, 0)
    }

    // A menu that opens at the pointer when the widget whose response this
    // is is right-clicked: a context menu.
    pub fn context_menu<'a>(&mut self, name: &str, owner: &Response, items: &'a [MenuItem<'a>]) -> Option<&'a str> {
        let menu_id = id(name);
        if (owner.secondary_clicked || (owner.secondary_pressed && owner.held_button.is_none()))
            && let Some(pointer) = self.pointer()
        {
            self.open_menu(menu_id, name, pointer);
        }
        if !self.popup_open(menu_id) {
            return None;
        }
        self.show_menu(name, menu_id, items, 0)
    }

    // Opens the context menu with the name at the point, as if right-clicked
    // there.
    pub fn open_context_menu(&mut self, name: &str, at: Point) {
        self.open_menu(id(name), name, at);
    }

    fn open_menu(&mut self, owner: Id, name: &str, origin: Point) {
        self.open_popup(owner, name);
        let state = self.memory::<MenuState>(id(name));
        state.origin = origin;
        state.highlighted = None;
        state.open_submenu = None;
    }

    // Shows an open menu, raised for its level of nesting, and returns the
    // label chosen from it or its submenus.
    fn show_menu<'a>(&mut self, name: &str, owner: Id, items: &'a [MenuItem<'a>], level: u32) -> Option<&'a str> {
        let menu_id = id(name);
        let has_icons = items.iter().any(|item| matches!(item, MenuItem::Command { icon: Some(_), .. } | MenuItem::Submenu { icon: Some(_), .. }));
        let has_checks = items.iter().any(|item| matches!(item, MenuItem::Command { checked: Some(_), .. }));
        let indent = MENU_ITEM_PLACEHOLDER * (has_icons as u32 + has_checks as u32) as f32;
        let mut width: f32 = MENU_FLYOUT_MIN_WIDTH;
        for item in items {
            let text = match item {
                MenuItem::Command { label, accelerator, .. } => {
                    self.measure(label, BODY).width + accelerator.map_or(0.0, |text| MENU_ITEM_ACCELERATOR_GAP + self.measure(text, CAPTION).width)
                }
                MenuItem::Submenu { label, .. } => self.measure(label, BODY).width + MENU_ITEM_ACCELERATOR_GAP + MENU_ITEM_CHEVRON_SIZE,
                MenuItem::Separator => 0.0,
            };
            width = width.max(MENU_ITEM_MARGIN.horizontal() + MENU_ITEM_PADDING.horizontal() + indent + text);
        }
        let width = width.ceil();
        let height = MENU_FLYOUT_PADDING.vertical() + items.iter().map(MenuItem::height).sum::<f32>();
        let bounds = self.bounds();
        let origin = self.memory::<MenuState>(menu_id).origin;
        // It opens down from its origin, or up if there's no room below.
        let mut y = origin.y;
        if y + height > bounds.bottom() - SPACE_S {
            y = (origin.y - height).max(bounds.y + SPACE_S);
        }
        let x = origin.x.clamp(bounds.x + SPACE_S, (bounds.right() - width - SPACE_S).max(bounds.x));
        let panel = Rect::new(x.round(), y.round(), width, height);

        let layer = self.layer();
        self.draw.set_layer(Layer::Popup);
        self.draw.push_clip(bounds, true);
        self.register(name, panel, Sense::NONE, "MenuFlyout");
        {
            let mut painter = self.painter();
            let theme = painter.theme();
            painter.shadow(panel, RADIUS_OVERLAY, elevation(ELEVATION_FLYOUT.z + (ELEVATION_SUBMENU.z - ELEVATION_FLYOUT.z) * level as f32));
            painter.shape(panel, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
        }

        // The keyboard moves the highlight while this is the top menu.
        let mut state = std::mem::take(self.memory::<MenuState>(menu_id));
        let on_top = self.popup_on_top(owner);
        let selectable: Vec<usize> = (0..items.len()).filter(|index| items[*index].is_enabled()).collect();
        let mut chosen = None;
        let mut open_child: Option<usize> = state.open_submenu;
        if on_top && !selectable.is_empty() {
            let position = state.highlighted.and_then(|highlighted| selectable.iter().position(|index| *index == highlighted));
            let (down, up) = (self.input().key_pressed(Key::DOWN), self.input().key_pressed(Key::UP));
            if down {
                state.highlighted = Some(selectable[position.map_or(0, |at| (at + 1) % selectable.len())]);
            }
            if up {
                state.highlighted = Some(selectable[position.map_or(selectable.len() - 1, |at| (at + selectable.len() - 1) % selectable.len())]);
            }
            if down || up {
                self.set_focus_visible(true);
            }
            if let Some(highlighted) = state.highlighted {
                let input = self.input();
                let opens = matches!(items[highlighted], MenuItem::Submenu { .. });
                if (input.key_pressed_once(Key::ENTER) || input.key_pressed_once(Key::SPACE)) && !opens {
                    chosen = Some(items[highlighted].label());
                } else if (input.key_pressed(Key::RIGHT) || input.key_pressed_once(Key::ENTER)) && opens {
                    open_child = Some(highlighted);
                }
            }
            if level > 0 && self.input().key_pressed(Key::LEFT) {
                self.close_popup(owner);
            }
        }

        let mut top = panel.y + MENU_FLYOUT_PADDING.top;
        let mut child_anchor = None;
        for (index, item) in items.iter().enumerate() {
            let slot = Rect::new(panel.x, top, panel.width, item.height());
            top += item.height();
            if let MenuItem::Separator = item {
                let line = Rect::new(panel.x + 1.0, slot.y + 1.0, panel.width - 2.0, 1.0);
                let color = self.theme().divider;
                self.painter().fill(line, color);
                continue;
            }
            let rect = slot.pad(MENU_ITEM_MARGIN);
            let item_name = format!("{name}/{}", item.label());
            let enabled = self.enabled();
            self.set_enabled(enabled && item.is_enabled());
            let response = self.interact(&item_name, rect, Sense::CLICK, "MenuFlyoutItem");
            self.set_enabled(enabled);
            self.set_label(&item_name, item.label());
            if response.hovered {
                state.highlighted = Some(index);
                if matches!(item, MenuItem::Submenu { .. }) {
                    open_child = Some(index);
                } else if open_child.is_some() {
                    open_child = None;
                }
            }
            let submenu_open = open_child == Some(index);
            if response.clicked {
                match item {
                    MenuItem::Submenu { .. } => open_child = Some(index),
                    _ => chosen = Some(item.label()),
                }
            }
            let state_now = response.state();
            let highlighted = state.highlighted == Some(index) && self.focus_visible() && !response.hovered;
            let theme = self.theme();
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
            let background = self.animate_color(response.id, 0, background, DURATION_FASTER);
            let content = rect.pad(MENU_ITEM_PADDING);
            let mut painter = self.painter();
            painter.rounded(rect, RADIUS_CONTROL, background);
            let mut x = content.x;
            if has_checks {
                if let MenuItem::Command { checked: Some(true), .. } = item {
                    painter.icon(Point::new(x + MENU_ITEM_CHECK_GLYPH / 2.0, rect.center().y), Icon::CheckMark, MENU_ITEM_CHECK_GLYPH, foreground);
                }
                x += MENU_ITEM_PLACEHOLDER;
            }
            if has_icons {
                let icon = match item {
                    MenuItem::Command { icon, .. } | MenuItem::Submenu { icon, .. } => *icon,
                    MenuItem::Separator => None,
                };
                if let Some(icon) = icon {
                    painter.icon(Point::new(x + MENU_ITEM_ICON_SIZE / 2.0, rect.center().y), icon, MENU_ITEM_ICON_SIZE, foreground);
                }
                x += MENU_ITEM_PLACEHOLDER;
            }
            painter.text_in(Rect::new(x, rect.y, content.right() - x, rect.height), item.label(), BODY, foreground, Align::Start);
            match item {
                MenuItem::Command { accelerator: Some(text), .. } => {
                    painter.text_in(content, text, CAPTION, secondary, Align::End);
                }
                MenuItem::Submenu { .. } => {
                    let center = Point::new(content.right() - MENU_ITEM_CHEVRON_SIZE / 2.0, rect.center().y);
                    painter.icon(center, Icon::ChevronRight, MENU_ITEM_CHEVRON_SIZE, secondary);
                    if submenu_open {
                        child_anchor = Some((index, response.id, rect));
                    }
                }
                _ => {}
            }
        }
        self.draw.pop_clip();
        self.draw.set_layer(layer);
        state.open_submenu = open_child;
        *self.memory::<MenuState>(menu_id) = state;

        // The open submenu, beside its item, and whatever's chosen from it.
        match open_child.zip(child_anchor) {
            Some((index, (_, item_id, item_rect))) => {
                let MenuItem::Submenu { label, items: children, .. } = items[index] else {
                    unreachable!("only submenus open");
                };
                let child_name = format!("{name}/{label}/Menu");
                if !self.popup_open(item_id) {
                    self.open_popup(item_id, &child_name);
                    let child = self.memory::<MenuState>(id(&child_name));
                    child.origin = Point::new(item_rect.right() + MENU_ITEM_MARGIN.right, item_rect.y - MENU_FLYOUT_PADDING.top - MENU_ITEM_MARGIN.top);
                    child.highlighted = None;
                    child.open_submenu = None;
                }
                if let Some(label) = self.show_menu(&child_name, item_id, children, level + 1) {
                    chosen = Some(label);
                }
            }
            None => {
                // Any submenu that was open from this menu closes.
                for item in items {
                    if let MenuItem::Submenu { label, .. } = item {
                        self.close_popup(id(&format!("{name}/{label}")));
                    }
                }
            }
        }
        if chosen.is_some() {
            self.close_popups();
        }
        chosen
    }
}

// WinUI's MenuBar: menus' names along a bar, each opening its menu below it.
// While one is open, the pointer moving onto another opens that one.
pub struct MenuBar<'a> {
    menus: &'a [(&'a str, &'a [MenuItem<'a>])],
}

impl<'a> MenuBar<'a> {
    pub fn new(menus: &'a [(&'a str, &'a [MenuItem<'a>])]) -> Self {
        Self { menus }
    }

    // The width its items take.
    pub fn width(&self, ui: &mut Ui) -> f32 {
        self.menus.iter().map(|(title, _)| ui.measure(title, BODY).width.ceil() + MENU_BAR_ITEM_PADDING.horizontal() + MENU_BAR_ITEM_MARGIN.horizontal()).sum()
    }
}

impl<'a> Widget for MenuBar<'a> {
    // The label of the item chosen from one of its menus.
    type Response = Option<&'a str>;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Option<&'a str> {
        let mut x = rect.x;
        let mut chosen = None;
        let any_open = self.menus.iter().any(|(title, _)| ui.popup_open(id(&format!("{name}/{title}"))));
        for (title, items) in self.menus {
            let width = ui.measure(title, BODY).width.ceil() + MENU_BAR_ITEM_PADDING.horizontal();
            let slot = Rect::new(x, rect.y, width + MENU_BAR_ITEM_MARGIN.horizontal(), rect.height);
            x += slot.width;
            let item = slot.pad(MENU_BAR_ITEM_MARGIN);
            let item_name = format!("{name}/{title}");
            let mut response = ui.interact(&item_name, item, Sense::BUTTON, "MenuBarItem");
            ui.set_label(&item_name, title);
            let open = ui.popup_open(response.id);
            if any_open && !open && response.hovered {
                // Moving across the bar while a menu is open opens this one.
                response.clicked = true;
            }
            if response.focused && ui.input().key_pressed(Key::DOWN) && !open {
                response.clicked = true;
            }
            let state = response.state();
            let theme = ui.theme();
            let background = match state {
                ControlState::Pressed => theme.subtle_fill_tertiary,
                _ if open => theme.subtle_fill_tertiary,
                ControlState::PointerOver => theme.subtle_fill_secondary,
                _ => theme.subtle_fill_transparent,
            };
            let foreground = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
            let background = ui.animate_color(response.id, 0, background, DURATION_FASTER);
            let mut painter = ui.painter();
            painter.rounded(item, RADIUS_CONTROL, background);
            painter.text_in(item, title, BODY, foreground, Align::Center);
            ui.focus_visual_if(response.focused, item, RADIUS_CONTROL);
            if let Some(label) = ui.flyout_menu(&format!("{item_name}/Menu"), &response, items) {
                chosen = Some(label);
            }
        }
        chosen
    }
}
