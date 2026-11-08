// What the modeler edits: the mesh, its paints and its selection, with the
// history that Undo and Redo go through, and the file it's saved as. A file
// is a JSON object with the mesh, as PolygonMesh writes it, and the paints:
//
//     { "Mesh": { "Positions": [...], "Faces": [...], ... }, "Paints": [{ "Name": "Blue", "Paint": {...} }] }

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use json::Value;
use polygon_mesh::{Edge, Mesh, Selection, primitives};

use crate::paint::{MAXIMUM_PAINTS, Paint, default_paints};

// Undo goes back at most this many steps.
const HISTORY: usize = 200;
// A new document's name, until it's saved as another.
pub const UNTITLED: &str = "Untitled";

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub mesh: Mesh,
    pub paints: Vec<Paint>,
    pub selection: Selection,
}

pub struct Document {
    pub mesh: Mesh,
    pub paints: Vec<Paint>,
    pub selection: Selection,
    // The name it's saved as, without the folder or extension.
    pub name: String,
    // Whether it changed since it was saved or opened.
    pub dirty: bool,
    undo: Vec<(String, Snapshot)>,
    redo: Vec<(String, Snapshot)>,
    // Counts every change, so that what's drawn from the mesh knows to remake
    // itself.
    pub version: u64,
    // The mesh's edges, for the version they're of.
    edges: RefCell<Option<(u64, Rc<BTreeSet<Edge>>)>>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    // A new document: a cube, in the default paints.
    pub fn new() -> Self {
        Self {
            mesh: primitives::cube(2.0),
            paints: default_paints(),
            selection: Selection::new(),
            name: UNTITLED.to_string(),
            dirty: false,
            undo: Vec::new(),
            redo: Vec::new(),
            version: 1,
            edges: RefCell::new(None),
        }
    }

