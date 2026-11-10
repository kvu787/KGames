// Tests how GameInput's readings become the interface's events: keys and
// their text and repeats, the mouse's buttons and wheel, the gamepad, focus,
// and the time of a frame's newest input, which its input latency starts at.
// The readings, the cursor and the focus changes are made up, with times on
// the performance counter.

#include "UserInterface/Input/Reader.h"
#include "Testing/Testing.h"
#include "UserInterface/Platform/Clock.h"

#include <algorithm>
#include <initializer_list>
#include <optional>
#include <tuple>
#include <vector>

using namespace user_interface;

namespace {

constexpr size_t KeyboardDevice = 1;
constexpr size_t MouseDevice = 2;
constexpr size_t GamepadDevice = 3;
using KeyCodes = std::pair<std::uint8_t, std::uint32_t>;
constexpr KeyCodes A{0x41, 0x1E};
constexpr KeyCodes LeftShift{0xA0, 0x2A};
constexpr KeyCodes RightShift{0xA1, 0x36};
using Timed = std::tuple<std::int64_t, game_input::GameInputKind, Reading>;

Timed keyboard(std::int64_t time, std::initializer_list<KeyCodes> keys) {
    Reading reading{
        .device = KeyboardDevice,
        .kinds = game_input::GameInputKindKeyboard,
    };
    for (auto [virtual_key, scan_code] : keys) {
        reading.keys.push_back(game_input::GameInputKeyState{
            .scanCode = scan_code,
            .codePoint = 0,
            .virtualKey = virtual_key,
            .isDeadKey = false,
        });
    }
    return {time, game_input::GameInputKindKeyboard, reading};
}

Timed mouse(std::int64_t time, game_input::GameInputMouseButtons buttons, std::int64_t wheel) {
    Reading reading{
        .device = MouseDevice,
        .kinds = game_input::GameInputKindMouse,
    };
    reading.mouse.buttons = buttons;
    reading.mouse.wheelY = wheel;
    return {time, game_input::GameInputKindMouse, reading};
}

Timed gamepad(std::int64_t time, game_input::GameInputGamepadButtons buttons) {
    Reading reading{
        .device = GamepadDevice,
        .kinds = game_input::GameInputKindGamepad,
    };
    reading.gamepad.buttons = buttons;
    return {time, game_input::GameInputKindGamepad, reading};
}

// The window with the cursor on it, which may get or lose focus at a time.
WindowState window(std::optional<std::pair<std::int64_t, bool>> focus) {
    WindowState state{
        .cursor = Point(100.0f, 50.0f),
        .inside = true,
    };
    if (focus) {
        state.focus_changes.push_back(FocusChange{.time = focus->first, .gained = focus->second});
    }
    return state;
}

FrameInput read(InputReader& reader, std::initializer_list<Timed> readings, std::initializer_list<size_t> disconnected, const WindowState& state, std::int64_t now) {
    std::vector<Timed> all(readings);
    std::vector<size_t> gone(disconnected);
    return reader.read(all, gone, state, now);
}

InputReader reader() {
    return InputReader::with_repeat(true, 0.5, 10.0);
}

// A reader whose window got focus at time 1.
InputReader focused() {
    InputReader made = reader();
    read(made, {}, {}, window({{1, true}}), 1);
    return made;
}

std::vector<Event> keys(const std::vector<Event>& events) {
    std::vector<Event> kept;
    for (const Event& event : events) {
        if (base::is<Event::KeyPressed>(event) || base::is<Event::KeyReleased>(event) || base::is<Event::Text>(event)) {
            kept.push_back(event);
        }
    }
    return kept;
}

bool contains(const std::vector<Event>& events, const Event& event) {
    return std::ranges::find(events, event) != events.end();
}

template <class T> bool any(const std::vector<Event>& events) {
    return std::ranges::any_of(events, [](const Event& event) { return base::is<T>(event); });
}

} // namespace

TEST(a_key_press_types_the_character_that_the_layout_gives) {
    InputReader reader = focused();
    FrameInput frame = read(reader, {keyboard(10, {A})}, {}, window(std::nullopt), 10);
    CHECK_EQ(keys(frame.events), (std::vector<Event>{Event::KeyPressed{Key::A, false}, Event::Text{U'a'}}));
    frame = read(reader, {keyboard(20, {}), keyboard(30, {LeftShift}), keyboard(40, {LeftShift, A})}, {}, window(std::nullopt), 40);
    CHECK_EQ(keys(frame.events), (std::vector<Event>{
                                     Event::KeyReleased{Key::A},
                                     Event::KeyPressed{Key::Shift, false},
                                     Event::KeyPressed{Key::A, false},
                                     Event::Text{U'A'},
                                 }));
}

