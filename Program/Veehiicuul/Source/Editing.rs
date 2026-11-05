// What the editors share: where their cards and views are, the view of the
// ground that they pan and zoom, a card's column of labeled controls, color
// fields, the controls that paint a model's slots with SimplePaint, and what
// they draw over the view. They follow Library/UserInterface's design
// system: colors from the theme's roles, and sizes from its tokens.
//
// Programs only support two window setups (see Library/UserInterface's
// README), so the editors' cards have fixed places in pixels: the app bar at
// the top, with the tabs, a card down the left side of the page, and in the
// vehicle editor, another down the right, the status bar at the bottom, and
// the 3D view under all of them, which fills the window. The cards float
// over the view, so it takes input only where they don't cover it.

use simple_paint::Parameters;
use user_interface::tokens::{
    APP_BAR_HEIGHT, BODY, BODY_STRONG, CAPTION, CONTROL_HEIGHT, PAGE_PADDING, RADIUS_CONTROL, SPACE_M, SPACE_S, SPACE_XS,
    STATUS_BAR_HEIGHT,
};
use user_interface::{
    Align, Button, Color, Corners, Cursor, Icon, InfoBar, Key, Painter, Point, Rect, Response, Sense, Severity, Slider,
    Stroke, TextBox, Ui,
};

use crate::camera::{self, Eye};
use crate::files::{self, Color as FileColor};

// How wide the editors' cards are.
pub const CARD_WIDTH: f32 = 520.0;
// The width of a row's label, and of the value after a slider.
const LABEL_WIDTH: f32 = 128.0;
const VALUE_WIDTH: f32 = 80.0;

// The editors' card down the left side of the page, as tall as the page.
pub fn left_card(page: Rect) -> Rect {
    page.left_strip(CARD_WIDTH)
}

// The vehicle editor's second card, down the right side.
pub fn right_card(page: Rect) -> Rect {
    page.right_strip(CARD_WIDTH)
}

// Where the 3D view takes input: the window between the app bar and the
// status bar, all the way across. The cards, and the notices under the app
// bar, cover it.
pub fn view_area(ui: &Ui) -> Rect {
    let size = ui.size();
    Rect::new(0.0, APP_BAR_HEIGHT, size.width, size.height - APP_BAR_HEIGHT - STATUS_BAR_HEIGHT)
}

// The part of the view that a card down the left side leaves, whose middle
// is where new things go.
pub fn visible(view: Rect) -> Rect {
    view.inset_by(PAGE_PADDING + CARD_WIDTH, 0.0, 0.0, 0.0)
}

pub fn window_size(ui: &Ui) -> [f32; 2] {
    [ui.size().width, ui.size().height]
}

// What happened in an editor's view in a frame.
pub struct ViewInput {
    pub response: Response,
    // The point on the ground under the pointer, if it's on the view.
    pub ground: Option<[f64; 2]>,
    pub cursor: Option<[f32; 2]>,
    // Whether Space is down, so that a drag pans instead of editing.
    pub panning: bool,
}

// A view of the ground for editing: straight down, or at the game's angle.
// Space and a drag pan it, and the wheel zooms it around the pointer. It
// fills the window, so its middle is the window's.
pub struct Viewport {
    pub eye: Eye,
    // The point on the ground that a pan holds under the pointer.
    grab: Option<[f64; 2]>,
}

impl Viewport {
    pub fn new(center: [f64; 2], height: f64) -> Viewport {
        Viewport {
            eye: Eye::top_down(center, height),
            grab: None,
        }
    }

    pub fn game_angle(&self) -> bool {
        self.eye.pitch < std::f64::consts::FRAC_PI_2 - 1e-6
    }

    pub fn set_game_angle(&mut self, game: bool) {
        self.eye.pitch = if game { camera::PITCH } else { std::f64::consts::FRAC_PI_2 };
    }

    // The view as a target, named like "Terrain/View", in `area`: its
    // presses, and the pan and zoom.
    pub fn interact(&mut self, ui: &mut Ui, name: &str, area: Rect) -> ViewInput {
        let size = window_size(ui);
        let response = ui.interact(name, area, Sense::CLICK, "View");
        // A press on the view takes keyboard focus from a text field, so
        // that the keys after it, like Space, are the view's.
        if response.pressed {
            ui.clear_focus();
        }
        let on = response.hovered || response.held;
        let cursor = ui.pointer().filter(|_| on).map(|point| [point.x, point.y]);
        let notches = ui.input().wheel.y;
        if notches != 0.0
            && response.hovered
            && let Some(cursor) = cursor
        {
            self.eye.zoom_at(camera::NOTCH.powf(-f64::from(notches)), cursor, size);
        }
        let space = ui.key_down(Key::SPACE);
        if response.pressed && space {
            self.grab = cursor.map(|cursor| self.eye.to_ground(cursor, size));
        }
        if !response.held {
            self.grab = None;
        }
        if let (Some(grab), Some(cursor)) = (self.grab, cursor) {
            let under = self.eye.to_ground(cursor, size);
            self.eye.center[0] += grab[0] - under[0];
            self.eye.center[1] += grab[1] - under[1];
        }
        let panning = space || self.grab.is_some();
        if panning && on {
            ui.set_cursor(Cursor::Move);
        }
        ViewInput {
            response,
            ground: cursor.map(|cursor| self.eye.to_ground(cursor, size)),
            cursor,
            panning,
        }
    }
}

