// A palette: swatches of colors in rounded squares, left to right and then
// in rows, of which one may be picked, which shows an accent ring around it.
// Each swatch is a target named "<name>/<index>", labeled with its color.

#pragma once

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cstddef>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

template <class M> class Palette : public Widget<M> {
public:
    // Swatches a control tall, a small space apart.
    Palette(std::optional<size_t> picked, std::span<const Color> colors) : picked(picked), colors(colors.begin(), colors.end()) {}
    Palette(const Palette&) = delete;
    Palette& operator=(const Palette&) = delete;
    Palette(Palette&&) = default;
    Palette& operator=(Palette&&) = default;

    // What a click on a swatch sends, with its index: `reaction` is called
    // with a size_t, and returns the message.
    template <class F> Palette&& on_pick(F&& reaction) && {
        on_pick_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // Each swatch's side.
    Palette&& size(float size) && {
        size_ = size;
        return std::move(*this);
    }

    // The space between swatches, across and down.
    Palette&& gap(float gap) && {
        gap_ = gap;
        return std::move(*this);
    }

    // How tall its rows of swatches are in the width.
    float height(float width) const {
        size_t columns = this->columns(width);
        float rows = static_cast<float>(std::max<size_t>(colors.size() / columns + (colors.size() % columns != 0 ? 1 : 0), 1));
        return rows * size_ + (rows - 1.0f) * gap_;
    }

    std::string_view kind() const override {
        return "Palette";
    }

    std::optional<std::string> label() const override {
        if (picked && *picked < colors.size()) {
            return colors[*picked].to_hex();
        }
        return std::nullopt;
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        for (size_t index = 0; index < colors.size(); index++) {
            ctx.part(Part::Index{index}, std::to_string(index), swatch(rect, index), Sense::Button, "Swatch", colors[index].to_hex());
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        const Part* activated = interaction.activated_part();
        const Part::Index* part = activated != nullptr ? base::as<Part::Index>(*activated) : nullptr;
        if (part != nullptr && picked != part->index && on_pick_) {
            ctx.send(on_pick_(part->index));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        const Look& look = ctx.look();
        for (size_t index = 0; index < colors.size(); index++) {
            Color color = colors[index];
            Part part = Part::Index{index};
            Rect area = swatch(rect, index);
            bool is_picked = picked == index;
            {
                Painter painter = ctx.painter();
                const Theme& theme = painter.theme();
                // A picked swatch's color is inset, inside its ring.
                Rect inner = area.inset(is_picked ? 3.0f : 0.0f);
                painter.shape(inner, std::max(tokens::RadiusSmall, tokens::RadiusControl - 1.0f), color, Border(theme.control_stroke_secondary, 1.0f));
                if (is_picked) {
                    painter.shape(area, tokens::RadiusControl + 1.0f, Color::Transparent, Border(theme.accent, 2.0f));
                } else if (look.hovered(part)) {
                    painter.shape(area, tokens::RadiusControl, Color::Transparent, Border(theme.control_strong_stroke, 1.0f));
                }
            }
            bool focused = look.focused && look.focused_part == part;
            ctx.focus_visual_if(focused, area, tokens::RadiusControl);
        }
    }

private:
    // How many swatches a row of the width has.
    size_t columns(float width) const {
        return std::max<size_t>(base::saturating_cast<size_t>((width + gap_) / (size_ + gap_)), 1);
    }

    Rect swatch(Rect rect, size_t index) const {
        size_t columns = this->columns(rect.width);
        size_t row = index / columns;
        size_t column = index % columns;
        return Rect(rect.x + static_cast<float>(column) * (size_ + gap_), rect.y + static_cast<float>(row) * (size_ + gap_), size_, size_);
    }

    std::optional<size_t> picked;
    std::vector<Color> colors;
    float size_ = tokens::ControlHeight;
    float gap_ = tokens::SpaceS;
    Reaction<size_t, M> on_pick_;
};

} // namespace user_interface
