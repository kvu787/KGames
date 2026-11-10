#include "Veehiicuul/TerrainEditor.h"

#include "Base/Numbers.h"
#include "Veehiicuul/Program.h"

#include <algorithm>
#include <format>
#include <string>
#include <vector>

namespace veehiicuul::terrain_editor {

namespace tokens = user_interface::tokens;

using editing::GroundInput;
using editing::interface_color;
using editing::Projection;
using editing::SliderRange;
using editing::Viewport;
using json::Value;
using program::Message;
using terrain::Cell;
using terrain::disk;
using terrain::PaletteSize;
using terrain::Terrain;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Field;
using user_interface::Form;
using user_interface::Icon;
using user_interface::Painter;
using user_interface::Palette;
using user_interface::Rect;
using user_interface::Segmented;
using user_interface::Tree;
using user_interface::Ui;

namespace {

constexpr std::array<Icon, 4> ToolIcons{Icon::Brush, Icon::Erase, Icon::Fill, Icon::Eyedropper};
// How far a fill reaches from the hexel it starts at, in hexels.
constexpr std::int32_t FillReach = 60;
// The width of the labels of the lines about the hexels.
constexpr float LabelWidth = 160.0f;

} // namespace

void TerrainEditor::view_input(const GroundInput& input, Terrain& terrain) {
    hover = input.ground ? std::optional(terrain.cell_at(*input.ground)) : std::nullopt;
    if (hover && input.held && !input.panning) {
        use_tool(terrain, *hover, input.pressed);
    }
}

void TerrainEditor::apply(const TerrainMsg& message, Terrain& terrain) {
    base::match(
        message, [&](const TerrainMsg::Tool& chosen) { tool = chosen.tool; }, [&](const TerrainMsg::Radius& chosen) { radius = chosen.radius; },
        [&](const TerrainMsg::PickColor& picked) { color = static_cast<std::uint8_t>(picked.index); },
        [&](const TerrainMsg::Color& edited) {
            if (static_cast<size_t>(color) < terrain.palette.size()) {
                terrain.palette[color] = edited.color;
            }
        },
        [&](const TerrainMsg::AddColor&) {
            if (terrain.palette.size() < PaletteSize) {
                files::Color copied = static_cast<size_t>(color) < terrain.palette.size() ? terrain.palette[color] : files::Color{128, 128, 128};
                terrain.palette.push_back(copied);
                color = static_cast<std::uint8_t>(terrain.palette.size() - 1);
            }
        },
        [&](const TerrainMsg::RemoveColor&) {
            if (terrain.palette.size() > 1) {
                terrain.remove_color(color);
            }
        },
        [&](const TerrainMsg::Size& sized) { terrain.size = sized.size; }, [&](const TerrainMsg::Orientation& chosen) { terrain.flat_top = chosen.orientation == 1; });
    color = std::min(color, static_cast<std::uint8_t>(base::saturating_sub(terrain.palette.size(), size_t{1})));
}

void TerrainEditor::use_tool(Terrain& terrain, Cell cell, bool pressed) {
    std::int32_t brush = base::saturating_cast<std::int32_t>(radius);
    bool in_palette = static_cast<size_t>(color) < terrain.palette.size();
    if (tool == 0 && in_palette) {
        for (Cell under : disk(cell, brush)) {
            terrain.set(under, color);
        }
    } else if (tool == 1) {
        for (Cell under : disk(cell, brush)) {
            terrain.set(under, std::nullopt);
        }
    } else if (tool == 2 && pressed && in_palette) {
        for (Cell joined : terrain.region(cell, FillReach)) {
            terrain.set(joined, color);
        }
    } else if (tool == 3) {
        if (std::optional<std::uint8_t> painted = terrain.get(cell)) {
            color = *painted;
        }
    }
}

void TerrainEditor::view(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Viewport& viewport, const Terrain& terrain) const {
    // The hexels that the tool would use, outlined.
    if (hover) {
        std::vector<Cell> cells = tool < 2 ? disk(*hover, base::saturating_cast<std::int32_t>(radius)) : std::vector<Cell>{*hover};
        std::vector<std::vector<std::array<double, 2>>> outlines;
        outlines.reserve(cells.size());
        for (Cell cell : cells) {
            std::array<std::array<double, 2>, 6> corners = terrain.corners(cell);
            outlines.emplace_back(corners.begin(), corners.end());
        }
        Projection projection(viewport.eye, editing::window_size(ui));
        tree.draw(view_rect, Canvas([outlines = std::move(outlines), projection](Painter& painter, Rect rect) {
                      user_interface::Color stroke = painter.theme().text_primary;
                      painter.push_clip(rect);
                      for (const std::vector<std::array<double, 2>>& outline : outlines) {
                          projection.outline(painter, outline, 1.5f, stroke);
                      }
                      painter.pop_clip();
                  }));
    }
    card(ui, tree, page, terrain);
}

void TerrainEditor::card(Ui& ui, Tree<Message>& tree, Rect page, const Terrain& terrain) const {
    Card frame = Card("Terrain").subtitle("Hexels painted on the ground").floating();
    tree.sized_card(ui, std::nullopt, editing::left_card(page), std::move(frame), [&](Ui& inner, Tree<Message>& items, CardArea area) {
        Form form(area.content);
        std::array<std::pair<Icon, std::string_view>, 4> tools;
        for (size_t index = 0; index < tools.size(); index++) {
            tools[index] = {ToolIcons[index], Tools[index]};
        }
        items.add("Terrain/Tool", form.take(tokens::ControlHeight),
                  Segmented<Message>(tool, tools).on_select([](size_t chosen) { return Message::Terrain{TerrainMsg::Tool{chosen}}; }));
        SliderRange brush(0.0, 8.0, 1.0, 0);
        editing::slider_row(form, items, "Terrain/Radius", "Brush radius", radius, brush, [](double value) { return Message::Terrain{TerrainMsg::Radius{value}}; });

        form.heading(items, "Palette");
        size_t picked = std::min(static_cast<size_t>(color), base::saturating_sub(terrain.palette.size(), size_t{1}));
        std::vector<user_interface::Color> colors;
        colors.reserve(terrain.palette.size());
        for (files::Color listed : terrain.palette) {
            colors.push_back(interface_color(listed));
        }
        Palette<Message> palette = Palette<Message>(picked, colors).on_pick([](size_t index) { return Message::Terrain{TerrainMsg::PickColor{index}}; });
        Rect palette_area = form.take(palette.height(form.rest().width));
        items.add("Palette", palette_area, std::move(palette));
        auto [color_column, add, remove] = form.columns<3>(tokens::ControlHeight);
        if (picked < terrain.palette.size()) {
            editing::color_field(items, "Terrain/Color", color_column, terrain.palette[picked], [](files::Color edited) { return Message::Terrain{TerrainMsg::Color{edited}}; });
        }
        items.add("Terrain/AddColor", add, Button<Message>("Add a color").icon(Icon::Add).on_press(Message::Terrain{TerrainMsg::AddColor{}}))
            .set_enabled(terrain.palette.size() < PaletteSize)
            .set_tooltip("Adds a copy of the color at the palette's end");
        items.add("Terrain/RemoveColor", remove, Button<Message>("Remove it").icon(Icon::Delete).on_press(Message::Terrain{TerrainMsg::RemoveColor{}}))
            .set_enabled(terrain.palette.size() > 1)
            .set_tooltip("Removes the color, and takes it off its hexels");
        form.caption(inner, items, std::format("{} of {} colors. Removing one takes it off its hexels.", terrain.palette.size(), PaletteSize));

        form.heading(items, "Grid");
        SliderRange sizes(0.5, 10.0, 10.0, 1);
        editing::slider_row(form, items, "Terrain/Size", "Hexel size", terrain.size, sizes, [](double value) { return Message::Terrain{TerrainMsg::Size{value}}; });
        size_t orientation = terrain.flat_top ? 1 : 0;
        Rect orientation_area = form.control(items, "Orientation");
        items.add("Terrain/Orientation", orientation_area,
                  Segmented<Message>(orientation, {{"Pointy tops", "Flat tops"}}).on_select([](size_t chosen) { return Message::Terrain{TerrainMsg::Orientation{chosen}}; }));

        form.heading(items, "Hexels");
        items.draw(form.take(tokens::Body.line_height), Field("Painted", LabelWidth, std::to_string(terrain.cells.size())));
        std::string under = hover ? std::format("Hexel {}, {}", hover->first, hover->second) : std::string("Nothing");
        items.draw(form.take(tokens::Body.line_height), Field("Under the pointer", LabelWidth, std::move(under)));
        form.caption(inner, items, "A press or a drag in the view uses the tool.");
        return form.bottom();
    });
}

Value TerrainEditor::state(const Terrain& terrain) const {
    Value::Array palette;
    palette.values.reserve(terrain.palette.size());
    for (files::Color listed : terrain.palette) {
        palette.values.emplace_back(files::color_text(listed));
    }
    return json::object({
        {"Tool", Tools[tool]},
        {"Color", static_cast<std::uint32_t>(color)},
        {"Radius", radius},
        {"Painted", terrain.cells.size()},
        {"Palette", std::move(palette)},
    });
}

} // namespace veehiicuul::terrain_editor
