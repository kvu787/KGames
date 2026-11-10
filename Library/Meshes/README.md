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
headlights and the wheels. `Generator`, a program of its own, `MeshesGenerator.exe`, makes it as
Meshes builds, from the model that Blender exported for Veehiicuul,
`Program/Veehiicuul/Models/SlopeCar.glb`, which it reads with [Gltf](../Gltf/README.md), so it's
always the car in that file. `Meshes.vcxproj` builds the generator first, and runs it when the
model or the generator has changed. It writes the car as C++, `Car.h`, into the project's
generated folder, `target\Intermediate\<Configuration>\Meshes\Generated`, and `Meshes.cpp`
includes it. To change the car, change the model. `Vertices` and `Indices` are the model's, in
its order, as spans, and each part's constant is the material index of the material that has
its name in the file:

| Constant     | Value | Material in the file |
| ------------ | ----- | -------------------- |
| `Axles`      | 0     | `SlopeCarAxles`      |
| `Body`       | 1     | `SlopeCarBodyBlue`   |
| `Cabin`      | 2     | `SlopeCarCabinBlue`  |
| `Headlights` | 3     | `SlopeCarHeadlights` |
| `Wheels`     | 4     | `SlopeCarWheels`     |

`MaterialCount` is 5. `Meshes.h` has the constants, and what the generator writes doesn't
compile unless each is its part's place in the generator's list. A material in the file that
isn't one of these, or one of these that isn't in the file, fails the build, and says which. The
model is in Git LFS, so in a new worktree, until Git LFS checks it out, it's a small pointer
file, and the build fails and says to run `git lfs checkout`.
