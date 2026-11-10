// Validated transforms for SimplePaint.hlsl's vertex stage. Projection is
// always orthographic, and object and view transforms are affine. Only clip
// XYZ is stored; the vertex shader supplies the invariant clip W = 1.
//
// Matrices follow Library/Math's conventions, which are DirectXMath's.

use math::Matrix;

use crate::Error;

// What SimplePaint.hlsl binds at b0: three columns of the world-to-clip
// transform, and three of the normal-to-view one.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObjectTransforms {
    pub world_to_clip: [[f32; 4]; 3],
    pub normal_to_view: [[f32; 4]; 3],
}

const _: () = assert!(size_of::<ObjectTransforms>() == 96);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub scale: [f32; 3],
    pub depth_offset: f32,
}

// A right-handed orthographic projection of a `width` x `height` view, from
// `near` to `far` in front of the camera, to depths 0 to 1.
pub fn make_projection(width: f32, height: f32, near: f32, far: f32) -> Result<Projection, Error> {
    if !width.is_finite()
        || !height.is_finite()
        || !near.is_finite()
        || !far.is_finite()
        || width < 1.0e-4
        || height < 1.0e-4
        || width > 1.0e6
        || height > 1.0e6
        || near < 0.0
        || far > 1.0e6
        || far - near < 1.0e-4
    {
        return Err(Error(
            "Orthographic projection requires bounded positive dimensions and 0 <= near < far."
                .to_string(),
        ));
    }
    let depth_scale = 1.0 / (near - far);
    Ok(Projection {
        scale: [2.0 / width, 2.0 / height, depth_scale],
        depth_offset: depth_scale * near,
    })
}

// Packs an object's world-view transform with the projection. Only rotations,
// reflections and uniform scale in [1/1024, 1024] are supported; shear and
// nonuniform scale are rejected instead of shading wrong normals.
pub fn build_object_transforms(
    world_view: &Matrix,
    projection: &Projection,
) -> Result<ObjectTransforms, Error> {
    let error = |message: &str| Err(Error(message.to_string()));
    let [scale_x, scale_y, scale_z] = projection.scale;
    let offset = projection.depth_offset;
    if !scale_x.is_finite()
        || !scale_y.is_finite()
        || !scale_z.is_finite()
        || !offset.is_finite()
        || scale_x <= 0.0
        || scale_y <= 0.0
        || scale_z >= 0.0
        || scale_x > 2.0e4
        || scale_y > 2.0e4
        || scale_z < -1.0e4
        || offset.abs() > 1.0e10
    {
        return error("Invalid packed orthographic projection.");
    }
    let m = &world_view.0;
    if m.iter().flatten().any(|value| !value.is_finite() || value.abs() > 1.0e6) {
        return error("SimplePaint transforms must be finite and bounded.");
    }
    if m[0][3] != 0.0 || m[1][3] != 0.0 || m[2][3] != 0.0 || m[3][3] != 1.0 {
        return error(
            "SimplePaint supports affine object/view transforms and orthographic projection only.",
        );
    }
    // The rows' squared lengths: how each basis vector is scaled.
    let lengths: [f64; 3] =
        std::array::from_fn(|i| (0..3).map(|k| m[i][k] as f64 * m[i][k] as f64).sum());
    for i in 0..3 {
        if lengths[i] < 1.0 / (1024.0 * 1024.0)
            || lengths[i] > 1024.0 * 1024.0
            || (lengths[i] - lengths[0]).abs() > 1.0e-5 * lengths[0]
        {
            return error("SimplePaint requires uniform scale in [1/1024,1024].");
        }
        for j in 0..i {
            let dot: f64 = (0..3).map(|k| m[i][k] as f64 * m[j][k] as f64).sum();
            if dot.abs() > 1.0e-5 * lengths[0] {
                return error("SimplePaint does not support sheared object/view transforms.");
            }
        }
    }
    let columns = world_view.transpose().0;
    let scaled = |column: [f32; 4], factor: f32| column.map(|value| value * factor);
    let mut depth = scaled(columns[2], scale_z);
    depth[3] += offset;
    Ok(ObjectTransforms {
        world_to_clip: [scaled(columns[0], scale_x), scaled(columns[1], scale_y), depth],
        // Normals use XYZ only; the validated object and view scale is uniform.
        normal_to_view: [columns[0], columns[1], columns[2]],
    })
}
