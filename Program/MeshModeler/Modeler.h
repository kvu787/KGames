// The modeler, as a program that Library/UserInterface runs: the document,
// the editor, the camera and how the view shades the mesh, and the commands
// that the menus, the cards' buttons and the keys all go through. Cards.h
// lays out and draws the interface; render draws the mesh into the view's
// card before the interface draws over the rest of the frame.

#pragma once

#include "Base/Sum.h"
#include "Graphics/Graphics.h"
#include "Math/Math.h"
#include "MeshModeler/Camera.h"
#include "MeshModeler/Document.h"
#include "MeshModeler/Editing.h"
#include "MeshModeler/Viewport.h"
#include "PolygonMesh/PolygonMesh.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Widgets/Dialog.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/Layout.h"
#include "Json/Json.h"

#include <array>
#include <concepts>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

namespace mesh_modeler::modeler {

inline constexpr std::string_view Title = "Mesh Modeler";
// The most levels of subdivision that the view shows.
inline constexpr std::uint32_t MaximumLevels = 4;
// Smooth shading splits normals at edges sharper than this, as Blender's auto
// smooth does, so that a cube stays flat and a sphere is smooth.
inline constexpr float SmoothAngleDegrees = 30.0f;

struct Command {
    struct New {
        bool operator==(const New&) const = default;
    };
    struct Open {
        bool operator==(const Open&) const = default;
    };
    struct Save {
        bool operator==(const Save&) const = default;
    };
    struct SaveAs {
        bool operator==(const SaveAs&) const = default;
    };
    struct ExportObj {
        bool operator==(const ExportObj&) const = default;
    };
    struct Undo {
        bool operator==(const Undo&) const = default;
    };
    struct Redo {
        bool operator==(const Redo&) const = default;
    };
    struct SelectAll {
        bool operator==(const SelectAll&) const = default;
    };
    struct SelectNone {
        bool operator==(const SelectNone&) const = default;
    };
    struct InvertSelection {
        bool operator==(const InvertSelection&) const = default;
    };
    struct VertexMode {
        bool operator==(const VertexMode&) const = default;
    };
    struct EdgeMode {
        bool operator==(const EdgeMode&) const = default;
    };
    struct FaceMode {
        bool operator==(const FaceMode&) const = default;
    };
    struct SelectTool {
        bool operator==(const SelectTool&) const = default;
    };
    struct PaintTool {
        bool operator==(const PaintTool&) const = default;
    };
    struct LoopCutTool {
        bool operator==(const LoopCutTool&) const = default;
    };
    struct Move {
        bool operator==(const Move&) const = default;
    };
    struct Rotate {
        bool operator==(const Rotate&) const = default;
    };
    struct Scale {
        bool operator==(const Scale&) const = default;
    };
    struct Extrude {
        bool operator==(const Extrude&) const = default;
    };
    struct Inset {
        bool operator==(const Inset&) const = default;
    };
    struct Fill {
        bool operator==(const Fill&) const = default;
    };
    struct Merge {
        bool operator==(const Merge&) const = default;
    };
    struct Subdivide {
        bool operator==(const Subdivide&) const = default;
    };
    struct Delete {
        bool operator==(const Delete&) const = default;
    };
    struct DeleteFaces {
        bool operator==(const DeleteFaces&) const = default;
    };
    struct FlipNormals {
        bool operator==(const FlipNormals&) const = default;
    };
    struct RecalculateNormals {
        bool operator==(const RecalculateNormals&) const = default;
    };
    struct Crease {
        bool operator==(const Crease&) const = default;
    };
    struct ClearCrease {
        bool operator==(const ClearCrease&) const = default;
    };
    struct AssignPaint {
        bool operator==(const AssignPaint&) const = default;
    };
    struct AddPaint {
        bool operator==(const AddPaint&) const = default;
    };
    struct RemovePaint {
        bool operator==(const RemovePaint&) const = default;
    };
    struct Add {
        editing::Primitive primitive;
        bool operator==(const Add&) const = default;
    };
    struct FrameAll {
        bool operator==(const FrameAll&) const = default;
    };
    struct FrameSelected {
        bool operator==(const FrameSelected&) const = default;
    };
    struct View {
        camera::Viewpoint viewpoint;
        bool operator==(const View&) const = default;
    };
    struct Level {
        std::uint32_t levels;
        bool operator==(const Level&) const = default;
    };
    struct ToggleShader {
        bool operator==(const ToggleShader&) const = default;
    };
    struct ToggleSmooth {
        bool operator==(const ToggleSmooth&) const = default;
    };
    struct ToggleXray {
        bool operator==(const ToggleXray&) const = default;
    };
    struct ToggleGrid {
        bool operator==(const ToggleGrid&) const = default;
    };
    struct ToggleTheme {
        bool operator==(const ToggleTheme&) const = default;
    };
    struct VerticalSync {
        bool operator==(const VerticalSync&) const = default;
    };
    struct Fullscreen {
        bool operator==(const Fullscreen&) const = default;
    };
    std::variant<New, Open, Save, SaveAs, ExportObj, Undo, Redo, SelectAll, SelectNone, InvertSelection, VertexMode, EdgeMode, FaceMode, SelectTool, PaintTool, LoopCutTool, Move,
                 Rotate, Scale, Extrude, Inset, Fill, Merge, Subdivide, Delete, DeleteFaces, FlipNormals, RecalculateNormals, Crease, ClearCrease, AssignPaint, AddPaint,
                 RemovePaint, Add, FrameAll, FrameSelected, View, Level, ToggleShader, ToggleSmooth, ToggleXray, ToggleGrid, ToggleTheme, VerticalSync, Fullscreen>
        value;

