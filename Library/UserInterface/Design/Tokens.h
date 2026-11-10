// Fluent 2's design tokens on Windows: spacing, corner radii, sizes, the type
// ramp, motion and elevation, with the values WinUI 3 gives them. Size and
// space everything with these, and color it with the theme's roles (see
// Theme.h), so that every program looks like one family, and like Windows
// 11. Controls' own sizes, like a checkbox's box, are in Controls.h.
//
// Fluent 2: https://fluent2.microsoft.design. WinUI 3's values come from
// microsoft/microsoft-ui-xaml's theme resources.

#pragma once

#include "UserInterface/Drawing/Text.h"

#include <array>

namespace user_interface::tokens {

// Spacing, on Fluent's 4-pixel grid, with its 2-pixel and 6-pixel nudges.
inline constexpr float SpaceNone = 0.0f;
inline constexpr float SpaceXxs = 2.0f;
inline constexpr float SpaceXs = 4.0f;
inline constexpr float SpaceSNudge = 6.0f;
inline constexpr float SpaceS = 8.0f;
inline constexpr float SpaceMNudge = 10.0f;
inline constexpr float SpaceM = 12.0f;
inline constexpr float SpaceL = 16.0f;
inline constexpr float SpaceXl = 20.0f;
inline constexpr float SpaceXxl = 24.0f;
inline constexpr float SpaceXxxl = 32.0f;
inline constexpr float SpaceHuge = 40.0f;

// Corner radii: ControlCornerRadius for controls, and OverlayCornerRadius
// for what floats or groups, like flyouts, dialogs and cards.
inline constexpr float RadiusNone = 0.0f;
inline constexpr float RadiusSmall = 2.0f;
inline constexpr float RadiusControl = 4.0f;
inline constexpr float RadiusOverlay = 8.0f;

// Strokes.
inline constexpr float StrokeThin = 1.0f;
inline constexpr float StrokeThick = 2.0f;

// Controls' heights: WinUI's standard, compact and large.
inline constexpr float ControlHeight = 32.0f;
inline constexpr float ControlHeightCompact = 24.0f;
inline constexpr float ControlHeightLarge = 40.0f;
// Icons in controls, and on their own: WinUI's glyph sizes.
inline constexpr float IconSmall = 12.0f;
inline constexpr float Icon = 16.0f;
inline constexpr float IconLarge = 20.0f;
inline constexpr float IconExtraLarge = 24.0f;

// The type ramp: Windows' sizes and line heights, each in the optical size
// of Segoe UI Variable that Fluent 2 sets it in.
inline constexpr TextStyle Caption = TextStyle(Font::Small, Weight::Regular, 12.0f, 16.0f);
inline constexpr TextStyle CaptionStrong = TextStyle(Font::Small, Weight::Semibold, 12.0f, 16.0f);
inline constexpr TextStyle Body = TextStyle(Font::Text, Weight::Regular, 14.0f, 20.0f);
inline constexpr TextStyle BodyStrong = TextStyle(Font::Text, Weight::Semibold, 14.0f, 20.0f);
inline constexpr TextStyle BodyLarge = TextStyle(Font::Text, Weight::Regular, 18.0f, 24.0f);
inline constexpr TextStyle BodyLargeStrong = TextStyle(Font::Text, Weight::Semibold, 18.0f, 24.0f);
inline constexpr TextStyle Subtitle = TextStyle(Font::Display, Weight::Semibold, 20.0f, 28.0f);
inline constexpr TextStyle Title = TextStyle(Font::Display, Weight::Semibold, 28.0f, 36.0f);
inline constexpr TextStyle TitleLarge = TextStyle(Font::Display, Weight::Semibold, 40.0f, 52.0f);
inline constexpr TextStyle Display = TextStyle(Font::Display, Weight::Semibold, 68.0f, 92.0f);
// Cascadia Mono, for text whose columns line up, like logs and codes, and at
// the caption's size.
inline constexpr TextStyle Mono = TextStyle(Font::Mono, Weight::Regular, 13.0f, 20.0f);
inline constexpr TextStyle MonoStrong = TextStyle(Font::Mono, Weight::Semibold, 13.0f, 20.0f);
inline constexpr TextStyle MonoCaption = TextStyle(Font::Mono, Weight::Regular, 12.0f, 16.0f);
// The PerfBar along the top of every window: Cascadia Mono at 14 pixels.
// Its ink, from the top of "/", 1610 of its 2048 units a size above the
// baseline, to the bottom of "y", 480 below, is 14.29 pixels tall, which the
// bar centers in the 16 above its divider, in a line as tall as the bar.
inline constexpr TextStyle PerfBarText = TextStyle(Font::Mono, Weight::Regular, 14.0f, 17.0f);
// How far Cascadia Mono's ink is below the middle of its ascent and descent,
// which a line centers, in sizes: ((1900 - 480) - (1610 - 480)) / 2 / 2048.
inline constexpr float MonoInkDrop = 0.0708f;

// How long the pointer rests on a control before its tooltip shows.
inline constexpr double TooltipDelay = 0.5;

// Motion: nothing moves or fades between states. A hover, a press, a
// selection, or a menu or dialog that opens shows whole on the frame that
// makes it. Only what must move to do its job does: the indeterminate
// progress bar and ring, whose motion says that work goes on, and a text
// field's blinking caret.
//
// An easing curve, as a cubic Bézier from (0, 0) to (1, 1) through two
// points, like XAML's KeySpline: the progress bar's and ring's, in
// Controls.h.
struct Curve {
    float x1;
    float y1;
    float x2;
    float y2;

