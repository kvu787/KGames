// Draws frames offscreen with the GPU, without a window, and checks their
// pixels: the theme's background, a card, an accent button's fill and label,
// a standard button's elevation border, that widgets draw a new state whole
// on the frame that makes it, that a press draws all it did on its own frame,
// that a selection that moves leaves its old item on that frame, that a
// release shows on its own frame, whether the PerfBar says that vertical sync
// is on, the PerfBar's colors in each theme, and the page for an unsupported
// setup.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Base/Vec.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdlib>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::Image;
using graphics::write_png;
using user_interface::Button;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::CheckBox;
using user_interface::Color;
using user_interface::ComboBox;
using user_interface::Context;
using user_interface::Event;
using user_interface::Headless;
using user_interface::Key;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::MenuBar;
using user_interface::MenuItem;
using user_interface::MouseButton;
using user_interface::Palette;
using user_interface::Program;
using user_interface::RadioGroup;
using user_interface::Rect;
using user_interface::Segmented;
using user_interface::SelectorBar;
using user_interface::Slider;
using user_interface::Text;
using user_interface::Theme;
using user_interface::ThemeKind;
using user_interface::ToggleSwitch;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::unsupported_setup;
using user_interface::controls::MenuBarHeight;
using user_interface::tokens::PerfBarHeight;

namespace rendering_tests {

using Pixel = std::array<std::uint8_t, 4>;

class Page : public Program<std::monostate> {
public:
    std::string title() const {
        return "Page";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        tree.card(ui, std::nullopt, Rect(40.0f, 40.0f, 400.0f, 200.0f), Card("A card"), [&](Ui&, Tree<std::monostate>& card, CardArea area) {
            auto [accent, standard] = area.content.top_strip(32.0f).columns<2>(8.0f);
            card.add("Accent", accent, Button<std::monostate>("Accent").accent());
            card.add("Standard", standard, Button<std::monostate>("Standard"));
        });
        return tree;
    }
};

bool close(Pixel a, Pixel b) {
    for (size_t index = 0; index < 3; index++) {
        if (std::abs(static_cast<std::int32_t>(a[index]) - static_cast<std::int32_t>(b[index])) > 2) {
            return false;
        }
    }
    return true;
}

void save(const Image& image, std::string_view name) {
    std::filesystem::path folder = std::filesystem::path(folders::Repository) / "target" / "TestOutput";
    CHECK(base::create_dir_all(folder).has_value());
    write_png(folder / std::format("{}.png", name), image);
}

// The row of the frame's pixels that the PerfBar ends at, and the rest.
std::span<const std::uint8_t> below_the_perf_bar(const Image& image) {
    size_t below = static_cast<size_t>(base::saturating_cast<std::uint32_t>(PerfBarHeight) * image.width * 4);
    return std::span(image.pixels).subspan(below);
}

TEST(a_frame_draws_the_background_the_widgets_and_their_text) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    for (ThemeKind kind : {ThemeKind::Light, ThemeKind::Dark}) {
        Headless<Page> app = Headless<Page>::with_theme(Page{}, kind);
        Image image = app.render(gpu);
        Theme theme(kind);
        save(image, std::format("Page{}", user_interface::name(kind)));
        CHECK_EQ(std::tuple(image.width, image.height), std::tuple(2560u, 1369u));
        CHECK(close(image.pixel(1000, 1000), theme.background.to_bytes()), "{}", testing::describe(image.pixel(1000, 1000)));
        // The card is its fill over the background.
        CHECK(close(image.pixel(400, 220), theme.card_opaque().to_bytes()), "{}", testing::describe(image.pixel(400, 220)));
        // The accent button's fill, between its label and its edge.
        Rect button = app.target("Accent");
        Pixel fill = image.pixel(base::saturating_cast<std::uint32_t>(button.x) + 6, base::saturating_cast<std::uint32_t>(button.center().y));
        CHECK(close(fill, theme.accent.over(theme.card_opaque()).to_bytes()), "{}", testing::describe(fill));
        // Its label is in text_on_accent: some pixel in its middle is.
        bool found = false;
        for (std::uint32_t x = base::saturating_cast<std::uint32_t>(button.x) + 40; x < base::saturating_cast<std::uint32_t>(button.right()) - 40; x++) {
            for (std::uint32_t y = base::saturating_cast<std::uint32_t>(button.y) + 8; y < base::saturating_cast<std::uint32_t>(button.bottom()) - 8; y++) {
                found |= close(image.pixel(x, y), theme.text_on_accent.to_bytes());
            }
        }
        CHECK(found, "the button's label didn't draw");
        // A standard button's elevation border is darker on one edge than
        // the other: the bottom in the light theme, the top in the dark one.
        Rect standard = app.target("Standard");
        std::uint32_t x = base::saturating_cast<std::uint32_t>(standard.center().x) + 40;
        Pixel top = image.pixel(x, base::saturating_cast<std::uint32_t>(standard.y));
        Pixel bottom = image.pixel(x, base::saturating_cast<std::uint32_t>(standard.bottom()) - 1);
        auto [stronger, weaker] = kind == ThemeKind::Light ? std::pair(bottom, top) : std::pair(top, bottom);
        auto brightness = [](Pixel pixel) {
            return static_cast<std::int32_t>(pixel[0]) + static_cast<std::int32_t>(pixel[1]) + static_cast<std::int32_t>(pixel[2]);
        };
        switch (kind) {
        case ThemeKind::Light:
            CHECK(brightness(stronger) < brightness(weaker), "{} {}", testing::describe(top), testing::describe(bottom));
            break;
        case ThemeKind::Dark:
            CHECK(brightness(stronger) > brightness(weaker), "{} {}", testing::describe(top), testing::describe(bottom));
            break;
        }
    }
}

