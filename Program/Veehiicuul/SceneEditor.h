// The scene editor: arranges the current scene's decorations, which are
// models, and the vehicles parked in it, which a press in the view picks
// and a drag moves, and sets the scene's background, its fixed camera, and
// the paint of a decoration's model, which every use of the model shares.

#pragma once

#include "Base/Sum.h"
#include "SimplePaint/Material.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Document.h"
#include "Veehiicuul/Editing.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Models.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <optional>
#include <string>
#include <utility>
#include <variant>

namespace veehiicuul::scene_editor {

// What's picked: a decoration, or a parked vehicle, by its index.
struct Picked {
    struct Decoration {
        size_t index;
        bool operator==(const Decoration&) const = default;
    };
    struct Parked {
        size_t index;
        bool operator==(const Parked&) const = default;
    };
    std::variant<Decoration, Parked> value;

    template <base::AlternativeOf<Picked> T> Picked(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Picked&) const = default;
};

// How a failed check in a test shows what's picked.
std::string describe_value(const Picked& picked);

// What the card's controls do. What needs the view's middle on the ground
// carries it from the frame it was pressed on.
struct SceneMsg {
    struct Pick {
        size_t row;
    };
    struct Model {
        size_t model;
    };
    struct Vehicle {
        size_t vehicle;
    };
    struct Add {
        std::array<double, 2> center;
    };
    struct Park {
        std::array<double, 2> center;
    };
    struct Delete {};
    struct Camera {
        std::array<double, 2> center;
        double height;
    };
    struct Background {
        files::Color color;
    };
    struct FollowHeight {
        double height;
    };
    struct Heading {
        double heading;
    };
    struct Scale {
        double scale;
    };
    struct Slot {
        size_t slot;
    };
    struct Paint {
        simple_paint::Parameters parameters;
    };
    std::variant<Pick, Model, Vehicle, Add, Park, Delete, Camera, Background, FollowHeight, Heading, Scale, Slot, Paint> value;

    template <base::AlternativeOf<SceneMsg> T> SceneMsg(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class SceneEditor {
public:
    std::optional<Picked> picked;
    size_t model = 0;
    size_t vehicle = 0;
    size_t slot = 0;

    // Drops a pick that's gone, like after an undo, even one during a drag.
    void keep_picked(const document::Document& document, size_t scene);

    // What the pointer in the view does: a press picks what's under it, and
    // a drag moves it.
    void view_input(const editing::GroundInput& input, document::Document& document, const models::Models& models, size_t scene);

    // A control on the card.
    void apply(const SceneMsg& message, document::Document& document, const models::Models& models, size_t scene);

    // The overlay over the view, then the card, over both.
    void view(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const editing::Viewport& viewport,
              const document::Document& document, const models::Models& models, size_t scene) const;

    json::Value state(const document::Document& document, size_t scene) const;

private:
    // How far the picked thing is from the point on the ground that a drag
    // grabbed it at.
    std::optional<std::array<double, 2>> grab;

    Picked row_of(const document::Document& document, size_t scene, size_t row) const;

    void card(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const editing::Viewport& viewport,
              const document::Document& document, const models::Models& models, size_t scene) const;
};

} // namespace veehiicuul::scene_editor
