// The packed orthographic transforms against full matrix products, and the
// projection's near and far depth endpoints.

#[path = "Reference.rs"]
mod reference;

use math::{Matrix, dot};
use reference::Random;
use simple_paint::orthographic::{build_object_transforms, make_projection};

fn require_near(actual: f32, expected: f32) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 2.0e-5 * expected.abs().max(1.0),
        "Packed orthographic transform differs from the matrix product: {actual} isn't {expected}"
    );
}

fn main() {
    let mut random = Random::new(0x4F52_5448);
    let mut position = || random.range(-10.0, 10.0) as f32;
    let mut count = 0;
    for aspect in [0.5625f32, 1.0, 16.0 / 9.0, 32.0 / 9.0] {
        let projection = make_projection(5.0 * aspect, 5.0, 1.0, 20.0).unwrap();
        let reference_projection = Matrix::orthographic_rh(5.0 * aspect, 5.0, 1.0, 20.0);
        // Near and far must map correctly, for depth clipping and overlap.
        for (z, depth) in [(-1.0f32, 0.0f32), (-20.0, 1.0)] {
            require_near(z * projection.scale[2] + projection.depth_offset, depth);
        }
        for _ in 0..1000 {
            let angles: [f32; 3] = std::array::from_fn(|_| position() * 0.314);
            let world = Matrix::scaling(0.4, 0.4, 0.4)
                * Matrix::rotation_z(angles[0])
                * Matrix::rotation_x(angles[1])
                * Matrix::rotation_y(angles[2])
                * Matrix::translation(position(), position(), position());
            let view = Matrix::look_at_rh(
                [1.6889755, 3.6863865, 2.9253915],
                [0.0; 3],
                [-0.36863866, 0.6755902, -0.63850087],
            );
            let world_view = world * view;
            let compact = build_object_transforms(&world_view, &projection).unwrap();
            let vertex = [position(), position(), position(), 1.0];
            let normal = [position(), position(), position()];
            let expected_position = (world_view * reference_projection).transform(vertex);
            let expected_normal = world_view.transform_direction(normal);
            for axis in 0..3 {
                let clip: f32 = (0..4).map(|k| vertex[k] * compact.world_to_clip[axis][k]).sum();
                let row = compact.normal_to_view[axis];
                let view_normal = dot(normal, [row[0], row[1], row[2]]);
                require_near(clip, expected_position[axis]);
                require_near(view_normal, expected_normal[axis]);
            }
            require_near(expected_position[3], 1.0);
            count += 1;
        }
    }
    println!("{count} packed transforms, depth endpoints passed.");
}
