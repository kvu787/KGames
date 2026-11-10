// The track builder: edits the current scene's track by its curves, the
// road's outer edge and its islands' edges, whose control points the view
// shows, and the barriers and the road follow them. It also moves the
// start, where the car starts, and turns it, by its handle ahead of it.
//
// | Tool           | What a press and a drag in the view do                     |
// | -------------- | ---------------------------------------------------------- |
// | Move points    | Moves the control point, the start, or the start's handle  |
// | Add points     | Adds a control point to the curve, near where it's closest |
// | Delete points  | Deletes the control point, if its curve has more than 3    |
// | Move track     | Moves every curve, and the start                           |

#pragma once

#include "Base/Sum.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Editing.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Track.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>

namespace veehiicuul::track_editor {

inline constexpr std::array<std::string_view, 4> Tools{"Move points", "Add points", "Delete points", "Move track"};

// What the card's controls do.
struct TrackMsg {
    struct Tool {
        size_t tool;
    };
    struct Curve {
        size_t curve;
    };
    // An island: a circle of 8 points in the middle of the view, this big.
    struct AddIsland {
        std::array<double, 2> center;
        double radius;
    };
    struct DeleteIsland {};
    struct Heading {
        double heading;
    };
    struct BarrierWidth {
        double width;
    };
    struct BarrierLength {
        double length;
    };
    struct RoadColor {
        files::Color color;
    };
    struct BarrierColor {
        size_t index;
        files::Color color;
    };
    std::variant<Tool, Curve, AddIsland, DeleteIsland, Heading, BarrierWidth, BarrierLength, RoadColor, BarrierColor> value;

    template <base::AlternativeOf<TrackMsg> T> TrackMsg(T&& alternative) : value(std::forward<T>(alternative)) {}
};

std::string curve_name(size_t index);

class TrackEditor {
public:
    size_t tool = 0;
    // The curve that points are added to: 0 for the road, and then the
    // islands.
    size_t curve = 0;

    // What the pointer in the view does: the tool, on the curves' points,
    // the start, or the whole track.
    void view_input(const editing::GroundInput& input, track::Track& track, camera::Eye eye, std::array<float, 2> size);

    // A control on the card.
    void apply(const TrackMsg& message, track::Track& track);

    // The overlay over the view, then the card, over both.
    void view(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const editing::Viewport& viewport,
              const track::Track& track, std::span<const std::string> problems) const;

    json::Value state(const track::Track& track, std::span<const std::string> problems) const;

private:
    // What a drag in the view moves.
    struct Drag {
        struct Point {
            size_t curve;
            size_t index;
        };
        struct Start {};
        struct Heading {};
        // The point on the ground that the track was grabbed at.
        struct Track {
            std::array<double, 2> grabbed;
        };
        std::variant<Point, Start, Heading, Track> value;

        template <base::AlternativeOf<Drag> T> Drag(T&& alternative) : value(std::forward<T>(alternative)) {}
    };

    std::optional<Drag> drag;
    // The control point under the pointer, for the ring the overlay draws.
    std::optional<std::array<double, 2>> under;

    // The curves' control points, and the lines between them, the start's
    // handle, and a ring around the point under the pointer.
    void overlay(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect view_rect, const editing::Viewport& viewport,
                 const track::Track& track) const;

    void card(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const editing::Viewport& viewport,
              const track::Track& track, std::span<const std::string> problems) const;
};

} // namespace veehiicuul::track_editor
