// Made-up input that scripts and Headless share: what a gesture, like a
// click or a drag, does frame by frame. A gesture is the input of its first
// frame, then that of each frame after it, which a script schedules on its
// frames and Headless runs one frame each.
//
// Scripts and tests point at places, and the frame's input has only the
// mice's motion, as the pointer is the interface's (see input/events.rs). So
// a gesture moves the pointer to a place, and that becomes the motion that
// takes the pointer there from where it is as the frame's input comes to it,
// which lands on the place exactly, if it's inside the frame.

use crate::drawing::geometry::{Point, Size};
use crate::input::devices::{Key, Modifiers, MouseButton};
use crate::input::events::{Event, motion_to, moved_pointer};
use crate::input::typing::key_for;

// Input as a test or a script makes it: events, and moves of the pointer to
// places.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Made {
    Event(Event),
    PointTo(Point),
}

pub(crate) type Frames = Vec<Vec<Made>>;

// A frame's events from its made-up input, with the pointer where it is as
// the frame's input comes to it, in a frame of the size: each move becomes
// the motion that takes the pointer there from where the input before it
// left it.
pub(crate) fn events(made: Vec<Made>, mut pointer: Option<Point>, size: Size) -> Vec<Event> {
    made.into_iter()
        .map(|made| {
            let event = match made {
                Made::Event(event) => event,
                Made::PointTo(point) => motion_to(pointer, point, size),
            };
            match event {
                Event::MouseMoved { x, y } => pointer = Some(moved_pointer(pointer, x, y, size)),
                Event::PointerPlaced(at) | Event::PointerMoved(at) => pointer = Some(moved_pointer(Some(at), 0.0, 0.0, size)),
                Event::ButtonPressed(_) | Event::ButtonReleased(_) | Event::Wheel(_) if pointer.is_none() => pointer = Some(moved_pointer(None, 0.0, 0.0, size)),
                _ => {}
            }
            event
        })
        .collect()
}

// Events, as they are.
pub(crate) fn made(events: &[Event]) -> Vec<Made> {
    events.iter().cloned().map(Made::Event).collect()
}

// The pointer to the point, and the button down there.
pub(crate) fn press(at: Point, button: MouseButton) -> Frames {
    vec![vec![Made::PointTo(at), Made::Event(Event::ButtonPressed(button))]]
}

// A press, and its release the next frame.
pub(crate) fn click(at: Point, button: MouseButton) -> Frames {
    let mut frames = press(at, button);
    frames.push(made(&[Event::ButtonReleased(button)]));
    frames
}

// Two clicks of the left button, a frame apart.
pub(crate) fn double_click(at: Point) -> Frames {
    let click = [Event::ButtonPressed(MouseButton::Left), Event::ButtonReleased(MouseButton::Left)];
    let mut first = vec![Made::PointTo(at)];
    first.extend(made(&click));
    vec![first, made(&click)]
}

// A press at one point, a move a frame toward the other over `steps`
// frames, and the release there the frame after.
pub(crate) fn drag(from: Point, to: Point, steps: usize, button: MouseButton) -> Frames {
    let steps = steps.max(1);
    let mut frames = press(from, button);
    frames.extend((1..=steps).map(|step| vec![Made::PointTo(from.lerp(to, step as f32 / steps as f32))]));
    frames.push(made(&[Event::ButtonReleased(button)]));
    frames
}

// The wheel turned over the point, by notches, up for more than 0.
pub(crate) fn scroll(at: Point, notches: f32) -> Frames {
    vec![vec![Made::PointTo(at), Made::Event(Event::Wheel(Point::new(0.0, notches)))]]
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
        keys.iter().map(|key| Made::Event(Event::KeyPressed(*key))).collect(),
        keys.iter().rev().map(|key| Made::Event(Event::KeyReleased(*key))).collect(),
    ]
}

// The keys that type the text, all in one frame: each character's key,
// pressed and let go of, with Shift down for those that need it and up for
// those that don't, and as `shift_held` says it was once the text is typed,
// so that a Shift that a test or a script holds stays held. The text is what
// the keys type on a US keyboard (see input/typing.rs), so a character that
// none of them types is an error, which says which.
pub(crate) fn type_text(text: &str, shift_held: bool) -> Result<Vec<Event>, String> {
    let shift = |down: bool| if down { Event::KeyPressed(Key::SHIFT) } else { Event::KeyReleased(Key::SHIFT) };
    let mut events = Vec::new();
    let mut shift_down = shift_held;
    for character in text.chars() {
        let (key, needs_shift) = key_for(character).ok_or_else(|| format!("can't type {character:?}, which no key of a US keyboard types"))?;
        if needs_shift != shift_down {
            events.push(shift(needs_shift));
            shift_down = needs_shift;
        }
        events.extend([Event::KeyPressed(key), Event::KeyReleased(key)]);
    }
    if shift_down != shift_held {
        events.push(shift(shift_held));
    }
    Ok(events)
}
