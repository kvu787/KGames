// Tests the interface's rules for input with a small program: which widget
// the pointer is on, presses, which act as the button goes down, keyboard
// focus and the keys that widgets take, popups, menus, dialogs, tooltips,
// notices, editing, and palettes.

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Vec.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <array>
#include <cmath>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

using json::Value;
using user_interface::AppBar;
using user_interface::Button;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::CheckBox;
using user_interface::Clipboard;
using user_interface::Color;
using user_interface::ColorField;
using user_interface::ComboBox;
using user_interface::Context;
using user_interface::Cover;
using user_interface::Dialog;
using user_interface::DialogArea;
using user_interface::DialogResult;
using user_interface::Element;
using user_interface::Event;
using user_interface::Fonts;
using user_interface::Form;
using user_interface::Group;
using user_interface::Headless;
using user_interface::Icon;
using user_interface::Key;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::MenuBar;
using user_interface::MenuItem;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::NumberBox;
using user_interface::Palette;
using user_interface::Point;
using user_interface::Program;
using user_interface::Rect;
using user_interface::Segmented;
using user_interface::Sense;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Slider;
using user_interface::Startup;
using user_interface::Text;
using user_interface::TextBox;
using user_interface::ThemeKind;
using user_interface::TooltipLine;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::View;
using user_interface::ViewInput;
using user_interface::controls::MenuBarHeight;
using user_interface::controls::MenuBarItemMargin;
using user_interface::controls::MenuBarItemPadding;
using user_interface::tokens::AppBarHeight;
using user_interface::tokens::PagePadding;
using user_interface::tokens::TooltipDelay;

namespace interaction_tests {

// Where the program's part of the window starts, under the PerfBar.
constexpr float PerfBar = user_interface::tokens::PerfBarHeight;

// The name of a widget that the interface is on, or none.
using Name = std::optional<std::string_view>;

struct Msg {
    struct Click {
        bool operator==(const Click&) const = default;
    };
    struct Under {
        bool operator==(const Under&) const = default;
    };
    struct Maybe {
        bool operator==(const Maybe&) const = default;
    };
    struct Trapped {
        bool operator==(const Trapped&) const = default;
    };
    struct Shortcut {
        bool operator==(const Shortcut&) const = default;
    };
    struct Late {
        bool operator==(const Late&) const = default;
    };
    struct Text {
        std::string text;
        bool operator==(const Text&) const = default;
    };
    struct Choice {
        size_t choice;
        bool operator==(const Choice&) const = default;
    };
    struct Rows {
        size_t row;
        bool operator==(const Rows&) const = default;
    };
    struct Check {
        bool checked;
        bool operator==(const Check&) const = default;
    };
    struct Level {
        double level;
        bool operator==(const Level&) const = default;
    };
    struct Number {
        double number;
        bool operator==(const Number&) const = default;
    };
    struct Chosen {
        std::string label;
        bool operator==(const Chosen&) const = default;
    };
    struct Area {
        ViewInput input;
        bool operator==(const Area&) const = default;
    };
    struct Ask {
        bool operator==(const Ask&) const = default;
    };
    struct Answer {
        DialogResult answer;
        bool operator==(const Answer&) const = default;
    };
    struct Pick {
        size_t pick;
        bool operator==(const Pick&) const = default;
    };
    struct Shrink {
        size_t shrink;
        bool operator==(const Shrink&) const = default;
    };
    struct Beneath {
        size_t row;
        bool operator==(const Beneath&) const = default;
    };
    struct Replace {
        bool operator==(const Replace&) const = default;
    };
    std::variant<Click, Under, Maybe, Trapped, Shortcut, Late, Text, Choice, Rows, Check, Level, Number, Chosen, Area, Ask, Answer, Pick, Shrink, Beneath, Replace> value;
    template <base::AlternativeOf<Msg> T> Msg(T&& alternative) : value(std::forward<T>(alternative)) {}
    bool operator==(const Msg&) const = default;
};

std::vector<MenuItem<Msg>> recent() {
    return base::vec(MenuItem<Msg>("First").on_choose(Msg::Chosen{"First"}), MenuItem<Msg>("Second").on_choose(Msg::Chosen{"Second"}));
}

std::vector<MenuItem<Msg>> menu() {
    return base::vec(MenuItem<Msg>("Open").accelerator("Ctrl+O").on_choose(Msg::Chosen{"Open"}), MenuItem<Msg>::separator(), MenuItem<Msg>::submenu("Recent", recent()));
}

class Test : public Program<Msg> {
public:
    std::uint32_t clicks = 0;
    std::uint32_t shortcut = 0;
    std::string text;
    size_t choice = 0;
    bool covered = false;
    bool disabled = false;
    std::optional<size_t> rows;
    bool checked = false;
    double level = 0.0;
    std::int32_t number = 0;
    std::vector<std::string> chosen;
    bool dialog = false;
    std::optional<DialogResult> answer;
    // Right presses on the area.
    std::uint32_t secondary = 0;
    // A combo box's items, which a test can take away while its list is open.
    std::vector<std::string> items;
    size_t shrink = 0;
    // Replaces the field's text with a two-byte character, once.
    bool replace = false;
    // A list that a context menu can open over.
    std::optional<size_t> beneath;
    // A combo box in the dialog.
    size_t pick = 0;
    // Presses of D, which the dialog gates from the frame it opens.
    std::uint32_t late = 0;

    std::string title() const {
        return "Test";
    }

    std::vector<Msg> input(const Ui& ui) const {
        std::vector<Msg> messages;
        if (ui.key_pressed(Key::A)) {
            messages.push_back(Msg::Shortcut{});
        }
        if (ui.key_pressed(Key::D)) {
            messages.push_back(Msg::Late{});
        }
        if (replace) {
            messages.push_back(Msg::Replace{});
        }
        return messages;
    }

