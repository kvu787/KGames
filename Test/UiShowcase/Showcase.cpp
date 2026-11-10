// Tests the showcase without a window: what each widget does with input, in
// the cards' real layout, at both window heights and in both themes.

#include "Testing/Testing.h"
#include "UiShowcase/UiShowcase.h"
#include "UserInterface/UserInterface.h"

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <optional>
#include <print>
#include <ranges>
#include <string>
#include <string_view>
#include <tuple>
#include <vector>

using json::Value;
using ui_showcase::showcase::Showcase;
using user_interface::Event;
using user_interface::Gamepad;
using user_interface::GamepadButtons;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Rect;
using user_interface::slider_rail;
using user_interface::Target;
using user_interface::ThemeKind;
namespace controls = user_interface::controls;
namespace tokens = user_interface::tokens;

namespace {

Headless<Showcase> showcase() {
    return Headless<Showcase>(Showcase());
}

std::optional<Value> interface_(const Headless<Showcase>& app, std::string_view member) {
    Value state = app.ui.state();
    const Value* found = state.get(member);
    return found == nullptr ? std::nullopt : std::optional(*found);
}

// The log's newest lines, at most so many.
std::vector<std::string> newest(const Headless<Showcase>& app, size_t count) {
    std::vector<std::string> lines;
    for (const std::string& line : app.program.log_ | std::views::take(count)) {
        lines.push_back(line);
    }
    return lines;
}

} // namespace

TEST(every_widget_fits_in_the_window_at_both_heights_and_in_both_themes) {
    for (ThemeKind theme : {ThemeKind::Light, ThemeKind::Dark}) {
        for (bool fullscreen : {false, true}) {
            Headless<Showcase> app = Headless<Showcase>::with_theme(Showcase(), theme);
            app.set_fullscreen(fullscreen).frame();
            float height = fullscreen ? tokens::FullscreenHeight : tokens::MaximizedHeight;
            for (const Target& target : app.ui.targets()) {
                if (!target.listed) {
                    continue;
                }
                Rect rect = target.rect;
                CHECK(rect.x >= 0.0f && rect.y >= 0.0f && rect.right() <= 2560.0f && rect.bottom() <= height, "{} is at {}", target.name, testing::describe(rect));
            }
        }
    }
}

TEST(buttons_count_their_presses) {
    Headless<Showcase> app = showcase();
    app.click("Standard").click("Accent").click("Subtle").click("Save").click("Add").click("Add").click("Remove");
    CHECK_EQ(app.program.presses, (std::array<std::uint32_t, 4>{1, 1, 1, 1}));
    CHECK_EQ(app.program.count, 1);
    app.click("Pin");
    CHECK(app.program.pinned);
    app.click("Link");
    CHECK_EQ(app.program.link_clicks, 1);
    app.click("Disabled");
    CHECK_EQ(app.program.presses, (std::array<std::uint32_t, 4>{1, 1, 1, 1}));
}

TEST(a_press_presses_a_button_as_it_goes_down_even_if_it_slides_off) {
    Headless<Showcase> app = showcase();
    app.press("Standard");
    CHECK_EQ(app.program.presses[0], 1);
    app.move_point(Point(1200.0f, 900.0f)).release();
    CHECK_EQ(app.program.presses[0], 1);
}

TEST(the_first_checkbox_disables_the_options) {
    Headless<Showcase> app = showcase();
    app.click("Option/Show rulers");
    CHECK(app.program.options[1]);
    app.click("OptionsEnabled").click("Option/Spell check");
    CHECK(!app.program.options_enabled && !app.program.options[3]);
    CHECK(!app.is_enabled("Option/Spell check"));
}

