// Tests the library's parts that need no window: geometry, colors, easing,
// the design system's values, keys, how events fold into a frame's input,
// settings, the command line, and text layout.

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Base/Vec.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <tuple>
#include <variant>
#include <vector>

using user_interface::Arguments;
using user_interface::Color;
using user_interface::Event;
using user_interface::Fonts;
using user_interface::Input;
using user_interface::Insets;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Settings;
using user_interface::Shortcut;
using user_interface::Theme;
using user_interface::controls::ProgressBarCurve;
using user_interface::controls::ProgressRingCurve;
using user_interface::tokens::Body;
using user_interface::tokens::Curve;
using user_interface::tokens::elevation;
using user_interface::tokens::ElevationDialog;
using user_interface::tokens::ElevationFlyout;
using user_interface::tokens::ElevationTooltip;

TEST(rectangles_cut_into_strips_and_columns) {
    Rect area(10.0f, 20.0f, 300.0f, 200.0f);
    Rect top = area.cut_top(32.0f, 8.0f);
    CHECK_EQ(top, Rect(10.0f, 20.0f, 300.0f, 32.0f));
    CHECK_EQ(area, Rect(10.0f, 60.0f, 300.0f, 160.0f));
    Rect right = area.cut_right(100.0f, 10.0f);
    CHECK_EQ(right, Rect(210.0f, 60.0f, 100.0f, 160.0f));
    CHECK_EQ(area.width, 190.0f);
    auto [a, b, c] = Rect(0.0f, 0.0f, 320.0f, 10.0f).columns<3>(10.0f);
    CHECK_EQ(std::tuple(a.x, a.width), std::tuple(0.0f, 100.0f));
    CHECK_EQ(std::tuple(b.x, c.x), std::tuple(110.0f, 220.0f));
    auto [first, second] = Rect(0.0f, 0.0f, 10.0f, 100.0f).rows<2>(10.0f);
    CHECK_EQ(std::tuple(first.height, second.y), std::tuple(45.0f, 55.0f));
    std::vector<Rect> weighted = Rect(0.0f, 0.0f, 310.0f, 10.0f).columns_weighted({{1.0f, 2.0f}}, 10.0f);
    CHECK_EQ(std::tuple(weighted[0].width, weighted[1].right()), std::tuple(100.0f, 310.0f));
    CHECK(Rect(0.0f, 0.0f, 10.0f, 10.0f).contains(Point(0.0f, 9.9f)));
    CHECK(!Rect(0.0f, 0.0f, 10.0f, 10.0f).contains(Point(10.0f, 5.0f)));
    CHECK(Rect(0.0f, 0.0f, 10.0f, 10.0f).intersect(Rect(20.0f, 0.0f, 5.0f, 5.0f)).is_empty());
    CHECK_EQ(Rect(0.0f, 0.0f, 100.0f, 40.0f).pad(Insets(11.0f, 5.0f, 11.0f, 6.0f)), Rect(11.0f, 5.0f, 78.0f, 29.0f));
    CHECK_EQ(Rect(10.0f, 10.0f, 100.0f, 20.0f).at_fraction(Point(0.25f, 0.5f)), Point(35.0f, 20.0f));
}

TEST(colors_parse_and_blend_like_xaml_writes_them) {
    CHECK_EQ(Color::parse("#1CDA61"), std::optional(Color::hex(0x1CDA61)));
    CHECK_EQ(Color::parse("#E4000000"), std::optional(Color::argb(0xE4000000)));
    CHECK_EQ(Color::parse("1CDA61"), std::optional(Color::hex(0x1CDA61)));
    CHECK_EQ(Color::parse("#12345"), std::optional<Color>());
    CHECK_EQ(Color::hex(0x1CDA61).to_hex(), "#1CDA61");
    CHECK_EQ(Color::argb(0x80000000).to_hex(), "#80000000");
    Color half_black = Color::argb(0x80000000).over(Color::White);
    CHECK(std::abs(half_black.r - 0.498f) < 0.01f && half_black.a == 1.0f);
    // Premultiplied for the GPU: a half-transparent white is half bright.
    CHECK_EQ(Color::rgba(1.0f, 1.0f, 1.0f, 0.5f).pack_premultiplied(), 0x80808080);
    auto [r, g, b] = Color::from_linear(0.5f, 0.0f, 1.0f).to_linear();
    CHECK(std::abs(r - 0.5f) < 0.01f && g == 0.0f && std::abs(b - 1.0f) < 1e-6f);
    // Hue, saturation and value, and back.
    Color color = Color::hex(0x1E90FF);
    auto [hue, saturation, value] = color.to_hsv();
    CHECK(std::abs(hue - 209.6f) < 0.1f && std::abs(saturation - 0.882f) < 0.01f && value == 1.0f);
    CHECK_EQ(Color::from_hsv(hue, saturation, value).to_hex(), "#1E90FF");
    CHECK_EQ(Color::from_hsv(120.0f, 1.0f, 1.0f).to_hex(), "#00FF00");
}

