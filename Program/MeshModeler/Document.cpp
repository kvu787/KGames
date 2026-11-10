#include "MeshModeler/Document.h"

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "Json/Json.h"

#include <algorithm>
#include <format>
#include <system_error>

namespace mesh_modeler::document {

using paint::default_paints;
using paint::MaximumPaints;
using paint::Paint;
using polygon_mesh::Edge;
using polygon_mesh::Face;
using polygon_mesh::Mesh;

namespace {

// Undo goes back at most this many steps.
constexpr size_t History = 200;

// The text as UTF-8, or nothing if it isn't Unicode, as a lone half of a
// surrogate pair in a file's name isn't: OsStr::to_str.
std::optional<std::string> to_str(std::wstring_view text) {
    for (size_t index = 0; index < text.size(); index++) {
        wchar_t unit = text[index];
        if (unit >= 0xD800 && unit <= 0xDBFF) {
            if (index + 1 >= text.size() || text[index + 1] < 0xDC00 || text[index + 1] > 0xDFFF) {
                return std::nullopt;
            }
            index += 1;
        } else if (unit >= 0xDC00 && unit <= 0xDFFF) {
            return std::nullopt;
        }
    }
    return base::narrow(text);
}

// Unicode's control characters, Cc: char::is_control.
bool is_control(char32_t character) {
    return character <= 0x1F || (character >= 0x7F && character <= 0x9F);
}

} // namespace

Document::Document() : mesh(polygon_mesh::primitives::cube(2.0f)), paints(default_paints()), name(Untitled) {}

std::shared_ptr<const std::set<Edge>> Document::edges() const {
    if (edges_ && edges_->first == version) {
        return edges_->second;
    }
    std::shared_ptr<const std::set<Edge>> made = std::make_shared<const std::set<Edge>>(mesh.edges());
    edges_ = std::pair(version, made);
    return made;
}

Snapshot Document::snapshot() const {
    return Snapshot{
        .mesh = mesh,
        .paints = paints,
        .selection = selection,
    };
}

void Document::restore(Snapshot snapshot) {
    mesh = std::move(snapshot.mesh);
    paints = std::move(snapshot.paints);
    selection = std::move(snapshot.selection);
    displaced = std::nullopt;
    version += 1;
}

bool Document::commit(std::string_view step, Snapshot before) {
    if (before == snapshot()) {
        return false;
    }
    undo_.emplace_back(std::string(step), std::move(before));
    std::optional<Step> oldest;
    if (undo_.size() > History) {
        oldest = std::move(undo_.front());
        undo_.erase(undo_.begin());
    }
    version += 1;
    displaced = Displaced{
        .oldest = std::move(oldest),
        .redo = std::exchange(redo_, {}),
        .dirty = dirty,
        .version = version,
    };
    dirty = true;
    return true;
}

bool Document::withdraw(std::uint64_t at_version) {
    if (version != at_version) {
        return false;
    }
    if (!displaced || displaced->version != at_version) {
        return false;
    }
    Displaced taken = std::move(*displaced);
    displaced = std::nullopt;
    if (undo_.empty()) {
        base::panic("the step that displaced them is the last");
    }
    Snapshot before = std::move(undo_.back().second);
    undo_.pop_back();
    restore(std::move(before));
    if (taken.oldest) {
        undo_.insert(undo_.begin(), std::move(*taken.oldest));
    }
    redo_ = std::move(taken.redo);
    dirty = taken.dirty;
    return true;
}

void Document::touch() {
    version += 1;
}

std::optional<std::string> Document::undo() {
    if (undo_.empty()) {
        return std::nullopt;
    }
    Step step = std::move(undo_.back());
    undo_.pop_back();
    Snapshot now = snapshot();
    restore(std::move(step.second));
    redo_.emplace_back(step.first, std::move(now));
    dirty = true;
    return std::move(step.first);
}

std::optional<std::string> Document::redo() {
    if (redo_.empty()) {
        return std::nullopt;
    }
    Step step = std::move(redo_.back());
    redo_.pop_back();
    Snapshot now = snapshot();
    restore(std::move(step.second));
    undo_.emplace_back(step.first, std::move(now));
    dirty = true;
    return std::move(step.first);
}

std::optional<std::string_view> Document::undo_name() const {
    if (undo_.empty()) {
        return std::nullopt;
    }
    return undo_.back().first;
}

std::optional<std::string_view> Document::redo_name() const {
    if (redo_.empty()) {
        return std::nullopt;
    }
    return redo_.back().first;
}

size_t Document::undo_steps() const {
    return undo_.size();
}

std::optional<size_t> Document::add_paint(Paint paint) {
    if (paints.size() >= MaximumPaints) {
        return std::nullopt;
    }
    paints.push_back(std::move(paint));
    return paints.size() - 1;
}

bool Document::remove_paint(size_t index) {
    if (paints.size() <= 1 || index >= paints.size()) {
        return false;
    }
    paints.erase(paints.begin() + static_cast<std::ptrdiff_t>(index));
    for (Face& face : mesh.faces) {
        size_t material = face.material;
        if (material < index) {
            face.material = static_cast<std::uint32_t>(material);
        } else if (material == index) {
            face.material = 0;
        } else {
            face.material = static_cast<std::uint32_t>(material) - 1;
        }
    }
    return true;
}

std::string Document::to_json() const {
    std::expected<json::Value, json::Error> written = json::parse(mesh.to_json());
    if (!written) {
        base::panic("PolygonMesh writes valid JSON");
    }
    json::Value::Array list;
    list.values.reserve(paints.size());
    for (const Paint& paint : paints) {
        list.values.push_back(paint.to_value());
    }
    json::Value::Object object;
    object.members.reserve(2);
    object.members.emplace_back("Mesh", std::move(*written));
    object.members.emplace_back("Paints", json::Value(std::move(list)));
    return json::Value(std::move(object)).to_json() + "\n";
}

std::expected<Document, std::string> Document::from_json(std::string_view text, std::string_view document_name) {
    std::expected<json::Value, json::Error> root = json::parse(text);
    if (!root) {
        return std::unexpected(std::format("It isn't valid JSON, at line {}, column {}: {}.", root.error().line, root.error().column, root.error().message));
    }
    const json::Value* written = root->get("Mesh");
    if (written == nullptr) {
        return std::unexpected("It has no Mesh.");
    }
    std::expected<Mesh, std::string> read = Mesh::from_json(written->to_json());
    if (!read) {
        return std::unexpected(std::move(read.error()));
    }
    std::vector<Paint> read_paints;
    if (const json::Value* listed = root->get("Paints")) {
        const json::Value::Array* items = base::as<json::Value::Array>(*listed);
        if (items == nullptr) {
            return std::unexpected(std::format("Its Paints must be a list, not {}.", listed->describe()));
        }
        read_paints.reserve(items->values.size());
        for (const json::Value& item : items->values) {
            std::expected<Paint, std::string> paint = Paint::from_value(item);
            if (!paint) {
                return std::unexpected(std::move(paint.error()));
            }
            read_paints.push_back(std::move(*paint));
        }
    } else {
        read_paints = default_paints();
    }
    if (read_paints.empty() || read_paints.size() > MaximumPaints) {
        return std::unexpected(std::format("It must have from 1 to {} paints, not {}.", MaximumPaints, read_paints.size()));
    }
    for (size_t face = 0; face < read->faces.size(); face++) {
        if (read->faces[face].material >= read_paints.size()) {
            return std::unexpected(std::format("Face {} has paint {}, but there are only {} paints.", face, read->faces[face].material, read_paints.size()));
        }
    }
    Document document;
    document.mesh = std::move(*read);
    document.paints = std::move(read_paints);
    document.name = document_name;
    return document;
}

std::expected<std::filesystem::path, std::string> Document::save(const std::filesystem::path& folder) {
    // A file that from_json would reject must not be written.
    if (std::expected<void, std::string> valid = mesh.validate(); !valid) {
        return std::unexpected(std::format("The model can't be saved: {}", valid.error()));
    }
    if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
        return std::unexpected(std::format("Couldn't make the folder {}: {}.", base::display(folder), made.error()));
    }
    std::filesystem::path path = file_path(folder, name);
    if (std::expected<void, std::string> written = base::write(path, std::string_view(to_json())); !written) {
        return std::unexpected(std::format("Couldn't write {}: {}.", base::display(path), written.error()));
    }
    dirty = false;
    return path;
}