TEST(shift_is_one_key_on_the_left_and_the_right) {
    InputReader reader = focused();
    read(reader, {keyboard(10, {LeftShift}), keyboard(20, {LeftShift, RightShift})}, {}, window(std::nullopt), 20);
    FrameInput frame = read(reader, {keyboard(30, {RightShift})}, {}, window(std::nullopt), 30);
    CHECK_EQ(keys(frame.events), std::vector<Event>{});
    frame = read(reader, {keyboard(40, {})}, {}, window(std::nullopt), 40);
    CHECK_EQ(keys(frame.events), (std::vector<Event>{Event::KeyReleased{Key::Shift}}));
}

TEST(a_held_key_repeats_after_the_delay_at_the_rate) {
    std::int64_t second = clock::frequency();
    InputReader reader = focused();
    read(reader, {keyboard(10, {A})}, {}, window(std::nullopt), 10);
    FrameInput frame = read(reader, {}, {}, window(std::nullopt), 10 + second * 4 / 10);
    CHECK_EQ(keys(frame.events), std::vector<Event>{});
    // At half a second, then a tenth of a second apart.
    frame = read(reader, {}, {}, window(std::nullopt), 10 + second * 75 / 100);
    size_t repeats = static_cast<size_t>(std::ranges::count(frame.events, Event(Event::KeyPressed{Key::A, true})));
    CHECK_EQ(repeats, 3u);
    frame = read(reader, {keyboard(10 + second, {})}, {}, window(std::nullopt), 10 + second * 2);
    CHECK_EQ(keys(frame.events), (std::vector<Event>{Event::KeyReleased{Key::A}}));
}

TEST(without_focus_keys_count_for_nothing_and_ones_held_when_it_comes_count_once_let_go) {
    InputReader made = reader();
    FrameInput frame = read(made, {keyboard(10, {A})}, {}, window(std::nullopt), 10);
    CHECK_EQ(keys(frame.events), std::vector<Event>{});
    CHECK_EQ(frame.input_time, std::optional<std::int64_t>());
    frame = read(made, {}, {}, window({{20, true}}), 20);
    CHECK(contains(frame.events, Event::FocusGained{{Key::A}}));
}

TEST(readings_and_focus_changes_happen_in_the_order_of_their_times) {
    InputReader made = reader();
    // Pressed before focus came, and let go after, in one frame.
    FrameInput frame = read(made, {keyboard(10, {A}), keyboard(30, {})}, {}, window({{20, true}}), 30);
    CHECK_EQ(std::ranges::count_if(frame.events, [](const Event& event) { return base::is<Event::FocusGained>(event); }), 1);
    CHECK(contains(frame.events, Event::FocusGained{{Key::A}}));
    CHECK(!contains(frame.events, Event::KeyPressed{Key::A, false}));
    CHECK(contains(frame.events, Event::KeyReleased{Key::A}));
}

TEST(the_click_that_gives_the_window_focus_does_only_that) {
    InputReader made = reader();
    FrameInput frame = read(made, {mouse(10, game_input::GameInputMouseLeftButton, 0), mouse(30, game_input::GameInputMouseNone, 0)}, {}, window({{20, true}}), 30);
    CHECK(!any<Event::ButtonPressed>(frame.events) && !any<Event::ButtonReleased>(frame.events));
}

TEST(a_press_while_the_cursor_is_off_the_window_is_another_window_s) {
    InputReader reader = focused();
    WindowState off = window(std::nullopt);
    off.inside = false;
    FrameInput frame = read(reader, {mouse(10, game_input::GameInputMouseLeftButton, 0)}, {}, off, 10);
    CHECK(!any<Event::ButtonPressed>(frame.events));
    CHECK(contains(frame.events, Event::PointerLeft{}));
    frame = read(reader, {mouse(20, game_input::GameInputMouseNone, 0)}, {}, window(std::nullopt), 20);
    CHECK(!any<Event::ButtonReleased>(frame.events));
}

TEST(a_press_on_the_window_is_where_the_cursor_is_and_holds_the_pointer_past_its_edge) {
    InputReader made = reader();
    WindowState moved = window({{1, true}});
    moved.cursor = Point(300.0f, 200.0f);
    FrameInput frame = read(made, {mouse(10, game_input::GameInputMouseLeftButton, 0)}, {}, moved, 10);
    auto moved_at = std::ranges::find(frame.events, Event(Event::PointerMoved{Point(300.0f, 200.0f)}));
    auto pressed_at = std::ranges::find(frame.events, Event(Event::ButtonPressed{MouseButton::Left}));
    CHECK(moved_at != frame.events.end() && pressed_at != frame.events.end() && moved_at < pressed_at);
    CHECK(made.capturing());
    WindowState past{
        .cursor = Point(-20.0f, 50.0f),
        .inside = false,
    };
    frame = read(made, {}, {}, past, 20);
    CHECK(contains(frame.events, Event::PointerMoved{Point(-20.0f, 50.0f)}));
    frame = read(made, {mouse(30, game_input::GameInputMouseNone, 0)}, {}, past, 30);
    CHECK(contains(frame.events, Event::ButtonReleased{MouseButton::Left}));
    CHECK(!made.capturing());
}