TEST(switches_turn_tooltips_and_the_dark_theme_on_and_off) {
    Headless<Showcase> app = showcase();
    // The press's own frame switches the theme, and logs it with the press
    // and the focus it gave, newest first.
    app.press("DarkTheme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
    CHECK_EQ(newest(app, 3), (std::vector<std::string>{"Press on DarkTheme", "Focus on DarkTheme", "Dark theme"}));
    app.release();
    app.click("Tooltips");
    CHECK(!app.ui.tooltips_enabled());
    // F9 is the library's, which the theme's switch follows.
    app.key(Key::F9).key(Key::F1);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    CHECK(app.ui.tooltips_enabled());
    CHECK(std::ranges::any_of(app.program.log_, [](const std::string& line) { return line == "Light theme"; }));
    app.click("Theme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
}

// F9 and the program's own theme command in one frame are two toggles, which
// cancel: the theme is the same at the frame's end, so it doesn't change.
TEST(two_theme_toggles_in_one_frame_cancel) {
    Headless<Showcase> app = showcase();
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    // The button fires on its press, so its press frame also presses F9.
    auto about_the_theme = [](const std::string& line) {
        return line.ends_with("theme");
    };
    size_t themed = static_cast<size_t>(std::ranges::count_if(app.program.log_, about_the_theme));
    app.queue({{Event::KeyPressed{.key = Key::F9, .repeat = false}}});
    app.press("Theme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    // The button's toggle logged the theme the frame draws in, once; the
    // watcher re-reads the theme when it applies, so it adds nothing, this
    // frame or the next.
    app.release();
    std::vector<std::string> after;
    for (const std::string& line : app.program.log_) {
        if (about_the_theme(line)) {
            after.push_back(line);
        }
    }
    CHECK_EQ(after.size(), themed + 1, "{}", testing::describe(after));
    CHECK_EQ(after.front(), "Light theme");
    // Either alone still switches.
    app.queue({{Event::KeyReleased{Key::F9}}}).frame();
    app.key(Key::F9);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
    app.click("Theme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
}

// A frame can hold both a typed letter, which re-filters the list, and a
// press on a row. The press means the item the row showed, so it still
// lands there.
TEST(a_press_lands_on_the_item_its_row_showed_even_as_the_search_changes) {
    Headless<Showcase> app = showcase();
    app.click("Search");
    Point center = app.target("Items/0").center();
    app.queue({{Event::Text{U'b'}, Event::PointerMoved{center}, Event::ButtonPressed{MouseButton::Left}}}).frame();
    CHECK_EQ(app.program.search, "b", "the letter reached the search box");
    CHECK_EQ(app.program.selected, std::optional<size_t>(0), "the press selected what the row was showing");
    app.release();
}

TEST(radio_buttons_and_segments_pick_one) {
    Headless<Showcase> app = showcase();
    app.click("Size/Large").click("Alignment/Right").click("Tool/Pick");
    CHECK_EQ(std::tuple(app.program.size, app.program.alignment, app.program.tool), (std::tuple<size_t, size_t, size_t>(2, 2, 3)));
    // A segmented control is one focus stop, whose arrows move it.
    app.click("Tool/Erase");
    CHECK_EQ(interface_(app, "Focused"), std::optional(Value("Tool")));
    app.key(Key::Left);
    CHECK_EQ(app.program.tool, 1);
}

TEST(a_press_on_a_slider_sets_it_and_a_drag_moves_it_past_its_ends) {
    Headless<Showcase> app = showcase();
    Rect rect = app.target("Volume");
    auto [left, right] = slider_rail(rect);
    app.press_point(Point(left + (right - left) * 0.25f, rect.center().y));
    CHECK_EQ(app.program.volume, 25.0f);
    app.move_point(Point(2400.0f, rect.center().y)).release();
    CHECK_EQ(app.program.volume, 100.0f);
    CHECK_EQ(app.label("Volume"), std::optional<std::string>("100"));
}

TEST(the_wheel_and_the_keys_move_a_slider_by_its_steps) {
    Headless<Showcase> app = showcase();
    app.scroll("Opacity", -3.0f);
    CHECK(std::abs(app.program.opacity - 0.747) < 1e-9, "{}", app.program.opacity);
    app.click("Angle").key(Key::Home).key(Key::Right).key(Key::PageUp);
    CHECK(std::abs(app.program.angle - 15.1f) < 1e-3f, "{}", app.program.angle);
    app.key(Key::End);
    CHECK_EQ(app.program.angle, 360.0f);
}

TEST(parts_of_notches_add_up_on_a_slider) {
    Headless<Showcase> app = showcase();
    for (int turn = 0; turn < 4; turn++) {
        app.scroll("Volume", 0.25f);
    }
    CHECK_EQ(app.program.volume, 51.0f);
}

TEST(a_combo_box_opens_its_list_over_itself_and_a_press_picks_from_it) {
    Headless<Showcase> app = showcase();
    app.click("Role");
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>("Role"));
    CHECK(std::abs(app.target("Role/0").center().y - app.target("Role").center().y) < 1.0f);
    app.click("Role/3");
    CHECK_EQ(app.program.role, 3);
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>());
}

TEST(a_press_outside_a_combo_box_list_closes_it_and_does_nothing_else) {
    Headless<Showcase> app = showcase();
    app.click("Role").click("Standard");
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>());
    CHECK_EQ(app.program.presses[0], 0);
}

TEST(a_text_field_takes_typing_and_enter_greets) {
    Headless<Showcase> app = showcase();
    app.click("Name").type_text("Ada Lovelace").key_with(Modifiers::Control, Key::Left).key_with(Modifiers::Shift, Key::End).type_text("Byron");
    app.key(Key::Enter);
    CHECK_EQ(app.program.name, "Ada Byron");
    CHECK_EQ(app.program.greeting, std::optional<std::string>("Hello, Ada Byron!"));
    app.click("Greeting/Close");
    CHECK_EQ(app.program.greeting, std::optional<std::string>());
}

TEST(a_double_click_selects_a_word) {
    Headless<Showcase> app = showcase();
    app.click("Name").type_text("one two");
    app.double_click("Name").type_text("three");
    CHECK(app.program.name == "one three" || app.program.name == "three two", "{}", app.program.name);
}

TEST(the_search_shows_the_items_it_matches_and_its_button_clears_it) {
    Headless<Showcase> app = showcase();
    app.click("Search").type_text("ember");
    CHECK_EQ(app.program.matches().size(), 10);
    CHECK_EQ(app.label("Items/0"), std::optional<std::string>("Ember 1"));
    app.click("Search/Clear");
    CHECK_EQ(app.program.matches().size(), 120);
}

TEST(number_boxes_take_typing_and_their_spin_buttons) {
    Headless<Showcase> app = showcase();
    app.click("Quantity/Up").click("Quantity/Up");
    CHECK_EQ(app.program.quantity, 5);
    app.click("Scale").key_with(Modifiers::Control, Key::A).type_text("9").key(Key::Enter);
    CHECK_EQ(app.program.scale, 4.0);
    app.key(Key::Down);
    CHECK_EQ(app.program.scale, 3.75);
}

TEST(the_list_selects_scrolls_and_opens) {
    Headless<Showcase> app = showcase();
    app.click("Items/2").key(Key::Down);
    CHECK_EQ(app.program.selected, std::optional<size_t>(3));
    app.scroll("Items", -10.0f);
    CHECK(!app.has_target("Items/0"));
    std::optional<std::string> row;
    size_t skipped = 0;
    for (const Target& target : app.ui.targets()) {
        if (target.name.starts_with("Items/") && target.listed) {
            if (skipped == 2) {
                row = target.name;
                break;
            }
            skipped += 1;
        }
    }
    CHECK(row.has_value());
    app.double_click(*row);
    std::string_view number = *row;
    while (number.starts_with("Items/")) {
        number.remove_prefix(std::string_view("Items/").size());
    }
    size_t index = static_cast<size_t>(base::parse_u64(number).value());
    CHECK_EQ(app.program.opened, std::optional<size_t>(index));
}

TEST(the_caution_closes_and_comes_back) {
    Headless<Showcase> app = showcase();
    app.click("Caution/Close");
    CHECK(!app.program.caution_shown);
    app.click("BringBack");
    CHECK(app.program.caution_shown);
}

TEST(a_tooltip_shows_once_the_pointer_rests_and_sits_above_it) {
    Headless<Showcase> app = showcase();
    app.move_to("Save").wait(tokens::TooltipDelay / 2.0);
    CHECK_EQ(app.ui.tooltip_name(), std::optional<std::string_view>());
    app.wait(tokens::TooltipDelay);
    CHECK_EQ(app.ui.tooltip_name(), std::optional<std::string_view>("Save"));
    std::optional<Value> tooltip = interface_(app, "Tooltip");
    CHECK(tooltip.has_value());
    const Value::Object* members = base::as<Value::Object>(*tooltip);
    CHECK(members != nullptr);
    auto found = std::ranges::find_if(members->members, [](const std::pair<std::string, Value>& member) { return member.first == "Area"; });
    CHECK(found != members->members.end());
    const Value::Array* area = base::as<Value::Array>(found->second);
    CHECK(area != nullptr);
    const Value::Number* y = base::as<Value::Number>(area->values[1]);
    const Value::Number* height = base::as<Value::Number>(area->values[3]);
    CHECK(y != nullptr && height != nullptr);
    CHECK_EQ(static_cast<float>(y->value + height->value), app.pointer().y - 20.0f);
}

TEST(the_menu_bar_lines_up_with_the_cards_under_the_title) {
    Headless<Showcase> app = showcase();
    // The File menu's name starts on the cards' left edge.
    Rect file = app.target("Menu/File");
    CHECK_EQ(file.x + controls::MenuBarItemPadding.left, tokens::PagePadding);
    CHECK_EQ(file.y, tokens::PerfBarHeight + tokens::AppBarHeight + controls::MenuBarItemMargin.top);
    // The theme's button is at the other end of the menu bar's row, on the
    // cards' right edge.
    Rect theme = app.target("Theme");
    CHECK_EQ(theme.center().y, file.center().y);
    CHECK_EQ(theme.right(), 2560.0f - tokens::PagePadding);
}

TEST(menus_choose_commands) {
    Headless<Showcase> app = showcase();
    app.click("Menu/View").move_to("Menu/View/Menu/Theme");
    app.click("Menu/View/Menu/Theme/Menu/Dark theme");
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
    CHECK_EQ(app.program.menu_choice, std::optional<std::string>("Dark theme"));
    app.click("More").click("More/Menu/Pinned");
    CHECK(app.program.pinned);
    // The context menu opens at a right-click.
    Point area = app.target("ContextArea").center();
    app.press_button(area, MouseButton::Right).release_button(MouseButton::Right);
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>("ContextArea/ContextMenu"));
    app.click("ContextArea/ContextMenu/Copy");
    CHECK_EQ(app.program.menu_choice, std::optional<std::string>("Copy"));
}

