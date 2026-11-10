// MeshModeler's files: a document's JSON keeps its mesh and paints; Save as,
// Open and New go through their dialogs, which ask before changes are lost;
// and files that are wrong say what's wrong with them.

#include "Base/Files.h"
#include "Base/Text.h"
#include "MeshModeler/MeshModeler.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <expected>
#include <filesystem>
#include <format>
#include <limits>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <vector>

using mesh_modeler::document::Document;
using mesh_modeler::modeler::Modeler;
using mesh_modeler::paint::MaximumPaints;
using mesh_modeler::paint::Paint;
using polygon_mesh::Edge;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Point;

namespace {

std::filesystem::path folder_for(std::string_view test) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "MeshModelerFilesTests" / test;
    std::error_code ignored;
    std::filesystem::remove_all(folder, ignored);
    std::filesystem::create_directories(folder);
    return folder;
}

// The name of the dialog that's open, if one is.
std::optional<std::string_view> prompt_name(const Headless<Modeler>& app) {
    if (!app.program.prompt) {
        return std::nullopt;
    }
    return app.program.prompt->name();
}

// What a test expects of a dialog's name.
std::optional<std::string_view> named(std::string_view name) {
    return name;
}

} // namespace

TEST(a_document_keeps_its_mesh_creases_and_paints_through_json) {
    Document document;
    document.mesh = polygon_mesh::primitives::torus(1.0f, 0.25f, 12, 8);
    document.mesh.assign_material({3, 4}, 2);
    document.mesh.set_crease(Edge(0, 1), 2.5f);
    document.paints[1].set_color(user_interface::Color::hex(0x336699));
    document.paints[1].parameters.shift = 0.25;
    document.paints[1].name = "Steel";
    std::string text = document.to_json();
    Document read = Document::from_json(text, "Torus").value();
    CHECK_EQ(read.mesh, document.mesh);
    CHECK_EQ(read.paints.size(), document.paints.size());
    CHECK_EQ(read.paints[1].name, "Steel");
    CHECK(std::abs(read.paints[1].parameters.shift - 0.25) < 1e-9);
    CHECK_EQ(read.paints[1].color().to_hex(), document.paints[1].color().to_hex());
    CHECK_EQ(read.name, "Torus");
    CHECK(!read.dirty);
}

TEST(wrong_files_say_what_s_wrong) {
    auto error = [](std::string_view text) {
        std::expected<Document, std::string> read = Document::from_json(text, "Wrong");
        CHECK(!read.has_value());
        return read.error();
    };
    CHECK(error("{").contains("isn't valid JSON"));
    CHECK(error("{}").contains("no Mesh"));
    std::string cube = Document().to_json();
    std::string no_paints = base::replace(cube, "\"Paints\"", "\"Unused\"");
    CHECK_EQ(Document::from_json(no_paints, "Cube").value().paints.size(), 4);
    Document many;
    many.paints.clear();
    for (size_t index = 0; index <= MaximumPaints; index++) {
        many.paints.push_back(Paint(std::format("P{}", index), 0x808080));
    }
    CHECK(error(many.to_json()).contains("from 1 to 16 paints"));
    Document painted;
    painted.mesh.faces[2].material = 7;
    CHECK(error(painted.to_json()).contains("Face 2 has paint 7"));
}

TEST(names_are_checked_for_what_windows_forbids) {
    for (std::string_view good : {"Cube", "My model 2", "a.b"}) {
        CHECK(mesh_modeler::document::valid_name(good), "{}", good);
    }
    for (std::string_view bad : {"", "   ", "a/b", "a:b", "what?", "dot.", "a\tb", "NUL", "nul", "con", "COM1", "lpt9", "Aux.old"}) {
        CHECK(!mesh_modeler::document::valid_name(bad), "{}", base::debug(bad));
    }
    for (std::string_view good : {"Null", "Console", "COM", "LPT10", "Nulls"}) {
        CHECK(mesh_modeler::document::valid_name(good), "{}", good);
    }
}

