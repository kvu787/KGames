// The hexel terrain editor: paints the current scene's terrain, a hexel at a
// time or a brush of them, with the colors of its palette, under the track.
//
// | Tool  | What a press and a drag in the view do                              |
// | ----- | ------------------------------------------------------------------- |
// | Paint | Paints the hexels under the brush with the palette's color          |
// | Erase | Takes the paint off them, so that the background shows             |
// | Fill  | Paints the hexel and those joined to it that are its color, nearby  |
// | Pick  | Makes the hexel's color the palette's color                         |

#pragma once

#include "Base/Sum.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Editing.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Terrain.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string_view>
#include <utility>
#include <variant>

namespace veehiicuul::terrain_editor {

inline constexpr std::array<std::string_view, 4> Tools{"Paint", "Erase", "Fill", "Pick"};

// What the card's controls do.
struct TerrainMsg {
    struct Tool {
        size_t tool;
    };
    struct Radius {
        double radius;
    };
    // The palette's swatch picked, and an edit to the picked color.
    struct PickColor {
        size_t index;
    };
    struct Color {
        files::Color color;
    };
    struct AddColor {};
    struct RemoveColor {};
    struct Size {
        double size;
    };
    struct Orientation {
        size_t orientation;
    };
    std::variant<Tool, Radius, PickColor, Color, AddColor, RemoveColor, Size, Orientation> value;

    template <base::AlternativeOf<TerrainMsg> T> TerrainMsg(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class TerrainEditor {
public:
    size_t tool = 0;
    // The palette's color that Paint and Fill use.
    std::uint8_t color = 0;
    // The brush's radius, in hexels around the one under the pointer.
    double radius = 1.0;
    // The hexel under the pointer, if it's on the view.
    std::optional<terrain::Cell> hover;

    // What the pointer in the view does: the tool, on the hexels under it.
    void view_input(const editing::GroundInput& input, terrain::Terrain& terrain);

    // A control on the card.
    void apply(const TerrainMsg& message, terrain::Terrain& terrain);

    // The overlay over the view, then the card, over both.
    void view(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, user_interface::Rect view_rect, const editing::Viewport& viewport,
              const terrain::Terrain& terrain) const;

    json::Value state(const terrain::Terrain& terrain) const;

private:
    void use_tool(terrain::Terrain& terrain, terrain::Cell cell, bool pressed);

    void card(user_interface::Ui& ui, user_interface::Tree<program::Message>& tree, user_interface::Rect page, const terrain::Terrain& terrain) const;
};

} // namespace veehiicuul::terrain_editor