TEST(the_dialog_asks_and_takes_the_page_until_answered) {
    Headless<Showcase> app = showcase();
    app.click("Ask");
    CHECK(app.program.dialog_open && app.ui.modal());
    app.click("Standard");
    CHECK_EQ(app.program.presses[0], 0);
    app.click("Question/Secondary");
    CHECK_EQ(app.program.dialog_result, std::optional<std::string>("Don't save"));
    CHECK(!app.program.dialog_open);
    app.click("Menu/Help").click("Menu/Help/Menu/Ask a question").key(Key::Escape);
    CHECK_EQ(app.program.dialog_result, std::optional<std::string>("Cancel"));
}

TEST(the_color_picker_and_field_and_palette_agree) {
    Headless<Showcase> app = showcase();
    app.click("Palette/3");
    CHECK_EQ(app.program.color.to_hex(), "#107C10");
    CHECK_EQ(app.label("Hex"), std::optional<std::string>("#107C10"));
    app.click("Hex").key_with(Modifiers::Control, Key::A).type_text("#FF0000").key(Key::Enter);
    CHECK_EQ(app.program.color.to_hex(), "#FF0000");
    // The spectrum's top left is red, at full saturation.
    Rect spectrum = app.target("Picker/Spectrum");
    app.click_point(Point(spectrum.x + spectrum.width / 3.0f, spectrum.y + 1.0f));
    user_interface::Color green = app.program.color;
    CHECK(green.g > 0.99f && green.r < 0.02f && green.b < 0.02f, "{}", green.to_hex());
}

