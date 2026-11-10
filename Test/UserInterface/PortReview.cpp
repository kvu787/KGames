// Pins what the adversarial review of the port to C++ found in the library,
// which Documentation/LogicReview.md records: places where the Rust stopped
// the program, or wrote something else, and the C++ at first did neither.

#include "Base/Panic.h"
#include "Base/Vec.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <limits>
#include <optional>
#include <string>
#include <variant>
#include <vector>

namespace port_review_tests {

using user_interface::Button;
using user_interface::Context;
using user_interface::Headless;
using user_interface::MenuItem;
using user_interface::NumberBox;
using user_interface::Point;
using user_interface::Program;
using user_interface::Rect;
using user_interface::Size;
using user_interface::Slider;
using user_interface::TextBox;
using user_interface::ThemeKind;
using user_interface::Tree;
using user_interface::Ui;

// A button with a menu, at the top left of a window.
class Narrow : public Program<std::monostate> {
public:
    std::string title() const {
        return "Narrow";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui&) const {
        Tree<std::monostate> tree;
        tree.add("Menu", Rect(0.0f, 0.0f, 40.0f, 32.0f), Button<std::monostate>("M").menu(base::vec(MenuItem<std::monostate>("An item of the menu"))));
        return tree;
    }
};

// A menu's place is clamped between the window's two sides, less a margin.
// In a window narrower than the menu, the least place is past the most:
// Rust's clamp panics there, and std::clamp is undefined. The program stops,
// on the frame of the press that opens the menu.
TEST(a_menu_wider_than_its_window_stops_the_program) {
    Headless<Narrow> app = Headless<Narrow>::with_size(Narrow{}, ThemeKind::Light, Size(100.0f, 400.0f));
    CHECK_THROWS(app.press_point(Point(20.0f, 16.0f)));
    // In a window with room for it, it opens.
    Headless<Narrow> wide = Headless<Narrow>::with_size(Narrow{}, ThemeKind::Light, Size(800.0f, 400.0f));
    wide.press_point(Point(20.0f, 16.0f));
    CHECK(wide.ui.any_popup());
}

// What a slider and a number box show for each of two values.
class Values : public Program<std::monostate> {
public:
    double value = 0.0;
    std::string text;

    std::string title() const {
        return "Values";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        float top = ui.content_top() + 10.0f;
        tree.add("Slider", Rect(10.0f, top, 300.0f, 32.0f), Slider<std::monostate>(value, 0.0, 1.0).suffix(" m"));
        tree.add("Number", Rect(10.0f, top + 40.0f, 300.0f, 32.0f), NumberBox<std::monostate>(value).range(0.0, 1.0));
        tree.add("Field", Rect(10.0f, top + 80.0f, 300.0f, 32.0f), TextBox<std::monostate>(text));
        return tree;
    }
};

// Rust writes a value that isn't a number as NaN, and std::format as nan,
// which showed in the labels and in the state that scripts read.
TEST(a_value_that_isnt_a_number_reads_nan_as_rust_writes_it) {
    Headless app(Values{});
    app.frame();
    CHECK(app.label("Slider").value().ends_with(" m"));
    app.program.value = std::numeric_limits<double>::quiet_NaN();
    app.frame();
    CHECK_EQ(app.label("Slider"), std::optional<std::string>("NaN m"));
    CHECK_EQ(app.label("Number"), std::optional<std::string>("NaN"));
}

// A text that starts inside a character, which no text in Rust can, had the
// text box search back for a character's start forever: the start of a text
// wasn't one, to the C++, when its first byte continues a character.
TEST(a_text_box_whose_text_starts_inside_a_character_draws) {
    Headless app(Values{});
    app.program.text = "\x80"
                       "abc";
    app.frame();
    app.click("Field").frame();
    CHECK(app.has_target("Field"));
}

// Sets its flag when it's destroyed.
struct Guard {
    bool* destroyed;

    ~Guard() {
        *destroyed = true;
    }
};

// A panic leaves the main loop through a handler, so that what the loop
// holds is destroyed, as Rust drops it when a panic unwinds: the session
// that times presents stops, and the presenter waits for the GPU. Uncaught,
// it would stop the program with all of that left as it was.
TEST(a_panic_in_the_main_loop_destroys_what_the_loop_holds) {
    bool destroyed = false;
    int code = user_interface::run_detail::unwinding([&]() -> int {
        Guard guard{&destroyed};
        base::panic("the cascade didn't settle");
    });
    CHECK(destroyed);
    CHECK_EQ(code, user_interface::run_detail::StoppedExitCode);
    CHECK_EQ(user_interface::run_detail::unwinding([] { return 0; }), 0);
    CHECK_EQ(user_interface::run_detail::unwinding([] { return 1; }), 1);
}

} // namespace port_review_tests
