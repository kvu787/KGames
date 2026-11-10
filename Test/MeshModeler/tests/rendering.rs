// MeshModeler's frames, drawn offscreen with the GPU, as the window would
// draw them: the mesh in the view's card, painted with SimplePaint or lit
// flat, subdivided, and with what's selected tinted in the accent.

use std::path::Path;

use mesh_modeler::modeler::Modeler;
use mesh_modeler::viewport::Shader;
use user_interface::graphics::{AdapterChoice, Gpu, Image};
use user_interface::{Headless, Key, Modifiers, Point};

fn pixel(image: &Image, point: Point) -> [u8; 3] {
    let [r, g, b, _] = image.pixel(point.x as u32, point.y as u32);
    [r, g, b]
}

fn differs(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).any(|(a, b)| a.abs_diff(b) > 8)
}

#[test]
fn the_view_draws_the_mesh_with_either_shader_subdivided_and_selected() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MeshModelerRendering");
    let mut app = Headless::new(Modeler::new(folder));
    app.frames(2);
    let [r, g, b, _] = app.ui.theme().card_opaque().to_bytes();
    let card = [r, g, b];

    let image = app.render(&mut gpu).unwrap();
    let view = app.program.view;
    let corner = view.origin() + Point::new(4.0, 4.0);
    assert!(!differs(pixel(&image, corner), card), "{:?}", pixel(&image, corner));
    // The cube's +Z face is in the middle of the view, in the first paint, a
    // blue.
    let center = view.center() + Point::new(-30.0, 30.0);
    let painted = pixel(&image, center);
    assert!(painted[2] > painted[0] + 40 && painted[2] > painted[1], "{painted:?}");

    // The flat shader colors it differently.
    app.program.shader = Shader::Flat;
    app.frame();
    let flat = pixel(&app.render(&mut gpu).unwrap(), center);
    assert!(differs(flat, painted), "{flat:?} {painted:?}");
    assert!(flat[2] > flat[0], "{flat:?}");

    // Subdivided, the cube rounds into a ball, so the top face's corner at
    // the back of the outline, away from the cage's edges, is no longer
    // drawn.
    let top_corner = app.program.camera.to_screen([-0.85, 1.0, -0.85], view).0;
    let before = pixel(&app.render(&mut gpu).unwrap(), top_corner);
    assert!(differs(before, card), "{before:?}");
    app.key_with(Modifiers::CONTROL, Key::DIGIT_3);
    let rounded = pixel(&app.render(&mut gpu).unwrap(), top_corner);
    assert!(!differs(rounded, card), "{rounded:?}");

    // A selected face is tinted with the accent.
    app.key_with(Modifiers::CONTROL, Key::DIGIT_0);
    let unselected = pixel(&app.render(&mut gpu).unwrap(), center);
    app.click_point(view.center());
    app.move_point(view.origin() + Point::new(10.0, 10.0));
    let selected = pixel(&app.render(&mut gpu).unwrap(), center);
    assert!(differs(selected, unselected), "{selected:?} {unselected:?}");

    // At fullscreen's height, the view's taller, and the mesh still in it.
    app.set_fullscreen(true).frames(2);
    let image = app.render(&mut gpu).unwrap();
    assert!(app.program.view.height > view.height);
    assert!(differs(pixel(&image, app.program.view.center()), card));
}