TEST(the_expander_shows_and_hides) {
    Headless<Showcase> app = showcase();
    CHECK(app.program.expanded);
    app.click("Expander");
    CHECK(!app.program.expanded);
}

TEST(tab_moves_focus_through_the_widgets_and_esc_takes_it_away) {
    Headless<Showcase> app = showcase();
    for (int press = 0; press < 5; press++) {
        app.key(Key::Tab);
    }
    CHECK_EQ(interface_(app, "Focused"), std::optional(Value("Standard")));
    app.key(Key::Space);
    CHECK_EQ(app.program.presses[0], 1);
    app.key_with(Modifiers::Shift, Key::Tab);
    CHECK_EQ(interface_(app, "Focused"), std::optional(Value("Theme")));
    app.key(Key::Escape);
    CHECK_EQ(interface_(app, "Focused"), std::optional(Value()));
}

TEST(ctrl_r_resets_every_widget) {
    Headless<Showcase> app = showcase();
    app.click("Standard").click("Size/Large");
    app.key_with(Modifiers::Control, Key::R);
    CHECK_EQ(app.program.presses[0], 0);
    CHECK_EQ(app.program.size, 1);
    CHECK(app.program.log_.front() != nullptr);
    CHECK_EQ(*app.program.log_.front(), "Reset every widget");
}

