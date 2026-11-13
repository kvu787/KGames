// Choosing a color: WinUI's ColorPicker's spectrum, a box of hue across and
// saturation down, with a slider of the value under it, whose thumbs are a
// ring and a slider's thumb; and a color field, a text field for a color's
// hex code with a swatch of it. Both send the color as the input leaves it.

use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{COLOR_SLIDER_HEIGHT, COLOR_SLIDER_INNER_THUMB, COLOR_SLIDER_RADIUS, COLOR_SPECTRUM_RING, SLIDER_THUMB_OVERHANG, SLIDER_THUMB_RADIUS, SLIDER_THUMB_SIZE};
use crate::design::tokens::{RADIUS_CONTROL, SPACE_L};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Border, Brush, Corners};
use crate::widgets::text_box::{FieldSettings, field_draw, field_interact, field_parts};
use crate::widgets::widget::{Elevation, Reaction, react};

pub struct ColorPicker<M> {
    color: Color,
    on_change: Option<Reaction<Color, M>>,
}

// The color as hue, saturation and value, kept so that a gray keeps the hue
// it had.
#[derive(Default)]
struct Hsv {
    hue: f32,
    saturation: f32,
    value: f32,
    last: Option<Color>,
}

impl Hsv {
    // Takes the color, unless it's the one this came from.
    fn sync(&mut self, color: Color) {
        if self.last == Some(color) {
            return;
        }
        let [hue, saturation, value] = color.to_hsv();
        if saturation > 0.0 && value > 0.0 {
            self.hue = hue;
        }
        if value > 0.0 {
            self.saturation = saturation;
        }
        self.value = value;
        self.last = Some(color);
    }
}

// The hues around the color wheel, for the spectrum's gradient.
const HUES: [u32; 7] = [0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x0000FF, 0xFF00FF, 0xFF0000];

const SPECTRUM: &str = "Spectrum";
const VALUE: &str = "Value";

impl<M: Clone + 'static> ColorPicker<M> {
    pub fn new(color: Color) -> Self {
        Self { color, on_change: None }
    }

    // What moving a thumb sends, with the color.
    pub fn on_change(mut self, reaction: impl Fn(Color) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    // How tall it is for a width: a square spectrum, and its value slider.
    pub fn height(width: f32) -> f32 {
        width + SPACE_L + SLIDER_THUMB_SIZE + SLIDER_THUMB_OVERHANG * 2.0
    }

    // The spectrum, and the value slider's row under it.
    fn areas(rect: Rect) -> (Rect, Rect) {
        let slider_height = SLIDER_THUMB_SIZE + SLIDER_THUMB_OVERHANG * 2.0;
        let spectrum = Rect::new(rect.x, rect.y, rect.width, (rect.height - SPACE_L - slider_height).max(0.0));
        let slider_row = Rect::new(rect.x, rect.bottom() - slider_height, rect.width, slider_height);
        (spectrum, slider_row)
    }

    fn rail(slider_row: Rect) -> (f32, f32) {
        (slider_row.x + SLIDER_THUMB_SIZE / 2.0, slider_row.right() - SLIDER_THUMB_SIZE / 2.0)
    }
}