    template <base::AlternativeOf<Command> T> constexpr Command(T&& alternative) : value(std::forward<T>(alternative)) {}

    std::string_view label() const;

    // The shortcut that menus and tooltips show, if it has one.
    std::optional<std::string_view> shortcut() const;

    static std::optional<Command> from_label(std::string_view label);

    bool operator==(const Command&) const = default;
};

// A command's shortcuts, like "Ctrl+Y" and "Ctrl+Shift+Z": up to
// MostShortcuts of them, kept in place, so that the table of commands is one
// constant, with no lists beside it for its rows to point at. A range of
// std::string_views.
class Shortcuts {
public:
    static constexpr size_t MostShortcuts = 4;

    // Of the texts, which last as long as the program, like literals.
    template <std::convertible_to<std::string_view>... Texts>
        requires(sizeof...(Texts) <= MostShortcuts)
    constexpr Shortcuts(Texts&&... given) : texts{std::string_view(given)...}, count(sizeof...(Texts)) {}

    constexpr const std::string_view* begin() const {
        return texts.data();
    }

    constexpr const std::string_view* end() const {
        return texts.data() + count;
    }

    constexpr size_t size() const {
        return count;
    }

    constexpr bool empty() const {
        return count == 0;
    }

private:
    std::array<std::string_view, MostShortcuts> texts;
    size_t count;
};

// A command with its label, which menus show and return, and its shortcuts,
// of which menus and tooltips show the first.
struct CommandEntry {
    Command command;
    std::string_view label;
    Shortcuts shortcuts;
};

// Every command.
extern const std::span<const CommandEntry> Commands;

// What a dialog's done with, after the changes are saved or not.
enum class Pending {
    New,
    Open,
    Close,
};

// The dialog that's open.
struct Prompt {
    // Asks for the name to save as.
    struct SaveAs {
        bool operator==(const SaveAs&) const = default;
    };
    // Lists the saved models to open.
    struct Open {
        bool operator==(const Open&) const = default;
    };
    // Asks whether to save the changes first.
    struct Unsaved {
        Pending pending;
        bool operator==(const Unsaved&) const = default;
    };
    std::variant<SaveAs, Open, Unsaved> value;

    template <base::AlternativeOf<Prompt> T> Prompt(T&& alternative) : value(std::forward<T>(alternative)) {}

    std::string_view name() const;

