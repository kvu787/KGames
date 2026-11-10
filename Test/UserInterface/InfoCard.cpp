// The card that test programs float over their scenes: its name, where it
// goes, and F1, which shows and hides it.

#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <optional>
#include <string>
#include <tuple>
#include <variant>
#include <vector>

using user_interface::Context;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Key;
using user_interface::Program;
using user_interface::Rect;
using user_interface::Tree;
using user_interface::Ui;
namespace tokens = user_interface::tokens;

namespace info_card_tests {

class Test : public Program<std::monostate> {
public:
    bool shown = false;

    std::string title() const {
        return "Test";
    }

    std::vector<std::monostate> input(const Ui& ui) const {
        if (InfoCard::toggled(ui.input())) {
            return {std::monostate{}};
        }
        return {};
    }

    void update(std::monostate, Context<std::monostate>&) {
        shown = !shown;
    }

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        if (shown) {
            tree.info_card(ui, InfoCard{
                                   .title = "A test",
                                   .description = "What it shows",
                                   .rows = {{{"Row", "Value"}}},
                                   .keys = {{{"S", "Something"}}},
                               });
        }
        return tree;
    }
};

TEST(the_card_floats_at_the_bottom_left_and_f1_hides_it) {
    Headless app(Test{.shown = true});
    CHECK_EQ(app.label(InfoCard::Name), std::optional<std::string>("A test"));
    Rect card = app.target(InfoCard::Name);
    CHECK_EQ(std::tuple(card.x, card.width), std::tuple(tokens::PagePadding, InfoCard::Width));
    CHECK_EQ(card.bottom(), app.size().height - tokens::PagePadding);
    app.key(Key::F1);
    CHECK(!app.has_target(InfoCard::Name));
    app.key(Key::F1);
    CHECK(app.has_target(InfoCard::Name));
}

} // namespace info_card_tests
