// The game as a Program, which Library/UserInterface's main loop runs. Tabs
// in the app bar go between playing and the editors, which make the game's
// scenes and vehicles:
//
// | Tab      | What it does                                                         |
// | -------- | -------------------------------------------------------------------- |
// | Play     | Drives the current scene's track, with a card over the bottom left   |
// |          | corner that says what's going on and what the controls do            |
// | Scene    | Arranges decorations and parked vehicles, and sets the background,   |
// |          | the camera, and models' paint                                        |
// | Terrain  | Paints the hexel terrain                                             |
// | Track    | Builds the track from curves, which make the road and its barriers   |
// | Vehicles | Edits the vehicles, their footprints, and their paint                |
//
// After the tabs: in the editors, whether their view is at the game's
// angle, and at the right, the scene to play and edit, a new scene, undo,
// redo, and save, which writes the files that changed. The status bar says
// whether there are changes to save. Drawing.h draws the 3D view under all
// of it, over the whole window.
//
// A frame's input becomes messages: the view's pointer and the cards'
// controls from the widgets, then the shortcuts and, in Play, a Tick with
// the frame's controls, and last Prepare, which commits a change when the
// button is up and rebuilds what the 3D view shows. The frame then draws
// from the state they made, so nothing shows a frame late.

#pragma once

#include "Base/Sum.h"
#include "Graphics/Graphics.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Controls.h"
#include "Veehiicuul/Document.h"
#include "Veehiicuul/Drawing.h"
#include "Veehiicuul/Editing.h"
#include "Veehiicuul/Game.h"
#include "Veehiicuul/SceneEditor.h"
#include "Veehiicuul/TerrainEditor.h"
#include "Veehiicuul/TrackEditor.h"
#include "Veehiicuul/VehicleEditor.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <expected>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace veehiicuul::program {

inline constexpr std::array<std::string_view, 5> Modes{"Play", "Scene", "Terrain", "Track", "Vehicles"};
inline constexpr size_t Play = 0;
inline constexpr size_t Scene = 1;
inline constexpr size_t Terrain = 2;
inline constexpr size_t Track = 3;
inline constexpr size_t Vehicles = 4;

// What a frame's input means to the game.
struct Message {
    // The app bar: a tab, and the commands at its right.
    struct Mode {
        size_t mode;
    };
    struct Save {};
    struct Undo {};
    struct Redo {};
    struct NewScene {};
    struct GoScene {
        size_t scene;
    };
    struct GameAngle {
        bool game_angle;
    };
    // The pointer in the view, with whether Space pans, which the mode
    // routes: the game's zoom, or an editor's viewport and tool.
    struct View {
        user_interface::ViewInput input;
        bool space;
    };
    // The editors' cards.
    struct Scene {
        scene_editor::SceneMsg message;
    };
    struct Terrain {
        terrain_editor::TerrainMsg message;
    };
    struct Track {
        track_editor::TrackMsg message;
    };
    struct Vehicle {
        vehicle_editor::VehicleMsg message;
    };
    // Once a frame in Play: the controls drive the car.
    struct Tick {
        controls::Controls controls;
    };
    // Once a frame, after everything else: keeps the game in the document,
    // commits a change once the button is up, shapes the track, and rebuilds
    // what the 3D view shows.
    struct Prepare {};
    std::variant<Mode, Save, Undo, Redo, NewScene, GoScene, GameAngle, View, Scene, Terrain, Track, Vehicle, Tick, Prepare> value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}
};

class Veehiicuul : public user_interface::Program<Message> {
public:
    game::Game game;
    size_t mode;
    // The view that the scene, terrain and track editors share.
    editing::Viewport viewport;
    scene_editor::SceneEditor scene_editor;
    terrain_editor::TerrainEditor terrain_editor;
    track_editor::TrackEditor track_editor;
    vehicle_editor::VehicleEditor vehicle_editor;
    // The document as of the last save, or as it was loaded.
    document::Document saved;
    // What the last save, or anything else, said.
    std::string status;

    // The game, saving to `save_folder`.
    Veehiicuul(game::Game loaded, std::filesystem::path folder);

    // The game as it starts: with its settings, from the files in its
    // folder, saving to the data folder, which is its folder, or a script's
    // output folder, so that a scripted run leaves the game's files alone.
    // Settings that name what isn't there make notices.
    static std::expected<Veehiicuul, std::string> start(user_interface::Startup& startup);

    bool unsaved() const;

    bool can_undo() const;

    bool can_redo() const;

    void undo();

    void redo();

    // Writes the files that changed since the last save, and deletes the
    // files of scenes the document no longer has, like an undone new scene.
    void save();

    void new_scene(double seconds);

    // Goes to a tab. Going to Play puts the car at the start.
    void go_to(size_t tab, double seconds);

    std::string title() const;

    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ctx);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    void prepare(graphics::Gpu& gpu);

    void render(graphics::Gpu& gpu, const graphics::Frame& frame);

    json::Value state() const;

private:
    // The document before each change, and after each undone one, and the
    // document as of the last change.
    std::vector<document::Document> undo_;
    std::vector<document::Document> redo_;
    document::Document committed;
    // Whether the document differed from `saved` as the frame's messages
    // ended, for the bars.
    bool unsaved_now;
    // The wheel's notches over Play's view this frame, for its Tick.
    double pending_notches;
    // Where Save writes: the game's folder, or a script's output folder.
    std::filesystem::path save_folder;
    // What draws the 3D view, once prepare made it. Tests do without.
    std::optional<drawing::Drawing> drawing;
    // What the frame's 3D view shows.
    camera::Eye eye;
    std::vector<drawing::Instance> instances;

    // Keeps the document as of now, for undo, if it changed.
    void commit();

    // The pointer in the view: in Play, the wheel zooms, through the frame's
    // Tick, and in the editors, the viewport pans and zooms and the tool
    // edits.
    void view_input(const user_interface::ViewInput& input, bool space, const user_interface::Context<Message>& ctx);

    // The app bar's commands: the tabs, in the editors whether their view is
    // at the game's angle, and at its right end, the scene and the commands.
    void commands(user_interface::Ui& ui, user_interface::Tree<Message>& tree, user_interface::Rect bar) const;

    // The status bar: the shortcuts, what the last save said, and whether
    // there are changes to save.
    std::vector<user_interface::Status> status_items() const;

    // What the 3D view shows: the scene's decorations and parked vehicles,
    // and the car, where it is in Play, and at the start in the editors, or
    // in Vehicles, the vehicle alone.
    void build_view();
};

// The card over the bottom left corner of the page in Play: the scene, the
// vehicle, how it's going, and what the controls do, as a table of the
// gamepad's and the keyboard's. It's as big as what it says. Returns its
// area, which scripts and tests see as "Play/Card".
user_interface::Rect play_card(const game::Game& game, user_interface::Ui& ui, user_interface::Tree<Message>& tree, user_interface::Rect page);

} // namespace veehiicuul::program