std::expected<Document, std::string> Document::open(const std::filesystem::path& folder, std::string_view document_name) {
    std::filesystem::path path = file_path(folder, document_name);
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        return std::unexpected(std::format("Couldn't read {}: {}.", base::display(path), text.error()));
    }
    std::expected<Document, std::string> document = Document::from_json(*text, document_name);
    if (!document) {
        return std::unexpected(std::format("{}: {}", base::display(path), document.error()));
    }
    return document;
}

std::expected<std::filesystem::path, std::string> Document::export_obj(const std::filesystem::path& folder) const {
    if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
        return std::unexpected(std::format("Couldn't make the folder {}: {}.", base::display(folder), made.error()));
    }
    std::filesystem::path path = folder / base::path(std::format("{}.obj", name));
    if (std::expected<void, std::string> written = base::write(path, std::string_view(mesh.to_obj())); !written) {
        return std::unexpected(std::format("Couldn't write {}: {}.", base::display(path), written.error()));
    }
    return path;
}

std::filesystem::path file_path(const std::filesystem::path& folder, std::string_view name) {
    return folder / base::path(std::format("{}.json", name));
}

std::vector<std::string> saved_names(const std::filesystem::path& folder) {
    std::vector<std::string> names;
    std::error_code error;
    std::filesystem::directory_iterator entries(folder, error);
    while (!error && entries != std::filesystem::directory_iterator()) {
        std::filesystem::path path = entries->path();
        if (path.extension().native() == L".json") {
            if (std::optional<std::string> name = to_str(path.stem().native())) {
                names.push_back(std::move(*name));
            }
        }
        entries.increment(error);
    }
    std::ranges::stable_sort(names);
    return names;
}

bool valid_name(std::string_view name) {
    std::string_view trimmed = base::trim(name);
    std::string stem = base::to_ascii_uppercase(trimmed.substr(0, trimmed.find('.')));
    bool device = stem == "CON" || stem == "PRN" || stem == "AUX" || stem == "NUL" ||
                  (stem.size() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && stem[3] >= '0' && stem[3] <= '9');
    constexpr std::u32string_view Forbidden = U"<>:\"/\\|?*";
    bool forbidden = false;
    for (char32_t character : base::chars(trimmed)) {
        if (Forbidden.contains(character) || is_control(character)) {
            forbidden = true;
            break;
        }
    }
    return !trimmed.empty() && trimmed.size() <= 64 && !forbidden && !trimmed.ends_with('.') && !device;
}

} // namespace mesh_modeler::document
