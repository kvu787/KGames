#include "MeshModeler/Cards.h"

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Vec.h"
#include "MeshModeler/Camera.h"
#include "MeshModeler/Document.h"
#include "MeshModeler/Editing.h"
#include "MeshModeler/Viewport.h"
#include "SimplePaint/Material.h"
#include "UserInterface/UserInterface.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <format>
#include <initializer_list>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace mesh_modeler::cards {

namespace tokens = user_interface::tokens;

using camera::Viewpoint;
using document::Document;
using editing::AllModes;
using editing::AllPrimitives;
using editing::AllTools;
using editing::Mode;
using editing::Primitive;
using editing::Tool;
using modeler::Command;
using modeler::MaximumLevels;
using modeler::Message;
using modeler::Modeler;
using modeler::PaintEdit;
using modeler::Prompt;
using modeler::Title;
using polygon_mesh::Mesh;
using polygon_mesh::Selection;
using user_interface::AppBar;
using user_interface::Border;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Color;
using user_interface::ColorField;
using user_interface::ColorPicker;
using user_interface::ComboBox;
using user_interface::Cursor;
using user_interface::Dialog;
using user_interface::DialogArea;
using user_interface::Field;
using user_interface::Form;
using user_interface::Icon;
using user_interface::Insets;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::MenuBar;
using user_interface::MenuItem;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Segmented;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Size;
using user_interface::Slider;
using user_interface::Status;
using user_interface::Text;
using user_interface::TextBox;
using user_interface::TextColor;
using user_interface::ToggleSwitch;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::View;
using viewport::Shader;

