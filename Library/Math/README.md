# Math

Vectors and matrices for 3D, in the conventions of DirectXMath, which the C++ code that KGames
ports from uses: vectors are rows that a matrix transforms as `v * M`, so transforms compose
from left to right, like `scaling * rotation * translation`. Matrices are stored row by row,
spaces are right-handed, and angles are in radians.

`Matrix::look_at_rh` and `Matrix::orthographic_rh` match `XMMatrixLookAtRH` and
`XMMatrixOrthographicRH`. [Test/Math](../../Test/Math/Matrices.rs) checks them.
