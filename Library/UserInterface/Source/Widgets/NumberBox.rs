// WinUI's NumberBox: a text field for a number, with its spin buttons inline.
// What's typed takes effect with Enter or when the field loses focus, inside
// its range and on its steps; the spin buttons, the arrows and Page Up and
// Page Down, and the wheel while it has focus, step it.

use crate::core::control::{Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{NUMBER_BOX_SPIN_COLUMN, TEXT_BOX_BUTTON_MARGIN};
use crate::design::icons::Icon;
use crate::drawing::geometry::Rect;
use crate::input::devices::Key;
use crate::widgets::slider::Number;
use crate::widgets::text_box::{TextBox, draw_text_control_button};

pub struct NumberBox {
    value: f64,
    minimum: f64,
    maximum: f64,
    step: f64,
    large_step: f64,
    decimals: usize,
    field: TextBox,
    editing: bool,
    spin_up: Option<Response>,
    spin_down: Option<Response>,
}

fn numeric(character: char) -> bool {
    character.is_ascii_digit() || matches!(character, '.' | '-' | '+' | 'e' | 'E')
}

impl NumberBox {
    pub fn new(value: impl Number) -> Self {
        let mut this = Self {
            value: value.to_f64(),
            minimum: f64::MIN,
            maximum: f64::MAX,
            step: 1.0,
            large_step: 10.0,
            decimals: 0,
            field: TextBox::new("").accept(numeric).trailing(NUMBER_BOX_SPIN_COLUMN),
            editing: false,
            spin_up: None,
            spin_down: None,
        };
        this.field.set_text(&this.format(this.value));
        this
    }

    pub fn range(mut self, minimum: f64, maximum: f64) -> Self {
        self.minimum = minimum.min(maximum);
        self.maximum = minimum.max(maximum);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self.large_step = step * 10.0;
        self
    }

    pub fn large_step(mut self, step: f64) -> Self {
        self.large_step = step;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.field = std::mem::replace(&mut self.field, TextBox::new("")).placeholder(placeholder);
        self
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn value_as<T: Number>(&self) -> T {
        T::from_f64(self.value)
    }

    pub fn set_value(&mut self, value: impl Number) {
        self.value = self.clamp(value.to_f64());
        self.editing = false;
        self.field.set_text(&self.format(self.value));
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }

    fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.minimum, self.maximum)
    }
}

impl Control for NumberBox {
    fn kind(&self) -> &'static str {
        "NumberBox"
    }

    fn sense(&self) -> Sense {
        Sense::TEXT
    }

    fn label(&self) -> Option<String> {
        Some(self.format(self.value))
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<NumberBox>(previous) {
            self.value = self.clamp(previous.value);
            self.field.set_text(&self.format(self.value));
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let (name, rect, response) = (&ctx.name, ctx.rect, &ctx.response);
        let current = self.value;
        let mut value = current;
        if !response.focused && !self.editing {
            let text = self.format(current);
            self.field.set_text(&text);
        }
        let outcome = self.field.input(ui, name, rect, response);
        // It's a NumberBox to scripts, not a TextBox.
        ui.retarget(name, "NumberBox");
        if outcome.changed {
            self.editing = true;
        }
        let commit = outcome.submitted || (self.editing && !response.focused);
        if commit {
            if let Ok(typed) = self.field.text().trim().parse::<f64>() {
                value = self.clamp(typed);
            }
            self.editing = false;
            let text = self.format(value);
            self.field.set_text(&text);
        }
        // The spin buttons, inside the field's right end.
        let column = Rect::new(rect.right() - NUMBER_BOX_SPIN_COLUMN, rect.y, NUMBER_BOX_SPIN_COLUMN, rect.height);
        let [up, down] = column.columns::<2>(0.0);
        let up = up.pad(crate::drawing::geometry::Insets::new(4.0, 4.0, 0.0, 4.0));
        let down = down.pad(TEXT_BOX_BUTTON_MARGIN);
        let up_response = ui.interact(&format!("{name}/Up"), up, Sense::CLICK, "Button");
        let down_response = ui.interact(&format!("{name}/Down"), down, Sense::CLICK, "Button");
        let mut steps = 0.0;
        if up_response.clicked {
            steps += 1.0;
        }
        if down_response.clicked {
            steps -= 1.0;
        }
        self.spin_up = Some(up_response);
        self.spin_down = Some(down_response);
        if response.focused {
            let input = ui.input();
            if input.key_pressed(Key::UP) {
                steps += 1.0;
            }
            if input.key_pressed(Key::DOWN) {
                steps -= 1.0;
            }
            if input.key_pressed(Key::PAGE_UP) {
                steps += self.large_step / self.step;
            }
            if input.key_pressed(Key::PAGE_DOWN) {
                steps -= self.large_step / self.step;
            }
        }
        // The wheel steps it while it has focus and the pointer is on it, or
        // on one of its buttons, as WinUI's does; elsewhere, the wheel is for
        // what's under the pointer.
        let on_it = ui
            .hovered_name()
            .and_then(|hovered| hovered.strip_prefix(name.as_str()))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
        if response.focused && on_it {
            steps += ui.input().wheel.y.round() as f64;
        }
        if steps != 0.0 {
            if self.editing && let Ok(typed) = self.field.text().trim().parse::<f64>() {
                value = typed;
            }
            value = self.clamp(value + steps * self.step);
            self.editing = false;
            let text = self.format(value);
            self.field.set_text(&text);
        }
        if value != current {
            self.value = value;
            ctx.emit(ui, UiEventKind::Changed(value));
        }
        ui.set_label(name, &self.format(self.value));
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        self.field.paint(ui, ctx.rect, &ctx.response);
        if let Some(up) = self.spin_up {
            draw_text_control_button(ui, &up, Icon::ChevronUp);
        }
        if let Some(down) = self.spin_down {
            draw_text_control_button(ui, &down, Icon::ChevronDown);
        }
    }
}