namespace {

// Rows of the paints' list that show at once.
constexpr float PaintRows = 4.0f;
constexpr float RowHeight = 40.0f;
constexpr float PickerWidth = 176.0f;
constexpr float Swatch = 20.0f;
constexpr float FieldLabelWidth = 128.0f;
// The Open dialog's list.
constexpr float OpenRows = 6.0f;

MenuItem<Message> item(const Modeler& modeler, Command command, std::optional<Icon> icon) {
    MenuItem<Message> made = MenuItem<Message>(std::string(command.label())).enabled(modeler.can(command)).on_choose(Message::Command{command});
    if (std::optional<std::string_view> shortcut = command.shortcut()) {
        made = MenuItem<Message>(std::move(made)).accelerator(std::string(*shortcut));
    }
    if (icon) {
        made = MenuItem<Message>(std::move(made)).icon(*icon);
    }
    if (std::optional<bool> checked = modeler.checked(command)) {
        made = MenuItem<Message>(std::move(made)).checked(*checked);
    }
    return made;
}

AppBar app_bar(const Modeler& modeler) {
    std::string_view dirty = modeler.document.dirty ? " · Not saved" : "";
    return AppBar(Icon::Edit, std::string(Title)).subtitle(std::format("{}{}", modeler.document.name, dirty));
}

void menu_bar(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect bar) {
    const Modeler& m = modeler;
    std::vector<MenuItem<Message>> file = base::vec(item(m, Command::New{}, Icon::Document), item(m, Command::Open{}, Icon::OpenFile), item(m, Command::Save{}, Icon::Save),
                                                    item(m, Command::SaveAs{}, Icon::SaveAs), MenuItem<Message>::separator(), item(m, Command::ExportObj{}, Icon::Export));
    std::vector<MenuItem<Message>> edit = base::vec(
        item(m, Command::Undo{}, Icon::Undo), item(m, Command::Redo{}, Icon::Redo), MenuItem<Message>::separator(), item(m, Command::SelectAll{}, Icon::SelectAll),
        item(m, Command::SelectNone{}, std::nullopt), item(m, Command::InvertSelection{}, std::nullopt), MenuItem<Message>::separator(),
        item(m, Command::VertexMode{}, std::nullopt), item(m, Command::EdgeMode{}, std::nullopt), item(m, Command::FaceMode{}, std::nullopt), MenuItem<Message>::separator(),
        item(m, Command::SelectTool{}, std::nullopt), item(m, Command::PaintTool{}, Icon::Brush), item(m, Command::LoopCutTool{}, Icon::Cut));
    std::vector<MenuItem<Message>> primitives;
    primitives.reserve(AllPrimitives.size());
    for (Primitive primitive : AllPrimitives) {
        primitives.push_back(item(m, Command::Add{primitive}, std::nullopt));
    }
    std::vector<MenuItem<Message>> mesh =
        base::vec(item(m, Command::Move{}, Icon::Move), item(m, Command::Rotate{}, Icon::Rotate), item(m, Command::Scale{}, std::nullopt), MenuItem<Message>::separator(),
                  item(m, Command::Extrude{}, std::nullopt), item(m, Command::Inset{}, std::nullopt), item(m, Command::Fill{}, Icon::Fill), item(m, Command::Merge{}, std::nullopt),
                  item(m, Command::Subdivide{}, Icon::Tiles), MenuItem<Message>::separator(), item(m, Command::Delete{}, Icon::Delete),
                  item(m, Command::DeleteFaces{}, std::nullopt), MenuItem<Message>::separator(), item(m, Command::FlipNormals{}, std::nullopt),
                  item(m, Command::RecalculateNormals{}, std::nullopt), MenuItem<Message>::separator(), item(m, Command::Crease{}, std::nullopt),
                  item(m, Command::ClearCrease{}, std::nullopt), MenuItem<Message>::separator(), MenuItem<Message>::submenu("Add", std::move(primitives)).icon(Icon::Add));
    std::vector<MenuItem<Message>> paint = base::vec(item(m, Command::AssignPaint{}, Icon::Brush), item(m, Command::PaintTool{}, std::nullopt), MenuItem<Message>::separator(),
                                                     item(m, Command::AddPaint{}, Icon::Add), item(m, Command::RemovePaint{}, Icon::Remove));
    std::vector<MenuItem<Message>> viewpoints;
    viewpoints.reserve(6);
    for (Viewpoint viewpoint : {Viewpoint::Front, Viewpoint::Back, Viewpoint::Right, Viewpoint::Left, Viewpoint::Top, Viewpoint::Bottom}) {
        viewpoints.push_back(item(m, Command::View{viewpoint}, std::nullopt));
    }
    std::vector<MenuItem<Message>> levels;
    levels.reserve(MaximumLevels + 1);
    for (std::uint32_t level = 0; level <= MaximumLevels; level++) {
        levels.push_back(item(m, Command::Level{level}, std::nullopt));
    }
    std::vector<MenuItem<Message>> view = base::vec(item(m, Command::FrameAll{}, Icon::Home), item(m, Command::FrameSelected{}, Icon::Search),
                                                    MenuItem<Message>::submenu("Look from", std::move(viewpoints)).icon(Icon::Camera), MenuItem<Message>::separator(),
                                                    MenuItem<Message>::submenu("Subdivision", std::move(levels)).icon(Icon::Tiles), item(m, Command::ToggleShader{}, std::nullopt),
                                                    item(m, Command::ToggleSmooth{}, std::nullopt), item(m, Command::ToggleXray{}, std::nullopt),
                                                    item(m, Command::ToggleGrid{}, std::nullopt), MenuItem<Message>::separator(), item(m, Command::ToggleTheme{}, Icon::Moon),
                                                    item(m, Command::VerticalSync{}, std::nullopt).checked(ui.vertical_sync()), item(m, Command::Fullscreen{}, Icon::FullScreen));
    std::vector<MenuBar<Message>::Menu> menus =
        base::vec(MenuBar<Message>::Menu("File", std::move(file)), MenuBar<Message>::Menu("Edit", std::move(edit)), MenuBar<Message>::Menu("Mesh", std::move(mesh)),
                  MenuBar<Message>::Menu("Paint", std::move(paint)), MenuBar<Message>::Menu("View", std::move(view)));
    MenuBar<Message> menu(std::move(menus));
    float width = menu.width(ui);
    tree.add("Menu", Rect(bar.x, bar.y, width, bar.height), std::move(menu));
}

std::vector<Status> status_items(const Modeler& modeler) {
    std::optional<std::pair<Severity, std::string>> message;
    if (modeler.editor.message) {
        message = std::pair(Severity::Warning, *modeler.editor.message);
    } else {
        message = modeler.message;
    }
    std::vector<Status> items;
    if (modeler.editor.operation) {
        const editing::Operation& operation = *modeler.editor.operation;
        std::string amount = std::format("{} · {} · {}", operation.name, editing::name(operation.axis), operation.amount);
        items = base::vec(Status::shortcut("X", "X axis"), Status::shortcut("Y", "Y axis"), Status::shortcut("Z", "Z axis"), Status::shortcut("N", "Normal"),
                          Status::shortcut("Ctrl", "Snap"), Status::shortcut("Enter", "Keep"), Status::shortcut("Esc", "Cancel"), Status(Status::Spacer{}),
                          Status::text(std::move(amount)));
    } else {
        items = base::vec(Status::shortcut("1", "Vertices"), Status::shortcut("2", "Edges"), Status::shortcut("3", "Faces"), Status::shortcut("G", "Move"),
                          Status::shortcut("R", "Rotate"), Status::shortcut("S", "Scale"), Status::shortcut("E", "Extrude"), Status::shortcut("Ctrl+R", "Loop cut"),
                          Status::shortcut("Alt", "Click a loop"), Status::shortcut("Middle", "Orbit"), Status::shortcut("Shift+Middle", "Pan"), Status(Status::Spacer{}));
        if (message) {
            items.push_back(Status::indicator(message->first, std::move(message->second)));
        }
    }
    return items;
}

// A button for a command, with its shortcut in its tooltip.
void command_button(const Modeler& modeler, Tree<Message>& tree, Rect rect, Command command, std::string_view label, std::optional<Icon> icon) {
    Button<Message> button = Button<Message>(std::string(label)).on_press(Message::Command{command});
    if (icon) {
        button = Button<Message>(std::move(button)).icon(*icon);
    }
    std::string tip;
    if (std::optional<std::string_view> shortcut = command.shortcut()) {
        tip = std::format("{} ({})", command.label(), *shortcut);
    } else {
        tip = std::string(command.label());
    }
    tree.add(command.label(), rect, std::move(button)).set_enabled(modeler.can(command)).set_tooltip(std::move(tip));
}

Rect select_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    return tree.sized_card(ui, std::nullopt, rect, Card("Select"), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form(area.content);
        std::vector<std::string_view> modes;
        modes.reserve(AllModes.size());
        for (Mode mode : AllModes) {
            switch (mode) {
            case Mode::Vertices:
                modes.push_back("Vertices");
                break;
            case Mode::Edges:
                modes.push_back("Edges");
                break;
            case Mode::Faces:
                modes.push_back("Faces");
                break;
            }
        }
        auto found_mode = std::ranges::find(AllModes, modeler.editor.mode);
        size_t mode = found_mode != AllModes.end() ? static_cast<size_t>(found_mode - AllModes.begin()) : 2;
        Rect mode_row = form.control(card, "Selection mode");
        card.add("Mode", mode_row, Segmented<Message>(mode, modes).on_select([](size_t index) {
            return Message::Command{std::array<Command, 3>{Command::VertexMode{}, Command::EdgeMode{}, Command::FaceMode{}}[index]};
        }));
        std::array<std::pair<Icon, std::string_view>, 3> tools{{{Icon::Edit, "Select"}, {Icon::Brush, "Paint"}, {Icon::Cut, "Loop cut"}}};
        auto found_tool = std::ranges::find(AllTools, modeler.editor.tool);
        size_t tool = found_tool != AllTools.end() ? static_cast<size_t>(found_tool - AllTools.begin()) : 0;
        Rect tool_row = form.control(card, "Tool");
        card.add("Tool", tool_row, Segmented<Message>(tool, tools).on_select([](size_t index) {
            return Message::Command{std::array<Command, 3>{Command::SelectTool{}, Command::PaintTool{}, Command::LoopCutTool{}}[index]};
        }));
        auto [all, none, invert] = form.columns<3>(tokens::ControlHeight);
        command_button(modeler, card, all, Command::SelectAll{}, "All", std::nullopt);
        command_button(modeler, card, none, Command::SelectNone{}, "None", std::nullopt);
        command_button(modeler, card, invert, Command::InvertSelection{}, "Invert", std::nullopt);
        return form.bottom();
    });
}