TEST(the_wheel_turns_in_notches_of_120) {
    InputReader reader = focused();
    read(reader, {mouse(10, game_input::GameInputMouseNone, 0)}, {}, window(std::nullopt), 10);
    FrameInput frame = read(reader, {mouse(20, game_input::GameInputMouseNone, 240)}, {}, window(std::nullopt), 20);
    CHECK(contains(frame.events, Event::Wheel{Point(0.0f, 2.0f)}));
}

TEST(a_frame_s_input_time_is_its_newest_reading_s) {
    InputReader reader = focused();
    FrameInput frame = read(reader, {keyboard(10, {A}), mouse(30, game_input::GameInputMouseNone, 0), keyboard(20, {})}, {}, window(std::nullopt), 40);
    CHECK_EQ(frame.input_time, std::optional<std::int64_t>(30));
    frame = read(reader, {}, {}, window(std::nullopt), 50);
    CHECK_EQ(frame.input_time, std::optional<std::int64_t>());
}

TEST(the_gamepad_counts_as_input_only_for_a_program_that_reads_it) {
    InputReader without = InputReader::with_repeat(false, 0.5, 10.0);
    read(without, {}, {}, window({{1, true}}), 1);
    FrameInput frame = read(without, {gamepad(10, game_input::GameInputGamepadA)}, {}, window(std::nullopt), 10);
    CHECK_EQ(frame.input_time, std::optional<std::int64_t>());
    CHECK(!any<Event::Gamepad>(frame.events));
    InputReader reader = focused();
    frame = read(reader, {gamepad(10, game_input::GameInputGamepadA)}, {}, window(std::nullopt), 10);
    CHECK_EQ(frame.input_time, std::optional<std::int64_t>(10));
}

TEST(the_gamepad_is_its_newest_reading_until_it_disconnects) {
    InputReader reader = focused();
    FrameInput frame = read(reader, {gamepad(10, game_input::GameInputGamepadA), gamepad(20, game_input::GameInputGamepadB)}, {}, window(std::nullopt), 20);
    const Event::Gamepad* state = nullptr;
    for (const Event& event : frame.events) {
        if (const Event::Gamepad* found = base::as<Event::Gamepad>(event)) {
            state = found;
        }
    }
    CHECK(state != nullptr, "no gamepad");
    CHECK(state->reading.connected);
    CHECK_EQ(state->reading.buttons.names(), std::vector<std::string_view>{"B"});
    frame = read(reader, {}, {GamepadDevice}, window(std::nullopt), 30);
    CHECK(!frame.events.empty());
    CHECK_EQ(frame.events.back(), Event(Event::Gamepad{}));
}

TEST(keys_and_buttons_held_on_a_device_that_is_unplugged_are_let_go_of) {
    InputReader reader = focused();
    read(reader, {keyboard(10, {A}), mouse(11, game_input::GameInputMouseLeftButton, 0)}, {}, window(std::nullopt), 11);
    FrameInput frame = read(reader, {}, {KeyboardDevice, MouseDevice}, window(std::nullopt), 20);
    CHECK_EQ(keys(frame.events), (std::vector<Event>{Event::KeyReleased{Key::A}}));
    CHECK(contains(frame.events, Event::ButtonReleased{MouseButton::Left}));
    // Nothing is held any more, so the next press is a press.
    frame = read(reader, {keyboard(30, {A})}, {}, window(std::nullopt), 30);
    CHECK(contains(frame.events, Event::KeyPressed{Key::A, false}));
}

// Only "no reading after this one" means that there's nothing new. Any other
// failure of GetNextReading starts over from the newest reading, so that a
// failure that keeps coming can't stop a kind of input for good.
TEST(only_reading_not_found_means_nothing_newer) {
    CHECK_EQ(after_next_reading_failed(game_input::GAMEINPUT_E_READING_NOT_FOUND), NextReadingFailure::NothingNewer);
    for (HRESULT code : {game_input::GAMEINPUT_E_REFERENCE_READING_TOO_OLD, game_input::GAMEINPUT_E_OBJECT_NO_LONGER_EXISTS, game_input::GAMEINPUT_E_DEVICE_DISCONNECTED}) {
        CHECK_EQ(after_next_reading_failed(code), NextReadingFailure::FindFromNewest);
    }
}