TEST(curves_go_from_0_to_1) {
    Curve linear(0.0f, 0.0f, 1.0f, 1.0f);
    for (Curve curve : {linear, ProgressBarCurve, ProgressRingCurve}) {
        CHECK(std::abs(curve.ease(0.0f)) < 1e-4f);
        CHECK(std::abs(curve.ease(1.0f) - 1.0f) < 1e-4f);
        // Each is symmetric, so it's half way at half the time.
        CHECK(std::abs(curve.ease(0.5f) - 0.5f) < 0.01f);
    }
    CHECK(std::abs(linear.ease(0.25f) - 0.25f) < 0.01f);
    // The indeterminate bar's indicators start slowly.
    CHECK(ProgressBarCurve.ease(0.1f) < 0.05f);
}

TEST(the_themes_have_winui_3s_values) {
    Theme light = Theme::light();
    Theme dark = Theme::dark();
    // AccentFillColorDefault is Windows' default blue's Dark1 in the light
    // theme and Light2 in the dark one.
    CHECK_EQ(light.accent.to_hex(), "#0067C0");
    CHECK_EQ(dark.accent.to_hex(), "#4CC2FF");
    CHECK_EQ(light.accent_secondary.to_hex(), "#E60067C0");
    CHECK_EQ(light.text_primary.to_hex(), "#E4000000");
    CHECK_EQ(dark.text_secondary.to_hex(), "#C5FFFFFF");
    CHECK_EQ(light.control_fill.to_hex(), "#B3FFFFFF");
    CHECK_EQ(light.control_stroke_secondary.to_hex(), "#29000000");
    CHECK_EQ(light.background.to_hex(), "#F3F3F3");
    CHECK_EQ(dark.background.to_hex(), "#202020");
    CHECK_EQ(light.acrylic.to_hex(), "#F9F9F9");
    CHECK_EQ(light.accent_selected_text_background.to_hex(), "#0078D4");
}

TEST(elevations_follow_themeshadows_recipe) {
    // Up to 16, only the directional shadow: half the Z blurs, down a
    // quarter of the Z.
    CHECK_EQ(ElevationTooltip.shadows[0].opacity_light, 0.0f);
    CHECK_EQ(std::tuple(ElevationTooltip.shadows[1].blur, ElevationTooltip.shadows[1].offset_y), std::tuple(8.0f, 4.0f));
    CHECK_EQ(ElevationTooltip.shadows[1].opacity_light, 0.14f);
    CHECK_EQ(std::tuple(ElevationFlyout.shadows[1].blur, ElevationFlyout.shadows[1].offset_y), std::tuple(16.0f, 8.0f));
    CHECK_EQ(ElevationFlyout.shadows[1].opacity_dark, 0.26f);
    // Above, an ambient one too, and both stronger.
    CHECK_EQ(ElevationDialog.shadows[0].opacity_light, 0.15f);
    CHECK_EQ(std::tuple(ElevationDialog.shadows[1].blur, ElevationDialog.shadows[1].offset_y), std::tuple(64.0f, 32.0f));
    CHECK_EQ(ElevationDialog.shadows[1].opacity_dark, 0.37f);
    CHECK(std::ranges::all_of(elevation(2.0f).shadows, [](const user_interface::tokens::Shadow& shadow) { return shadow.opacity_light == 0.0f; }));
}

TEST(keys_have_names_that_parse_back) {
    for (Key key : {Key::A, Key::F11, Key::Left, Key::Space, Key::Enter, Key::Escape, Key('7'), Key::PageDown, Key::Delete}) {
        CHECK_EQ(Key::parse(key.name()), std::optional(key), "{}", key.name());
    }
    CHECK_EQ(Key::parse("f11"), std::optional(Key::F11));
    CHECK_EQ(Key::parse("Nope"), std::optional<Key>());
    CHECK_EQ(Shortcut::parse("Ctrl+S"), std::optional(Shortcut(Modifiers::Control, Key::S)));
    CHECK_EQ(Shortcut::parse("Ctrl+Shift+Z"), std::optional(Shortcut(Modifiers::ControlShift, Key::Z)));
    CHECK_EQ(Shortcut::parse("Hyper+S"), std::optional<Shortcut>());
    CHECK_EQ(Shortcut(Modifiers::ControlShift, Key::S).name(), "Ctrl+Shift+S");
}