Rect edit_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    return tree.sized_card(ui, std::nullopt, rect, Card("Edit"), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form = Form(area.content).gap(tokens::SpaceS);
        struct Edit {
            Command command;
            std::string_view label;
            std::optional<Icon> icon;
        };
        const std::array<Edit, 2> rows[] = {
            {Edit{Command::Move{}, "Move", Icon::Move}, Edit{Command::Rotate{}, "Rotate", Icon::Rotate}},
            {Edit{Command::Scale{}, "Scale", std::nullopt}, Edit{Command::Extrude{}, "Extrude", std::nullopt}},
            {Edit{Command::Inset{}, "Inset", std::nullopt}, Edit{Command::LoopCutTool{}, "Loop cut", Icon::Cut}},
            {Edit{Command::Fill{}, "Fill", Icon::Fill}, Edit{Command::Merge{}, "Merge", std::nullopt}},
            {Edit{Command::Subdivide{}, "Subdivide", Icon::Tiles}, Edit{Command::Delete{}, "Delete", Icon::Delete}},
            {Edit{Command::FlipNormals{}, "Flip", std::nullopt}, Edit{Command::RecalculateNormals{}, "Recalculate", std::nullopt}},
        };
        for (const std::array<Edit, 2>& row : rows) {
            std::array<Rect, 2> columns = form.columns<2>(tokens::ControlHeight);
            for (size_t index = 0; index < row.size(); index++) {
                command_button(modeler, card, columns[index], row[index].command, row[index].label, row[index].icon);
            }
        }
        form.space(tokens::SpaceS);
        Rect slider = form.take(Slider<Message>::height(true));
        card.add("Sharpness", slider, Slider<Message>(modeler.crease, 0.0, 10.0).step(0.25).header("Crease sharpness").show_value(2).on_change(base::make<Message::Crease>));
        auto [crease, clear] = form.columns<2>(tokens::ControlHeight);
        command_button(modeler, card, crease, Command::Crease{}, "Crease", std::nullopt);
        command_button(modeler, card, clear, Command::ClearCrease{}, "Clear crease", std::nullopt);
        return form.bottom();
    });
}

