# Meshes

Meshes for [SimplePaint](../SimplePaint/README.md): a UV sphere, a box, and a plane, made in
code, and the car, made in Blender. Each made in code has one material, smooth normals on the
sphere and flat ones on the others. All have triangles wound counterclockwise when seen from
outside, the right-handed convention. The sphere has 4 to 512 vertices around each ring, and 2
to 512 bands from pole to pole. With 3 around, its triangles near the equator fail
SimplePaint's check that their normals share a cone. `Test/Meshes` checks that every mesh
passes SimplePaint's checks.

The car, `meshes::car`, faces +Z, with Y up and its wheels on Y = 0. It's 3 units wide and about
6 long, and has five parts, each with its own material: the axles, the body, the cabin, the
headlights and the wheels. `build.rs` makes it as the crate builds, from the model that Blender
exported for Veehiicuul, `Program/Veehiicuul/Models/SlopeCar.glb`, which it reads with
[Gltf](../Gltf/README.md), so it's always the car in that file. To change the car, change the
model. `VERTICES` and `INDICES` are the model's, in its order, and each part's constant is the
material index of the material that has its name in the file:

| Constant     | Value | Material in the file |
| ------------ | ----- | -------------------- |
| `AXLES`      | 0     | `SlopeCarAxles`      |
| `BODY`       | 1     | `SlopeCarBodyBlue`   |
| `CABIN`      | 2     | `SlopeCarCabinBlue`  |
| `HEADLIGHTS` | 3     | `SlopeCarHeadlights` |
| `WHEELS`     | 4     | `SlopeCarWheels`     |

`MATERIAL_COUNT` is 5. A material in the file that isn't one of these, or one of these that
isn't in the file, fails the build, and says which. The model is in Git LFS, so in a new
worktree, until Git LFS checks it out, it's a small pointer file, and the build fails and says
to run `git lfs checkout`.
