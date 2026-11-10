// The demo, as a program that Library/UserInterface runs: the editor, its
// cards, and the sphere's preview, which render draws into the preview
// card's view before the interface draws over the rest of the frame.

#pragma once

#include "Base/Sum.h"
#include "Graphics/Graphics.h"
#include "SimplePaint/Material.h"
#include "SimplePaintDemo/Editor.h"
#include "SimplePaintDemo/Preview.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"
#include "Json/Json.h"

#include <cstddef>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace simple_paint_demo::demo {

inline constexpr std::string_view Title = "SimplePaint Demo";

// What the input on the cards means.
struct Message {
    // A key's command: select, nudge or save.
    struct Command {
        editor::Command command;
        bool operator==(const Command&) const = default;
    };
    struct Reset {
        bool operator==(const Reset&) const = default;
    };
    struct Save {
        bool operator==(const Save&) const = default;
    };
    // A press on a setting's row, by the setting's index.
    struct Select {
        size_t index;
        bool operator==(const Select&) const = default;
    };
    // A setting's slider moved, dragged or by the wheel.
    struct Set {
        size_t index;
        double value;
        bool operator==(const Set&) const = default;
    };
    // A press took hold of a setting's slider, or let go of it.
    struct Drag {
        size_t index;
        bool held;
        bool operator==(const Drag&) const = default;
    };
    std::variant<Command, Reset, Save, Select, Set, Drag> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Message&) const = default;
};

class Demo : public user_interface::Program<Message> {
public:
    editor::Editor editor;
    // Where Ctrl+S and Save write the material: the demo's folder, or a
    // script's output folder.
    std::filesystem::path paint_path;
    // The cards whose content didn't fit in them in the last frame, which
    // the tests check is none of them. The view notes it, as the one place
    // that lays the cards out.
    mutable std::vector<std::string_view> overflow;
    // Where the last frame's preview card has its view, which render fills
    // with the sphere, and the card's color, opaque, around the sphere. The
    // view notes them, as the one place that knows the layout and the theme.
    mutable std::optional<user_interface::Rect> view_;
    mutable user_interface::Color background = user_interface::Color::White;

    Demo(const simple_paint::Parameters& parameters, std::filesystem::path paint_path);
    Demo(const Demo&) = delete;
    Demo& operator=(const Demo&) = delete;
    Demo(Demo&&) = default;
    Demo& operator=(Demo&&) = default;

    // The demo, with the material in Paint.json in its data folder, if it's
    // there, which a notice says what's wrong with if it can't be used. A
    // scripted run starts from SimplePaint's default material, so that its
    // scripts start the same every time.
    static Demo start(user_interface::Startup& startup);

    std::string title() const;

    // Each key pressed this frame, in order, with its repeats, unless a
    // widget with focus takes it, like a button takes Space.
    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    void prepare(graphics::Gpu& gpu);

    void render(graphics::Gpu& gpu, const graphics::Frame& frame);

    json::Value state() const;

private:
    // Made by prepare.
    std::optional<preview::Preview> preview;
};

} // namespace simple_paint_demo::demo