Rect add_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    return tree.sized_card(ui, std::nullopt, rect, Card("Add"), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form(area.content);
        Rect row = form.control(card, "Primitive");
        Rect button = row.cut_right(96.0f, tokens::SpaceS);
        std::vector<std::string_view> names;
        names.reserve(AllPrimitives.size());
        for (Primitive primitive : AllPrimitives) {
            names.push_back(Command(Command::Add{primitive}).label());
        }
        card.add("Primitive", row, ComboBox<Message>(modeler.primitive, names).on_select(base::make<Message::Primitive>));
        Primitive primitive = AllPrimitives[std::min(modeler.primitive, AllPrimitives.size() - 1)];
        card.add("AddPrimitive", button, Button<Message>("Add").icon(Icon::Add).on_press(Message::Command{Command::Add{primitive}}))
            .set_tooltip("Adds it at the origin, in the paint picked, and selects it");
        return form.bottom();
    });
}

void model_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    tree.card(ui, std::nullopt, rect, Card("Model"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        const Document& document = modeler.document;
        const Mesh& mesh = document.mesh;
        const Selection& selection = document.selection;
        size_t surface_faces = modeler.surface() != nullptr ? modeler.surface()->mesh.faces.size() : mesh.faces.size();
        std::array<std::pair<std::string_view, std::string>, 9> fields{{
            {"Name", document.name},
            {"Saved", document.dirty ? "No" : "Yes"},
            {"Vertices", std::format("{} · {} selected", mesh.positions.size(), selection.vertices.size())},
            {"Edges", std::format("{} · {} selected", document.edges()->size(), selection.edges.size())},
            {"Faces", std::format("{} · {} selected", mesh.faces.size(), selection.faces.size())},
            {"Creased edges", std::to_string(mesh.creases.size())},
            {"Surface faces", std::to_string(surface_faces)},
            {"Undo", std::string(document.undo_name().value_or("Nothing"))},
            {"Redo", std::string(document.redo_name().value_or("Nothing"))},
        }};
        for (auto& [label, value] : fields) {
            if (area.height < tokens::Body.line_height) {
                break;
            }
            Rect row = area.cut_top(tokens::Body.line_height, tokens::SpaceXs);
            card.draw(row, Field(std::string(label), FieldLabelWidth, std::move(value)));
        }
    });
}