// The nearest of some points on the ground to the pointer, within `reach`
// pixels of it on the screen.
pub fn nearest(eye: &Eye, size: [f32; 2], cursor: [f32; 2], points: &[[f64; 2]], reach: f32) -> Option<usize> {
    points
        .iter()
        .map(|&point| {
            let [x, y] = eye.ground_to_screen(point, size);
            (x - cursor[0]).hypot(y - cursor[1])
        })
        .enumerate()
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

// The heading, in degrees from +Z toward +X, from one point toward another.
pub fn heading_toward(from: [f64; 2], to: [f64; 2]) -> f64 {
    (to[0] - from[0]).atan2(to[1] - from[1]).to_degrees()
}

// A file's color as the interface takes it.
pub fn interface_color([red, green, blue]: FileColor) -> Color {
    Color::rgba(red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, 1.0)
}

// What an editor draws over its view, from points on the ground, clipped to
// the view. It draws before the cards, which go over it.
pub struct Overlay<'a> {
    painter: Painter<'a>,
    eye: Eye,
    size: [f32; 2],
    area: Rect,
}

impl<'a> Overlay<'a> {
    pub fn new(ui: &'a mut Ui, eye: Eye, area: Rect) -> Overlay<'a> {
        let size = window_size(ui);
        let mut painter = ui.painter();
        painter.push_clip(area);
        Overlay { painter, eye, size, area }
    }

    // Where a point on the ground is on the screen.
    pub fn screen(&self, point: [f64; 2]) -> Point {
        let [x, y] = self.eye.ground_to_screen(point, self.size);
        Point::new(x, y)
    }

    // A closed outline through points on the ground.
    pub fn outline(&mut self, points: &[[f64; 2]], width: f32, color: Color) {
        for (index, &point) in points.iter().enumerate() {
            let next = points[(index + 1) % points.len()];
            let (from, to) = (self.screen(point), self.screen(next));
            self.line(from, to, width, color);
        }
    }

    // A line, cut where it leaves the view, so that one that goes far past
    // it doesn't make the GPU cover the whole window.
    pub fn line(&mut self, from: Point, to: Point, width: f32, color: Color) {
        let area = self.area.expand(width);
        // Liang and Barsky's clipping, along the line from 0 to 1.
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let (mut start, mut end) = (0.0_f32, 1.0_f32);
        for (p, q) in [
            (-dx, from.x - area.x),
            (dx, area.right() - from.x),
            (-dy, from.y - area.y),
            (dy, area.bottom() - from.y),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return;
                }
            } else {
                let t = q / p;
                if p < 0.0 {
                    start = start.max(t);
                } else {
                    end = end.min(t);
                }
            }
        }
        if start <= end {
            self.painter.line(from.lerp(to, start), from.lerp(to, end), width, color);
        }
    }

    pub fn dot(&mut self, center: Point, radius: f32, color: Color) {
        if self.area.expand(radius).contains(center) {
            self.painter.circle(center, radius, color);
        }
    }

    pub fn ring(&mut self, center: Point, radius: f32, width: f32, color: Color) {
        if self.area.expand(radius).contains(center) {
            self.painter.ring(center, radius, width, color);
        }
    }
}

impl Drop for Overlay<'_> {
    fn drop(&mut self) {
        self.painter.pop_clip();
    }
}

// The values a slider makes: from `start` at its left end to `end` at its
// right, a whole number of steps, between `minimum` and `maximum`, which can
// be inside its ends, like for a value that must be above 0. A value from a
// file can be between steps, and stays as it is until the slider moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    pub start: f64,
    pub end: f64,
    pub minimum: f64,
    pub maximum: f64,
    // Steps per unit, like 1000 for steps of 0.001.
    pub steps: f64,
}

impl SliderRange {
    pub const fn new(start: f64, end: f64, steps: f64) -> SliderRange {
        SliderRange {
            start,
            end,
            minimum: start,
            maximum: end,
            steps,
        }
    }

