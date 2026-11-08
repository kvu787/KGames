// WinUI's NumberBox: a text field for a number, with its spin buttons inline.
// What's typed takes effect with Enter or when the field loses focus, inside
// its range and on its steps; the spin buttons, the arrows and Page Up and
// Page Down, and the wheel while it has focus, step it. The value is the
// program's; what's typed, until it takes effect, is the box's own.

use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{NUMBER_BOX_SPIN_COLUMN, NUMBER_BOX_SPIN_GLYPH, TEXT_BOX_BUTTON_MARGIN};
use crate::design::icons::Icon;
use crate::drawing::geometry::{Insets, Rect};
use crate::input::devices::Key;
use crate::widgets::buttons::draw_text_control_button;
use crate::widgets::text_box::{FieldSettings, field_draw, field_interact, field_parts};
use crate::widgets::widget::{Reaction, react};

pub struct NumberBox<M> {
    value: f64,
    minimum: f64,
    maximum: f64,
    step: f64,
    large_step: f64,
    decimals: usize,
    placeholder: String,
    on_change: Option<Reaction<f64, M>>,
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

impl<M: Clone + 'static> NumberBox<M> {
    pub fn new(value: impl Into<f64>) -> Self {
        Self {
            value: value.into(),
            minimum: f64::MIN,
            maximum: f64::MAX,
            step: 1.0,
            large_step: 10.0,
            decimals: 0,
            placeholder: String::new(),
            on_change: None,
        }
    }

    // What a change sends, with the value.
    pub fn on_change(mut self, reaction: impl Fn(f64) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
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

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }

    fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.minimum, self.maximum)
    }

    fn settings(&self) -> FieldSettings {
        FieldSettings {
            placeholder: self.placeholder.clone(),
            accept: Some(numeric),
            trailing: NUMBER_BOX_SPIN_COLUMN,
            ..FieldSettings::default()
        }
    }

    // The text the field shows: what's typed, or the value.
    fn shown_text(&self, draft: &Draft, focused: bool) -> String {
        if focused && draft.editing { draft.text.clone() } else { self.format(self.value) }
    }

    // The spin buttons, inside the field's right end.
    fn spin_buttons(rect: Rect) -> (Rect, Rect) {
        let column = Rect::new(rect.right() - NUMBER_BOX_SPIN_COLUMN, rect.y, NUMBER_BOX_SPIN_COLUMN, rect.height);
        let [up, down] = column.columns::<2>(0.0);
        (up.pad(Insets::new(4.0, 4.0, 0.0, 4.0)), down.pad(TEXT_BOX_BUTTON_MARGIN))
    }
}

impl<M: Clone + 'static> Widget<M> for NumberBox<M> {
    fn kind(&self) -> &'static str {
        "NumberBox"
    }

    fn label(&self) -> Option<String> {
        Some(self.format(self.value))
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::TEXT.takes_wheel_while_focused())
    }

    fn parts(&self, ctx: &mut Parts) {
        let focused = ctx.is_focused();
        let text = self.shown_text(ctx.memory::<Draft>(), focused);
        field_parts(ctx, &text, &self.settings());
        let (up, down) = Self::spin_buttons(ctx.rect());
        ctx.part(Part::Named("Up".to_string()), "Up", up, Sense::CLICK, "Button", None);
        ctx.part(Part::Named("Down".to_string()), "Down", down, Sense::CLICK, "Button", None);
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let focused = interaction.focused;
        let mut draft = std::mem::take(ctx.memory::<Draft>());
        let current = self.value;
        // What's typed stays until it's taken or left, even as the field
        // loses focus, when it's taken - never thrown away.
        if !draft.editing {
            draft.text = self.format(current);
        }
        let mut value = current;
        let edited = field_interact(&draft.text, &self.settings(), interaction, ctx);
        if let Some(text) = edited.text {
            draft.text = text;
            draft.editing = true;
        }
        let commit = edited.submitted || (draft.editing && !ctx.is_focused());
        if commit {
            if let Ok(typed) = draft.text.trim().parse::<f64>() {
                value = self.clamp(typed);
            }
            draft.editing = false;
            draft.text = self.format(value);
        }
        let mut steps = 0.0;
        if interaction.clicked(&Part::Named("Up".to_string())) {
            steps += 1.0;
        }
        if interaction.clicked(&Part::Named("Down".to_string())) {
            steps -= 1.0;
        }
        if focused {
            let input = ctx.input();
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
        if focused && interaction.hovered_part().is_some() {
            steps += interaction.wheel().y.round() as f64;
        }
        if steps != 0.0 {
            if draft.editing && let Ok(typed) = draft.text.trim().parse::<f64>() {
                value = typed;
            }
            value = self.clamp(value + steps * self.step);
            draft.editing = false;
            draft.text = self.format(value);
        }
        *ctx.memory::<Draft>() = draft;
        if value != current
            && let Some(reaction) = &self.on_change
        {
            ctx.send(reaction(value));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let focused = ctx.focused();
        let text = self.shown_text(ctx.memory::<Draft>(), focused);
        field_draw(ctx, &text, &self.settings());
        let (up, down) = Self::spin_buttons(ctx.rect());
        draw_text_control_button(ctx, &Part::Named("Up".to_string()), up, Icon::ChevronUp, NUMBER_BOX_SPIN_GLYPH);
        draw_text_control_button(ctx, &Part::Named("Down".to_string()), down, Icon::ChevronDown, NUMBER_BOX_SPIN_GLYPH);
    }
}