void left_column(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect column) {
    float bottom = select_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, tokens::CardGap);
    bottom = edit_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, tokens::CardGap);
    bottom = add_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, tokens::CardGap);
    model_card(modeler, ui, tree, column);
}

std::vector<MenuItem<Message>> context_items(const Modeler& modeler) {
    const Modeler& m = modeler;
    return base::vec(item(m, Command::Move{}, Icon::Move), item(m, Command::Rotate{}, Icon::Rotate), item(m, Command::Scale{}, std::nullopt), MenuItem<Message>::separator(),
                     item(m, Command::Extrude{}, std::nullopt), item(m, Command::Inset{}, std::nullopt), item(m, Command::LoopCutTool{}, Icon::Cut),
                     item(m, Command::Fill{}, Icon::Fill), item(m, Command::Merge{}, std::nullopt), item(m, Command::Subdivide{}, Icon::Tiles), MenuItem<Message>::separator(),
                     item(m, Command::Crease{}, std::nullopt), item(m, Command::AssignPaint{}, Icon::Brush), MenuItem<Message>::separator(),
                     item(m, Command::Delete{}, Icon::Delete));
}

// The view: a card whose content the modeler's render fills with the mesh,
// with what's under way over its top left, and the box being selected.
void view_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    std::string subtitle = std::format("{} mode · {} tool · {} shader{}{}", editing::name(modeler.editor.mode), editing::name(modeler.editor.tool), viewport::name(modeler.shader),
                                       modeler.levels > 0 ? std::format(" · Subdivided {}×", modeler.levels) : std::string(), modeler.xray ? " · X-ray" : "");
    std::optional<std::pair<Point, Point>> selecting_box = modeler.editor.selecting_box;
    // What's under way, at the view's top left.
    std::optional<std::string> label;
    if (modeler.editor.operation) {
        const editing::Operation& operation = *modeler.editor.operation;
        label = std::format("{} · {} · {}", operation.name, editing::name(operation.axis), operation.amount);
    } else {
        switch (modeler.editor.tool) {
        case Tool::LoopCut:
            label = "Click an edge to cut the loop across it";
            break;
        case Tool::Paint:
            label = std::format("Painting with {}", modeler.document.paints[modeler.editor.active_paint].name);
            break;
        case Tool::Select:
            break;
        }
    }
    std::vector<MenuItem<Message>> items = context_items(modeler);
    tree.card(ui, std::nullopt, rect, Card("View").subtitle(std::move(subtitle)).with_view(), [&](Ui&, Tree<Message>& card, CardArea area) {
        card.add("View", area.content, View<Message>().every_frame().on_input(base::make<Message::View>)).set_context_menu(std::move(items));
        // The box being selected.
        if (selecting_box) {
            card.draw(Rect::from_corners(selecting_box->first, selecting_box->second), Canvas([](Painter& painter, Rect box) {
                          Color accent = painter.theme().accent;
                          painter.shape(box, 0.0f, accent.fade(0.12f), Border(accent, 1.0f));
                      }));
        }
        if (label) {
            Rect at = area.content.pad(Insets::all(tokens::SpaceM));
            card.draw(Rect(at.x, at.y, at.width, tokens::Mono.line_height), Text::label(std::move(*label)).style(tokens::Mono).secondary());
        }
    });
}