    pub const fn between(self, minimum: f64, maximum: f64) -> SliderRange {
        SliderRange { minimum, maximum, ..self }
    }

    // The step nearest `value`, within the range. Dividing whole numbers,
    // like 107 / 1000, gives the double nearest the decimal, so values save
    // as they show, like 0.107.
    pub fn nearest(&self, value: f64) -> f64 {
        ((value * self.steps).round() / self.steps).clamp(self.minimum, self.maximum)
    }
}

// A card's column of controls, from the top down: rows a control tall, each
// with its label at its left, headings over groups of them, and lines of
// text. Every row is SPACE_S from the next.
pub struct Form {
    pub area: Rect,
}

impl Form {
    pub fn new(area: Rect) -> Form {
        Form { area }
    }

    pub fn next(&mut self, height: f32) -> Rect {
        self.area.cut_top(height, SPACE_S)
    }

    pub fn skip(&mut self, height: f32) {
        self.area.cut_top(height, 0.0);
    }

    // A group's heading, under a line that sets it apart from what's above.
    pub fn heading(&mut self, ui: &mut Ui, text: &str) {
        self.skip(SPACE_XS);
        let line = self.area.cut_top(1.0, SPACE_M);
        ui.divider(line);
        let color = ui.theme().text_primary;
        let area = self.next(BODY_STRONG.line_height);
        ui.text(area, text, BODY_STRONG, color);
    }

    // A line of body text.
    pub fn text(&mut self, ui: &mut Ui, text: &str, color: Color) {
        let area = self.next(BODY.line_height);
        ui.text(area, text, BODY, color);
    }

    // A line of text that explains, in the secondary color.
    pub fn caption(&mut self, ui: &mut Ui, text: &str) {
        let area = self.next(CAPTION.line_height);
        ui.caption(area, text);
    }

    // A warning, like why a vehicle can't go.
    pub fn warning(&mut self, ui: &mut Ui, name: &str, title: &str, message: &str) {
        let info = InfoBar::new(Severity::Warning, title, message);
        let height = ui.info_bar_height(self.area.width, &info);
        let area = self.next(height);
        ui.info_bar(name, area, info);
    }

    // A row's label, before its control, and the area left for the control.
    pub fn row(&mut self, ui: &mut Ui, label: &str) -> Rect {
        let area = self.next(CONTROL_HEIGHT);
        let color = ui.theme().text_secondary;
        ui.text(area.left_strip(LABEL_WIDTH - SPACE_S), label, BODY, color);
        area.inset_by(LABEL_WIDTH, 0.0, 0.0, 0.0)
    }

    // A slider with its label before it and its value after it. Returns
    // whether it changed the value.
    pub fn slider(
        &mut self,
        ui: &mut Ui,
        name: &str,
        label: &str,
        value: &mut f64,
        range: SliderRange,
        format: impl Fn(f64) -> String,
    ) -> bool {
        let mut rail = self.row(ui, label);
        let value_area = rail.cut_right(VALUE_WIDTH, SPACE_M);
        let mut shown = *value as f32;
        let slider = Slider::new(range.start as f32, range.end as f32).step((1.0 / range.steps) as f32);
        let before = *value;
        if ui.slider(name, rail, &mut shown, slider).changed {
            *value = range.nearest(f64::from(shown));
        }
        let color = ui.theme().text_primary;
        ui.text_aligned(value_area, &format(*value), BODY, color, Align::End);
        *value != before
    }

    // A combo box with its label before it. Returns whether it changed.
    pub fn combo<S: AsRef<str>>(&mut self, ui: &mut Ui, name: &str, label: &str, items: &[S], selected: &mut usize) -> bool {
        let area = self.row(ui, label);
        ui.combo_box(name, area, selected, items).changed
    }

    // An editor's tools, as segments with icons, as wide as the card, one of
    // which is on.
    pub fn tools(&mut self, ui: &mut Ui, name: &str, items: &[(Icon, &str)], selected: &mut usize) -> bool {
        let area = self.next(CONTROL_HEIGHT);
        ui.segmented_icons(name, area, selected, items).changed
    }

    // Segments, one of which is on, with a label before them.
    pub fn segmented(&mut self, ui: &mut Ui, name: &str, label: &str, labels: &[&str], selected: &mut usize) -> bool {
        let area = self.row(ui, label);
        ui.segmented(name, area, selected, labels).changed
    }

    // A text field with its label before it, which takes keyboard focus,
    // unlike the editors' other controls.
    pub fn text_box(&mut self, ui: &mut Ui, name: &str, label: &str, text: &mut String, placeholder: &str) -> bool {
        let area = self.row(ui, label);
        ui.set_focusable(true);
        let changed = ui.text_box(name, area, text, TextBox::new().placeholder(placeholder).max_length(40)).changed;
        ui.set_focusable(false);
        changed
    }

