// Choosing a color: WinUI's ColorPicker's spectrum, a box of hue across and
// saturation down, with a slider of the value under it, whose thumbs are a
// ring and a slider's thumb; and a color field, a text field for a color's
// hex code with a swatch of it.

use crate::core::interaction::{Response, Sense, Wheel, id};
use crate::core::ui::Ui;
use crate::design::controls::{COLOR_SLIDER_HEIGHT, COLOR_SLIDER_INNER_THUMB, COLOR_SLIDER_RADIUS, COLOR_SPECTRUM_RING, SLIDER_THUMB_OVERHANG, SLIDER_THUMB_RADIUS, SLIDER_THUMB_SIZE};
use crate::design::tokens::{RADIUS_CONTROL, SPACE_L};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Border, Brush, Corners};
use crate::widgets::text_box::TextBox;
use crate::widgets::widget::{Elevation, Widget};

pub struct ColorPicker<'v> {
    color: &'v mut Color,
}

impl<'v> ColorPicker<'v> {
    pub fn new(color: &'v mut Color) -> Self {
        Self { color }
    }

    // How tall it is for a width: a square spectrum, and its value slider.
    pub fn height(width: f32) -> f32 {
        width + SPACE_L + SLIDER_THUMB_SIZE + SLIDER_THUMB_OVERHANG * 2.0
    }
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

// The hues around the color wheel, for the spectrum's gradient.
const HUES: [u32; 7] = [0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x0000FF, 0xFF00FF, 0xFF0000];

impl Widget for ColorPicker<'_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let picker_id = id(name);
        let mut hsv = std::mem::take(ui.memory::<Hsv>(picker_id));
        if hsv.last != Some(*self.color) {
            let [hue, saturation, value] = self.color.to_hsv();
            if saturation > 0.0 && value > 0.0 {
                hsv.hue = hue;
            }
            if value > 0.0 {
                hsv.saturation = saturation;
            }
            hsv.value = value;
        }
        let slider_height = SLIDER_THUMB_SIZE + SLIDER_THUMB_OVERHANG * 2.0;
        let spectrum = Rect::new(rect.x, rect.y, rect.width, (rect.height - SPACE_L - slider_height).max(0.0));
        let slider_row = Rect::new(rect.x, rect.bottom() - slider_height, rect.width, slider_height);
        let mut response = ui.interact(&format!("{name}/Spectrum"), spectrum, Sense::CLICK, "ColorSpectrum");
        if response.held && let Some(pointer) = response.pointer {
            let fraction = spectrum.fraction_of(pointer);
            hsv.hue = fraction.x.clamp(0.0, 1.0) * 360.0;
            hsv.saturation = 1.0 - fraction.y.clamp(0.0, 1.0);
            if hsv.value <= 0.0 {
                hsv.value = 1.0;
            }
            response.changed = true;
        }
        let value_response = ui.interact(&format!("{name}/Value"), slider_row, Sense::CLICK, "ColorSlider");
        let (left, right) = (slider_row.x + SLIDER_THUMB_SIZE / 2.0, slider_row.right() - SLIDER_THUMB_SIZE / 2.0);
        if value_response.held && let Some(pointer) = value_response.pointer {
            hsv.value = ((pointer.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0);
            response.changed = true;
        }
        if response.changed {
            *self.color = Color::from_hsv(hsv.hue, hsv.saturation, hsv.value).with_alpha(self.color.a);
        }
        hsv.last = Some(*self.color);
        let full = Color::from_hsv(hsv.hue, hsv.saturation, 1.0);
        let (hue, saturation, value) = (hsv.hue, hsv.saturation, hsv.value);
        *ui.memory::<Hsv>(picker_id) = hsv;
        ui.set_label(&format!("{name}/Spectrum"), &self.color.to_hex());

        let mut painter = ui.painter();
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
        response
    }
}

// A text field for a color's hex code, like "#1E90FF", with a swatch of the
// color in it. What's typed takes effect with Enter or when the field loses
// focus, if it's a color.
pub struct ColorField<'v> {
    color: &'v mut Color,
}

impl<'v> ColorField<'v> {
    pub fn new(color: &'v mut Color) -> Self {
        Self { color }
    }
}

#[derive(Default)]
struct ColorDraft {
    text: String,
    editing: bool,
}

fn hex_digit(character: char) -> bool {
    character.is_ascii_hexdigit() || character == '#'
}

impl Widget for ColorField<'_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let draft_id = id(&format!("{name}/Draft"));
        let mut draft = std::mem::take(ui.memory::<ColorDraft>(draft_id));
        // What's typed stays until it's taken or left, even as the field
        // loses focus, when it's taken.
        if !draft.editing {
            draft.text = self.color.to_hex();
        }
        let swatch_size = 16.0;
        let result = ui.add(name, rect, TextBox::new(&mut draft.text).accept(hex_digit).max_length(9).leading(swatch_size + 8.0));
        let mut response = result.response;
        if result.changed {
            draft.editing = true;
        }
        if result.submitted || (draft.editing && !response.focused) {
            if let Some(color) = Color::parse(&draft.text)
                && color != *self.color {
                    *self.color = color;
                    response.changed = true;
                }
            draft.editing = false;
            draft.text = self.color.to_hex();
        }
        ui.retarget(name, "ColorField", Wheel::Ignored);
        ui.set_label(name, &self.color.to_hex());
        let swatch = Rect::new(rect.x + 8.0, rect.center().y - swatch_size / 2.0, swatch_size, swatch_size).round();
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.shape(swatch, RADIUS_CONTROL - 2.0, *self.color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
        *ui.memory::<ColorDraft>(draft_id) = draft;
        response
    }
}