Rect shading_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    return tree.sized_card(ui, std::nullopt, rect, Card("Shading"), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form(area.content);
        size_t shader = modeler.shader == Shader::SimplePaint ? 0 : 1;
        Rect row = form.control(card, "Shader");
        card.add("Shader", row, Segmented<Message>(shader, {{"SimplePaint", "Flat"}}).on_select([](size_t) { return Message::Command{Command::ToggleShader{}}; }));
        Rect slider = form.take(Slider<Message>::height(true));
        card.add("Subdivision", slider,
                 Slider<Message>(static_cast<double>(modeler.levels), 0.0, static_cast<double>(MaximumLevels))
                     .step(1.0)
                     .header("Catmull-Clark subdivision levels")
                     .show_value(0)
                     .on_change([](double value) { return Message::Command{Command::Level{base::saturating_cast<std::uint32_t>(std::round(value))}}; }));
        struct Toggle {
            std::string_view name;
            std::string_view label;
            Command command;
        };
        for (const Toggle& toggle :
             {Toggle{"Smooth", "Smooth shading", Command::ToggleSmooth{}}, Toggle{"XRay", "X-ray", Command::ToggleXray{}}, Toggle{"Grid", "Grid", Command::ToggleGrid{}}}) {
            bool value = modeler.checked(toggle.command).value_or(false);
            Rect toggle_row = form.take(tokens::ControlHeight);
            card.add(toggle.name, toggle_row,
                     ToggleSwitch<Message>(value, std::string(toggle.label)).on_change([command = toggle.command](bool) { return Message::Command{command}; }));
        }
        return form.bottom();
    });
}

// What a slider of the paint's card sends: the edit that the alternative is,
// of its value.
template <class Alternative> PaintEdit edit_of(double value) {
    return Alternative{value};
}

