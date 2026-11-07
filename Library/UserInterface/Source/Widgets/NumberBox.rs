// WinUI's NumberBox: a text field for a number, with its spin buttons inline.
// What's typed takes effect with Enter or when the field loses focus, inside
// its range and on its steps; the spin buttons, the arrows and Page Up and
// Page Down, and the wheel while it has focus, step it.

use crate::core::interaction::Response;
use crate::core::ui::Ui;
use crate::design::controls::{NUMBER_BOX_SPIN_COLUMN, TEXT_BOX_BUTTON_MARGIN};
use crate::design::icons::Icon;
use crate::drawing::geometry::Rect;
use crate::input::devices::Key;
use crate::widgets::slider::Number;
use crate::widgets::text_box::{TextBox, text_control_button};
use crate::widgets::widget::Widget;

pub struct NumberBox<'a, 'v, T: Number> {
    value: &'v mut T,
    minimum: f64,
    maximum: f64,
    step: f64,
    large_step: f64,
    decimals: usize,
    placeholder: &'a str,
}

impl<'a, 'v, T: Number> NumberBox<'a, 'v, T> {
    pub fn new(value: &'v mut T) -> Self {
        Self {
            value,
            minimum: f64::MIN,
            maximum: f64::MAX,
            step: 1.0,
            large_step: 10.0,
            decimals: 0,
            placeholder: "",
        }
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

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }

    fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.minimum, self.maximum)
    }
}

// What's typed, while the field has focus.
#[derive(Default)]
struct Draft {
    text: String,
    editing: bool,
}

fn numeric(character: char) -> bool {
    character.is_ascii_digit() || matches!(character, '.' | '-' | '+' | 'e' | 'E')
}

impl<T: Number> Widget for NumberBox<'_, '_, T> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let draft_id = crate::core::interaction::id(&format!("{name}/Draft"));
        let focused = ui.is_focused(name);
        let mut draft = std::mem::take(ui.memory::<Draft>(draft_id));
        let current = self.value.to_f64();
        if !focused || !draft.editing {
            draft.text = self.format(current);
        }
        let mut value = current;
        let field = TextBox::new(&mut draft.text).placeholder(self.placeholder).accept(numeric).trailing(NUMBER_BOX_SPIN_COLUMN);
        let result = ui.add(name, rect, field);
        let mut response = result.response;
        // It's a NumberBox to scripts, not a TextBox.
        ui.retarget(name, "NumberBox");
        if result.changed {
            draft.editing = true;
        }
        let commit = result.submitted || (draft.editing && !response.focused);
        if commit {
            if let Ok(typed) = draft.text.trim().parse::<f64>() {
                value = self.clamp(typed);
            }
            draft.editing = false;
            draft.text = self.format(value);
        }
        // The spin buttons, inside the field's right end.
        let column = Rect::new(rect.right() - NUMBER_BOX_SPIN_COLUMN, rect.y, NUMBER_BOX_SPIN_COLUMN, rect.height);
        let [up, down] = column.columns::<2>(0.0);
        let up = up.pad(crate::drawing::geometry::Insets::new(4.0, 4.0, 0.0, 4.0));
        let down = down.pad(TEXT_BOX_BUTTON_MARGIN);
        let mut steps = 0.0;
        if text_control_button(ui, &format!("{name}/Up"), up, Icon::ChevronUp).clicked {
            steps += 1.0;
        }
        if text_control_button(ui, &format!("{name}/Down"), down, Icon::ChevronDown).clicked {
            steps -= 1.0;
        }
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
            .and_then(|hovered| hovered.strip_prefix(name))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
        if response.focused && on_it {
            steps += ui.input().wheel.y.round() as f64;
        }
        if steps != 0.0 {
            if draft.editing && let Ok(typed) = draft.text.trim().parse::<f64>() {
                value = typed;
            }
            value = self.clamp(value + steps * self.step);
            draft.editing = false;
            draft.text = self.format(value);
        }
        if value != current {
            *self.value = T::from_f64(value);
            response.changed = true;
        }
        ui.set_label(name, &self.format(self.value.to_f64()));
        *ui.memory::<Draft>(draft_id) = draft;
        response
    }
}

