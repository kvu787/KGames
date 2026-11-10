// What the modeler edits: the mesh, its paints and its selection, with the
// history that Undo and Redo go through, and the file it's saved as. A file
// is a JSON object with the mesh, as PolygonMesh writes it, and the paints:
//
//     { "Mesh": { "Positions": [...], "Faces": [...], ... }, "Paints": [{ "Name": "Blue", "Paint": {...} }] }

#pragma once

#include "MeshModeler/Paint.h"
#include "PolygonMesh/PolygonMesh.h"

#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <memory>
#include <optional>
#include <set>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

namespace mesh_modeler::document {

// A new document's name, until it's saved as another.
inline constexpr std::string_view Untitled = "Untitled";

struct Snapshot {
    polygon_mesh::Mesh mesh;
    std::vector<paint::Paint> paints;
    polygon_mesh::Selection selection;

    bool operator==(const Snapshot&) const = default;
};

class Document {
    // A step of the history: its name, and the document before it.
    using Step = std::pair<std::string, Snapshot>;

    // What the last step put out of the history, and how the document was
    // marked before it, which withdrawing the step puts back: the oldest step,
    // if the history was full, and the steps to redo. And the version that the
    // step left the document at.
    struct Displaced {
        std::optional<Step> oldest;
        std::vector<Step> redo;
        bool dirty;
        std::uint64_t version;
    };

public:
    polygon_mesh::Mesh mesh;
    std::vector<paint::Paint> paints;
    polygon_mesh::Selection selection;
    // The name it's saved as, without the folder or extension.
    std::string name;
    // Whether it changed since it was saved or opened.
    bool dirty = false;

private:
    std::vector<Step> undo_;
    std::vector<Step> redo_;
    std::optional<Displaced> displaced;

public:
    // Counts every change, so that what's drawn from the mesh knows to remake
    // itself.
    std::uint64_t version = 1;

private:
    // The mesh's edges, for the version they're of.
    mutable std::optional<std::pair<std::uint64_t, std::shared_ptr<const std::set<polygon_mesh::Edge>>>> edges_;

public:
    // A new document: a cube, in the default paints.
    Document();
    Document(const Document&) = delete;
    Document& operator=(const Document&) = delete;
    Document(Document&&) = default;
    Document& operator=(Document&&) = default;

    // The mesh's edges, made once for each version of the document: the
    // overlay, the cards and picking ask for them every frame.
    std::shared_ptr<const std::set<polygon_mesh::Edge>> edges() const;

    Snapshot snapshot() const;

    void restore(Snapshot snapshot);

    // Records a change: what the document was before it, under the change's
    // name, for Undo. Returns whether there was a change, which is a step.
    bool commit(std::string_view step, Snapshot before);

    // Takes the last step back as if it was never made, if it left the
    // document at the version and nothing changed the document since, like
    // a press's selection that a drag turns into a box: unlike Undo, it
    // leaves nothing to redo, and what the step put out of the history comes
    // back. Returns whether it did.
    bool withdraw(std::uint64_t at_version);

    // Changes the document with `make`, as one step of the history.
    template <class Make> decltype(auto) change(std::string_view step, Make&& make) {
        Snapshot before = snapshot();
        if constexpr (std::is_void_v<std::invoke_result_t<Make, Document&>>) {
            std::forward<Make>(make)(*this);
            commit(step, std::move(before));
            version += 1;
        } else {
            auto result = std::forward<Make>(make)(*this);
            commit(step, std::move(before));
            version += 1;
            return result;
        }
    }

    // Marks the drawn mesh as out of date, like after a change that isn't a
    // step of its own yet.
    void touch();

    std::optional<std::string> undo();

    std::optional<std::string> redo();

    // The names of the steps that Undo and Redo would take.
    std::optional<std::string_view> undo_name() const;

    std::optional<std::string_view> redo_name() const;

    size_t undo_steps() const;

    // Adds a paint, if there's room for one. Returns its index.
    std::optional<size_t> add_paint(paint::Paint paint);

    // Removes a paint, if it isn't the last, and repaints the faces that had
    // it with the first paint.
    bool remove_paint(size_t index);

    std::string to_json() const;

    // A document from a file's text, named `document_name`, or what's wrong
    // with it.
    static std::expected<Document, std::string> from_json(std::string_view text, std::string_view document_name);

    std::expected<std::filesystem::path, std::string> save(const std::filesystem::path& folder);

    static std::expected<Document, std::string> open(const std::filesystem::path& folder, std::string_view document_name);

    // Writes the mesh as a Wavefront OBJ file beside the document's.
    std::expected<std::filesystem::path, std::string> export_obj(const std::filesystem::path& folder) const;
};

std::filesystem::path file_path(const std::filesystem::path& folder, std::string_view name);

// The documents in the folder, by name, in order.
std::vector<std::string> saved_names(const std::filesystem::path& folder);

// Whether a name can be a file's: not empty, without what Windows forbids
// in names, and not one of its devices, like NUL, which a file's name can't
// be, with or without an extension; writing to NUL.json "succeeds", into
// nowhere.
bool valid_name(std::string_view name);

} // namespace mesh_modeler::document