void paints_card(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect rect) {
    const Document& document = modeler.document;
    std::vector<size_t> counts;
    counts.reserve(document.paints.size());
    for (size_t paint = 0; paint < document.paints.size(); paint++) {
        counts.push_back(static_cast<size_t>(std::ranges::count_if(document.mesh.faces, [&](const polygon_mesh::Face& face) { return face.material == paint; })));
    }
    std::vector<std::pair<std::string, Color>> paints;
    paints.reserve(document.paints.size());
    for (const paint::Paint& paint : document.paints) {
        paints.emplace_back(paint.name, paint.color());
    }
    size_t index = modeler.editor.active_paint;
    std::vector<std::string> labels;
    labels.reserve(paints.size());
    for (const std::pair<std::string, Color>& paint : paints) {
        labels.push_back(paint.first);
    }
    ListView<Message> list =
        ListView<Message>(paints.size(), RowHeight).selection(index).on_select(base::make<Message::PickPaint>).labels([labels = std::move(labels)](size_t row) {
            return labels[row];
        });

    tree.card(ui, std::nullopt, rect, Card("Paints").subtitle("Each in SimplePaint's settings"), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        Form form(area.content);

        // The paints, of which the one picked is what Paint and new
        // primitives use.
        Rect list_rect = form.take(RowHeight * PaintRows);
        ListRows rows = list.rows(ui, "Paints", list_rect);
        card.container("Paints", list_rect, std::move(list), [&](Tree<Message>& items) {
            for (size_t row : rows.indices()) {
                Rect content = rows.content(row);
                const std::string& name = paints[row].first;
                Color color = paints[row].second;
                Rect swatch(content.x, content.center().y - Swatch / 2.0f, Swatch, Swatch);
                items.draw(swatch, user_interface::Swatch(color));
                Rect text(content.x + Swatch + tokens::SpaceM, content.y, content.width - Swatch - tokens::SpaceM, content.height);
                std::string count = counts[row] == 1 ? std::string("1 face") : std::format("{} faces", counts[row]);
                Rect count_rect = text.cut_right(96.0f, tokens::SpaceS);
                items.draw(text, Text::label(name));
                items.draw(count_rect, Text::caption(std::move(count)).end());
            }
        });
        auto [add, remove, assign] = form.columns<3>(tokens::ControlHeight);
        command_button(modeler, card, add, Command::AddPaint{}, "Add", Icon::Add);
        command_button(modeler, card, remove, Command::RemovePaint{}, "Remove", Icon::Remove);
        command_button(modeler, card, assign, Command::AssignPaint{}, "Paint", Icon::Brush);

        // The paint picked: its color, beside its name and its color's field.
        Rect row = form.take(ColorPicker<Message>::height(PickerWidth));
        Rect picker = row.cut_left(PickerWidth, tokens::SpaceM);
        Color color = document.paints[index].color();
        card.add("Paint/Color", picker, ColorPicker<Message>(color).on_change([](Color picked) { return Message::Paint{PaintEdit::Color{picked}}; }));
        Form side(row);
        Rect name_row = side.control(card, "Name");
        card.add("Paint/Name", name_row,
                 TextBox<Message>(document.paints[index].name).max_length(32).on_change([](std::string name) { return Message::Paint{PaintEdit::Name{std::move(name)}}; }));
        Rect hex_row = side.control(card, "Base color");
        card.add("Paint/Hex", hex_row, ColorField<Message>(color).on_change([](Color picked) { return Message::Paint{PaintEdit::Color{picked}}; }));

        // SimplePaint's other settings.
        simple_paint::Parameters parameters = document.paints[index].parameters;
        using Edit = PaintEdit (*)(double);
        struct Setting {
            std::string_view name;
            std::string_view header;
            double value;
            double minimum;
            double maximum;
            size_t decimals;
            std::string_view suffix;
            Edit edit;
        };
        const Setting sliders[] = {
            {"Paint/Brightness", "Brightness", parameters.brightness, simple_paint::Margin, simple_paint::InteriorMaximum, 2, "", edit_of<PaintEdit::Brightness>},
            {"Paint/Shift", "Shift", parameters.shift, 0.0, simple_paint::InteriorMaximum, 2, "", edit_of<PaintEdit::Shift>},
            {"Paint/Rotation", "Rotation", parameters.rotation_degrees, 0.0, 359.0, 0, "°", edit_of<PaintEdit::Rotation>},
            {"Paint/Dark", "Dark point", parameters.dark_point, 0.0, simple_paint::InteriorMaximum, 2, "", edit_of<PaintEdit::Dark>},
            {"Paint/Light", "Light point", parameters.light_point, simple_paint::Margin, 1.0, 2, "", edit_of<PaintEdit::Light>},
        };
        for (const Setting& setting : sliders) {
            Rect slider_rect = form.take(Slider<Message>::height(true));
            double step = setting.decimals == 0 ? 1.0 : 0.01;
            Slider<Message> slider = Slider<Message>(setting.value, setting.minimum, setting.maximum)
                                         .step(step)
                                         .header(std::string(setting.header))
                                         .show_value(setting.decimals)
                                         .suffix(std::string(setting.suffix))
                                         .on_change([edit = setting.edit](double value) { return Message::Paint{edit(value)}; });
            card.add(setting.name, slider_rect, std::move(slider));
        }
    });
}

void right_column(const Modeler& modeler, Ui& ui, Tree<Message>& tree, Rect column) {
    float bottom = shading_card(modeler, ui, tree, column).bottom();
    column.cut_top(bottom - column.y, tokens::CardGap);
    paints_card(modeler, ui, tree, column);
}

