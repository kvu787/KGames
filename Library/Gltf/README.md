# Gltf

Reads models from binary glTF files, `.glb`, like the ones that Blender exports with
File > Export > glTF 2.0, so that models made in Blender come into KGames' programs as they are.
`gltf::load` reads a file into a `Model`: every mesh of the file's default scene as one list of
triangles, with each node's transform applied, and the materials that the triangles use.

```rust
let model = gltf::load(Path::new("Models/SlopeCar.glb"))?;
for material in &model.material_list {
    println!("{}: {:?}", material.name, material.base_color);
}
```

| `Model` member  | What it has                                                                  |
| --------------- | ---------------------------------------------------------------------------- |
| `positions`     | Each vertex's position, in the model's space                                 |
| `normals`       | Each vertex's normal, of length 1                                            |
| `materials`     | Each vertex's material, as an index into `material_list`                     |
| `indices`       | Three vertices a triangle, wound counterclockwise when seen from its front   |
| `material_list` | Each material's `name`, its `base_color` as linear RGBA, and its `extras`,   |
|                 | which Blender fills with the material's custom properties                    |

glTF is Y up and right-handed, and Blender's exporter turns its Z up into Y up, so a model that
faces -Y in Blender faces +Z here. A node's normals go through the inverse transpose of its
transform, so they stay at right angles to its surface under any scale, and a node that mirrors
has its triangles' winding reversed, so their fronts stay out. A primitive without normals gets
flat ones, and one without a material gets one called `Default`, which is white.

Only what KGames uses is read. Texture coordinates, textures, animations, skins, cameras and
lights are ignored. A primitive that isn't triangles, sparse accessors, and data in other files
are errors, which say what's wrong. `Test/Gltf` tests it.
