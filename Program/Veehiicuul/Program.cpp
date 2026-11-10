#include "Veehiicuul/Program.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Veehiicuul/Angles.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Settings.h"
#include "Veehiicuul/Veehiicuul.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <span>

namespace veehiicuul::program {

namespace tokens = user_interface::tokens;

using camera::Eye;
using controls::Controls;
using document::Document;
using drawing::Drawing;
using drawing::Ground;
using drawing::Instance;
using editing::GroundInput;
using editing::GroundView;
using editing::Viewport;
using game::Game;
using json::Value;
using scene_editor::SceneMsg;
using settings::GameSettings;
using terrain_editor::TerrainMsg;
using track_editor::TrackMsg;
using user_interface::AppBar;
using user_interface::Button;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::CheckBox;
using user_interface::ComboBox;
using user_interface::Context;
using user_interface::Divider;
using user_interface::Field;
using user_interface::Icon;
using user_interface::InfoBar;
using user_interface::Insets;
using user_interface::Key;
using user_interface::KeycapHeight;
using user_interface::Keycaps;
using user_interface::Modifiers;
using user_interface::MouseButton;
using user_interface::Point;
using user_interface::Rect;
using user_interface::SelectorBar;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Startup;
using user_interface::Status;
using user_interface::Text;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::View;
using user_interface::ViewInput;
using vehicle_editor::ShownHeading;
using vehicle_editor::VehicleMsg;

namespace {

// How many changes undo goes back.
constexpr size_t History = 200;
// How wide the app bar's combo box of scenes is.
constexpr float SceneWidth = 200.0f;

// What a control does in Play, with the gamepad's buttons, then the
// keyboard's keys. A way that's empty isn't one.
struct ControlRow {
    std::string_view does;
    std::array<std::string_view, 2> gamepad;
    std::array<std::string_view, 2> keyboard;
};

// What each control does in Play, with the gamepad's buttons, then the
// keyboard's keys, each on a keycap. A "+" joins keys that go together, like
// a shortcut's.
constexpr std::array<ControlRow, 9> ControlRows{{
    {"Drive", {"Right stick"}, {"Arrows", "WASD"}},
    {"Brake", {"LT"}, {"Space"}},
    {"Back to the start", {"X"}, {"R"}},
    {"Vehicle", {"D-pad"}, {"Q", "E"}},
    {"Scene", {"D-pad"}, {"PageUp", "PageDown"}},
    {"Camera", {"Menu"}, {"C"}},
    {"Zoom", {"RB+Left stick"}, {"Wheel"}},
    {"Zoom back", {"Y"}, {"Home"}},
    {"Hide this", {"View"}, {"F1"}},
}};
// The label column's width in the card, the gap before the keyboard's
// column, and a row of keycaps' height.
constexpr float LabelWidth = 136.0f;
constexpr float ColumnGap = tokens::SpaceXl;
constexpr float KeycapRow = 28.0f;
// The gap between the card's lines of what's going on.
constexpr float LineGap = tokens::SpaceXs;

// The ways that a row has, without the empty ones after them.
std::span<const std::string_view> ways_of(const std::array<std::string_view, 2>& ways) {
    return std::span(ways).first(ways[1].empty() ? 1 : 2);
}

// How wide the keycaps of the ways to do something are, a small space apart.
// Each way is a key, or keys that go together, like "RB+Left stick", which
// Keycaps joins.
float ways_width(Ui& ui, std::span<const std::string_view> ways) {
    float width = 0.0f;
    for (size_t index = 0; index < ways.size(); index++) {
        if (index > 0) {
            width += tokens::SpaceS;
        }
        width += Keycaps::width(ui.fonts(), ways[index]);
    }
    return width;
}

// Draws the keycaps of the ways from `origin`.
void draw_ways(Ui& ui, Tree<Message>& tree, Point origin, std::span<const std::string_view> ways) {
    float x = origin.x;
    for (size_t index = 0; index < ways.size(); index++) {
        if (index > 0) {
            x += tokens::SpaceS;
        }
        float width = Keycaps::width(ui.fonts(), ways[index]);
        tree.draw(Rect(x, origin.y, width, KeycapHeight), Keycaps(std::string(ways[index])));
        x += width;
    }
}

} // namespace

Veehiicuul::Veehiicuul(Game loaded, std::filesystem::path folder) :
    game(std::move(loaded)),
    mode(Play),
    viewport(game.current_scene().camera_center, game.current_scene().camera_height),
    saved(game.document),
    committed(game.document),
    unsaved_now(false),
    pending_notches(0.0),
    save_folder(std::move(folder)),
    eye(Eye::game(game.camera_center(), game.camera.height)) {}

std::expected<Veehiicuul, std::string> Veehiicuul::start(Startup& startup) {
    GameSettings settings = GameSettings::read(startup.settings);
    std::vector<std::string> notices;
    std::expected<Game, std::string> loaded = Game::load(base::path(Folder), settings, notices);
    if (!loaded) {
        return std::unexpected(std::move(loaded.error()));
    }
    for (const std::string& notice : notices) {
        startup.notify(Severity::Warning, "Settings.json", notice);
    }
    return Veehiicuul(std::move(*loaded), startup.data_folder);
}

bool Veehiicuul::unsaved() const {
    return game.document != saved;
}

void Veehiicuul::commit() {
    if (game.document != committed) {
        Document before = std::exchange(committed, game.document);
        undo_.push_back(std::move(before));
        if (undo_.size() > History) {
            undo_.erase(undo_.begin());
        }
        redo_.clear();
    }
}

bool Veehiicuul::can_undo() const {
    return !undo_.empty() || game.document != committed;
}

bool Veehiicuul::can_redo() const {
    return !redo_.empty();
}

void Veehiicuul::undo() {
    commit();
    if (!undo_.empty()) {
        Document before = std::move(undo_.back());
        undo_.pop_back();
        redo_.push_back(std::exchange(game.document, std::move(before)));
        committed = game.document;
        game.keep_in_document();
    }
}

void Veehiicuul::redo() {
    if (!redo_.empty()) {
        Document after = std::move(redo_.back());
        redo_.pop_back();
        undo_.push_back(std::exchange(game.document, std::move(after)));
        committed = game.document;
        game.keep_in_document();
    }
}

void Veehiicuul::save() {
    std::vector<std::pair<std::string, std::string>> changed = game.document.changed_files(saved);
    std::vector<std::string> removed = game.document.removed_files(saved);
    if (changed.empty() && removed.empty()) {
        status = "Nothing to save";
        return;
    }
    for (const std::pair<std::string, std::string>& file : changed) {
        if (std::expected<void, std::string> written = files::write(save_folder / base::path(file.first), file.second); !written) {
            status = std::move(written.error());
            return;
        }
    }
    for (const std::string& path : removed) {
        // A file that's gone already is as it should be.
        if (!DeleteFileW((save_folder / base::path(path)).c_str())) {
            std::string error = base::os_error(GetLastError());
            if (!base::is_not_found(error)) {
                status = std::format("Couldn't remove {}: {}", path, error);
                return;
            }
        }
    }
    std::vector<std::string_view> names;
    names.reserve(changed.size());
    for (const std::pair<std::string, std::string>& file : changed) {
        names.push_back(file.first);
    }
    if (names.empty()) {
        status = std::format("Removed {}", base::join(removed, ", "));
    } else if (removed.empty()) {
        status = std::format("Saved {}", base::join(names, ", "));
    } else {
        status = std::format("Saved {} and removed {}", base::join(names, ", "), base::join(removed, ", "));
    }
    saved = game.document;
}

void Veehiicuul::new_scene(double seconds) {
    const std::vector<scene::Scene>& scenes = game.document.scenes;
    std::string name;
    for (size_t number = scenes.size() + 1;; number++) {
        name = std::format("Scene {}", number);
        if (std::ranges::all_of(scenes, [&](const scene::Scene& scene) { return scene.name != name; })) {
            break;
        }
    }
    scene::Scene scene(name);
    viewport = Viewport(scene.camera_center, scene.camera_height);
    game.document.scenes.push_back(std::move(scene));
    game.go_to_scene(game.document.scenes.size() - 1, seconds);
}

void Veehiicuul::go_to(size_t tab, double seconds) {
    mode = tab;
    game.keep_in_document();
    if (tab == Play) {
        game.put_at_start(seconds);
    }
}

void Veehiicuul::view_input(const ViewInput& input, bool space, const Context<Message>& ctx) {
    std::array<float, 2> size{ctx.size().width, ctx.size().height};
    size_t scene = game.scene;
    if (mode == Play) {
        pending_notches += static_cast<double>(input.wheel.y);
    } else if (mode == Scene) {
        GroundInput ground = viewport.apply(input, space, size);
        scene_editor.view_input(ground, game.document, game.models, scene);
    } else if (mode == Terrain) {
        GroundInput ground = viewport.apply(input, space, size);
        terrain_editor.view_input(ground, game.document.scenes[scene].terrain);
    } else if (mode == Track) {
        GroundInput ground = viewport.apply(input, space, size);
        track_editor.view_input(ground, game.document.scenes[scene].track, viewport.eye, size);
    } else {
        GroundInput ground = vehicle_editor.viewport.apply(input, space, size);
        vehicle_editor.view_input(ground, game.document, game.vehicle, size);
    }
}

void Veehiicuul::commands(Ui& ui, Tree<Message>& tree, Rect bar) const {
    // The tabs take the bar's whole height, so that their text is in
    // its middle, with their pills under it.
    Rect tabs_area = bar.cut_left(SelectorBar<Message>::width(ui, Modes), tokens::SpaceXxl);
    Rect tabs(tabs_area.x, ui.content_top(), tabs_area.width, tokens::AppBarHeight);
    tree.add("Mode", tabs, SelectorBar<Message>(mode, Modes).on_select(base::make<Message::Mode>));

    // From the right end: save, redo and undo, then the scene.
    Button<Message> save_button = Button<Message>("Save").icon(Icon::Save).on_press(Message::Save{});
    if (unsaved_now) {
        save_button = std::move(save_button).accent();
    }
    Button<Message> undo_button = Button<Message>("Undo").icon(Icon::Undo).on_press(Message::Undo{});
    Button<Message> redo_button = Button<Message>("Redo").icon(Icon::Redo).on_press(Message::Redo{});
    Button<Message> new_scene_button = Button<Message>("New scene").icon(Icon::Add).on_press(Message::NewScene{});
    Rect save_area = bar.cut_right(save_button.width(ui), tokens::SpaceS);
    Rect redo_area = bar.cut_right(redo_button.width(ui), tokens::SpaceS);
    Rect undo_area = bar.cut_right(undo_button.width(ui), tokens::SpaceL);
    Rect divider = bar.cut_right(1.0f, tokens::SpaceL);
    Rect new_scene_area = bar.cut_right(new_scene_button.width(ui), tokens::SpaceS);
    Rect scene_area = bar.cut_right(SceneWidth, tokens::SpaceS);
    Rect label_area = bar.cut_right(std::ceil(ui.measure("Scene", tokens::Body).width), tokens::SpaceL);

    tree.add("Save", save_area, std::move(save_button)).set_tooltip("Writes the files that changed (Ctrl+S)");
    tree.add("Redo", redo_area, std::move(redo_button)).set_enabled(can_redo()).set_tooltip("Does again what undo undid (Ctrl+Y)");
    tree.add("Undo", undo_area, std::move(undo_button)).set_enabled(can_undo()).set_tooltip("Takes back the last change (Ctrl+Z)");
    tree.draw(divider.pad(Insets::symmetric(0.0f, tokens::SpaceXs)), Divider());
    tree.add("NewScene", new_scene_area, std::move(new_scene_button)).set_tooltip("Makes an empty scene, and goes to it");
    std::vector<std::string_view> names;
    names.reserve(game.document.scenes.size());
    for (const scene::Scene& scene : game.document.scenes) {
        names.push_back(scene.name);
    }
    tree.add("Scene", scene_area, ComboBox<Message>(game.scene, names).on_select(base::make<Message::GoScene>));
    tree.draw(label_area, Text::label("Scene").secondary());

    if (mode != Play) {
        std::string_view label = "The game's angle";
        Rect area = bar.left_strip(CheckBox<Message>::width(ui, label));
        const Viewport& shown = mode == Vehicles ? vehicle_editor.viewport : viewport;
        tree.add("GameAngle", area, CheckBox<Message>(shown.game_angle(), std::string(label)).on_change(base::make<Message::GameAngle>))
            .set_tooltip("Shows the view as the game does, instead of straight down");
    }
}

std::vector<Status> Veehiicuul::status_items() const {
    std::vector<Status> items = base::vec(Status::shortcut("Ctrl+S", "Save"), Status::shortcut("Ctrl+Z", "Undo"), Status::shortcut("Ctrl+Y", "Redo"));
    if (mode == Play) {
        items.push_back(Status::shortcut("F1", game.show_card ? "Hide the card" : "Show the card"));
    } else {
        items.push_back(Status::shortcut("Space", "Pan, with a drag"));
        items.push_back(Status::shortcut("Wheel", "Zoom"));
    }
    items.push_back(Status::shortcut("F9", "Theme"));
    items.push_back(Status::shortcut("F10", "VSync"));
    items.push_back(Status::shortcut("F11", "Fullscreen"));
    items.push_back(Status::Spacer{});
    if (!status.empty()) {
        items.push_back(Status::text(status));
    }
    items.push_back(unsaved_now ? Status::indicator(Severity::Warning, "Unsaved changes") : Status::indicator(Severity::Success, "No unsaved changes"));
    return items;
}

void Veehiicuul::build_view() {
    const Document& document = game.document;
    const models::Models& models = game.models;
    std::vector<Instance> shown;
    if (mode == Vehicles) {
        if (std::optional<std::pair<size_t, std::vector<simple_paint::Parameters>>> painted = document.vehicle_materials(models, game.vehicle)) {
            shown.emplace_back(painted->first, std::array<double, 2>{0.0, 0.0}, to_radians(ShownHeading), 1.0, std::move(painted->second));
        }
        eye = vehicle_editor.viewport.eye;
        instances = std::move(shown);
        return;
    }
    const scene::Scene& scene = game.current_scene();
    for (const scene::Decoration& decoration : scene.decorations) {
        if (std::optional<size_t> model = models.find(decoration.model)) {
            shown.emplace_back(*model, decoration.pose.position, decoration.pose.radians(), decoration.scale, document.model_materials(models, *model));
        }
    }
    for (const scene::Parked& parked : scene.parked) {
        if (std::optional<size_t> vehicle = document.vehicles.find(parked.vehicle)) {
            if (std::optional<std::pair<size_t, std::vector<simple_paint::Parameters>>> painted = document.vehicle_materials(models, *vehicle)) {
                shown.emplace_back(painted->first, parked.pose.position, parked.pose.radians(), 1.0, std::move(painted->second));
            }
        }
    }
    std::array<double, 2> position = mode == Play ? game.car.position : scene.track.start.position;
    double heading = mode == Play ? game.car.heading : scene.track.start.radians();
    if (std::optional<std::pair<size_t, std::vector<simple_paint::Parameters>>> painted = document.vehicle_materials(models, game.vehicle)) {
        shown.emplace_back(painted->first, position, heading, 1.0, std::move(painted->second));
    }
    eye = mode == Play ? Eye::game(game.camera_center(), game.camera.height) : viewport.eye;
    instances = std::move(shown);
}

std::string Veehiicuul::title() const {
    return std::string(Name);
}

std::vector<Message> Veehiicuul::input(const Ui& ui) const {
    std::vector<Message> messages;
    if (ui.shortcut(Modifiers::Control, Key::S)) {
        messages.push_back(Message::Save{});
    }
    if (ui.shortcut(Modifiers::Control, Key::Z)) {
        messages.push_back(Message::Undo{});
    }
    if (ui.shortcut(Modifiers::Control, Key::Y)) {
        messages.push_back(Message::Redo{});
    }
    if (mode == Play) {
        messages.push_back(Message::Tick{Controls::read(ui, 0.0)});
    }
    messages.push_back(Message::Prepare{});
    return messages;
}

void Veehiicuul::update(Message message, Context<Message>& ctx) {
    base::match(
        message,
        [&](const Message::Mode& chosen) {
            if (chosen.mode != mode) {
                go_to(chosen.mode, ctx.time());
            }
        },
        [&](const Message::Save&) { save(); }, [&](const Message::Undo&) { undo(); }, [&](const Message::Redo&) { redo(); },
        [&](const Message::NewScene&) { new_scene(ctx.time()); },
        [&](const Message::GoScene& chosen) {
            if (chosen.scene != game.scene) {
                game.go_to_scene(chosen.scene, ctx.time());
                const scene::Scene& current = game.current_scene();
                viewport = Viewport(current.camera_center, current.camera_height);
            }
        },
        [&](const Message::GameAngle& chosen) {
            Viewport& shown = mode == Vehicles ? vehicle_editor.viewport : viewport;
            shown.set_game_angle(chosen.game_angle);
        },
        [&](const Message::View& pointed) { view_input(pointed.input, pointed.space, ctx); },
        [&](const Message::Scene& edit) { scene_editor.apply(edit.message, game.document, game.models, game.scene); },
        [&](const Message::Terrain& edit) {
            size_t scene = game.scene;
            terrain_editor.apply(edit.message, game.document.scenes[scene].terrain);
        },
        [&](const Message::Track& edit) {
            size_t scene = game.scene;
            track_editor.apply(edit.message, game.document.scenes[scene].track);
        },
        [&](const Message::Vehicle& edit) { vehicle_editor.apply(edit.message, game.document, game.models, game.vehicle); },
        [&](const Message::Tick& tick) {
            if (mode == Play) {
                Controls controls = tick.controls;
                controls.notches = std::exchange(pending_notches, 0.0);
                game.update(controls, ctx.time());
            }
        },
        [&](const Message::Prepare&) {
            pending_notches = 0.0;
            game.keep_in_document();
            // A pick that's gone, like after an undo, even during a drag,
            // and the track's and the terrain's choices the same way.
            scene_editor.keep_picked(game.document, game.scene);
            const scene::Scene& current = game.document.scenes[game.scene];
            track_editor.curve = std::min(track_editor.curve, current.track.islands.size());
            terrain_editor.color = std::min(terrain_editor.color, static_cast<std::uint8_t>(base::saturating_sub(current.terrain.palette.size(), size_t{1})));
            if (!ctx.is_focused("Vehicles/Name")) {
                vehicle_editor.sync_name(game.document, game.vehicle);
            }
            // A drag is one change, as of when it's let go of.
            if (!ctx.input().button_down(MouseButton::Left)) {
                commit();
            }
            game.shape();
            unsaved_now = unsaved();
            build_view();
        });
}

Tree<Message> Veehiicuul::view(Ui& ui) const {
    Tree<Message> tree;
    // Only text fields take keyboard focus, so that Space, the arrows
    // and the other keys that drive and pan never go to a control.
    tree.set_focusable(false);
    // The view first, under everything: the bars, the notices and the
    // cards cover it.
    Rect view_rect = editing::view_area(ui);
    if (mode == Play) {
        tree.add("Play/View", view_rect, View<Message>().on_input([](ViewInput input) { return Message::View{input, false}; }));
    } else {
        constexpr std::array<std::string_view, 5> names{"", "Scene/View", "Terrain/View", "Track/View", "Vehicles/View"};
        const Viewport& shown = mode == Vehicles ? vehicle_editor.viewport : viewport;
        tree.add(names[mode], view_rect, GroundView(shown.panning()));
    }
    Shell shell = tree.shell(ui, AppBar(Icon::Car, std::string(Name)), false, status_items());
    commands(ui, tree, shell.commands);
    size_t scene = game.scene;
    if (mode == Play) {
        if (game.show_card) {
            play_card(game, ui, tree, shell.page);
        }
    } else if (mode == Scene) {
        scene_editor.view(ui, tree, shell.page, view_rect, viewport, game.document, game.models, scene);
    } else if (mode == Terrain) {
        terrain_editor.view(ui, tree, shell.page, view_rect, viewport, game.document.scenes[scene].terrain);
    } else if (mode == Track) {
        track_editor.view(ui, tree, shell.page, view_rect, viewport, game.document.scenes[scene].track, game.last_shape().problems);
    } else {
        vehicle_editor.view(ui, tree, shell.page, view_rect, game.document, game.models, game.vehicle);
    }
    return tree;
}

void Veehiicuul::prepare(graphics::Gpu& gpu) {
    drawing = Drawing(gpu, game.models);
}

void Veehiicuul::render(graphics::Gpu& gpu, const graphics::Frame& frame) {
    if (!drawing) {
        return;
    }
    const scene::Scene& scene = game.current_scene();
    // The Vehicles tab shows its vehicle on the theme's background.
    std::optional<std::array<float, 3>> background;
    std::optional<Ground> ground;
    if (mode != Vehicles) {
        background = files::srgb(scene.background);
        ground.emplace(Ground{
            .terrain = scene.terrain,
            .track = scene.track,
            .shape = game.last_shape(),
        });
    }
    drawing::View shown{
        .eye = eye,
        .background = background,
        .ground = ground,
        .instances = instances,
    };
    drawing->draw(gpu, frame, shown);
}

Value Veehiicuul::state() const {
    Value::Object object;
    object.members.emplace_back("Mode", Value(Modes[mode]));
    object.members.emplace_back("Unsaved", Value(unsaved()));
    object.members.emplace_back("Status", Value(status));
    object.members.emplace_back("Undo", Value(undo_.size()));
    for (std::pair<std::string_view, Value>& member : game.state()) {
        object.members.emplace_back(std::string(member.first), std::move(member.second));
    }
    const Document& document = game.document;
    size_t scene = game.scene;
    const track::Track& track = document.scenes[scene].track;
    object.members.emplace_back("SceneEditor", scene_editor.state(document, scene));
    object.members.emplace_back("TerrainEditor", terrain_editor.state(document.scenes[scene].terrain));
    object.members.emplace_back("TrackEditor", track_editor.state(track, game.last_shape().problems));
    object.members.emplace_back("VehicleEditor", vehicle_editor.state(document, game.vehicle));
    return object;
}

Rect play_card(const Game& game, Ui& ui, Tree<Message>& tree, Rect page) {
    const vehicles::Vehicle& vehicle = game.current_vehicle();
    driving::Acceleration rates = vehicle.acceleration;
    std::string_view camera = game.camera.follow ? "Following the car" : "Fixed";
    std::string_view gamepad = ui.gamepad().connected ? "Connected" : "None, so drive with the keys";
    std::array<std::pair<std::string_view, std::string>, 6> rows{{
        {"Vehicle", std::format("{}, {} of {}", vehicle.name, game.vehicle + 1, game.document.vehicles.list.size())},
        {"Accelerates", std::format("{} ahead, {} in reverse, {} left, {} right", base::display(rates.forward), base::display(rates.reverse), base::display(rates.left),
                                    base::display(rates.right))},
        {"Speed", std::format("{:.1f} units a second", game.car.speed())},
        {"Camera", std::format("{}, {:.0f} units tall", camera, game.camera.height)},
        {"Crashes", std::to_string(game.crashes)},
        {"Gamepad", std::string(gamepad)},
    }};
    const std::vector<std::string>& problems = game.last_shape().problems;
    std::optional<std::string> problem = problems.empty() ? std::nullopt : std::optional(problems.front());
    const std::string& scene = game.current_scene().name;
    std::string scenes = std::format("Scene {} of {}", game.scene + 1, game.document.scenes.size());

    // As wide as its widest line.
    float scene_width = ui.measure(scene, tokens::BodyStrong).width;
    float scenes_width = ui.measure(scenes, tokens::Caption).width;
    float title_width = scene_width + tokens::SpaceS + scenes_width;
    float rows_width = 0.0f;
    for (const std::pair<std::string_view, std::string>& row : rows) {
        rows_width = std::max(rows_width, LabelWidth + ui.measure(row.second, tokens::Body).width);
    }
    float gamepad_header = ui.measure("Gamepad", tokens::Caption).width;
    float keyboard_header = ui.measure("Keyboard", tokens::Caption).width;
    float gamepad_width = gamepad_header;
    for (const ControlRow& control : ControlRows) {
        gamepad_width = std::max(gamepad_width, ways_width(ui, ways_of(control.gamepad)));
    }
    gamepad_width = std::ceil(gamepad_width);
    float keyboard_width = keyboard_header;
    for (const ControlRow& control : ControlRows) {
        keyboard_width = std::max(keyboard_width, ways_width(ui, ways_of(control.keyboard)));
    }
    float controls_width = LabelWidth + gamepad_width + ColumnGap + keyboard_width;
    float content_width = std::ceil(std::min(std::max(std::max(title_width, rows_width), controls_width), page.width - tokens::CardPadding * 2.0f));
    std::optional<InfoBar<Message>> problem_bar;
    if (problem) {
        problem_bar.emplace(Severity::Warning, "The track has a problem", *problem);
    }
    float problem_height = problem_bar ? tokens::SpaceM + problem_bar->height(ui, content_width) : 0.0f;
    float rows_height = tokens::Body.line_height * static_cast<float>(rows.size()) + LineGap * static_cast<float>(rows.size() - 1);
    float heading_height = tokens::BodyStrong.line_height + tokens::SpaceXs + 1.0f;
    float controls_height = tokens::SpaceL + heading_height + tokens::SpaceS + KeycapRow * static_cast<float>(ControlRows.size());
    float content_height = rows_height + problem_height + controls_height;
    Card card = Card(scene).subtitle(scenes).floating();
    float height = card.height_around(content_height);
    Rect area(page.x, page.bottom() - height, content_width + tokens::CardPadding * 2.0f, height);

    tree.card(ui, "Play/Card", area, std::move(card), [&](Ui& inner, Tree<Message>& items, CardArea areas) {
        Rect content = areas.content;
        for (size_t index = 0; index < rows.size(); index++) {
            float gap = index + 1 < rows.size() ? LineGap : 0.0f;
            items.draw(content.cut_top(tokens::Body.line_height, gap), Field(std::string(rows[index].first), LabelWidth, rows[index].second));
        }
        if (problem_bar) {
            content.cut_top(tokens::SpaceM, 0.0f);
            float bar_height = problem_bar->height(inner, content.width);
            items.add("Play/Problem", content.cut_top(bar_height, 0.0f), std::move(*problem_bar));
        }
        // The table's heading, with its columns' under a divider, as a
        // form's heading has it.
        content.cut_top(tokens::SpaceL, 0.0f);
        Rect heading = content.cut_top(heading_height, tokens::SpaceS);
        Rect line = heading.top_strip(tokens::BodyStrong.line_height);
        float keyboard_x = line.x + LabelWidth + gamepad_width + ColumnGap;
        items.draw(line, Text::heading("Controls"));
        items.draw(Rect(line.x + LabelWidth, line.y, gamepad_width, line.height), Text::caption("Gamepad"));
        items.draw(Rect(keyboard_x, line.y, line.right() - keyboard_x, line.height), Text::caption("Keyboard"));
        items.draw(heading.bottom_strip(1.0f), Divider());
        for (const ControlRow& control : ControlRows) {
            Rect row = content.cut_top(KeycapRow, 0.0f);
            float top = std::round(row.center().y - KeycapHeight / 2.0f);
            items.draw(row.left_strip(LabelWidth - tokens::SpaceS), Text::label(std::string(control.does)).secondary());
            draw_ways(inner, items, Point(row.x + LabelWidth, top), ways_of(control.gamepad));
            draw_ways(inner, items, Point(keyboard_x, top), ways_of(control.keyboard));
        }
    });
    return area;
}

} // namespace veehiicuul::program
