// Cards: the surfaces that group a page's controls, with a title in their
// header. A card is CardBackgroundFillColorDefault with a CardStroke border
// and OverlayCornerRadius corners, on the page's background. A floating card
// is opaque acrylic, raised like a flyout, for over a 3D view. A card with a
// view leaves a hole in its content for a 3D view that the program's render
// draws.
//
// A card's controls are its children, laid out in `area(ui, rect).content`:
//
//     tree.card(ui, "Settings", rect, Card("Settings"), [&](Ui&, Tree<Message>& card, CardArea area) {
//         Form form(area.content);
//         card.add("Volume", form.control(card, "Volume"), Slider<Message>(...));
//     });
//
// A card can size itself to its content with Tree::sized_card: its content
// is laid out from the top of all the height it may take, and the card ends
// where the content does.
//
// This defines Tree::card and Tree::sized_card, which Core/Tree.h declares.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Text.h"

#include <algorithm>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

enum class CardStyle : std::uint8_t {
    Standard,
    // Opaque and raised, for over a 3D view.
    Floating,
    // CardBackgroundFillColorSecondary, for a card inside a card.
    Subtle,
};

// Where a card's parts are.
struct CardArea {
    Rect rect;
    // The header, right of the title and its subtitle, for the card's own
    // commands; empty if the card has no title.
    Rect header;
    // Inside the padding, under the header.
    Rect content;
};

class Card : public Widget<> {
public:
    explicit Card(std::string title);

    // A caption after the title, like a count or what the card is for.
    Card&& subtitle(std::string subtitle) &&;

    Card&& floating() &&;

    Card&& subtle() &&;

    Card&& style(CardStyle style) &&;

    Card&& padding(float padding) &&;

    // A hole in the content for a 3D view, which the program's render draws
    // into. Clear the view to Theme::card_opaque for it to look like the
    // card.
    Card&& with_view() &&;

    // How tall the card is around content `content_height` tall: its
    // padding, and its header if it has a title. For a card that must end
    // where it ends, like at the window's bottom.
    float height_around(float content_height) const;

    // Where the card's parts are, in the rectangle.
    CardArea area(Ui& ui, Rect rect) const;

    std::string_view kind() const override;

    // A named card is a target that scripts see; every card is a surface
    // that the pointer stops at.
    std::optional<Sense> sense() const override;

    bool covers() const override;

    void draw(Draw& ctx) const override;

private:
    // Tree::sized_card reads the padding.
    template <class M> friend class Tree;

    CardArea area_with(Fonts& fonts, Rect rect) const;

    std::string title;
    std::optional<std::string> subtitle_;
    CardStyle style_;
    float padding_;
    // Leaves a hole in the content for a 3D view.
    bool view;
};

// A card in the rectangle, with the children that `build` adds, given
// where the card's parts are: it's called with a Ui&, the card's Tree<M>&
// and its CardArea. A named card is a target of kind "Card".
template <class M> template <class Build> Element<M>& Tree<M>::card(Ui& ui, std::optional<std::string_view> name, Rect rect, Card card, Build&& build) {
    CardArea area = card.area(ui, rect);
    return container(name, rect, std::move(card), [&](Tree<M>& children) { std::forward<Build>(build)(ui, children, area); });
}

// A card at the rectangle's top, as tall as its content: `build` lays
// the content out from the top of the area it's given, which may be as
// tall as the rectangle, and returns where the content ends. Returns
// where the card is.
template <class M> template <class Build> Rect Tree<M>::sized_card(Ui& ui, std::optional<std::string_view> name, Rect rect, Card card, Build&& build) {
    CardArea area = card.area(ui, rect);
    float padding = card.padding_;
    float bottom = rect.y;
    container(name, rect, std::move(card), [&](Tree<M>& children) { bottom = std::forward<Build>(build)(ui, children, area); });
    float padded = std::min(bottom + padding, rect.bottom());
    Rect sized(rect.x, rect.y, rect.width, std::max(padded - rect.y, 0.0f));
    if (!elements.empty()) {
        elements.back().rect_ = sized;
    }
    return sized;
}

} // namespace user_interface