    void update(Msg message, Context<Msg>&) {
        base::match(
            message, [&](const Msg::Click&) { clicks += 1; }, [&](const Msg::Under&) { covered = true; }, [&](const Msg::Maybe&) { clicks += 100; },
            [&](const Msg::Trapped&) { clicks += 1000; }, [&](const Msg::Shortcut&) { shortcut += 1; },
            // After the dialog's message in the frame's queue, so the frame
            // that opens the dialog gates it.
            [&](const Msg::Late&) {
                if (!dialog) {
                    late += 1;
                }
            },
            [&](Msg::Text& typed) { text = std::move(typed.text); }, [&](const Msg::Choice& picked) { choice = picked.choice; },
            [&](const Msg::Rows& picked) { rows = picked.row; }, [&](const Msg::Check& changed) { checked = changed.checked; },
            [&](const Msg::Level& changed) { level = changed.level; }, [&](const Msg::Number& changed) { number = base::saturating_cast<std::int32_t>(changed.number); },
            [&](Msg::Chosen& picked) { chosen.push_back(std::move(picked.label)); },
            [&](const Msg::Area& area) {
                if (area.input.secondary_pressed) {
                    secondary += 1;
                }
            },
            [&](const Msg::Ask&) { dialog = true; },
            [&](const Msg::Answer& given) {
                answer = given.answer;
                dialog = false;
            },
            [&](const Msg::Pick& picked) { pick = picked.pick; }, [&](const Msg::Shrink& picked) { shrink = picked.shrink; },
            [&](const Msg::Beneath& picked) { beneath = picked.row; },
            [&](const Msg::Replace&) {
                text = "é";
                replace = false;
            });
    }

