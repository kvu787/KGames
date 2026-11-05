// Small labels: chips, which wrap onto rows, like Material's filter chips
// that a press turns on or off, with a check while they're on, and chips that
// show what's on, like the buttons that are held; badges, like Fluent's info
// badges, which show a status; and keycaps, which show a key to press.

use canvas::{Align, Canvas, Color, Fonts, Icon, Rect, TextStyle, Theme};

use crate::interface::{Id, Layer, Sense};
use crate::surfaces::{focus_ring, raised};
use crate::tokens::*;
use crate::Frame;

const CHIP_RADIUS: f32 = 6.0;
const CHIP_GAP: f32 = 6.0;
// Inside a chip, at each end.
const CHIP_INSET: f32 = 10.0;
// A filter chip's check, and the gap after it.
const CHECK_ROOM: f32 = 12.0 + 6.0;

// What a chip shows: a label, and whether it's on.
#[derive(Clone, Copy, Debug)]
pub struct Chip<'a> {
    pub label: &'a str,
    pub on: bool,
    // Faint, for what a device or a setting doesn't have.
    pub enabled: bool,
}

// A chip's width for its label, with room for a check if it has one.
fn chip_width(fonts: &Fonts, label: &str, check: bool) -> f32 {
    fonts.measure(label, TextStyle::Body).ceil() + CHIP_INSET * 2.0 + if check { CHECK_ROOM } else { 0.0 }
}

// Places chips of these widths left to right, wrapping onto new rows to fit
// in `width`. Returns each chip's offset from the top left, and the height of
// all the rows.
fn flow(widths: &[f32], width: f32) -> (Vec<[f32; 2]>, f32) {
    let (mut x, mut y) = (0.0, 0.0);
    let mut places = Vec::new();
    for chip in widths {
        if x + chip > width && x > 0.0 {
            x = 0.0;
            y += CHIP_HEIGHT + CHIP_GAP;
        }
        places.push([x, y]);
        x += chip + CHIP_GAP;
    }
    (places, if widths.is_empty() { 0.0 } else { y + CHIP_HEIGHT })
}

// How tall chips with these labels are, as they wrap in `width`.
pub fn chips_height(fonts: &Fonts, width: f32, labels: &[&str]) -> f32 {
    let widths: Vec<f32> = labels.iter().map(|label| chip_width(fonts, label, false)).collect();
    flow(&widths, width).1
}

// Text on a color: black or white, whichever stands out more.
pub fn on_color(color: Color) -> Color {
    let [red, green, blue, _] = color.to_floats();
    let luminance = 0.299 * red + 0.587 * green + 0.114 * blue;
    if luminance > 0.6 { Color::rgb(0x000000) } else { Color::rgb(0xFFFFFF) }
}

// Draws chips that show what's on, in `on_color` while they're on, as many
// as fit down `area`, and returns how tall they are.
pub fn status_chips(canvas: &mut Canvas, theme: &Theme, area: Rect, items: &[Chip], on_color: Color) -> f32 {
    let widths: Vec<f32> = items.iter().map(|item| chip_width(canvas.fonts, item.label, false)).collect();
    let (places, height) = flow(&widths, area.width);
    for ((item, width), [x, y]) in items.iter().zip(widths).zip(places) {
        let chip = Rect::new(area.x + x, area.y + y, width, CHIP_HEIGHT);
        if chip.bottom() > area.bottom() + 0.5 {
            break;
        }
        let text = if item.on {
            canvas.fill(chip, CHIP_RADIUS, on_color);
            self::on_color(on_color)
        } else {
            canvas.stroke(chip, CHIP_RADIUS, 1.0, theme.divider.mix(theme.control_strong, 0.25));
            if item.enabled { theme.text_secondary } else { theme.text_disabled.alpha(0.6) }
        };
        canvas.text_in(chip, item.label, TextStyle::Body, text, Align::Center);
    }
    height
}

