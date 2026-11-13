// Made-up input that scripts and Headless share: what a gesture, like a
// click or a drag, does frame by frame. A gesture is the events of its first
// frame, then those of each frame after it, which a script schedules on its
// frames and Headless runs one frame each.

use crate::drawing::geometry::Point;
use crate::input::devices::{Key, Modifiers, MouseButton};
use crate::input::events::Event;

pub(crate) type Frames = Vec<Vec<Event>>;

// The pointer to the point, and the button down there.
pub(crate) fn press(at: Point, button: MouseButton) -> Frames {
    vec![vec![Event::PointerMoved(at), Event::ButtonPressed(button)]]
}

// A press, and its release the next frame.
pub(crate) fn click(at: Point, button: MouseButton) -> Frames {
    let mut frames = press(at, button);
    frames.push(vec![Event::ButtonReleased(button)]);
    frames
}

// Two clicks of the left button, a frame apart.
pub(crate) fn double_click(at: Point) -> Frames {
    vec![
        vec![Event::PointerMoved(at), Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)],
        vec![Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)],
    ]
}

// A press at one point, a move a frame toward the other over `steps`
// frames, and the release there the frame after.
pub(crate) fn drag(from: Point, to: Point, steps: usize, button: MouseButton) -> Frames {
    let steps = steps.max(1);
    let mut frames = press(from, button);
    frames.extend((1..=steps).map(|step| vec![Event::PointerMoved(from.lerp(to, step as f32 / steps as f32))]));
    frames.push(vec![Event::ButtonReleased(button)]);
    frames
}

// The wheel turned over the point, by notches, up for more than 0.
pub(crate) fn scroll(at: Point, notches: f32) -> Frames {
    vec![vec![Event::PointerMoved(at), Event::Wheel(Point::new(0.0, notches))]]
}

// The key with the modifiers, like Ctrl+S, pressed, and let go of the next
// frame, but those that are already held, which stay down.
pub(crate) fn key(modifiers: Modifiers, key: Key, held: &[Key]) -> Frames {
    let modifier_keys = [(modifiers.control, Key::CONTROL), (modifiers.shift, Key::SHIFT), (modifiers.alt, Key::ALT)];
    let keys: Vec<Key> = modifier_keys
        .into_iter()
        .filter_map(|(down, modifier)| down.then_some(modifier))
        .chain([key])
        .filter(|key| !held.contains(key))
        .collect();
    vec![
        keys.iter().map(|key| Event::KeyPressed { key: *key, repeat: false }).collect(),
        keys.iter().rev().map(|key| Event::KeyReleased(*key)).collect(),
    ]
}