struct Msg {
    struct On {
        bool on;
        bool operator==(const On&) const = default;
    };
    struct Checked {
        bool checked;
        bool operator==(const Checked&) const = default;
    };
    struct Size {
        size_t size;
        bool operator==(const Size&) const = default;
    };
    struct Level {
        double level;
        bool operator==(const Level&) const = default;
    };
    struct Alignment {
        size_t alignment;
        bool operator==(const Alignment&) const = default;
    };
    struct Tab {
        size_t tab;
        bool operator==(const Tab&) const = default;
    };
    struct Pick {
        size_t pick;
        bool operator==(const Pick&) const = default;
    };
    struct Row {
        size_t row;
        bool operator==(const Row&) const = default;
    };
    struct Color {
        size_t color;
        bool operator==(const Color&) const = default;
    };
    std::variant<On, Checked, Size, Level, Alignment, Tab, Pick, Row, Color> value;
    template <base::AlternativeOf<Msg> T> Msg(T&& alternative) : value(std::forward<T>(alternative)) {}
    bool operator==(const Msg&) const = default;
};

constexpr std::array<std::string_view, 3> Sizes{"Small", "Medium", "Large"};
constexpr std::array<Color, 4> Colors{Color::hex(0xD13438), Color::hex(0x107C10), Color::hex(0x0078D4), Color::hex(0x8764B8)};

std::vector<MenuBar<Msg>::Menu> menus() {
    std::vector<MenuItem<Msg>> recent = base::vec(MenuItem<Msg>("First"), MenuItem<Msg>("Second"));
    return base::vec(MenuBar<Msg>::Menu("File", base::vec(MenuItem<Msg>("Open"), MenuItem<Msg>::submenu("Recent", std::move(recent)))));
}

// A widget of each kind whose look follows the input, and a menu bar with a
// submenu.
class States : public Program<Msg> {
public:
    bool on = false;
    bool checked = false;
    size_t size = 0;
    double level = 0.0;
    size_t alignment = 0;
    size_t tab = 0;
    size_t pick = 0;
    std::optional<size_t> row;
    std::optional<size_t> color;