    Tree<Msg> view(Ui& ui) const {
        Tree<Msg> tree;
        // Under the PerfBar.
        float top = ui.content_top();
        tree.add("Button", Rect(10.0f, top + 10.0f, 100.0f, 32.0f), Button<Msg>("Button").on_press(Msg::Click{})).set_tooltip("A tip");
        tree.add("Field", Rect(10.0f, top + 60.0f, 200.0f, 32.0f), TextBox<Msg>(text).on_change(base::make<Msg::Text>));
        tree.add("Combo", Rect(10.0f, top + 110.0f, 200.0f, 32.0f), ComboBox<Msg>(choice, {{"Zero", "One", "Two"}}).on_select(base::make<Msg::Choice>));
        // A surface over a button covers it from the pointer.
        tree.add("Under", Rect(300.0f, top + 10.0f, 100.0f, 32.0f), Button<Msg>("Under").on_press(Msg::Under{}));
        tree.draw(Rect(290.0f, top, 200.0f, 60.0f), Cover());
        tree.add("Maybe", Rect(10.0f, top + 160.0f, 100.0f, 32.0f), Button<Msg>("Maybe").on_press(Msg::Maybe{})).set_enabled(!disabled);
        // A widget can't be enabled inside a disabled group.
        Tree<Msg> group;
        group.add("Trapped", Rect(120.0f, top + 160.0f, 100.0f, 32.0f), Button<Msg>("Trapped").on_press(Msg::Trapped{})).set_enabled(true);
        tree.push(Element<Msg>::unnamed(Rect(120.0f, top + 160.0f, 100.0f, 32.0f), Group()).enabled(false).children(std::move(group)));
        Rect rows_area(500.0f, top + 10.0f, 200.0f, 100.0f);
        ListView<Msg> list = ListView<Msg>(50, 20.0f).selection(rows).on_select(base::make<Msg::Rows>).labels([](size_t index) { return std::format("Row {}", index); });
        ListRows shown = list.rows(ui, "Rows", rows_area);
        tree.container("Rows", rows_area, std::move(list), [&](Tree<Msg>& list) {
            for (size_t index : shown.indices()) {
                list.draw(shown.content(index), Text::label(std::format("Row {}", index)));
            }
        });
        tree.add("Check", Rect(10.0f, top + 210.0f, 200.0f, 32.0f), CheckBox<Msg>(checked, "Check").on_change(base::make<Msg::Check>));
        tree.add("Level", Rect(10.0f, top + 260.0f, 220.0f, 32.0f), Slider<Msg>(level, 0.0, 1.0).step(0.1).allowed(0.2, 0.8).on_change(base::make<Msg::Level>));
        tree.add("Number", Rect(10.0f, top + 310.0f, 200.0f, 32.0f), NumberBox<Msg>(static_cast<double>(number)).range(0.0, 10.0).on_change(base::make<Msg::Number>));
        tree.add("MenuButton", Rect(10.0f, top + 360.0f, 100.0f, 32.0f), Button<Msg>("Menu").menu(menu()));
        tree.add("Area", Rect(800.0f, top + 400.0f, 200.0f, 200.0f), View<Msg>().on_input(base::make<Msg::Area>)).set_context_menu(recent());
        tree.sized_card(ui, "Sized", Rect(1200.0f, top + 10.0f, 300.0f, 1000.0f), Card("Sized"), [&](Ui&, Tree<Msg>& card, CardArea area) {
            Rect content = area.content;
            card.draw(content.cut_top(20.0f, 8.0f), Text::label("One"));
            card.draw(content.cut_top(20.0f, 0.0f), Text::label("Two"));
            return content.y;
        });
        tree.add("Ask", Rect(10.0f, top + 410.0f, 100.0f, 32.0f), Button<Msg>("Ask").on_press(Msg::Ask{}));
        if (dialog) {
            Dialog<Msg> question = Dialog<Msg>("Sure?", "Really.").primary("Yes").content_height(32.0f).on_result(base::make<Msg::Answer>);
            DialogArea area = question.area(ui);
            tree.container("Question", area.rect, std::move(question), [&](Tree<Msg>& question) {
                question.add("Pick", area.content.top_strip(32.0f), ComboBox<Msg>(pick, {{"Yes", "No"}}).on_select(base::make<Msg::Pick>));
            });
        }
        // Last, so that the tab order of the widgets above stays.
        tree.add("Shrink", Rect(240.0f, top + 110.0f, 200.0f, 32.0f), ComboBox<Msg>(shrink, items).on_select(base::make<Msg::Shrink>));
        Rect beneath_area(1000.0f, top + 400.0f, 300.0f, 100.0f);
        ListView<Msg> under = ListView<Msg>(50, 20.0f).selection(beneath).on_select(base::make<Msg::Beneath>);
        ListRows under_shown = under.rows(ui, "Beneath", beneath_area);
        tree.container("Beneath", beneath_area, std::move(under), [&](Tree<Msg>& list) {
            for (size_t index : under_shown.indices()) {
                list.draw(under_shown.content(index), Text::label(std::format("Row {}", index)));
            }
        });
        return tree;
    }
};

// The member of the interface's state with the name, if it has one.
std::optional<Value> member(const Value& state, std::string_view name) {
    const Value* found = state.get(name);
    return found == nullptr ? std::nullopt : std::optional(*found);
}

template <class P> std::optional<Value> focused(const Headless<P>& app) {
    return member(app.ui.state(), "Focused");
}

TEST(a_press_acts_on_the_frame_that_the_button_goes_down) {
    Headless app(Test{});
    app.press("Button");
    CHECK_EQ(app.program.clicks, 1, "the press, before its release");
    // Sliding off and letting go elsewhere neither takes it back nor
    // presses again.
    Point away(5.0f, 900.0f);
    app.move_point(away).release();
    CHECK_EQ(app.program.clicks, 1);
    // Nor does letting go on the button.
    app.press("Button").release();
    CHECK_EQ(app.program.clicks, 2);
    // A press and its release in one frame are one press.
    app.move_to("Button").send({{Event::ButtonPressed{MouseButton::Left}, Event::ButtonReleased{MouseButton::Left}}});
    CHECK_EQ(app.program.clicks, 3);
}

TEST(a_press_that_opens_a_popup_or_a_dialog_does_nothing_more_as_it_is_let_go_of) {
    Headless app(Test{});
    // The combo box's list opens over it, with the chosen item under the
    // pointer, which the release doesn't choose.
    app.press("Combo");
    CHECK_EQ(app.ui.popup_name(), Name("Combo"));
    app.release();
    CHECK_EQ(std::pair(app.ui.popup_name(), app.program.choice), std::pair(Name("Combo"), size_t{0}));
    // A press on an item chooses it, and closes the list, as it goes down.
    app.press("Combo/2");
    CHECK_EQ(std::pair(app.ui.popup_name(), app.program.choice), std::pair(Name(), size_t{2}));
    app.release();
    // A menu, likewise.
    app.press("MenuButton");
    CHECK_EQ(app.ui.popup_name(), Name("MenuButton/Menu"));
    app.release();
    CHECK_EQ(app.ui.popup_name(), Name("MenuButton/Menu"));
    app.press("MenuButton/Menu/Open");
    CHECK_EQ(std::pair(app.ui.popup_name(), app.program.chosen), std::pair(Name(), std::vector<std::string>{"Open"}));
    app.release();
    // A dialog that a press opens isn't answered by the release over it.
    app.press("Ask");
    CHECK_EQ(member(app.ui.state(), "Dialog"), std::optional(Value(true)));
    app.move_to("Question/Primary").release();
    CHECK_EQ(std::pair(app.program.dialog, app.program.answer), std::pair(true, std::optional<DialogResult>()));
}

TEST(a_covered_widget_is_not_under_the_pointer) {
    Headless app(Test{});
    app.click("Under");
    CHECK(!app.program.covered);
}

TEST(a_disabled_widget_takes_no_input) {
    Headless app(Test{.disabled = true});
    app.click("Maybe");
    CHECK_EQ(app.program.clicks, 0);
    CHECK(!app.is_enabled("Maybe"));
}

TEST(a_widget_cannot_be_enabled_inside_a_disabled_group) {
    Headless app(Test{});
    app.click("Trapped");
    CHECK_EQ(app.program.clicks, 0);
    CHECK(!app.is_enabled("Trapped"));
}

TEST(a_focused_text_field_takes_the_keys_that_type) {
    Headless app(Test{});
    app.key(Key::A);
    CHECK_EQ(app.program.shortcut, 1);
    app.click("Field").type_text("ab cd").key(Key::A);
    CHECK_EQ(app.program.text, "ab cd");
    CHECK_EQ(app.program.shortcut, 1);
    // Editing: a word goes with Ctrl+Backspace, everything with Ctrl+A.
    app.key_with(Modifiers::Control, Key::Backspace);
    CHECK_EQ(app.program.text, "ab ");
    app.key_with(Modifiers::Control, Key::A).key_with(Modifiers::Control, Key::C).key(Key::Backspace);
    CHECK_EQ(app.program.text, "");
    app.key_with(Modifiers::Control, Key::V);
    CHECK_EQ(app.program.text, "ab ");
    // Esc takes focus away, and the keys are the program's again.
    app.key(Key::Escape).key(Key::A);
    CHECK_EQ(app.program.shortcut, 2);
    CHECK_EQ(focused(app), std::optional(Value()));
}

// Cut takes the text away only once it's on the clipboard. Windows'
// clipboard can be unavailable for a moment, while another program has it
// open, and then the selection stays.
TEST(a_cut_keeps_the_text_unless_the_clipboard_takes_it) {
    Headless app(Test{});
    app.click("Field").type_text("keep");
    app.set_clipboard(Clipboard::Unavailable{});
    app.key_with(Modifiers::Control, Key::A).key_with(Modifiers::Control, Key::X);
    CHECK_EQ(app.program.text, "keep");
    app.set_clipboard(Clipboard::Own{});
    app.key_with(Modifiers::Control, Key::A).key_with(Modifiers::Control, Key::X);
    CHECK_EQ(app.program.text, "");
    CHECK_EQ(app.clipboard_text(), std::optional<std::string>("keep"));
}

TEST(tab_goes_through_the_widgets_in_order_and_skips_disabled_ones) {
    Headless app(Test{.disabled = true});
    app.key(Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Button")));
    app.key(Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Field")));
    // Under is covered from the pointer, but not from the keyboard, and
    // Maybe is disabled.
    app.key(Key::Tab).key(Key::Tab).key(Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Rows")));
    app.key_with(Modifiers::Shift, Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Under")));
}

TEST(a_popup_takes_the_pointer_from_what_is_under_it) {
    Headless app(Test{});
    app.click("Combo");
    CHECK_EQ(app.ui.popup_name(), Name("Combo"));
    // The chosen item is where the combo box's text is.
    Rect chosen = app.target("Combo/0");
    CHECK(std::abs(chosen.center().y - app.target("Combo").center().y) < 1.0f);
    app.click("Combo/2");
    CHECK_EQ(app.program.choice, 2);
    CHECK_EQ(app.ui.popup_name(), Name());
    CHECK_EQ(app.label("Combo"), std::optional<std::string>("Two"));
    // Esc closes it without choosing.
    app.click("Combo").key(Key::Up).key(Key::Escape);
    CHECK_EQ(std::pair(app.program.choice, app.ui.popup_name()), std::pair(size_t{2}, Name()));
}

TEST(a_tooltip_waits_for_the_pointer_to_rest_and_sits_above_it) {
    Headless app(Test{});
    app.move_point(Point(60.0f, PerfBar + 30.0f)).wait(TooltipDelay * 0.6);
    CHECK_EQ(app.ui.tooltip_name(), Name());
    app.wait(TooltipDelay);
    CHECK_EQ(app.ui.tooltip_name(), Name("Button"));
    // There's no room above a button at the window's top, so it's below.
    Value state = app.ui.state();
    const Value* tooltip = state.get("Tooltip");
    const Value* area = tooltip == nullptr ? nullptr : tooltip->get("Area");
    CHECK(area != nullptr);
    const Value::Array* sides = base::as<Value::Array>(*area);
    CHECK(sides != nullptr);
    const Value::Number* y = base::as<Value::Number>(sides->values[1]);
    CHECK(y != nullptr);
    CHECK(y->value >= static_cast<double>(PerfBar + 30.0f + 20.0f), "{}", y->value);
    app.press_point(Point(60.0f, PerfBar + 30.0f));
    CHECK_EQ(app.ui.tooltip_name(), Name());
}

TEST(a_list_shows_only_the_rows_in_view_and_the_keys_scroll_to_the_selection) {
    Headless app(Test{});
    CHECK(app.has_target("Rows/4") && !app.has_target("Rows/6"));
    app.click("Rows/1");
    CHECK_EQ(app.program.rows, std::optional<size_t>(1));
    app.key(Key::End);
    CHECK_EQ(app.program.rows, std::optional<size_t>(49));
    CHECK(app.has_target("Rows/49") && !app.has_target("Rows/1"));
    app.scroll("Rows", 2.0f);
    CHECK(app.has_target("Rows/43"));
}

TEST(toggles_sliders_and_number_boxes_change_their_values) {
    Headless app(Test{.level = 0.5});
    app.click("Check");
    CHECK(app.program.checked);
    // The slider stays inside its allowed values, though its rail is wider.
    float left = user_interface::slider_rail(app.target("Level")).first;
    app.press_point(Point(left, app.target("Level").center().y)).release();
    CHECK(std::abs(app.program.level - 0.2) < 1e-9, "{}", app.program.level);
    app.key(Key::Right);
    CHECK(std::abs(app.program.level - 0.3) < 1e-9);
    app.scroll("Level", 0.5f).scroll("Level", 0.5f);
    CHECK(std::abs(app.program.level - 0.4) < 1e-9, "half notches add up: {}", app.program.level);
    CHECK_EQ(app.label("Level"), std::optional<std::string>("0.40"));
    // A number box takes what's typed with Enter, inside its range.
    app.click("Number").key_with(Modifiers::Control, Key::A).type_text("42").key(Key::Enter);
    CHECK_EQ(app.program.number, 10);
    app.click("Number/Down");
    CHECK_EQ(app.program.number, 9);
}

TEST(menus_open_choose_and_close) {
    Headless app(Test{});
    app.click("MenuButton");
    CHECK_EQ(app.ui.popup_name(), Name("MenuButton/Menu"));
    app.click("MenuButton/Menu/Open");
    CHECK_EQ(app.program.chosen, std::vector<std::string>{"Open"});
    CHECK_EQ(app.ui.popup_name(), Name());
    // A submenu opens when the pointer is on its item.
    app.click("MenuButton").move_to("MenuButton/Menu/Recent");
    CHECK(app.has_target("MenuButton/Menu/Recent/Menu/Second"));
    app.click("MenuButton/Menu/Recent/Menu/Second");
    CHECK_EQ(app.program.chosen, (std::vector<std::string>{"Open", "Second"}));
    CHECK(!app.ui.any_popup());
    // A right press opens a context menu where it is, which stays open as
    // the button comes up.
    Point at(850.0f, 450.0f);
    app.press_button(at, MouseButton::Right);
    CHECK_EQ(app.ui.popup_name(), Name("Area/ContextMenu"));
    app.release_button(MouseButton::Right);
    CHECK_EQ(app.ui.popup_name(), Name("Area/ContextMenu"));
    Rect shown = app.target("Area/ContextMenu");
    CHECK(std::abs(shown.x - at.x) < 1.0f && std::abs(shown.y - at.y) < 1.0f);
    app.key(Key::Down).key(Key::Enter);
    CHECK_EQ(app.program.chosen, (std::vector<std::string>{"Open", "Second", "First"}));
    // A press outside a menu closes it, and does nothing else.
    app.click("MenuButton").click("Button");
    CHECK_EQ(std::pair(app.program.clicks, app.ui.any_popup()), std::pair(std::uint32_t{0}, false));
}

TEST(a_dialog_takes_the_input_until_it_is_answered) {
    Headless app(Test{});
    app.click("Ask");
    CHECK_EQ(member(app.ui.state(), "Dialog"), std::optional(Value(true)));
    app.click_point(Point(60.0f, PerfBar + 26.0f));
    CHECK_EQ(app.program.clicks, 0, "the page is under the smoke");
    app.key(Key::A);
    CHECK_EQ(app.program.shortcut, 0, "the program's keys wait too");
    app.key(Key::Escape);
    CHECK_EQ(app.program.answer, std::optional(DialogResult::Close));
    app.click("Ask").click("Question/Primary");
    CHECK_EQ(app.program.answer, std::optional(DialogResult::Primary));
    app.click("Button");
    CHECK_EQ(app.program.clicks, 1);
}

TEST(a_dialog_gates_the_keys_the_frame_it_opens) {
    Headless app(Test{});
    app.key(Key::D);
    CHECK_EQ(app.program.late, 1, "without a dialog the key counts");
    // The press that opens the dialog and a press of D land in one frame.
    // The widgets' messages come before the program's own, so the dialog
    // is open when the key's message is applied, that very frame.
    app.move_to("Ask").frame();
    app.send({{
        Event::ButtonPressed{MouseButton::Left},
        Event::KeyPressed{.key = Key::D, .repeat = false},
        Event::KeyReleased{Key::D},
    }});
    CHECK_EQ(member(app.ui.state(), "Dialog"), std::optional(Value(true)));
    CHECK_EQ(app.program.late, 1);
}

TEST(a_card_sizes_itself_to_its_content) {
    Headless app(Test{});
    // Padding, the header and its gap, two rows, and padding.
    CHECK_EQ(app.target("Sized").height, 16.0f + 32.0f + 8.0f + 20.0f + 8.0f + 20.0f + 16.0f);
    Value state = app.ui.state();
    const Value* targets = state.get("Targets");
    const Value* card = targets == nullptr ? nullptr : targets->get("Sized");
    std::optional<Value> kind = card == nullptr ? std::nullopt : member(*card, "Kind");
    CHECK_EQ(kind, std::optional(Value("Card")));
}

constexpr std::array<Color, 5> Colors_{Color::hex(0xD13438), Color::hex(0xCA5010), Color::hex(0xC19C00), Color::hex(0x107C10), Color::hex(0x0078D4)};

class Colors : public Program<size_t> {
public:
    std::optional<size_t> picked;

    std::string title() const {
        return "Colors";
    }

    void update(size_t index, Context<size_t>&) {
        picked = index;
    }

    Tree<size_t> view(Ui& ui) const {
        Tree<size_t> tree;
        float top = ui.content_top();
        tree.add("Colors", Rect(10.0f, top + 10.0f, 100.0f, 200.0f), Palette<size_t>(picked, Colors_).on_pick([](size_t index) { return index; }));
        return tree;
    }
};

TEST(a_palette_lays_its_swatches_out_in_rows_and_picks_the_one_clicked) {
    Headless app(Colors{});
    app.frame();
    // Swatches a control tall and 8 apart: two fit across 100 pixels, so
    // five make three rows.
    Rect first = app.target("Colors/0");
    Rect second = app.target("Colors/1");
    Rect third = app.target("Colors/2");
    CHECK_EQ(std::pair(first.width, first.height), std::pair(32.0f, 32.0f));
    CHECK_EQ(std::pair(second.x - first.x, second.y), std::pair(40.0f, first.y));
    CHECK_EQ(std::pair(third.x, third.y - first.y), std::pair(first.x, 40.0f));
    CHECK_EQ(Palette<size_t>(std::nullopt, Colors_).height(100.0f), 3.0f * 32.0f + 2.0f * 8.0f);
    CHECK_EQ(app.label("Colors/3"), std::optional<std::string>("#107C10"));
    CHECK_EQ(app.program.picked, std::optional<size_t>());
    app.click("Colors/3");
    CHECK_EQ(app.program.picked, std::optional<size_t>(3));
}

class Notices : public Program<std::monostate> {
public:
    std::string title() const {
        return "Notices";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        tree.shell(ui, AppBar(Icon::Info, "Notices"), false, std::nullopt);
        return tree;
    }
};

// A menu bar in the menu row under the app bar.
class MenuRow : public Program<std::string> {
public:
    std::optional<std::string> chosen;

    std::string title() const {
        return "MenuRow";
    }

    void update(std::string label, Context<std::string>&) {
        chosen = std::move(label);
    }

    Tree<std::string> view(Ui& ui) const {
        Tree<std::string> tree;
        Shell shell = tree.shell(ui, AppBar(Icon::Info, "MenuRow"), true, std::nullopt);
        Rect row = shell.menu_row.value();
        MenuBar<std::string> bar(base::vec(MenuBar<std::string>::Menu("File", base::vec(MenuItem<std::string>("Open").on_choose("Open"), MenuItem<std::string>::separator()))));
        float width = bar.width(ui);
        tree.add("Menus", row.left_strip(width), std::move(bar));
        // The page, as a card that a test can find.
        tree.add("Page", shell.page, Card(""));
        return tree;
    }
};

TEST(the_menu_row_puts_a_menu_bar_in_line_with_the_page_under_the_app_bar) {
    Headless app(MenuRow{});
    app.frame();
    float row_top = PerfBar + AppBarHeight;
    Rect file = app.target("Menus/File");
    // The first menu's name starts on the page's left edge, and its
    // backplate out in the padding.
    CHECK_EQ(std::pair(file.x + MenuBarItemPadding.left, file.y), std::pair(PagePadding, row_top + MenuBarItemMargin.top));
    CHECK_EQ(app.target("Page").y, row_top + MenuBarHeight + PagePadding);
    app.click("Menus/File").click("Menus/File/Menu/Open");
    CHECK_EQ(app.program.chosen, std::optional<std::string>("Open"));
}

TEST(notices_from_the_start_show_and_close) {
    std::filesystem::path folder = std::filesystem::temp_directory_path();
    Startup startup = Startup::for_tests(R"({ "Volume": "loud" })", folder);
    startup.settings.number("Volume", 1.0, [](double) -> std::expected<void, std::string> { return {}; });
    startup.notify(Severity::Error, "Paint.json", "It's broken.");
    Headless<Notices> app = Headless<Notices>::start(std::move(startup), [](Startup&) { return Notices{}; });
    CHECK(app.has_target("Notice/0") && app.has_target("Notice/1"));
    CHECK(app.label("Notice/0").value().contains("Volume must be a number"));
    app.click("Notice/0/Close");
    CHECK_EQ(app.ui.notices().size(), 1);
    CHECK_EQ(app.ui.notices()[0].title, "Paint.json");
}

// What porting the programs found: segments where widgets can't take focus,
// a color field left while it's typed in, a slider's steps of a thousandth,
// and a wide tooltip.
struct PortedMsg {
    struct Segment {
        size_t segment;
        bool operator==(const Segment&) const = default;
    };
    struct Color {
        user_interface::Color color;
        bool operator==(const Color&) const = default;
    };
    struct Other {
        std::string other;
        bool operator==(const Other&) const = default;
    };
    struct Fine {
        double fine;
        bool operator==(const Fine&) const = default;
    };
    std::variant<Segment, Color, Other, Fine> value;
    template <base::AlternativeOf<PortedMsg> T> PortedMsg(T&& alternative) : value(std::forward<T>(alternative)) {}
    bool operator==(const PortedMsg&) const = default;
};

class Ported : public Program<PortedMsg> {
public:
    size_t segment = 0;
    Color color;
    std::string other;
    double fine = 0.0;

    std::string title() const {
        return "Ported";
    }

    void update(PortedMsg message, Context<PortedMsg>&) {
        base::match(
            message, [&](const PortedMsg::Segment& changed) { segment = changed.segment; }, [&](const PortedMsg::Color& changed) { color = changed.color; },
            [&](PortedMsg::Other& changed) { other = std::move(changed.other); }, [&](const PortedMsg::Fine& changed) { fine = changed.fine; });
    }

    Tree<PortedMsg> view(Ui& ui) const {
        Tree<PortedMsg> tree;
        // Under the PerfBar.
        float top = ui.content_top();
        tree.set_focusable(false);
        tree.add("Segments", Rect(10.0f, top + 10.0f, 300.0f, 32.0f), Segmented<PortedMsg>(segment, {{"A", "B", "C"}}).on_select(base::make<PortedMsg::Segment>));
        tree.set_focusable(true);
        tree.add("Hex", Rect(10.0f, top + 60.0f, 200.0f, 32.0f), ColorField<PortedMsg>(color).on_change(base::make<PortedMsg::Color>));
        tree.add("Other", Rect(10.0f, top + 110.0f, 200.0f, 32.0f), TextBox<PortedMsg>(other).on_change(base::make<PortedMsg::Other>));
        tree.add("Fine", Rect(10.0f, top + 160.0f, 300.0f, Slider<PortedMsg>::height(false)), Slider<PortedMsg>(fine, 0.0, 1.0).step(0.001).on_change(base::make<PortedMsg::Fine>));
        std::string_view long_ = "A device's path, like HID#VID_045E&PID_0B13&IG_00#8&2f6c1e3b&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}, and more that goes on after it.";
        tree.add("Wide", Rect(10.0f, top + 600.0f, 100.0f, 32.0f), Button<PortedMsg>("Wide")).set_tooltip_wide(base::vec(TooltipLine::text(std::string(long_))), 640.0f);
        return tree;
    }
};

TEST(segments_take_no_focus_where_widgets_can_t) {
    Headless app(Ported{});
    app.frame();
    app.click("Segments/B");
    CHECK_EQ(app.program.segment, 1);
    CHECK_EQ(app.ui.focused_name(), Name());
}

TEST(a_color_field_takes_what_was_typed_when_it_loses_focus) {
    Headless app(Ported{});
    app.frame();
    app.click("Hex");
    app.key_with(Modifiers::Control, Key::A);
    app.type_text("#FF8000");
    CHECK_EQ(app.program.color, Color());
    app.click("Other");
    CHECK_EQ(app.program.color.to_hex(), "#FF8000");
}

// However focus leaves a number box - Esc, or a press on a view that takes
// focus without being focusable - the typed draft commits on that very
// frame, and no stale draft revives on the next edit.
TEST(a_number_box_commits_its_draft_however_focus_leaves) {
    Headless app(Test{});
    app.frame();
    app.click("Number").key_with(Modifiers::Control, Key::A);
    app.type_text("7");
    CHECK_EQ(app.program.number, 0, "not before it commits");
    app.queue({{Event::KeyPressed{.key = Key::Escape, .repeat = false}}}).frame();
    CHECK_EQ(app.program.number, 7, "Esc committed the draft on its frame");
    app.queue({{Event::KeyReleased{Key::Escape}}}).frame();
    // A fresh edit starts from the value, not a revived draft.
    app.click("Number").key_with(Modifiers::Control, Key::A);
    app.type_text("3");
    app.press("Area");
    CHECK_EQ(app.program.number, 3, "the view's press committed the draft on its frame");
    app.release();
}

// Enter in a dialog goes to the focused widget that takes it, like the
// combo box, and to the default button only when nothing does.
TEST(enter_in_a_dialog_is_the_focused_widgets_before_the_defaults) {
    Headless app(Test{});
    app.click("Ask");
    CHECK(app.has_target("Question"));
    // Tab goes through the dialog's own buttons first, then its content.
    app.key(Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Question/Primary")));
    app.key(Key::Tab);
    app.key(Key::Tab);
    CHECK_EQ(focused(app), std::optional(Value("Pick")));
    app.key(Key::Enter);
    CHECK_EQ(app.program.answer, std::optional<DialogResult>(), "Enter opened the combo, not the default");
    CHECK_NE(member(app.ui.state(), "Popup"), std::optional(Value()));
    app.key(Key::Escape);
    app.key(Key::Enter);
    CHECK_EQ(app.program.answer, std::optional<DialogResult>(), "the combo still has focus and takes Enter");
    app.key(Key::Escape);
    app.key(Key::Escape);
    CHECK_EQ(app.program.answer, std::optional(DialogResult::Close));
    // With nothing focused inside, Enter is the default's, even while a
    // widget under the dialog kept focus.
    app.click("Ask");
    app.key(Key::Enter);
    CHECK_EQ(app.program.answer, std::optional(DialogResult::Primary));
}

// F10's very frame already says the new setting, PerfBar and state alike.
TEST(f10_shows_the_new_vertical_sync_on_its_own_frame) {
    Headless app(Test{});
    app.frame();
    app.queue({{Event::KeyPressed{.key = Key::F10, .repeat = false}}}).frame();
    CHECK(!app.vertical_sync());
    CHECK_EQ(member(app.ui.state(), "VerticalSync"), std::optional(Value(false)));
}

TEST(f10_switches_vertical_sync_even_while_a_text_field_has_focus) {
    Headless app(Test{});
    // It starts on, as every program does.
    CHECK(app.vertical_sync());
    app.key(Key::F10);
    CHECK(!app.vertical_sync());
    CHECK_EQ(member(app.ui.state(), "VerticalSync"), std::optional(Value(false)));
    // A text field takes what's typed, but not the function keys.
    app.click("Field").key(Key::F10);
    CHECK(app.vertical_sync());
    CHECK_EQ(member(app.ui.state(), "VerticalSync"), std::optional(Value(true)));
    CHECK(app.program.text.empty());
}

TEST(f9_switches_the_theme_even_while_a_text_field_has_focus) {
    Headless app(Test{});
    // It starts light, as every program does.
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    app.key(Key::F9);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Dark);
    CHECK_EQ(member(app.ui.state(), "Theme"), std::optional(Value("Dark")));
    // A text field takes what's typed, but not the function keys.
    app.click("Field").key(Key::F9);
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
    CHECK_EQ(member(app.ui.state(), "Theme"), std::optional(Value("Light")));
    CHECK(app.program.text.empty());
}

// Notes the theme that each frame's view is in, and switches it with
// toggle_theme when asked, as a menu item does.
class Themes : public Program<std::monostate> {
public:
    mutable std::vector<ThemeKind> frames;

    std::string title() const {
        return "Themes";
    }

    void update(std::monostate, Context<std::monostate>& ui) {
        ui.toggle_theme();
    }

    Tree<std::monostate> view(Ui& ui) const {
        frames.push_back(ui.theme().kind);
        return Tree<std::monostate>();
    }
};

TEST(a_frame_is_drawn_whole_in_the_theme_its_messages_leave) {
    Headless app(Themes{});
    // F9 switches the theme from the frame that it's pressed in.
    app.send({{Event::KeyPressed{.key = Key::F9, .repeat = false}}});
    std::vector<ThemeKind> frames = app.program.frames;
    CHECK_EQ(std::vector<ThemeKind>(frames.end() - 2, frames.end()), (std::vector<ThemeKind>{ThemeKind::Light, ThemeKind::Dark}));
    app.send({{Event::KeyReleased{Key::F9}}});
    // A message's toggle_theme switches it for the frame that applies the
    // message, which is drawn in the new theme throughout.
    app.send_message(std::monostate{});
    CHECK(!app.program.frames.empty());
    CHECK_EQ(app.program.frames.back(), ThemeKind::Light);
    // Two toggles in one frame leave the theme as it was.
    app.queue_message(std::monostate{}).queue_message(std::monostate{}).frame();
    CHECK_EQ(app.ui.theme().kind, ThemeKind::Light);
}

TEST(a_slider_s_steps_land_on_their_decimals) {
    Headless app(Ported{});
    app.frame();
    for (int turn = 0; turn < 107; turn++) {
        app.scroll("Fine", 1.0f);
    }
    CHECK_EQ(app.program.fine, 0.107);
}

TEST(headed_columns_sit_side_by_side_under_their_headers) {
    Form form(Rect(400.0f, PerfBar + 10.0f, 400.0f, 200.0f));
    Tree<std::monostate> tree;
    auto [left, right] = form.headed_columns<2>(tree, {"Left", "Right"}, 32.0f);
    float header = user_interface::tokens::Body.line_height + user_interface::controls::TextBoxHeaderGap;
    CHECK_EQ(left.y, PerfBar + 10.0f + header);
    CHECK_EQ(std::pair(left.height, right.y), std::pair(32.0f, left.y));
    CHECK_EQ(right.x - left.right(), user_interface::tokens::CardPadding);
    CHECK_EQ(right.right(), 800.0f);
}

TEST(a_gamepad_s_combination_has_a_keycap_for_each_part) {
    Headless app(Ported{});
    Fonts& fonts = app.ui.fonts();
    float keys = user_interface::Keycaps::width(fonts, "RB+Left stick");
    float parts = user_interface::Keycaps::key_width(fonts, "RB") + user_interface::Keycaps::key_width(fonts, "Left stick");
    CHECK(keys > parts && keys < parts + 16.0f, "{} {}", keys, parts);
}

TEST(a_wide_tooltip_is_wider_than_winui_s) {
    Headless app(Ported{});
    app.frame();
    app.move_to("Wide").wait(TooltipDelay * 2.0);
    Value state = app.ui.state();
    const Value* tooltip = state.get("Tooltip");
    const Value* area = tooltip == nullptr ? nullptr : tooltip->get("Area");
    const Value::Array* sides = area == nullptr ? nullptr : base::as<Value::Array>(*area);
    CHECK(sides != nullptr, "no tooltip");
    const Value::Number* width = base::as<Value::Number>(sides->values[2]);
    CHECK(width != nullptr);
    CHECK(width->value > 320.0 && width->value <= 640.0, "{}", width->value);
}

TEST(a_right_press_that_closes_a_menu_goes_nowhere_else) {
    Headless app(Test{});
    app.click("MenuButton");
    CHECK(app.ui.any_popup());
    app.right_click("Area");
    CHECK(!app.ui.any_popup());
    CHECK_EQ(app.program.secondary, 0, "the press that closed the menu");
    app.right_click("Area");
    CHECK_EQ(app.program.secondary, 1);
}

TEST(enter_goes_to_an_open_list_in_a_dialog_not_to_its_default_button) {
    Headless app(Test{});
    app.click("Ask").click("Pick");
    CHECK(app.ui.any_popup());
    app.key(Key::Down).key(Key::Enter);
    CHECK_EQ(std::pair(app.program.pick, app.program.answer), std::pair(size_t{1}, std::optional<DialogResult>()));
    CHECK(!app.ui.any_popup());
    app.click("Question/Primary");
    CHECK_EQ(app.program.answer, std::optional(DialogResult::Primary));
}

TEST(a_list_whose_items_go_while_it_is_open_closes) {
    Headless app(Test{.items = {"Only"}});
    app.click("Shrink");
    CHECK(app.ui.any_popup());
    app.program.items.clear();
    // The frame that shows no items, and the one whose input finds the
    // list empty and closes it.
    app.frame().frame();
    CHECK(!app.ui.any_popup());
    CHECK_EQ(app.label("Shrink"), std::optional<std::string>(""));
}

TEST(a_caret_in_text_the_program_replaced_lands_on_a_character) {
    Headless app(Test{});
    app.click("Field").type_text("abc").key(Key::Left).key(Key::Left);
    // The caret was inside the two-byte character's bytes.
    app.program.replace = true;
    app.frame().type_text("x");
    CHECK_EQ(app.program.text, "xé");
}

TEST(the_wheel_over_a_menu_leaves_the_list_under_it_alone) {
    Headless app(Test{});
    Point at(999.0f, PerfBar + 450.0f);
    app.press_button(at, MouseButton::Right).release_button(MouseButton::Right);
    CHECK_EQ(app.ui.popup_name(), Name("Area/ContextMenu"));
    Point over = app.target("Area/ContextMenu/First").center();
    CHECK(app.target("Beneath").contains(over), "the menu opens over the list");
    app.scroll_point(over, -2.0f);
    CHECK(app.has_target("Beneath/0"), "the list didn't scroll under the menu");
    app.key(Key::Escape);
    app.scroll_point(over, -2.0f);
    CHECK(!app.has_target("Beneath/0"), "the list scrolls once the menu is gone");
}

TEST(a_number_box_takes_the_wheel_only_with_focus_and_the_pointer_on_it) {
    Headless app(Test{});
    app.scroll("Number", 1.0f);
    CHECK_EQ(app.program.number, 0, "without focus");
    app.click("Number").scroll("Check", 1.0f);
    CHECK_EQ(app.program.number, 0, "with the pointer elsewhere");
    app.scroll("Number", 1.0f);
    CHECK_EQ(app.program.number, 1);
}

TEST(a_message_s_effects_all_show_on_its_frame) {
    // Every widget shows what the input did, whatever order the tree lists
    // them in: the field's text, which the view builds from the model, is
    // the typed text on the very frame that types it.
    Headless app(Test{});
    app.click("Field").type_text("abc");
    CHECK_EQ(app.label("Field"), std::optional<std::string>("abc"));
    CHECK_EQ(app.messages_applied(), 1);
}

TEST(a_view_that_sends_nothing_applies_no_messages) {
    Headless app(Test{});
    app.frame();
    CHECK_EQ(app.messages_applied(), 0);
    (void)Sense::None;
}

// A button whose menu loses its last items when asked.
class Shrinking : public Program<std::monostate> {
public:
    bool few = false;

    std::string title() const {
        return "Shrinking";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        std::vector<MenuItem<std::monostate>> items = base::vec(MenuItem<std::monostate>("One"));
        if (!few) {
            items.push_back(MenuItem<std::monostate>("Two"));
            items.push_back(MenuItem<std::monostate>("Three"));
        }
        tree.add("Menu", Rect(10.0f, ui.content_top() + 10.0f, 100.0f, 32.0f), Button<std::monostate>("Menu").menu(std::move(items)));
        return tree;
    }
};

// A menu keeps which item is highlighted from frame to frame. If the item
// is gone from the menu by the frame that reads the keys for it, the program
// stops, as it does in Rust, where the items' bounds are checked, instead of
// reading past the items.
TEST(a_menu_whose_highlighted_item_goes_while_it_is_open_stops_the_program) {
    Headless app(Shrinking{});
    app.click("Menu").move_to("Menu/Menu/Three");
    app.program.few = true;
    // The frame that shows the shorter menu, and the one whose input finds
    // the highlight past its end.
    app.frame();
    CHECK_THROWS(app.frame());
}

} // namespace interaction_tests