TEST(events_fold_into_a_frame_of_input) {
    Input input;
    input.begin(0.0, {{Event::FocusGained{}, Event::PointerMoved{Point(5.0f, 5.0f)}, Event::ButtonPressed{MouseButton::Left}}});
    CHECK(input.button_pressed(MouseButton::Left) && input.button_down(MouseButton::Left));
    CHECK_EQ(input.clicks(MouseButton::Left), 1);
    input.begin(0.1, {{Event::ButtonReleased{MouseButton::Left}}});
    CHECK(input.button_released(MouseButton::Left) && !input.button_down(MouseButton::Left));
    // A second press, soon and close by, is a double-click.
    input.begin(0.2, {{Event::ButtonPressed{MouseButton::Left}}});
    CHECK_EQ(input.clicks(MouseButton::Left), 2);
    // Without focus there's no input, and losing it lets go of everything.
    input.begin(0.3, {{Event::KeyPressed{.key = Key::W, .repeat = false}, Event::FocusLost{}, Event::KeyPressed{.key = Key::A, .repeat = false}}});
    CHECK(input.focus_lost && !input.key_down(Key::W) && !input.key_down(Key::A) && !input.button_down(MouseButton::Left));
    // Keys held when focus comes count once they're let go of.
    input.begin(0.4, {{Event::FocusGained{{Key::Alt}}, Event::KeyPressed{.key = Key::Alt, .repeat = true}}});
    CHECK(!input.key_down(Key::Alt));
    input.begin(0.5, {{Event::KeyReleased{Key::Alt}, Event::KeyPressed{.key = Key::Alt, .repeat = false}}});
    CHECK(input.key_down(Key::Alt));
    input.begin(0.6, {{Event::KeyPressed{.key = Key::Control, .repeat = false}, Event::KeyPressed{.key = Key::S, .repeat = false}}});
    CHECK(input.shortcut_pressed(Shortcut(Modifiers(true, false, true), Key::S)));
}

TEST(settings_keep_their_defaults_when_missing_or_wrong) {
    Settings settings = Settings::from_text(R"({ "Volume": 3, "Mode": "fast", "Name": 5 })");
    CHECK_EQ(settings.number("Volume", 1.0,
                             [](double value) -> std::expected<void, std::string> {
                                 if (value <= 2.0) {
                                     return {};
                                 }
                                 return std::unexpected("must be at most 2");
                             }),
             1.0);
    CHECK_EQ(settings.choice("Mode", "Slow", {{"Slow", "Fast"}}), "Fast");
    CHECK_EQ(settings.string("Name", "Ada"), "Ada");
    CHECK(settings.boolean("Missing", true));
    CHECK_EQ(std::vector<std::string>(settings.problems().begin(), settings.problems().end()),
             (std::vector<std::string>{"Volume must be at most 2, so it's the default.", "Name must be text, not 5, so it's the default."}));
    Settings broken = Settings::from_text("{ \"A\": ");
    CHECK_EQ(broken.problems().size(), 1);
}

TEST(a_missing_settings_file_is_written_with_the_defaults) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / std::format("UserInterfaceTests{}", GetCurrentProcessId());
    CHECK(base::create_dir_all(folder).has_value());
    std::filesystem::path path = folder / "Settings.json";
    std::error_code ignored;
    std::filesystem::remove(path, ignored);
    Settings settings = Settings::load(path);
    settings.boolean("AllowTearing", true);
    settings.number("DeadZone", 0.1, [](double) -> std::expected<void, std::string> { return {}; });
    CHECK(settings.write_if_missing().has_value());
    std::string written = base::read_to_string(path).value();
    CHECK(written.contains("\"AllowTearing\": true") && written.contains("\"DeadZone\": 0.1"), "{}", written);
    std::filesystem::remove_all(folder);
}

TEST(vsync_arguments_override_the_setting_only_when_given) {
    auto parse = [](std::vector<std::string> arguments) {
        return Arguments::parse(std::move(arguments), {}).value().vsync;
    };
    CHECK_EQ(parse({}), std::optional<bool>());
    CHECK_EQ(parse({"--vsync"}), std::optional<bool>(true));
    CHECK_EQ(parse({"--no-vsync"}), std::optional<bool>(false));
    CHECK_EQ(parse({"--no-vsync", "--vsync"}), std::optional<bool>(true));
}