    std::string title() const {
        return "States";
    }

    void update(Msg message, Context<Msg>&) {
        base::match(
            message, [&](const Msg::On& changed) { on = changed.on; }, [&](const Msg::Checked& changed) { checked = changed.checked; },
            [&](const Msg::Size& changed) { size = changed.size; }, [&](const Msg::Level& changed) { level = changed.level; },
            [&](const Msg::Alignment& changed) { alignment = changed.alignment; }, [&](const Msg::Tab& changed) { tab = changed.tab; },
            [&](const Msg::Pick& changed) { pick = changed.pick; }, [&](const Msg::Row& changed) { row = changed.row; }, [&](const Msg::Color& changed) { color = changed.color; });
    }

    Tree<Msg> view(Ui& ui) const {
        Tree<Msg> tree;
        float top = ui.content_top();
        tree.add("Menus", Rect(40.0f, top + 8.0f, 200.0f, MenuBarHeight), MenuBar<Msg>(menus()));
        tree.card(ui, std::nullopt, Rect(40.0f, top + 300.0f, 480.0f, 800.0f), Card("States"), [&](Ui& ui, Tree<Msg>& card, CardArea card_area) {
            Rect area = card_area.content;
            card.add("Button", area.cut_top(32.0f, 8.0f), Button<Msg>("Button"));
            card.add("Switch", area.cut_top(32.0f, 8.0f), ToggleSwitch<Msg>(on, "Switch").on_change(base::make<Msg::On>));
            card.add("Check", area.cut_top(32.0f, 8.0f), CheckBox<Msg>(checked, "Check").on_change(base::make<Msg::Checked>));
            card.add("Size", area.cut_top(RadioGroup<Msg>::height(Sizes.size()), 8.0f), RadioGroup<Msg>(size, Sizes).on_select(base::make<Msg::Size>));
            card.add("Level", area.cut_top(32.0f, 8.0f), Slider<Msg>(level, 0.0, 1.0).on_change(base::make<Msg::Level>));
            card.add("Alignment", area.cut_top(32.0f, 8.0f), Segmented<Msg>(alignment, {{"Left", "Center", "Right"}}).on_select(base::make<Msg::Alignment>));
            card.add("Tabs", area.cut_top(48.0f, 8.0f), SelectorBar<Msg>(tab, {{"First", "Second", "Third"}}).on_select(base::make<Msg::Tab>));
            card.add("Pick", area.cut_top(32.0f, 8.0f), ComboBox<Msg>(pick, {{"One", "Two", "Three"}}).on_select(base::make<Msg::Pick>));
            // As tall as its rows, so that it has no scroll bar, whose thumb
            // waits for the pointer to rest on it before it widens.
            Rect rows = area.cut_top(120.0f, 8.0f);
            ListView<Msg> list = ListView<Msg>(3, 40.0f).selection(row).on_select(base::make<Msg::Row>);
            ListRows shown = list.rows(ui, "Rows", rows);
            card.container("Rows", rows, std::move(list), [&](Tree<Msg>& list) {
                for (size_t index : shown.indices()) {
                    list.draw(shown.content(index), Text::label(std::format("Row {}", index)));
                }
            });
            card.add("Colors", area.cut_top(32.0f, 8.0f), Palette<Msg>(color, Colors).on_pick(base::make<Msg::Color>));
        });
        return tree;
    }
};

