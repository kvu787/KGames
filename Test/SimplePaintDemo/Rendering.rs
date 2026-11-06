// SimplePaintDemo's frames, drawn offscreen with the GPU, as the window would
// draw them: the sphere fills the preview card's view, around the card's
// color, at both window heights, and is drawn again when the material changes.

use simple_paint::Parameters;
use simple_paint_demo::demo::Demo;
use user_interface::graphics::{AdapterChoice, Gpu, Image};
use user_interface::{Headless, Point, Rect};

fn pixel(image: &Image, point: Point) -> [u8; 3] {
    let [r, g, b, _] = image.pixel(point.x as u32, point.y as u32);
    [r, g, b]
}

fn near(actual: [u8; 3], expected: [u8; 3]) -> bool {
    actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 1)
}

// What the frame shows at the points: the card's color around the sphere,
// inside the view and outside it, and the sphere, right of its middle.
fn check(image: &Image, view: Rect, card: [u8; 3]) -> [u8; 3] {
    let middle = view.center();
    for point in [
        view.origin() + Point::new(8.0, 8.0),
        Point::new(view.right() - 8.0, view.bottom() - 8.0),
        Point::new(view.x - 8.0, middle.y),
        Point::new(view.x + 8.0, middle.y),
    ] {
        assert!(near(pixel(image, point), card), "{:?} at {point:?} isn't the card's {card:?}", pixel(image, point));
    }
    let sphere = pixel(image, middle + Point::new(300.0, 0.0));
    assert!(!near(sphere, card), "the sphere isn't in the view");
    sphere
}

#[test]
fn the_sphere_fills_the_preview_card_s_view_at_both_heights() {
    let mut gpu = Gpu::new(AdapterChoice::PowerSaving).unwrap();
    let mut app = Headless::new(Demo::new(Parameters::default(), r"C:\Nowhere\Paint.json".into()));
    let theme = app.ui.theme().clone();
    let [r, g, b, _] = theme.card_background.over(theme.background).to_bytes();
    let card = [r, g, b];

    let image = app.render(&mut gpu).unwrap();
    let view = app.program.view.unwrap();
    let blue = check(&image, view, card);
    // The default material is a blue.
    assert!(blue[2] > blue[0] + 64 && blue[2] > blue[1], "{blue:?}");

    // In fullscreen, the view is taller, and the sphere is still in its middle.
    app.set_fullscreen(true).frame();
    let image = app.render(&mut gpu).unwrap();
    let tall = app.program.view.unwrap();
    assert!(tall.height > view.height);
    assert_eq!(check(&image, tall, card), blue);

    // A new material draws the sphere again.
    app.program.editor.set(0, 0.9);
    app.program.editor.set(2, 0.1);
    app.frame();
    let image = app.render(&mut gpu).unwrap();
    let red = check(&image, tall, card);
    assert!(red[0] > red[2] + 64, "{red:?}");
}
