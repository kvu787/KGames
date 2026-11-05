# Meshes

Meshes for [SimplePaint](../SimplePaint/README.md): a UV sphere, a box, and a plane, made in
code, and the car, made in Blender. Each made in code has one material, smooth normals on the
sphere and flat ones on the others. All have triangles wound counterclockwise when seen from
outside, the right-handed convention. The sphere has 4 to 512 vertices around each ring, and 2
to 512 bands from pole to pole. With 3 around, its triangles near the equator fail
SimplePaint's check that their normals share a cone. `Test/Meshes` checks that every mesh
passes SimplePaint's checks.

The car, in `Source/Car.rs`, faces +Z, with Y up and its wheels on Y = 0. It's 3 units wide
and about 6 long, and has five parts, each with its own material: the axles, the body, the
cabin, the headlights and the wheels. It's a baked asset: don't edit it by hand.