    bool operator==(const Prompt&) const = default;
};

// An edit to the paint that's picked, from its card.
struct PaintEdit {
    struct Color {
        user_interface::Color color;
        bool operator==(const Color&) const = default;
    };
    struct Name {
        std::string name;
        bool operator==(const Name&) const = default;
    };
    struct Brightness {
        double value;
        bool operator==(const Brightness&) const = default;
    };
    struct Shift {
        double value;
        bool operator==(const Shift&) const = default;
    };
    struct Rotation {
        double value;
        bool operator==(const Rotation&) const = default;
    };
    struct Dark {
        double value;
        bool operator==(const Dark&) const = default;
    };
    struct Light {
        double value;
        bool operator==(const Light&) const = default;
    };
    std::variant<Color, Name, Brightness, Shift, Rotation, Dark, Light> value;

    template <base::AlternativeOf<PaintEdit> T> PaintEdit(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const PaintEdit&) const = default;
};

// What the input on the modeler's interface means.
struct Message {
    // A command, from a menu, a button or a key.
    struct Command {
        mesh_modeler::modeler::Command command;
        bool operator==(const Command&) const = default;
    };
    // The crease slider, and the Add card's primitive.
    struct Crease {
        double value;
        bool operator==(const Crease&) const = default;
    };
    struct Primitive {
        size_t index;
        bool operator==(const Primitive&) const = default;
    };
    // The pointer in the view, and on the modal edit's cover.
    struct View {
        user_interface::ViewInput input;
        bool operator==(const View&) const = default;
    };
    struct Operation {
        user_interface::ViewInput input;
        bool operator==(const Operation&) const = default;
    };
    // The modal edit's keys.
    struct Constrain {
        editing::Axis axis;
        bool operator==(const Constrain&) const = default;
    };
    struct ConfirmOperation {
        bool operator==(const ConfirmOperation&) const = default;
    };
    struct CancelOperation {
        bool operator==(const CancelOperation&) const = default;
    };
    // The paints: the one picked, an edit to it, and the edit's end, which
    // commits it as one step of the history.
    struct PickPaint {
        size_t index;
        bool operator==(const PickPaint&) const = default;
    };
    struct Paint {
        PaintEdit edit;
        bool operator==(const Paint&) const = default;
    };
    struct EndPaintEdit {
        bool operator==(const EndPaintEdit&) const = default;
    };
    // The dialogs: their fields, and their buttons.
    struct NameDraft {
        std::string name;
        bool operator==(const NameDraft&) const = default;
    };
    struct OpenChoice {
        size_t index;
        bool operator==(const OpenChoice&) const = default;
    };
    struct OpenActivated {
        bool operator==(const OpenActivated&) const = default;
    };
    struct Unsaved {
        user_interface::DialogResult result;
        bool operator==(const Unsaved&) const = default;
    };
    struct SaveAsResult {
        user_interface::DialogResult result;
        bool operator==(const SaveAsResult&) const = default;
    };
    struct OpenResult {
        user_interface::DialogResult result;
        bool operator==(const OpenResult&) const = default;
    };
    // Once a frame, after everything else: remakes what the view draws.
    struct Prepare {
        bool operator==(const Prepare&) const = default;
    };
    std::variant<Command, Crease, Primitive, View, Operation, Constrain, ConfirmOperation, CancelOperation, PickPaint, Paint, EndPaintEdit, NameDraft, OpenChoice, OpenActivated,
                 Unsaved, SaveAsResult, OpenResult, Prepare>
        value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Message&) const = default;
};

