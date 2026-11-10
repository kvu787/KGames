// The vehicle editor, and its footprint creator: picks a vehicle's model,
// its acceleration, and its paint over the model's, and draws its footprint,
// the rectangle that touches the track's edges, over its model seen from
// above. A side of the footprint follows a drag from it in the view, and the
// footprint can fit the model's own rectangle. The paint has a card of its
// own, down the right side, and the vehicle is between the two.

#pragma once

#include "Base/Sum.h"
#include "SimplePaint/Material.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Document.h"
#include "Veehiicuul/Editing.h"
#include "Veehiicuul/Models.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>

namespace veehiicuul::vehicle_editor {

// The vehicle faces up the screen, which is this way on the ground.
inline constexpr double ShownHeading = -135.0;
inline constexpr std::array<std::string_view, 4> Sides{"Left", "Right", "Front", "Back"};

// What the cards' controls do.
struct VehicleMsg {
    struct Select {
        size_t index;
    };
    struct Add {};
    struct Delete {};
    // What the name's field holds, which renames the vehicle while the name
    // is free.
    struct Name {
        std::string text;
    };
    struct Model {
        size_t model;
    };
    // One of the acceleration's rates, as `Rates` has them.
    struct Rate {
        size_t index;
        double value;
    };
    // How far the footprint goes to one side, as Sides has them.
    struct Side {
        size_t side;
        double value;
    };
    struct Fit {};
    struct Slot {
        size_t slot;
    };
    struct Paint {
        simple_paint::Parameters parameters;
    };
    struct ResetPaint {};
    std::variant<Select, Add, Delete, Name, Model, Rate, Side, Fit, Slot, Paint, ResetPaint> value;

    template <base::AlternativeOf<VehicleMsg> T> VehicleMsg(T&& alternative) : value(std::forward<T>(alternative)) {}
};

// The acceleration's rates, as Rate's index has them.
inline constexpr std::array<std::string_view, 4> Rates{"Forward", "Reverse", "Left", "Right"};

class VehicleEditor {
public:
    editing::Viewport viewport{{0.0, 0.0}, 12.0};
    size_t slot = 0;
    std::string message;

    // Keeps the name's field on the vehicle's name while it isn't being
    // edited, so that editing starts from it.
    void sync_name(const document::Document& document, size_t vehicle);

    // What the pointer in the view does: a drag moves a side of the
    // footprint.
    void view_input(const editing::GroundInput& input, document::Document& document, size_t vehicle, std::array<float, 2> size);

    // A control on the cards.
    void apply(const VehicleMsg& given, document::Document& document, const models::Models& models, size_t& vehicle);

    // The overlay over the view, then the cards, over both.
    void view(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const document::Document& document,
              const models::Models& models, size_t vehicle) const;

    json::Value state(const document::Document& document, size_t vehicle) const;

private:
    // The side of the footprint being dragged, as an index of Sides.
    std::optional<size_t> drag;
    // The side under the pointer, for the handles the overlay draws.
    std::optional<size_t> under;
    std::string name_text;

    void card(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, const document::Document& document, const models::Models& models,
              size_t vehicle) const;

    // The card down the right side, with the vehicle's paint over its
    // model's.
    void paint_card(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, const document::Document& document,
                    const models::Models& models, size_t vehicle) const;
};

} // namespace veehiicuul::vehicle_editor