impl<M: Clone + 'static> Widget<M> for ColorPicker<M> {
    fn kind(&self) -> &'static str {
        "ColorPicker"
    }

    fn label(&self) -> Option<String> {
        Some(self.color.to_hex())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let (spectrum, slider_row) = Self::areas(ctx.rect());
        ctx.part(Part::Named(SPECTRUM.to_string()), SPECTRUM, spectrum, Sense::CLICK, "ColorSpectrum", Some(self.color.to_hex()));
        ctx.part(Part::Named(VALUE.to_string()), VALUE, slider_row, Sense::CLICK, "ColorSlider", None);
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let (spectrum, slider_row) = Self::areas(ctx.rect());
        let mut hsv = std::mem::take(ctx.memory::<Hsv>());
        hsv.sync(self.color);
        let mut changed = false;
        if let Some(hit) = interaction.hit(&Part::Named(SPECTRUM.to_string()))
            && hit.held
            && let Some(pointer) = interaction.pointer
        {
            let fraction = spectrum.fraction_of(pointer);
            hsv.hue = fraction.x.clamp(0.0, 1.0) * 360.0;
            hsv.saturation = 1.0 - fraction.y.clamp(0.0, 1.0);
            if hsv.value <= 0.0 {
                hsv.value = 1.0;
            }
            changed = true;
        }
        if let Some(hit) = interaction.hit(&Part::Named(VALUE.to_string()))
            && hit.held
            && let Some(pointer) = interaction.pointer
        {
            let (left, right) = Self::rail(slider_row);
            hsv.value = ((pointer.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0);
            changed = true;
        }
        if changed {
            let color = Color::from_hsv(hsv.hue, hsv.saturation, hsv.value).with_alpha(self.color.a);
            hsv.last = Some(color);
            if color != self.color
                && let Some(reaction) = &self.on_change
            {
                ctx.send(reaction(color));
            }
        }
        *ctx.memory::<Hsv>() = hsv;
    }

    fn draw(&self, ctx: &mut Draw) {
        let (spectrum, slider_row) = Self::areas(ctx.rect());
        let (left, right) = Self::rail(slider_row);
        let hsv = ctx.memory::<Hsv>();
        hsv.sync(self.color);
        let (hue, saturation, value) = (hsv.hue, hsv.saturation, hsv.value);
        let full = Color::from_hsv(hue, saturation, 1.0);
        let mut painter = ctx.painter();
        let theme = painter.theme();
        // The hues across, in six gradients, the ends rounded, and white over
        // them toward the bottom, where saturation is 0.
        let segment = spectrum.width / 6.0;
        for index in 0..6 {
            let x = (spectrum.x + segment * index as f32).round();
            let right = (spectrum.x + segment * (index + 1) as f32).round();
            let part = Rect::new(x, spectrum.y, right - x, spectrum.height);
            let corners = match index {
                0 => Corners::left(RADIUS_CONTROL),
                5 => Corners::right(RADIUS_CONTROL),
                _ => Corners::ZERO,
            };
            painter.shape(part, corners, Brush::horizontal(part, Color::hex(HUES[index]), Color::hex(HUES[index + 1])), None);
        }
        painter.shape(spectrum, RADIUS_CONTROL, Brush::vertical(spectrum, Color::WHITE.with_alpha(0.0), Color::WHITE), None);
        painter.shape(spectrum, RADIUS_CONTROL, Color::TRANSPARENT, Some(Border::new(theme.control_stroke, 1.0)));
        let ring_center = Point::new(spectrum.x + spectrum.width * hue / 360.0, spectrum.y + spectrum.height * (1.0 - saturation));
        let ring = Rect::from_center(ring_center, Size::square(COLOR_SPECTRUM_RING));
        painter.shape(ring, COLOR_SPECTRUM_RING / 2.0, Color::TRANSPARENT, Some(Border::new(Color::WHITE, 2.0)));
        painter.shape(ring.expand(1.0), COLOR_SPECTRUM_RING / 2.0 + 1.0, Color::TRANSPARENT, Some(Border::new(Color::BLACK.with_alpha(0.6), 1.0)));

        // The value, from black to the full color.
        let track = Rect::new(slider_row.x, slider_row.center().y - COLOR_SLIDER_HEIGHT / 2.0, slider_row.width, COLOR_SLIDER_HEIGHT);
        painter.shape(track, COLOR_SLIDER_RADIUS, Brush::horizontal(track, Color::BLACK, full), Some(Border::new(theme.control_stroke, 1.0)));
        let thumb = Rect::from_center(Point::new(left + (right - left) * value, track.center().y), Size::square(SLIDER_THUMB_SIZE)).expand(SLIDER_THUMB_OVERHANG);
        painter.shape(thumb, SLIDER_THUMB_RADIUS, theme.control_solid_fill, Some(Elevation::Control.border(theme, thumb)));
        painter.circle(thumb.center(), COLOR_SLIDER_INNER_THUMB / 2.0, Color::from_hsv(hue, saturation, value));
    }
}

// A text field for a color's hex code, like "#1E90FF", with a swatch of the
// color in it. What's typed takes effect with Enter, if it's a color, and
// goes if the field loses focus first.
pub struct ColorField<M> {
    color: Color,
    on_change: Option<Reaction<Color, M>>,
}

// What's typed, while the field has focus.
#[derive(Default)]
struct ColorDraft {
    text: String,
    editing: bool,
}

const SWATCH_SIZE: f32 = 16.0;

fn hex_digit(character: char) -> bool {
    character.is_ascii_hexdigit() || character == '#'
}

fn settings() -> FieldSettings {
    FieldSettings {
        accept: Some(hex_digit),
        max_length: Some(9),
        leading: SWATCH_SIZE + 8.0,
        ..FieldSettings::default()
    }
}

impl<M: Clone + 'static> ColorField<M> {
    pub fn new(color: Color) -> Self {
        Self { color, on_change: None }
    }

    // What a typed color taking effect sends.
    pub fn on_change(mut self, reaction: impl Fn(Color) -> M + 'static) -> Self {
        self.on_change = Some(react(reaction));
        self
    }

    fn shown_text(&self, draft: &ColorDraft) -> String {
        if draft.editing { draft.text.clone() } else { self.color.to_hex() }
    }
}

impl<M: Clone + 'static> Widget<M> for ColorField<M> {
    fn kind(&self) -> &'static str {
        "ColorField"
    }

    fn label(&self) -> Option<String> {
        Some(self.color.to_hex())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::TEXT)
    }

    fn parts(&self, ctx: &mut Parts) {
        // What's typed and not taken with Enter goes as the field loses
        // focus.
        if !ctx.is_focused() {
            *ctx.memory::<ColorDraft>() = ColorDraft::default();
        }
        let text = self.shown_text(ctx.memory::<ColorDraft>());
        field_parts(ctx, &text, &settings());
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let mut draft = std::mem::take(ctx.memory::<ColorDraft>());
        if !draft.editing {
            draft.text = self.color.to_hex();
        }
        let edited = field_interact(&draft.text, &settings(), interaction, ctx);
        if let Some(text) = edited.text {
            draft.text = text;
            draft.editing = true;
        }
        if edited.submitted {
            if let Some(color) = Color::parse(&draft.text)
                && color != self.color
                && let Some(reaction) = &self.on_change
            {
                ctx.send(reaction(color));
            }
            draft.editing = false;
            draft.text = self.color.to_hex();
        }
        *ctx.memory::<ColorDraft>() = draft;
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let text = self.shown_text(ctx.memory::<ColorDraft>());
        field_draw(ctx, &text, &settings());
        let swatch = Rect::new(rect.x + 8.0, rect.center().y - SWATCH_SIZE / 2.0, SWATCH_SIZE, SWATCH_SIZE).round();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        painter.shape(swatch, RADIUS_CONTROL - 2.0, self.color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
    }
}