TEST(widgets_draw_a_new_state_whole_on_the_frame_that_makes_it) {
    // Each step is input that changes how a widget looks, and after which
    // nothing changes. The frame that reads the input is drawn, and then
    // the frame a second later, which must be the same below the PerfBar,
    // whose numbers do change. A widget that eased, faded or slid to its
    // new state, even over WinUI's fastest 83 ms, would be less than half
    // way there in the first.
    using Step = void (*)(Headless<States>&);
    std::array<std::pair<std::string_view, Step>, 19> steps{{
        {"the pointer onto a button", [](Headless<States>& app) { app.move_to("Button"); }},
        {"a press of the button", [](Headless<States>& app) { app.press("Button"); }},
        {"its release", [](Headless<States>& app) { app.release(); }},
        {"a press of a toggle switch, which turns it on", [](Headless<States>& app) { app.press("Switch"); }},
        {"its release", [](Headless<States>& app) { app.release(); }},
        {"a click of a checkbox", [](Headless<States>& app) { app.click("Check"); }},
        {"a click of a radio button", [](Headless<States>& app) { app.click("Size/Large"); }},
        {"a press of a slider", [](Headless<States>& app) { app.press("Level"); }},
        {"its release", [](Headless<States>& app) { app.release(); }},
        {"a click of a segment", [](Headless<States>& app) { app.click("Alignment/Right"); }},
        {"a click of a tab", [](Headless<States>& app) { app.click("Tabs/Third"); }},
        {"a click of a list's row", [](Headless<States>& app) { app.click("Rows/2"); }},
        {"a combo box's list opening", [](Headless<States>& app) { app.click("Pick"); }},
        {"the pointer onto its item", [](Headless<States>& app) { app.move_to("Pick/1"); }},
        {"choosing the combo box's item", [](Headless<States>& app) { app.click("Pick/1"); }},
        {"a menu opening", [](Headless<States>& app) { app.click("Menus/File"); }},
        {"its submenu opening", [](Headless<States>& app) { app.move_to("Menus/File/Menu/Recent"); }},
        {"the pointer onto the submenu's item", [](Headless<States>& app) { app.move_to("Menus/File/Menu/Recent/Menu/First"); }},
        {"choosing the submenu's item", [](Headless<States>& app) { app.click("Menus/File/Menu/Recent/Menu/First"); }},
    }};
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(States{});
    app.frame();
    for (size_t index = 0; index < steps.size(); index++) {
        auto [step, act] = steps[index];
        act(app);
        Image now = app.render(gpu);
        app.wait(1.0);
        Image later = app.render(gpu);
        if (std::ranges::equal(below_the_perf_bar(now), below_the_perf_bar(later))) {
            continue;
        }
        save(now, std::format("States{}Now", index));
        save(later, std::format("States{}Later", index));
        for (std::uint32_t y = base::saturating_cast<std::uint32_t>(PerfBarHeight); y < now.height; y++) {
            for (std::uint32_t x = 0; x < now.width; x++) {
                if (now.pixel(x, y) != later.pixel(x, y)) {
                    testing::fail(
                        std::format("after {}, ({}, {}) was {} and a second later {}", step, x, y, testing::describe(now.pixel(x, y)), testing::describe(later.pixel(x, y))));
                }
            }
        }
        base::unreachable();
    }
    // The steps did what they say.
    const States& program = app.program;
    CHECK(program.on && program.checked);
    CHECK_EQ(std::tuple(program.size, program.alignment, program.tab, program.pick, program.row),
             (std::tuple<size_t, size_t, size_t, size_t, std::optional<size_t>>(2, 2, 2, 1, 2)));
}

TEST(a_press_draws_all_it_did_on_its_own_frame) {
    // A press acts as the button goes down: the frame that reads it draws
    // the switch on, the box checked, the item selected or the menu open,
    // and is the same a second later, with the button still held.
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(States{});
    app.frame();
    for (std::string_view name : {"Switch", "Check", "Size/Medium", "Alignment/Center", "Tabs/Second", "Rows/1", "Colors/1", "Menus/File"}) {
        app.press(name);
        Image now = app.render(gpu);
        app.wait(1.0);
        Image later = app.render(gpu);
        if (!std::ranges::equal(below_the_perf_bar(now), below_the_perf_bar(later))) {
            save(now, "PressNow");
            save(later, "PressLater");
            testing::fail(std::format("the press of {} didn't draw all it did on its own frame", name));
        }
        app.release();
    }
    const States& program = app.program;
    CHECK(program.on && program.checked);
    CHECK_EQ(std::tuple(program.size, program.alignment, program.tab, program.row, program.color),
             (std::tuple<size_t, size_t, size_t, std::optional<size_t>, std::optional<size_t>>(1, 1, 1, 1, 1)));
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>("Menus/File/Menu"));
}