// The dialog that's open, if any.
void prompt(const Modeler& modeler, Ui& ui, Tree<Message>& tree) {
    if (!modeler.prompt) {
        return;
    }
    Prompt prompt = *modeler.prompt;
    if (base::is<Prompt::Unsaved>(prompt)) {
        std::string message = std::format("{} has changes that aren't saved.", modeler.document.name);
        Dialog<Message> dialog =
            Dialog<Message>("Save your changes?", std::move(message)).primary("Save").secondary("Don't save").close("Cancel").on_result(base::make<Message::Unsaved>);
        DialogArea area = dialog.area(ui);
        tree.add(prompt.name(), area.rect, std::move(dialog));
    } else if (base::is<Prompt::SaveAs>(prompt)) {
        std::optional<std::string> error = modeler.prompt_error;
        float height = tokens::Body.line_height + tokens::SpaceXs + tokens::ControlHeight + (error ? tokens::SpaceS + tokens::Caption.line_height * 2.0f : 0.0f);
        Dialog<Message> dialog = Dialog<Message>("Save as", "Models are saved in the program's Models folder.")
                                     .primary("Save")
                                     .close("Cancel")
                                     .content_height(height)
                                     .on_result(base::make<Message::SaveAsResult>);
        DialogArea area = dialog.area(ui);
        std::string draft = modeler.name_draft;
        tree.container(prompt.name(), area.rect, std::move(dialog), [&](Tree<Message>& content) {
            Form form = Form(area.content).gap(tokens::SpaceS);
            Rect row = form.control(content, "Name");
            content.add("SaveAs/Name", row, TextBox<Message>(draft).max_length(64).placeholder("Name the model").on_change(base::make<Message::NameDraft>));
            if (error) {
                content.draw(form.rest(), Text::paragraph(*error).style(tokens::Caption).color(TextColor::Custom{ui.theme().critical}));
            }
        });
    } else {
        std::vector<std::string> names = modeler.saved_names;
        std::string_view message = names.empty() ? "There are no models in the program's Models folder yet." : "Pick a model from the program's Models folder.";
        Dialog<Message> dialog = Dialog<Message>("Open a model", std::string(message)).close("Cancel").on_result(base::make<Message::OpenResult>);
        if (!names.empty()) {
            dialog = Dialog<Message>(std::move(dialog)).primary("Open");
        }
        float height = names.empty() ? 0.0f : RowHeight * OpenRows;
        dialog = Dialog<Message>(std::move(dialog)).content_height(height);
        DialogArea area = dialog.area(ui);
        if (names.empty()) {
            tree.add(prompt.name(), area.rect, std::move(dialog));
            return;
        }
        ListView<Message> list = ListView<Message>(names.size(), RowHeight)
                                     .selection(modeler.open_choice)
                                     .on_select(base::make<Message::OpenChoice>)
                                     .on_activate([](size_t) { return Message::OpenActivated{}; })
                                     .labels([names](size_t row) { return names[row]; });
        Rect list_rect = area.content;
        ListRows rows = list.rows(ui, "OpenModel/List", list_rect);
        tree.container(prompt.name(), area.rect, std::move(dialog), [&](Tree<Message>& content) {
            content.container("OpenModel/List", list_rect, std::move(list), [&](Tree<Message>& items) {
                for (size_t row : rows.indices()) {
                    items.draw(rows.content(row), Text::label(names[row]));
                }
            });
        });
    }
}

} // namespace

const Size SwatchSize = Size(Swatch, Swatch);

Rect view_content(Rect bounds) {
    float top = bounds.y + tokens::AppBarHeight + tokens::PagePadding;
    float bottom = bounds.bottom() - tokens::StatusBarHeight - tokens::PagePadding;
    Rect page(tokens::PagePadding, top, bounds.width - tokens::PagePadding * 2.0f, bottom - top);
    page.cut_left(LeftWidth, tokens::CardGap);
    page.cut_right(RightWidth, tokens::CardGap);
    Rect inner = page.inset(tokens::CardPadding);
    return inner.pad(Insets(0.0f, tokens::CardHeaderHeight + tokens::CardHeaderGap, 0.0f, 0.0f));
}

Tree<Message> view(const Modeler& modeler, Ui& ui) {
    Tree<Message> tree;
    std::vector<Status> status = status_items(modeler);
    Shell shell = tree.shell(ui, app_bar(modeler), false, std::move(status));
    menu_bar(modeler, ui, tree, shell.commands);

    Rect page = shell.page;
    Rect left = page.cut_left(LeftWidth, tokens::CardGap);
    Rect right = page.cut_right(RightWidth, tokens::CardGap);
    left_column(modeler, ui, tree, left);
    view_card(modeler, ui, tree, page);
    right_column(modeler, ui, tree, right);

    // A modal edit: a cover over the whole window takes the pointer from
    // everything else while it's under way, and hears it every frame.
    if (modeler.editor.operation) {
        tree.add("Operation", ui.bounds(), View<Message>().cursor(Cursor::Move).every_frame().on_input(base::make<Message::Operation>));
    }
    prompt(modeler, ui, tree);
    return tree;
}

} // namespace mesh_modeler::cards
