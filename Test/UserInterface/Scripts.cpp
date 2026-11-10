// Tests scripts: what they parse to, the input their steps make, and how
// expectations match the state.

#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <cstdint>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

using user_interface::Button;
using user_interface::compare;
using user_interface::Context;
using user_interface::Event;
using user_interface::Headless;
using user_interface::Key;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Program;
using user_interface::Rect;
using user_interface::Script;
using user_interface::ScriptRunner;
using user_interface::Tree;
using user_interface::Ui;

namespace scripts_tests {

class Page : public Program<std::monostate> {
public:
    std::string title() const {
        return "Page";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui&) const {
        Tree<std::monostate> tree;
        tree.add("Save", Rect(100.0f, 100.0f, 80.0f, 32.0f), Button<std::monostate>("Save"));
        return tree;
    }
};

TEST(scripts_say_what_is_wrong_with_them) {
    CHECK(!Script::parse("{").has_value());
    CHECK(Script::parse(R"({ "Steps": [ { "Move": [1, 2] } ] })").error().contains("Frame"));
    CHECK(Script::parse(R"({ "Steps": [ { "Frame": 1, "Fly": true } ] })").error().contains("Fly"));
    CHECK(!Script::parse(R"({ "Steps": [ { "Frame": 1, "Move": [1, 2], "Click": [1, 2] } ] })").has_value());
    Script script = Script::parse(R"({ "FrameTime": 0.01, "Steps": [ { "Frame": 7, "Key": "F1" }, { "Frame": 3, "Key": "F2" } ] })").value();
    CHECK_EQ(script.frame_time, 0.01);
    CHECK_EQ(script.last_frame(), 7);
}

TEST(steps_make_input_on_their_frames) {
    Headless app(Page{});
    Script script = Script::parse(
                        R"({ "Steps": [
            { "Frame": 1, "Click": "Save" },
            { "Frame": 3, "Key": "Ctrl+S" },
            { "Frame": 5, "Hold": ["W"] },
            { "Frame": 6, "Hold": [] },
            { "Frame": 7, "Drag": { "From": [0, 0], "To": [10, 0], "Frames": 2, "Button": "Middle" } },
            { "Frame": 12, "Press": { "At": [3, 4], "Button": "Right" } }
        ] })")
                        .value();
    ScriptRunner runner(std::move(script));
    auto frame = [&](std::uint64_t number) {
        return runner.frame(number, app.ui).value().events;
    };
    CHECK_EQ(frame(0)[0], Event(Event::FocusGained{}));
    std::vector<Event> first = frame(1);
    CHECK_EQ(first[0], Event(Event::PointerMoved{app.target("Save").center()}));
    CHECK_EQ(first[1], Event(Event::ButtonPressed{MouseButton::Left}));
    CHECK_EQ(frame(2)[0], Event(Event::ButtonReleased{MouseButton::Left}));
    std::vector<Event> third = frame(3);
    CHECK_EQ(std::vector<Event>(third.begin(), third.begin() + 2),
             (std::vector<Event>{Event::KeyPressed{.key = Key::Control, .repeat = false}, Event::KeyPressed{.key = Key::S, .repeat = false}}));
    std::vector<Event> fourth = frame(4);
    CHECK_EQ(std::vector<Event>(fourth.begin(), fourth.begin() + 2), (std::vector<Event>{Event::KeyReleased{Key::S}, Event::KeyReleased{Key::Control}}));
    CHECK_EQ(frame(5)[0], Event(Event::KeyPressed{.key = Key::W, .repeat = false}));
    CHECK_EQ(frame(6)[0], Event(Event::KeyReleased{Key::W}));
    CHECK_EQ(frame(7)[1], Event(Event::ButtonPressed{MouseButton::Middle}));
    frame(8);
    std::vector<Event> ninth = frame(9);
    const Event::PointerMoved* moved = base::as<Event::PointerMoved>(ninth[0]);
    CHECK(moved != nullptr && moved->point.x == 10.0f);
    CHECK_EQ(frame(10)[0], Event(Event::ButtonReleased{MouseButton::Middle}));
    frame(11);
    std::vector<Event> twelfth = frame(12);
    CHECK_EQ(std::vector<Event>(twelfth.begin(), twelfth.begin() + 2), (std::vector<Event>{Event::PointerMoved{Point(3.0f, 4.0f)}, Event::ButtonPressed{MouseButton::Right}}));
}

TEST(a_step_that_points_at_a_widget_the_frame_did_not_show_fails) {
    Headless app(Page{});
    Script missing = Script::parse(R"({ "Steps": [ { "Frame": 1, "Click": "Open" } ] })").value();
    std::string error = ScriptRunner(std::move(missing)).frame(1, app.ui).error();
    CHECK(error.contains("Open") && error.contains("Save"), "{}", error);
}

TEST(expectations_match_what_they_name) {
    json::Value state = json::parse(R"({ "Volume": 50.0000001, "Name": "Ada", "List": [1, 2], "Interface": { "Popup": null, "Focused": "Name" } })").value();
    auto check = [&](std::string_view expected) {
        std::vector<std::string> mismatches;
        std::string path;
        compare(json::parse(expected).value(), state, path, mismatches);
        return mismatches;
    };
    CHECK(check(R"({ "Volume": 50, "Interface": { "Popup": null } })").empty());
    CHECK_EQ(check(R"({ "Name": "Grace" })"), std::vector<std::string>{R"(Name is "Ada", not "Grace")"});
    CHECK_EQ(check(R"({ "Interface": { "Focused": null } })").size(), 1);
    CHECK_EQ(check(R"({ "Missing": 1 })"), std::vector<std::string>{"Missing is missing"});
    CHECK_EQ(check(R"({ "List": [1] })").size(), 1);
}

TEST(a_point_can_be_a_place_in_a_target) {
    Headless app(Page{});
    Script script = Script::parse(R"({ "Steps": [ { "Frame": 1, "Move": { "Target": "Save", "At": [0.25, 0.5] } } ] })").value();
    std::vector<Event> events = ScriptRunner(std::move(script)).frame(1, app.ui).value().events;
    Rect save = app.target("Save");
    CHECK_EQ(events[0], Event(Event::PointerMoved{Point(save.x + save.width / 4.0f, save.center().y)}));
}

} // namespace scripts_tests