impl Frame<'_> {
    // Chips that show what's on, like the buttons that are held, which
    // wrap onto more rows to fit across `area`. Returns their height.
    pub fn status_chips(&mut self, area: Rect, items: &[Chip], on_color: Color) -> f32 {
        let theme = self.theme();
        status_chips(&mut self.canvas, theme, area, items, on_color)
    }

    // Filter chips: chips that a press turns on or off, with a check while
    // they're on, one for each label, which wrap onto more rows to fit
    // across `area`. Each is called "<name>/<label>". Returns whether any
    // changed, and their height.
    pub fn filter_chips(&mut self, name: &str, area: Rect, labels: &[&str], values: &mut [bool]) -> (bool, f32) {
        // A press flips a chip before it's laid out, so that the frame shows
        // it as it is.
        let mut changed = false;
        for (label, value) in labels.iter().zip(values.iter_mut()) {
            let part = Id::new(&format!("{name}/{label}"));
            let clicks = self
                .events_for(part)
                .iter()
                .filter(|event| match event {
                    application::Event::Press(_) => true,
                    application::Event::Key { key, control: false, repeat: false, .. } => {
                        *key as i32 == windows_api::VK_SPACE || *key as i32 == windows_api::VK_RETURN
                    }
                    _ => false,
                })
                .count();
            if clicks % 2 == 1 {
                *value = !*value;
                changed = true;
            }
        }
        let fonts = self.fonts();
        let widths: Vec<f32> = labels.iter().zip(values.iter()).map(|(label, on)| chip_width(fonts, label, *on)).collect();
        let (places, height) = flow(&widths, area.width);
        let enabled = self.enabled();
        let theme = self.theme();
        for (((label, value), width), [x, y]) in labels.iter().zip(values.iter()).zip(widths).zip(places) {
            let chip = Rect::new(area.x + x, area.y + y, width, CHIP_HEIGHT);
            let part = format!("{name}/{label}");
            let id = Id::new(&part);
            let response = self.interact_on(&part, "Chip", "", chip, Sense::BUTTON, Layer::Base, None);
            let (hover, press) = self.hover_and_press(id, &response);
            let canvas = &mut self.canvas;
            let state = Color::TRANSPARENT.mix(theme.subtle_hover, hover).mix(theme.subtle_pressed, press);
            let (text, check) = match (*value, enabled) {
                (true, true) => {
                    canvas.fill(chip, CHIP_RADIUS, state.over(theme.accent_tonal));
                    (theme.text, theme.accent_text)
                }
                (true, false) => {
                    canvas.fill(chip, CHIP_RADIUS, theme.control_disabled);
                    (theme.text_disabled, theme.text_disabled)
                }
                (false, _) => {
                    canvas.fill_stroke(chip, CHIP_RADIUS, state, 1.0, theme.control_strong.alpha(0.45));
                    let text = if enabled { theme.text_secondary.mix(theme.text, hover) } else { theme.text_disabled };
                    (text, text)
                }
            };
            let mut text_area = chip.inset_by(CHIP_INSET, 0.0);
            if *value {
                canvas.icon([text_area.x + 6.0, chip.center()[1]], Icon::CheckMark, 12.0, check);
                text_area = Rect::from_edges(text_area.x + CHECK_ROOM, chip.y, text_area.right(), chip.bottom());
            }
            canvas.text_in(text_area, label, TextStyle::Body, text, Align::Start);
            if self.shows_focus(&response) {
                focus_ring(&mut self.canvas, theme, chip, CHIP_RADIUS);
            }
        }
        (changed, height)
    }

    // A badge: a pill with a dot and a few words, like a status, from
    // (x, y) at its left middle. Returns its area.
    pub fn badge(&mut self, [x, y]: [f32; 2], text: &str, color: Color) -> Rect {
        let theme = self.theme();
        badge(&mut self.canvas, theme, [x, y], text, color)
    }

    // A keycap: a key's name on a key, from (x, y) at its left middle.
    // Returns its area.
    pub fn keycap(&mut self, [x, y]: [f32; 2], key: &str) -> Rect {
        let theme = self.theme();
        keycap(&mut self.canvas, theme, [x, y], key)
    }
}

// How wide a badge is for its text.
pub fn badge_width(fonts: &Fonts, text: &str) -> f32 {
    8.0 + 8.0 + 6.0 + fonts.measure(text, TextStyle::Caption).ceil() + 10.0
}

pub fn badge(canvas: &mut Canvas, theme: &Theme, [x, y]: [f32; 2], text: &str, color: Color) -> Rect {
    let width = badge_width(canvas.fonts, text);
    let area = Rect::new(x, (y - 12.0).round(), width, 24.0);
    canvas.fill(area, 12.0, color.alpha(if theme.dark { 0.22 } else { 0.12 }));
    canvas.circle([area.x + 12.0, area.center()[1]], 4.0, color);
    let text_area = Rect::from_edges(area.x + 22.0, area.y, area.right() - 10.0, area.bottom());
    canvas.text_in(text_area, text, TextStyle::Caption, theme.text, Align::Start);
    area
}

// How wide a keycap is for a key's name.
pub fn keycap_width(fonts: &Fonts, key: &str) -> f32 {
    (fonts.measure(key, TextStyle::Caption).ceil() + 12.0).max(22.0)
}

pub fn keycap(canvas: &mut Canvas, theme: &Theme, [x, y]: [f32; 2], key: &str) -> Rect {
    let width = keycap_width(canvas.fonts, key);
    let area = Rect::new(x, (y - 11.0).round(), width, 22.0);
    raised(canvas, area, RADIUS_CONTROL, theme.control, theme.control_stroke, theme.control_stroke_bottom);
    canvas.text_in(area.translate([0.0, -0.5]), key, TextStyle::Caption, theme.text_secondary, Align::Center);
    area
}