    // A field for a color, like "#1CDA61", with a swatch of it. The color
    // changes as soon as the field holds one. `text` keeps what's typed.
    pub fn color(&mut self, ui: &mut Ui, name: &str, label: &str, text: &mut String, color: &mut FileColor) -> bool {
        let mut field = self.row(ui, label);
        let swatch = field.cut_right(CONTROL_HEIGHT, SPACE_S);
        // What's typed shows the color, unless it's being typed.
        if !ui.is_focused(name) {
            *text = files::color_text(*color);
        }
        ui.set_focusable(true);
        ui.text_box(name, field, text, TextBox::new().placeholder("#RRGGBB").max_length(7));
        ui.set_focusable(false);
        let mut changed = false;
        if let Some(parsed) = files::parse_color(text.trim())
            && parsed != *color
        {
            *color = parsed;
            changed = true;
        }
        swatch_rect(ui, swatch, interface_color(*color));
        changed
    }

    // A row of buttons, as wide as the card together. Returns the index of
    // the one clicked, if any.
    pub fn buttons(&mut self, ui: &mut Ui, buttons: &[(&str, Button)]) -> Option<usize> {
        let area = self.next(CONTROL_HEIGHT);
        let count = buttons.len().max(1) as f32;
        let width = (area.width - SPACE_S * (count - 1.0)) / count;
        let mut clicked = None;
        for (index, (name, button)) in buttons.iter().enumerate() {
            let column = Rect::new(area.x + index as f32 * (width + SPACE_S), area.y, width, area.height).round();
            if ui.button(name, column, *button).clicked {
                clicked = Some(index);
            }
        }
        clicked
    }

    // The sliders of a SimplePaint material, for one of a model's slots,
    // which the combo box picks. Returns whether the material changed.
    pub fn paint(&mut self, ui: &mut Ui, name: &str, slots: &[String], slot: &mut usize, parameters: &mut Parameters) -> bool {
        let mut area = self.row(ui, "Slot");
        let swatch = area.cut_right(CONTROL_HEIGHT, SPACE_S);
        ui.combo_box(&format!("{name}/Slot"), area, slot, slots);
        let [red, green, blue] = parameters.base_color_srgb.map(|channel| channel as f32);
        swatch_rect(ui, swatch, Color::rgba(red, green, blue, 1.0));
        let unit = SliderRange::new(0.0, 1.0, 1000.0);
        let (lowest, highest) = (simple_paint::MARGIN, simple_paint::INTERIOR_MAXIMUM);
        let fraction = |value: f64| format!("{value:.3}");
        let mut changed = false;
        let [red, green, blue] = &mut parameters.base_color_srgb;
        for (part, value) in [("Red", red), ("Green", green), ("Blue", blue)] {
            changed |= self.slider(ui, &format!("{name}/{part}"), part, value, unit.between(lowest, highest), fraction);
        }
        let sliders = [
            ("Brightness", "Brightness", &mut parameters.brightness, unit.between(lowest, highest)),
            ("Shift", "Shift", &mut parameters.shift, unit.between(0.0, highest)),
        ];
        for (part, label, value, range) in sliders {
            changed |= self.slider(ui, &format!("{name}/{part}"), label, value, range, fraction);
        }
        let degrees = SliderRange::new(0.0, 360.0, 10.0).between(0.0, 359.9);
        changed |= self.slider(ui, &format!("{name}/Rotation"), "Rotation", &mut parameters.rotation_degrees, degrees, |value| {
            format!("{value:.1}°")
        });
        let points = [
            ("DarkPoint", "Dark point", &mut parameters.dark_point, unit.between(0.0, highest)),
            ("LightPoint", "Light point", &mut parameters.light_point, unit.between(lowest, 1.0)),
        ];
        for (part, label, value, range) in points {
            changed |= self.slider(ui, &format!("{name}/{part}"), label, value, range, fraction);
        }
        changed
    }
}

// A square of a color, with a control's stroke around it.
pub fn swatch_rect(ui: &mut Ui, area: Rect, color: Color) {
    let stroke = ui.theme().control_stroke_secondary;
    ui.painter().shape(area, Corners::all(RADIUS_CONTROL), color, Some(Stroke::new(stroke, 1.0)));
}

// A card's subtitle, in its header, right of its title.
pub fn subtitle(ui: &mut Ui, header: Rect, text: &str) {
    let color = ui.theme().text_secondary;
    ui.text(header, text, BODY, color);
}