// The first pixel in the area that differs between the two frames.
std::optional<std::pair<std::uint32_t, std::uint32_t>> first_difference(const Image& now, const Image& later, Rect area) {
    for (std::uint32_t y = base::saturating_cast<std::uint32_t>(area.y); y < base::saturating_cast<std::uint32_t>(area.bottom()); y++) {
        for (std::uint32_t x = base::saturating_cast<std::uint32_t>(area.x); x < base::saturating_cast<std::uint32_t>(area.right()); x++) {
            if (now.pixel(x, y) != later.pixel(x, y)) {
                return std::pair(x, y);
            }
        }
    }
    return std::nullopt;
}

TEST(a_selection_leaves_its_old_item_on_the_frame_that_moves_it) {
    // A press selects a list's row, a radio button, a segment, a tab or a
    // swatch. The frame that reads the input applies its message and then
    // describes every item from the model, so even when the selection moves
    // to an item after the old one, that very frame draws the old item
    // without it, the same as a second later.
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(States{});
    app.frame();
    std::array<std::pair<std::string_view, std::string_view>, 5> moves{{
        {"Rows/0", "Rows/2"},
        {"Size/Small", "Size/Large"},
        {"Alignment/Left", "Alignment/Right"},
        {"Tabs/First", "Tabs/Third"},
        {"Colors/0", "Colors/2"},
    }};
    for (auto [old, new_] : moves) {
        app.click(old).press(new_);
        Image now = app.render(gpu);
        app.wait(1.0);
        Image later = app.render(gpu);
        app.release();
        if (std::optional<std::pair<std::uint32_t, std::uint32_t>> differing = first_difference(now, later, app.target(old))) {
            save(now, "SelectionNow");
            save(later, "SelectionLater");
            testing::fail(std::format("{} still looked selected on the frame that selected {}, at ({}, {})", old, new_, differing->first, differing->second));
        }
    }
    const States& program = app.program;
    CHECK_EQ(std::tuple(program.row, program.size, program.alignment, program.tab, program.color),
             (std::tuple<std::optional<size_t>, size_t, size_t, size_t, std::optional<size_t>>(2, 2, 2, 2, 2)));
}

TEST(a_release_shows_on_the_frame_that_reads_it) {
    // A widget that the left button holds down looks pressed until the
    // frame that reads the release, which draws it as it looks from then
    // on, with what its press did, like a switch that's flipped or a menu
    // that's open, and not pressed for a frame more.
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(States{});
    app.frame();
    for (std::string_view name : {"Button", "Switch", "Check", "Size/Medium", "Level", "Alignment/Center", "Tabs/Second", "Rows/1", "Colors/1", "Menus/File"}) {
        app.press(name).queue({{Event::ButtonReleased{MouseButton::Left}}}).frame();
        Image now = app.render(gpu);
        app.wait(1.0);
        Image later = app.render(gpu);
        if (std::optional<std::pair<std::uint32_t, std::uint32_t>> differing = first_difference(now, later, app.target(name))) {
            save(now, "ReleaseNow");
            save(later, "ReleaseLater");
            testing::fail(std::format("{} didn't look let go on the frame that let go of it, at ({}, {})", name, differing->first, differing->second));
        }
    }
    const States& program = app.program;
    CHECK(program.on && program.checked);
    CHECK_EQ(std::tuple(program.size, program.alignment, program.tab, program.row, program.color),
             (std::tuple<size_t, size_t, size_t, std::optional<size_t>, std::optional<size_t>>(1, 1, 1, 1, 1)));
    CHECK_EQ(app.ui.popup_name(), std::optional<std::string_view>("Menus/File/Menu"));
}

