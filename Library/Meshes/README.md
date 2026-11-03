# Meshes

Simple meshes for [SimplePaint](../SimplePaint/README.md), made in code: a UV sphere, a box,
and a plane. Each has one material, smooth normals on the sphere and flat ones on the others,
and triangles wound counterclockwise when seen from outside, the right-handed convention.
The sphere has 4 to 512 vertices around each ring, and 2 to 512 bands from pole to pole. With 3
around, its triangles near the equator fail SimplePaint's check that their normals share a
cone. `Test/Meshes` checks that every mesh passes SimplePaint's checks.