TEST(text_lays_out_with_directwrite) {
    Fonts fonts;
    user_interface::Size one = fonts.measure("Hello", Body);
    user_interface::Size two = fonts.measure("Hello Hello", Body);
    CHECK(one.width > 20.0f && two.width > one.width * 1.8f);
    CHECK_EQ(one.height, Body.line_height);
    user_interface::Size wrapped = fonts.measure_wrapped("one two three four five six seven", Body, 60.0f);
    CHECK(wrapped.height >= Body.line_height * 3.0f);
    // A caret goes between characters, and a hit finds the nearest one.
    const user_interface::TextLayout& layout = fonts.layout("Hello", Body, std::nullopt);
    CHECK_EQ(layout.caret_x(0), 0.0f);
    float end = layout.caret_x(5);
    CHECK(std::abs(end - one.width) < 0.5f);
    CHECK_EQ(layout.hit(Point(end + 20.0f, 5.0f)), 5);
    CHECK_EQ(layout.hit(Point(-5.0f, 5.0f)), 0);
    CHECK_EQ(fonts.family(user_interface::Font::Text), "Segoe UI Variable Text");
}

// Marks that aren't the layer's own, in order, stop the program, as the
// bounds of the layer's slice do in Rust, where unchecked they'd move what's
// outside the layer.
TEST(moving_instances_to_the_back_checks_its_marks) {
    using Bounds = std::array<float, 4>;
    Fonts fonts;
    Theme theme = Theme::light();
    user_interface::DrawList list;
    list.clear(user_interface::Size(100.0f, 100.0f));
    user_interface::Painter painter{list, fonts, theme};
    painter.fill(Rect(0.0f, 0.0f, 10.0f, 10.0f), theme.accent);
    painter.fill(Rect(20.0f, 0.0f, 10.0f, 10.0f), theme.accent);
    painter.fill(Rect(40.0f, 0.0f, 10.0f, 10.0f), theme.accent);
    // The last instance goes under the two before it.
    list.move_to_back(0, 2);
    std::span<const user_interface::Instance> instances = list.layer_instances(user_interface::Layer::Base);
    CHECK_EQ(instances.size(), 3);
    CHECK_EQ(std::tuple(instances[0].bounds, instances[1].bounds, instances[2].bounds),
             std::tuple(Bounds{40.0f, 0.0f, 50.0f, 10.0f}, Bounds{0.0f, 0.0f, 10.0f, 10.0f}, Bounds{20.0f, 0.0f, 30.0f, 10.0f}));
    // A mark past the layer's end, and marks out of order.
    CHECK_THROWS(list.move_to_back(0, 4));
    CHECK_THROWS(list.move_to_back(3, 2));
    CHECK_EQ(list.layer_len(), 3);
}

// The README's example program, which must keep working as it says.
namespace readme {

using user_interface::AppBar;
using user_interface::Button;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Context;
using user_interface::Headless;
using user_interface::Icon;
using user_interface::Program;
using user_interface::Shell;
using user_interface::Slider;
using user_interface::Status;
using user_interface::Tree;
using user_interface::Ui;

struct Message {
    struct Level {
        double level;
    };
    struct Mute {};
    std::variant<Level, Mute> value;
    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class Volume : public Program<Message> {
public:
    double level = 0.0;

    std::string title() const {
        return "Volume";
    }

    void update(Message message, Context<Message>&) {
        base::match(message, [&](const Message::Level& changed) { level = changed.level; }, [&](const Message::Mute&) { level = 0.0; });
    }

    Tree<Message> view(Ui& ui) const {
        Tree<Message> tree;
        Shell shell = tree.shell(ui, AppBar(Icon::Volume, "Volume"), false, base::vec(Status::shortcut("F11", "Fullscreen")));
        tree.card(ui, std::nullopt, shell.page.left_strip(400.0f).top_strip(200.0f), Card("Output"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
            Rect area = card_area.content;
            Slider<Message> slider = Slider<Message>(level, 0.0, 100.0).step(1.0).header("Level").show_value(0).on_change(base::make<Message::Level>);
            card.add("Level", area.cut_top(Slider<Message>::height(true), 8.0f), std::move(slider));
            card.add("Mute", area.cut_top(32.0f, 0.0f).left_strip(120.0f), Button<Message>("Mute").on_press(Message::Mute{}));
        });
        return tree;
    }
};

TEST(the_readme_s_example_works_as_it_says) {
    Headless app(Volume{.level = 50.0});
    app.frame();
    app.click("Mute");
    CHECK_EQ(app.program.level, 0.0);
    app.key(Key::F11);
    CHECK(app.fullscreen());
}

} // namespace readme
