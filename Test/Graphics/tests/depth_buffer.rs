// A depth buffer as big as the frame, which is made again when the frame's
// size changes, and cleared to the far plane for a frame.

use graphics::{AdapterChoice, DepthBuffer, Gpu, Offscreen};

#[test]
fn the_buffer_follows_the_frame_s_size() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut depth = DepthBuffer::new(&gpu).unwrap();
    let mut small = Offscreen::new(&gpu, 64, 32, [0.0; 4]).unwrap();
    let frame = small.begin().unwrap();
    let first = depth.clear(&gpu, &frame).unwrap();
    small.finish(&mut gpu, frame).unwrap();
    let mut large = Offscreen::new(&gpu, 128, 96, [0.0; 4]).unwrap();
    let frame = large.begin().unwrap();
    // The same view, of a new buffer, which the frame's commands clear.
    let second = depth.view(&gpu, &frame).unwrap();
    assert_eq!(first.ptr, second.ptr);
    let image = large.finish(&mut gpu, frame).unwrap();
    assert_eq!((image.width, image.height), (128, 96));
}
