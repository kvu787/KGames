// Scripts: input from a file instead of the user, on a fixed clock, which
// tests a program in its real window without touching the user's input or
// focus. RunInBackground runs them, and the README describes the format.
//
//     {
//         "FrameTime": 0.016666666666666666,
//         "Steps": [
//             { "Frame": 1, "Click": "Save" },
//             { "Frame": 3, "Drag": { "From": "Volume", "To": [400, 300], "Frames": 10 } },
//             { "Frame": 20, "Key": "Ctrl+S" },
//             { "Frame": 22, "Capture": "Saved" },
//             { "Frame": 22, "Expect": { "Saved": true, "Interface": { "Popup": null } } }
//         ]
//     }
//
// Frame N's steps happen before frame N's update, which sees them as input,
// except Capture and Expect, which happen after it. Frame N is at N times
// FrameTime seconds, a 60th of a second unless the script says. Frame 0,
// before the first step, gives the window focus and draws what steps can
// point at. The program exits after the last step's frame, and fails if an
// expectation wasn't met.

#pragma once

#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "Json/Json.h"

#include <cstdint>
#include <expected>
#include <filesystem>
#include <map>
#include <string>
#include <string_view>
#include <vector>

namespace user_interface {

class Interface;

inline constexpr double DefaultFrameTime = 1.0 / 60.0;

class Script {
public:
    double frame_time;

    static std::expected<Script, std::string> load(const std::filesystem::path& path);

    static std::expected<Script, std::string> parse(std::string_view text);

    Script(const Script&) = delete;
    Script& operator=(const Script&) = delete;
    Script(Script&&) = default;
    Script& operator=(Script&&) = default;

    // The frame of the last step.
    std::uint64_t last_frame() const;

private:
    // The runner reads the steps.
    friend class ScriptRunner;

    struct Step {
        std::uint64_t frame;
        std::string action;
        json::Value value;
    };

    Script(double frame_time, std::vector<Step> steps);

    std::vector<Step> steps;
};

// What a frame of a script does around the program's update.
struct ScriptFrame {
    std::vector<Event> events;
    std::vector<json::Value> program_steps;
    // These happen after the update.
    std::vector<json::Value> expectations;
    std::vector<std::string> captures;
};

class ScriptRunner {
public:
    std::vector<std::string> failures;

    explicit ScriptRunner(Script script);
    ScriptRunner(const ScriptRunner&) = delete;
    ScriptRunner& operator=(const ScriptRunner&) = delete;
    ScriptRunner(ScriptRunner&&) = default;
    ScriptRunner& operator=(ScriptRunner&&) = default;

    double frame_time() const;

    std::uint64_t last_frame() const;

    // What frame N does. `ui` has the last frame's targets, which names point
    // at.
    std::expected<ScriptFrame, std::string> frame(std::uint64_t frame, const Interface& ui);

    // Checks an expectation against the state, and remembers what didn't
    // match.
    void check(std::uint64_t frame, const json::Value& expected, const json::Value& state);

private:
    void schedule(std::uint64_t frame, Event event);

    Script script;
    // Events that steps put off to later frames, like a click's release.
    std::map<std::uint64_t, std::vector<Event>> scheduled;
    std::vector<Key> held;
    Gamepad gamepad;
};

// A gamepad from a script's Gamepad step.
std::expected<Gamepad, std::string> parse_gamepad(const json::Value& value);

// Whether `actual` has everything that `expected` has: objects match if each
// member that `expected` names matches, lists match item by item, and
// numbers match to a millionth. Each mismatch says where it is.
void compare(const json::Value& expected, const json::Value& actual, std::string& path, std::vector<std::string>& mismatches);

} // namespace user_interface