    // The mesh's edges, made once for each version of the document: the
    // overlay, the cards and picking ask for them every frame.
    pub fn edges(&self) -> Rc<BTreeSet<Edge>> {
        let mut cached = self.edges.borrow_mut();
        if let Some((version, edges)) = &*cached
            && *version == self.version
        {
            return edges.clone();
        }
        let edges = Rc::new(self.mesh.edges());
        *cached = Some((self.version, edges.clone()));
        edges
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            mesh: self.mesh.clone(),
            paints: self.paints.clone(),
            selection: self.selection.clone(),
        }
    }

    pub fn restore(&mut self, snapshot: Snapshot) {
        self.mesh = snapshot.mesh;
        self.paints = snapshot.paints;
        self.selection = snapshot.selection;
        self.version += 1;
    }

    // Records a change: what the document was before it, under the change's
    // name, for Undo.
    pub fn commit(&mut self, name: &str, before: Snapshot) {
        if before == self.snapshot() {
            return;
        }
        self.undo.push((name.to_string(), before));
        if self.undo.len() > HISTORY {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        self.version += 1;
    }

    // Changes the document with `change`, as one step of the history.
    pub fn change<R>(&mut self, name: &str, change: impl FnOnce(&mut Document) -> R) -> R {
        let before = self.snapshot();
        let result = change(self);
        self.commit(name, before);
        self.version += 1;
        result
    }

    // Marks the drawn mesh as out of date, like after a change that isn't a
    // step of its own yet.
    pub fn touch(&mut self) {
        self.version += 1;
    }

    pub fn undo(&mut self) -> Option<String> {
        let (name, before) = self.undo.pop()?;
        let now = self.snapshot();
        self.restore(before);
        self.redo.push((name.clone(), now));
        self.dirty = true;
        Some(name)
    }

    pub fn redo(&mut self) -> Option<String> {
        let (name, after) = self.redo.pop()?;
        let now = self.snapshot();
        self.restore(after);
        self.undo.push((name.clone(), now));
        self.dirty = true;
        Some(name)
    }

    // The names of the steps that Undo and Redo would take.
    pub fn undo_name(&self) -> Option<&str> {
        self.undo.last().map(|(name, _)| name.as_str())
    }

    pub fn redo_name(&self) -> Option<&str> {
        self.redo.last().map(|(name, _)| name.as_str())
    }

    pub fn undo_steps(&self) -> usize {
        self.undo.len()
    }

    // Adds a paint, if there's room for one. Returns its index.
    pub fn add_paint(&mut self, paint: Paint) -> Option<usize> {
        if self.paints.len() >= MAXIMUM_PAINTS {
            return None;
        }
        self.paints.push(paint);
        Some(self.paints.len() - 1)
    }

    // Removes a paint, if it isn't the last, and repaints the faces that had
    // it with the first paint.
    pub fn remove_paint(&mut self, index: usize) -> bool {
        if self.paints.len() <= 1 || index >= self.paints.len() {
            return false;
        }
        self.paints.remove(index);
        for face in &mut self.mesh.faces {
            let material = face.material as usize;
            face.material = match material.cmp(&index) {
                std::cmp::Ordering::Less => material as u32,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => material as u32 - 1,
            };
        }
        true
    }

    pub fn to_json(&self) -> String {
        let mesh = json::parse(&self.mesh.to_json()).expect("PolygonMesh writes valid JSON");
        let paints = Value::Array(self.paints.iter().map(Paint::to_value).collect());
        Value::Object(vec![("Mesh".to_string(), mesh), ("Paints".to_string(), paints)]).to_json() + "\n"
    }

    // A document from a file's text, named `name`, or what's wrong with it.
    pub fn from_json(text: &str, name: &str) -> Result<Document, String> {
        let root = json::parse(text).map_err(|error| format!("It isn't valid JSON, at line {}, column {}: {}.", error.line, error.column, error.message))?;
        let Some(mesh) = root.get("Mesh") else {
            return Err("It has no Mesh.".to_string());
        };
        let mesh = Mesh::from_json(&mesh.to_json())?;
        let paints = match root.get("Paints") {
            Some(Value::Array(items)) => items.iter().map(Paint::from_value).collect::<Result<Vec<Paint>, String>>()?,
            Some(other) => return Err(format!("Its Paints must be a list, not {}.", other.describe())),
            None => default_paints(),
        };
        if paints.is_empty() || paints.len() > MAXIMUM_PAINTS {
            return Err(format!("It must have from 1 to {MAXIMUM_PAINTS} paints, not {}.", paints.len()));
        }
        if let Some(face) = mesh.faces.iter().position(|face| face.material as usize >= paints.len()) {
            return Err(format!("Face {face} has paint {}, but there are only {} paints.", mesh.faces[face].material, paints.len()));
        }
        Ok(Document {
            mesh,
            paints,
            name: name.to_string(),
            ..Document::new()
        })
    }

    pub fn save(&mut self, folder: &Path) -> Result<PathBuf, String> {
        // A file that from_json would reject must not be written.
        self.mesh.validate().map_err(|error| format!("The model can't be saved: {error}"))?;
        std::fs::create_dir_all(folder).map_err(|error| format!("Couldn't make the folder {}: {error}.", folder.display()))?;
        let path = file_path(folder, &self.name);
        std::fs::write(&path, self.to_json()).map_err(|error| format!("Couldn't write {}: {error}.", path.display()))?;
        self.dirty = false;
        Ok(path)
    }

    pub fn open(folder: &Path, name: &str) -> Result<Document, String> {
        let path = file_path(folder, name);
        let text = std::fs::read_to_string(&path).map_err(|error| format!("Couldn't read {}: {error}.", path.display()))?;
        Document::from_json(&text, name).map_err(|error| format!("{}: {error}", path.display()))
    }

    // Writes the mesh as a Wavefront OBJ file beside the document's.
    pub fn export_obj(&self, folder: &Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(folder).map_err(|error| format!("Couldn't make the folder {}: {error}.", folder.display()))?;
        let path = folder.join(format!("{}.obj", self.name));
        std::fs::write(&path, self.mesh.to_obj()).map_err(|error| format!("Couldn't write {}: {error}.", path.display()))?;
        Ok(path)
    }
}

pub fn file_path(folder: &Path, name: &str) -> PathBuf {
    folder.join(format!("{name}.json"))
}

// The documents in the folder, by name, in order.
pub fn saved_names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    (path.extension().is_some_and(|extension| extension == "json")).then(|| path.file_stem()?.to_str().map(str::to_string)).flatten()
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

// Whether a name can be a file's: not empty, without what Windows forbids
// in names, and not one of its devices, like NUL, which a file's name can't
// be, with or without an extension; writing to NUL.json "succeeds", into
// nowhere.
pub fn valid_name(name: &str) -> bool {
    let name = name.trim();
    let device = {
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && stem.ends_with(|digit: char| digit.is_ascii_digit()))
    };
    !name.is_empty()
        && name.len() <= 64
        && !name.chars().any(|character| "<>:\"/\\|?*".contains(character) || character.is_control())
        && !name.ends_with('.')
        && !device
}