class Modeler : public user_interface::Program<Message> {
public:
    mesh_modeler::document::Document document;
    mesh_modeler::editing::Editor editor;
    mesh_modeler::camera::Camera camera;
    mesh_modeler::viewport::Shader shader = mesh_modeler::viewport::Shader::SimplePaint;
    // Levels of Catmull-Clark subdivision, which the view shows the mesh
    // with, around its cage.
    std::uint32_t levels = 0;
    bool smooth = true;
    bool xray = false;
    bool grid = true;
    // The sharpness that Crease gives edges.
    float crease = 1.0f;
    // The primitive that the Add card adds.
    size_t primitive = 0;
    // Where models are saved: Models in the data folder.
    std::filesystem::path folder;
    std::optional<Prompt> prompt;
    // What a dialog edits: the name to save as, and the model to open.
    std::string name_draft;
    std::optional<size_t> open_choice;
    std::vector<std::string> saved_names;

private:
    // What's pending until a Save as dialog saves.
    std::optional<Pending> after_save;

public:
    // What the last command said, like where a file was saved, for the
    // status bar.
    std::optional<std::pair<user_interface::Severity, std::string>> message;
    // Where the last frame's view is in the window.
    user_interface::Rect view_ = user_interface::Rect::Zero;
    mesh_modeler::viewport::Scene scene;
    // The surface that the view shows, when the mesh is subdivided: the
    // program's own, which `surface` reads.
    std::optional<polygon_mesh::SubdivisionSurface> surface_;

private:
    // What the scene's triangles are of: the document's version, the levels,
    // and whether they're smooth.
    std::optional<std::tuple<std::uint64_t, std::uint32_t, bool>> built;

public:
    // The loop cut's preview's edges, and what they're of.
    std::vector<std::array<math::Vector3, 2>> preview;

private:
    std::optional<std::tuple<std::uint64_t, polygon_mesh::Edge, std::int32_t>> preview_key;
    std::optional<mesh_modeler::viewport::Viewport> viewport;

public:
    // The paints' names and colors as their fields have them, for an edit
    // that's one step of history while a slider is dragged.
    std::optional<mesh_modeler::document::Snapshot> paint_edit;
    // What's wrong with the name in the Save as dialog.
    std::optional<std::string> prompt_error;

    explicit Modeler(std::filesystem::path folder);
    Modeler(const Modeler&) = delete;
    Modeler& operator=(const Modeler&) = delete;
    Modeler(Modeler&&) = default;
    Modeler& operator=(Modeler&&) = default;

    // The modeler, which saves models in Models in its data folder: its own
    // folder, or a script's output folder.
    static Modeler start(user_interface::Startup& startup);

    // The surface that the view shows, when the mesh is subdivided, or null.
    const polygon_mesh::SubdivisionSurface* surface() const;

    // Does a command, from a menu, a button or a key.
    void run(user_interface::Context<Message>& ui, Command command);

    // Whether a command can be done now, for its menu item and button.
    bool can(Command command) const;

    // Whether the command's toggle is on, for its menu item's check.
    std::optional<bool> checked(Command command) const;

    // Does what was pending, now that the changes are saved or not. Closing
    // is done by `finish`, which has the Context.
    void proceed(Pending pending);

    void prompt_save_as(std::optional<Pending> after);

    // Saves under the document's name, or asks for one if it has none yet.
    bool save();

    // Saves under the name in the Save as dialog. Returns what was pending
    // on the save, if it saved.
    std::expected<std::optional<Pending>, std::string> save_as();

    // Opens the model chosen in the Open dialog.
    void open_chosen();

    // A change to the paints from their card, which is one step of the
    // history until the control is let go of.
    void begin_paint_edit();

    void end_paint_edit();

    // Remakes what the view draws for this frame: the triangles, when the
    // mesh or how it's shaded changed, the transforms, the paints, and the
    // overlay.
    void prepare_scene(const user_interface::Theme& theme);

    // The modeler's state, for scripts and tests.
    json::Value state() const;

    // What Library/UserInterface runs the program by.

    std::string title() const;

    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message sent, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    void prepare(graphics::Gpu& gpu);

    void render(graphics::Gpu& gpu, const graphics::Frame& frame);

    bool may_close();

private:
    void say(user_interface::Severity severity, std::string text);

    // Asks whether to save the changes before what's pending, if there are
    // any, or else does it.
    void ask_first(Pending pending);

    // The loop cut's preview, remade only when the edge or where along it
    // changes, to a hundredth.
    void update_preview();

    // Does what was pending once the changes are saved or not.
    void finish(user_interface::Context<Message>& ui, Pending pending);

    // A dialog's answer.
    void answer(user_interface::Context<Message>& ui, Prompt answered, user_interface::DialogResult result);

    // An edit to the paint that's picked, which begins one step of the
    // history if none is under way.
    void edit_paint(PaintEdit edit);

    // Picks the last paint, if the one picked isn't among the paints any
    // more, which a document always has one of.
    void keep_paint_picked();
};

} // namespace mesh_modeler::modeler
