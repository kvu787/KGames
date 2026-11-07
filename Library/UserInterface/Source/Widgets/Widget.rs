// What every widget shares: the Widget trait, which `Ui::add` shows widgets
// through, the brushes of WinUI's visual states, its focus visual, and its
// elevation borders, the gradients that make controls look raised.

use crate::core::interaction::ControlState;
use crate::core::ui::Ui;
use crate::design::controls::FOCUS_VISUAL_MARGIN;
use crate::design::theme::{Theme, ThemeKind};
use crate::drawing::geometry::{Color, Insets, Point, Rect};
use crate::drawing::painter::{Border, Brush, Painter};

// A control that a frame shows, like a Button or a Slider. It's a value that
// says what the control is, and borrows what it changes, like a slider's
// value; `show` registers it under its name, draws it in its rectangle, and
// says what the input did.
pub trait Widget {
    type Response;
    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Self::Response;
}

impl Ui {
    // Shows a widget under the name, in the rectangle.
    pub fn add<W: Widget>(&mut self, name: &str, rect: Rect, widget: W) -> W::Response {
        widget.show(self, name, rect)
    }

    // Shows a widget enabled or not, leaving the widgets around it alone.
    // In a disabled scope it stays disabled either way.
    pub fn add_enabled<W: Widget>(&mut self, enabled: bool, name: &str, rect: Rect, widget: W) -> W::Response {
        self.with_enabled(enabled, |ui| ui.add(name, rect, widget))
    }
}

// A brush for each of WinUI's visual states, like ButtonBackground,
// ButtonBackgroundPointerOver, ButtonBackgroundPressed and
// ButtonBackgroundDisabled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateColors {
    pub normal: Color,
    pub pointer_over: Color,
    pub pressed: Color,
    pub disabled: Color,
}

impl StateColors {
    pub const fn new(normal: Color, pointer_over: Color, pressed: Color, disabled: Color) -> Self {
        Self {
            normal,
            pointer_over,
            pressed,
            disabled,
        }
    }

    pub const fn all(color: Color) -> Self {
        Self::new(color, color, color, color)
    }

    pub fn get(&self, state: ControlState) -> Color {
        match state {
            ControlState::Normal => self.normal,
            ControlState::PointerOver => self.pointer_over,
            ControlState::Pressed => self.pressed,
            ControlState::Disabled => self.disabled,
        }
    }
}

// WinUI's elevation border brushes: gradients across a control's border
// that shade one edge, so that it looks raised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    // ControlElevationBorderBrush, of buttons, combo boxes and the like.
    Control,
    // AccentControlElevationBorderBrush, of accent buttons.
    Accent,
    // CircleElevationBorderBrush, of round things like a slider's thumb.
    Circle,
    // TextControlElevationBorderBrush, of text fields at rest.
    TextControl,
    // TextControlElevationBorderFocusedBrush: the accent along the bottom.
    TextControlFocused,
}

impl Elevation {
    // The border's brush over a control in the rectangle.
    pub fn brush(self, theme: &Theme, rect: Rect) -> Brush {
        let x = rect.x;
        let top = rect.y;
        let bottom = rect.bottom();
        let line = |from: f32, to: f32, start: Color, end: Color| Brush::Linear {
            from: Point::new(x, from),
            to: Point::new(x, to),
            start,
            end,
        };
        match self {
            // From 0 to 3 pixels, mirrored to the bottom in the light theme.
            // The dark theme's isn't mirrored, so its lighter edge is the top.
            Elevation::Control => match theme.kind {
                ThemeKind::Light => line(bottom - 1.0, bottom - 3.0, theme.control_stroke_secondary, theme.control_stroke),
                ThemeKind::Dark => line(top + 1.0, top + 3.0, theme.control_stroke_secondary, theme.control_stroke),
            },
            Elevation::Accent => line(bottom - 1.0, bottom - 3.0, theme.control_stroke_on_accent_secondary, theme.control_stroke_on_accent),
            Elevation::Circle => line(top + rect.height * 0.5, top + rect.height * 0.7, theme.control_stroke, theme.control_stroke_secondary),
            Elevation::TextControl => line(bottom - 1.0, bottom - 2.0, theme.control_strong_stroke, theme.control_stroke),
            Elevation::TextControlFocused => line(bottom - 2.0, bottom - 2.01, theme.accent, theme.control_stroke),
        }
    }

    pub fn border(self, theme: &Theme, rect: Rect) -> Border {
        let widths = if self == Elevation::TextControlFocused { Insets::new(1.0, 1.0, 1.0, 2.0) } else { Insets::all(1.0) };
        Border::sides(self.brush(theme, rect), widths)
    }
}

// WinUI's focus visual: a 2-pixel outer stroke in FocusStrokeColorOuter and
// a 1-pixel inner one in FocusStrokeColorInner, around the control by its
// FocusVisualMargin, with its corners as much rounder as the margin is wide.
pub fn focus_visual(painter: &mut Painter, theme: &Theme, rect: Rect, radius: f32, margin: Insets) {
    let outer = Rect::new(rect.x - margin.left, rect.y - margin.top, rect.width + margin.horizontal(), rect.height + margin.vertical());
    let outer_radius = if radius > 0.0 { radius + (margin.left + margin.top) / 2.0 } else { 0.0 };
    painter.shape(outer, outer_radius, Color::TRANSPARENT, Some(Border::new(theme.focus_outer, 2.0)));
    painter.shape(outer.inset(2.0), (outer_radius - 2.0).max(0.0), Color::TRANSPARENT, Some(Border::new(theme.focus_inner, 1.0)));
}

impl Ui {
    // Draws the focus visual 3 pixels around the rectangle, if `focused` and
    // the keyboard moved focus there.
    pub fn focus_visual_if(&mut self, focused: bool, rect: Rect, radius: f32) {
        self.focus_visual_around(focused, rect, radius, FOCUS_VISUAL_MARGIN);
    }

    pub fn focus_visual_around(&mut self, focused: bool, rect: Rect, radius: f32, margin: Insets) {
        if focused && self.focus_visible() {
            let mut painter = self.painter();
            let theme = painter.theme();
            focus_visual(&mut painter, theme, rect, radius, margin);
        }
    }
}