TEST(the_perf_bar_says_whether_vertical_sync_is_on) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(Page{});
    // A second of frames, so that the PerfBar has a frame rate to show.
    app.wait(1.0);
    Image on = app.render(gpu);
    app.key(Key::F10);
    Image off = app.render(gpu);
    save(on, "VerticalSyncOn");
    save(off, "VerticalSyncOff");
    // "VSync = on" turns to "VSync = off" right after the other stats, which
    // start at the bar's left, so in its left half, and nothing else along
    // the bar changes: only "n " to "ff", two characters of Cascadia Mono, a
    // few pixels over 8 each.
    std::vector<std::uint32_t> changed;
    for (std::uint32_t x = 0; x < off.width; x++) {
        for (std::uint32_t y = 0; y < base::saturating_cast<std::uint32_t>(PerfBarHeight); y++) {
            if (off.pixel(x, y) != on.pixel(x, y)) {
                changed.push_back(x);
                break;
            }
        }
    }
    CHECK(!changed.empty(), "the PerfBar didn't change");
    std::uint32_t first = changed.front();
    std::uint32_t last = changed.back();
    CHECK(last - first < 30, "{}..={}", first, last);
    CHECK(last < off.width / 2, "{}..={}", first, last);
}

TEST(the_perf_bar_is_black_on_white_in_the_light_theme_and_white_on_black_in_the_dark) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    for (ThemeKind kind : {ThemeKind::Light, ThemeKind::Dark}) {
        Headless<Page> app = Headless<Page>::with_theme(Page{}, kind);
        Image image = app.render(gpu);
        save(image, std::format("PerfBar{}", user_interface::name(kind)));
        using Rgb = std::array<std::uint8_t, 3>;
        auto [background, text] = kind == ThemeKind::Light ? std::pair(Rgb{255, 255, 255}, Rgb{0, 0, 0}) : std::pair(Rgb{0, 0, 0}, Rgb{255, 255, 255});
        auto rgb = [&](std::uint32_t x, std::uint32_t y) {
            Pixel pixel = image.pixel(x, y);
            return Rgb{pixel[0], pixel[1], pixel[2]};
        };
        // Past its text, every row above its divider is its background.
        for (std::uint32_t y = 0; y < base::saturating_cast<std::uint32_t>(PerfBarHeight) - 1; y++) {
            CHECK_EQ(rgb(image.width - 8, y), background, "{} theme, row {}", user_interface::name(kind), y);
        }
        // Its text's strokes reach the text's color.
        bool found = false;
        for (std::uint32_t x = 16; x < 400; x++) {
            for (std::uint32_t y = 0; y < base::saturating_cast<std::uint32_t>(PerfBarHeight); y++) {
                found |= rgb(x, y) == text;
            }
        }
        CHECK(found, "{} theme: no pixel of the text is {}", user_interface::name(kind), testing::describe(text));
    }
}

class Unsupported : public Program<std::monostate> {
public:
    std::string title() const {
        return "Unsupported";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui&) const {
        return unsupported_setup<std::monostate>(std::vector<std::string>{
            "The display is 1920 × 1080 pixels. It must be 2560 × 1440.",
            "The window must be maximized, or borderless fullscreen with F11.",
        });
    }
};

TEST(an_unsupported_setup_says_what_is_wrong) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless<Unsupported> app = Headless<Unsupported>::with_size(Unsupported{}, ThemeKind::Light, user_interface::Size(1280.0f, 720.0f));
    Image image = app.render(gpu);
    save(image, "UnsupportedSetup");
    // The window's corner is the background, and the box's middle has the
    // critical background in it.
    Theme theme = Theme::light();
    CHECK(close(image.pixel(5, 5), theme.background.to_bytes()));
    Pixel critical = theme.critical_background.to_bytes();
    bool found = false;
    for (std::uint32_t x = 0; x < 1280; x += 8) {
        found |= close(image.pixel(x, 360), critical);
    }
    CHECK(found);
}

} // namespace rendering_tests