// A mesh that from_json would reject must not be written: save says what's
// wrong instead of losing the model to a file that can't be opened.
TEST(a_broken_mesh_refuses_to_save) {
    std::filesystem::path folder = folder_for("Broken");
    Headless<Modeler> app{Modeler(folder)};
    app.frames(2);
    app.program.document.mesh.positions[0][0] = std::numeric_limits<float>::infinity();
    app.program.name_draft = "Broken";
    std::expected<std::filesystem::path, std::string> saved = app.program.document.save(folder);
    CHECK(!saved.has_value());
    std::string problem = saved.error();
    CHECK(problem.contains("can't be saved"), "{}", problem);
    CHECK(std::filesystem::directory_iterator(folder) == std::filesystem::directory_iterator(), "nothing was written");
}

TEST(save_as_open_and_new_go_through_their_dialogs) {
    std::filesystem::path folder = folder_for("Dialogs");
    Headless<Modeler> app{Modeler(folder)};
    app.frames(2);
    // An untitled model asks for a name.
    app.key(Key::A);
    app.key_with(Modifiers::Control, Key::S);
    CHECK_EQ(prompt_name(app), named("SaveAs"));
    // A name that can't be a file's says so, and stays open.
    app.program.name_draft = "a/b";
    app.click("SaveAs/Primary");
    CHECK(app.program.prompt_error.value().contains("can't be empty"));
    CHECK_EQ(prompt_name(app), named("SaveAs"));
    app.program.name_draft = "Box";
    app.click("SaveAs/Primary");
    CHECK(!app.program.prompt.has_value());
    CHECK(std::filesystem::is_regular_file(folder / "Box.json"));
    CHECK_EQ(app.program.document.name, "Box");
    CHECK(!app.program.document.dirty);

    // Ctrl+S saves under its name, without asking.
    app.key_with(Modifiers::Alt, Key::A);
    Point center = app.program.view_.center();
    app.click_point(center);
    app.key(Key::E);
    app.key(Key::Enter);
    CHECK(app.program.document.dirty);
    app.key_with(Modifiers::Control, Key::S);
    CHECK(!app.program.prompt.has_value());
    CHECK(!app.program.document.dirty);
    Document saved = Document::open(folder, "Box").value();
    CHECK_EQ(saved.mesh.faces.size(), 10);

    // New with changes asks first; Cancel keeps them.
    app.key_with(Modifiers::Alt, Key::A);
    app.key_with(Modifiers::Control, Key::N);
    CHECK_EQ(prompt_name(app), named("Unsaved"));
    app.click("Unsaved/Close");
    CHECK_EQ(app.program.document.name, "Box");
    app.key_with(Modifiers::Control, Key::N);
    app.click("Unsaved/Secondary");
    CHECK_EQ(app.program.document.name, mesh_modeler::document::Untitled);
    CHECK_EQ(app.program.document.mesh.faces.size(), 6);

    // Open lists the saved models, and opens the one picked.
    app.key_with(Modifiers::Control, Key::O);
    CHECK_EQ(prompt_name(app), named("OpenModel"));
    CHECK_EQ(app.program.saved_names, std::vector<std::string>{"Box"});
    app.click("OpenModel/Primary");
    CHECK_EQ(app.program.document.name, "Box");
    CHECK_EQ(app.program.document.mesh.faces.size(), 10);
    CHECK(!app.program.prompt.has_value());

    // Export writes an OBJ file beside the model.
    app.key_with(Modifiers::Control, Key::E);
    std::string obj = base::read_to_string(folder / "Box.obj").value();
    CHECK_EQ(std::ranges::count_if(base::lines(obj), [](std::string_view line) { return line.starts_with("f "); }), 10);
}

TEST(saving_an_untitled_model_from_the_unsaved_dialog_asks_for_its_name_then_goes_on) {
    std::filesystem::path folder = folder_for("SaveThenNew");
    Headless<Modeler> app{Modeler(folder)};
    app.frames(2);
    app.key(Key::A);
    app.key_with(Modifiers::Control, Key::N);
    app.click("Unsaved/Primary");
    CHECK_EQ(prompt_name(app), named("SaveAs"));
    app.program.name_draft = "First";
    app.click("SaveAs/Primary");
    // Saved, then new, as asked.
    CHECK(std::filesystem::is_regular_file(folder / "First.json"));
    CHECK_EQ(app.program.document.name, mesh_modeler::document::Untitled);
    CHECK(!app.program.prompt.has_value());
}