    constexpr Curve(float x1, float y1, float x2, float y2) : x1(x1), y1(y1), x2(x2), y2(y2) {}

    // The curve's progress at time t, both from 0 to 1.
    float ease(float t) const;

    bool operator==(const Curve&) const = default;
};

struct Elevation;

// A drop shadow: black, below the surface, blurred as Windows' composition
// blurs, whose Gaussian's standard deviation is a third of the radius.
struct Shadow {
    float offset_y;
    float blur;
    float opacity_light;
    float opacity_dark;

    float sigma() const;

    bool operator==(const Shadow&) const = default;

private:
    static const Shadow None;

    friend constexpr Elevation elevation(float z);
};

inline constexpr Shadow Shadow::None{
    .offset_y = 0.0f,
    .blur = 0.0f,
    .opacity_light = 0.0f,
    .opacity_dark = 0.0f,
};

// An elevation: WinUI's ThemeShadow for a surface raised by Translation Z,
// which is two drop shadows, an ambient one and a directional one.
struct Elevation {
    float z;
    std::array<Shadow, 2> shadows;

    bool operator==(const Elevation&) const = default;
};

// ThemeShadow's recipe for a Translation Z, from WinUI's DropShadowRecipe.h:
// the elevation is half the Z, up to 64. Up to 16, there's only the
// directional shadow; above, the ambient one too, and both are stronger.
constexpr Elevation elevation(float z) {
    float level = z / 2.0f < 64.0f ? z / 2.0f : 64.0f;
    if (level < 2.0f) {
        return Elevation{
            .z = z,
            .shadows = {Shadow::None, Shadow::None},
        };
    }
    Shadow ambient = Shadow::None;
    float directional_light;
    float directional_dark;
    if (level <= 16.0f) {
        float light = level / 100.0f + 0.06f;
        directional_light = light < 0.14f ? light : 0.14f;
        directional_dark = 0.26f;
    } else {
        ambient = Shadow{
            .offset_y = 2.0f,
            .blur = level / 3.0f,
            .opacity_light = 0.15f,
            .opacity_dark = 0.37f,
        };
        directional_light = 0.19f;
        directional_dark = 0.37f;
    }
    Shadow directional{
        .offset_y = level * 0.5f,
        .blur = level,
        .opacity_light = directional_light,
        .opacity_dark = directional_dark,
    };
    return Elevation{
        .z = z,
        .shadows = {ambient, directional},
    };
}

// The Z that WinUI raises each kind of surface by.
inline constexpr Elevation ElevationTooltip = elevation(16.0f);
inline constexpr Elevation ElevationFlyout = elevation(32.0f);
inline constexpr Elevation ElevationSubmenu = elevation(40.0f);
inline constexpr Elevation ElevationDialog = elevation(128.0f);
// A card over a 3D view floats like a flyout.
inline constexpr Elevation ElevationFloatingCard = ElevationFlyout;

// How KGames lays out a program's window, in fixed pixels: the PerfBar
// along the very top, which every program has, an app bar under it, which
// is as tall as WinUI's TitleBar control, a status bar at the bottom,
// and cards between them, PagePadding from the window's edges and CardGap
// from each other.
inline constexpr float PerfBarHeight = 17.0f;
inline constexpr float AppBarHeight = 48.0f;
inline constexpr float StatusBarHeight = 32.0f;
inline constexpr float PagePadding = SpaceL;
inline constexpr float CardGap = SpaceL;
inline constexpr float CardPadding = SpaceL;
// A card's header is a control tall, so that controls fit beside its title,
// with a gap under it.
inline constexpr float CardHeaderHeight = ControlHeight;
inline constexpr float CardHeaderGap = SpaceS;
// A card's form rows: a label above a control, or beside it.
inline constexpr float FormLabelGap = SpaceXs;
inline constexpr float FormRowGap = SpaceM;

// The two supported window setups' client areas.
inline constexpr float WindowWidth = 2560.0f;
inline constexpr float MaximizedHeight = 1369.0f;
inline constexpr float FullscreenHeight = 1440.0f;

} // namespace user_interface::tokens