TEST(f10_and_the_view_menu_switch_vertical_sync) {
    Headless<Showcase> app = showcase();
    CHECK(app.vertical_sync());
    app.key(Key::F10);
    CHECK(!app.vertical_sync());
    CHECK_EQ(interface_(app, "VerticalSync"), std::optional(Value(false)));
    app.click("Menu/View").click("Menu/View/Menu/VSync");
    CHECK(app.vertical_sync());
}

TEST(f11_switches_to_fullscreen_and_back) {
    Headless<Showcase> app = showcase();
    app.key(Key::F11);
    CHECK(app.fullscreen());
    app.key(Key::F11);
    CHECK(!app.fullscreen());
}

TEST(without_focus_nothing_responds) {
    Headless<Showcase> app = showcase();
    app.focus(false).click("Standard").key(Key::F9);
    CHECK_EQ(app.program.presses[0], 0);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    app.focus(true).click("Standard");
    CHECK_EQ(app.program.presses[0], 1);
}

TEST(the_interface_card_shows_held_keys_and_the_gamepad) {
    Headless<Showcase> app = showcase();
    app.hold(Key::W);
    CHECK(app.ui.input().key_down(Key::W));
    app.release_key(Key::W);
    Gamepad gamepad{
        .connected = true,
        .left_stick = Point(0.5f, 0.5f),
        .buttons = GamepadButtons(GamepadButtons::A),
    };
    app.set_gamepad(gamepad);
    CHECK_EQ(app.ui.gamepad().buttons.names(), std::vector<std::string_view>{"A"});
}

TEST(a_held_key_repeats) {
    Headless<Showcase> app = showcase();
    app.click("Name").type_text("abc");
    app.hold(Key::Backspace).repeat(Key::Backspace).release_key(Key::Backspace);
    CHECK_EQ(app.program.name, "a");
}

TEST(benchmark_frame_time) {
    Headless<Showcase> app = showcase();
    app.frames(100);
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    size_t frames = 1000;
    user_interface::FramePhases phases;
    for (size_t index = 0; index < frames; index++) {
        app.frame();
        user_interface::FramePhases frame = app.ui.phases();
        phases.input += frame.input;
        phases.update += frame.update;
        phases.view += frame.view;
        phases.layout += frame.layout;
        phases.draw += frame.draw;
    }
    double per_frame = std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count() * 1000.0 / static_cast<double>(frames);
    double each = static_cast<double>(frames);
    std::println(stderr, "A frame of the showcase's interface takes {:.3f} ms to build", per_frame);
    std::println(stderr, "  input {:.3f}, update {:.3f}, view {:.3f}, layout {:.3f}, draw {:.3f}", phases.input / each, phases.update / each, phases.view / each,
                 phases.layout / each, phases.draw / each);
}
