// The track builder: edits the current scene's track by its curves, the
// road's outer edge and its islands' edges, whose control points the view
// shows, and the barriers and the road follow them. It also moves the
// start, where the car starts, and turns it, by its handle ahead of it.
//
// | Tool           | What a press and a drag in the view do                     |
// | -------------- | ---------------------------------------------------------- |
// | Move points    | Moves the control point, the start, or the start's handle  |
// | Add points     | Adds a control point to the curve, near where it's closest |
// | Delete points  | Deletes the control point, if its curve has more than 3    |
// | Move the track | Moves every curve, and the start                           |

use json::Value;
use user_interface::{Button, CONTROL_HEIGHT, Frame, Icon, SliderRange};

use crate::curve::{self, FEWEST};
use crate::editing::{self, Viewport, heading, label, slider};
use crate::track::Track;

pub const TOOLS: [&str; 4] = ["Move points", "Add points", "Delete points", "Move track"];
// How near the cursor a point counts as under it, in pixels.
const REACH: f32 = 12.0;
// How far ahead of the start its handle is, in units.
const HANDLE: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    Point(usize, usize),
    Start,
    Heading,
    // The point on the ground that the track was grabbed at.
    Track([f64; 2]),
}

#[derive(Default)]
pub struct TrackEditor {
    pub tool: usize,
    // The curve that points are added to: 0 for the road, and then the
    // islands.
    pub curve: usize,
    drag: Option<Drag>,
    road_text: String,
    barrier_texts: [String; 2],
}

pub fn curve_name(index: usize) -> String {
    if index == 0 { "Road".to_string() } else { format!("Island {index}") }
}

impl TrackEditor {
    pub fn frame(&mut self, frame: &mut Frame, viewport: &mut Viewport, track: &mut Track, problems: &[String]) {
        let theme = frame.theme();
        let inner = editing::card(frame, "Track", "curves whose edges make the road and barriers");
        let mut stack = editing::rows(inner);
        let focusable = frame.set_focusable(false);
        frame.segmented("Track/Tool", stack.next(CONTROL_HEIGHT), &TOOLS, &mut self.tool);

        heading(frame, &mut stack, "Curves");
        self.curve = self.curve.min(track.islands.len());
        let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
        let labels: Vec<&str> = names.iter().map(String::as_str).collect();
        let area = stack.next(CONTROL_HEIGHT * labels.len() as f32);
        frame.radio("Track/Curve", area, &labels, &mut self.curve);
        let add = Button::new("Add an island").icon(Icon::Add);
        let delete = Button::new("Delete the island").icon(Icon::Delete);
        match editing::buttons(frame, &mut stack, &[("Track/AddIsland", add), ("Track/DeleteIsland", delete)]) {
            Some(0) => {
                let size = [frame.width, frame.height];
                let center = viewport.eye.to_ground(editing::view_area(frame).center(), size);
                track.islands.push(curve::circle(center, viewport.eye.height / 12.0, 8));
                self.curve = track.islands.len();
            }
            Some(1) if self.curve > 0 => {
                track.islands.remove(self.curve - 1);
                self.curve -= 1;
            }
            _ => {}
        }
        let count = track.curves().nth(self.curve).map_or(0, Vec::len);
        label(frame, &mut stack, &format!("{} has {count} control points.", curve_name(self.curve)), theme.text);

        heading(frame, &mut stack, "Start");
        slider(
            frame,
            &mut stack,
            "Track/Heading",
            "Heading",
            &mut track.start.heading,
            &SliderRange::new(-180.0, 180.0, 10.0),
            |value| format!("{value:.1} deg"),
        );
        label(frame, &mut stack, "Drag the car, or its handle, to move or turn it.", theme.text_secondary);

        heading(frame, &mut stack, "Barriers");
        slider(
            frame,
            &mut stack,
            "Track/BarrierWidth",
            "Width",
            &mut track.barrier_width,
            &SliderRange::new(0.1, 5.0, 100.0),
            |value| format!("{value:.2}"),
        );
        slider(
            frame,
            &mut stack,
            "Track/BarrierLength",
            "Length",
            &mut track.barrier_length,
            &SliderRange::new(1.0, 50.0, 10.0),
            |value| format!("{value:.1}"),
        );
        frame.set_focusable(focusable);
        heading(frame, &mut stack, "Colors");
        editing::color_field(frame, &mut stack, "Track/RoadColor", "Road", &mut self.road_text, &mut track.road_color);
        let [first, second] = &mut track.barrier_colors;
        let [first_text, second_text] = &mut self.barrier_texts;
        editing::color_field(frame, &mut stack, "Track/BarrierColor0", "Barriers", first_text, first);
        editing::color_field(frame, &mut stack, "Track/BarrierColor1", "", second_text, second);

        heading(frame, &mut stack, "Problems");
        if problems.is_empty() {
            label(frame, &mut stack, "None. The track can be driven.", theme.success);
        }
        for problem in problems {
            label(frame, &mut stack, problem, theme.caution);
        }

        let input = viewport.interact(frame, "Track/View");
        let handle = |track: &Track| {
            let [sin, cos] = track.start.ahead();
            [track.start.position[0] + sin * HANDLE, track.start.position[1] + cos * HANDLE]
        };
        if input.response.pressed
            && !input.panning
            && let (Some(cursor), Some(ground)) = (input.cursor, input.ground)
        {
            let point_under = |track: &Track| {
                track.curves().enumerate().find_map(|(curve, points)| {
                    editing::nearest(frame, viewport, cursor, points, REACH).map(|index| (curve, index))
                })
            };
            match self.tool {
                0 => {
                    self.drag = if editing::nearest(frame, viewport, cursor, &[handle(track)], REACH).is_some() {
                        Some(Drag::Heading)
                    } else if editing::nearest(frame, viewport, cursor, &[track.start.position], REACH * 2.0).is_some() {
                        Some(Drag::Start)
                    } else {
                        point_under(track).map(|(curve, index)| Drag::Point(curve, index))
                    };
                    if let Some(Drag::Point(curve, _)) = self.drag {
                        self.curve = curve;
                    }
                }
                1 => {
                    let curve = self.curve;
                    if let Some(points) = track.curve_mut(curve) {
                        let index = curve::insertion(points, ground).min(points.len());
                        points.insert(index, ground);
                        self.drag = Some(Drag::Point(curve, index));
                    }
                }
                2 => {
                    if let Some((curve, index)) = point_under(track)
                        && let Some(points) = track.curve_mut(curve)
                        && points.len() > FEWEST
                    {
                        points.remove(index);
                        self.curve = curve;
                    }
                }
                _ => self.drag = Some(Drag::Track(ground)),
            }
        }
        if let (Some(drag), Some(ground)) = (self.drag, input.ground) {
            match drag {
                Drag::Point(curve, index) => {
                    if let Some(point) = track.curve_mut(curve).and_then(|points| points.get_mut(index)) {
                        *point = ground;
                    }
                }
                Drag::Start => track.start.position = ground,
                Drag::Heading => track.start.heading = editing::heading_toward(track.start.position, ground),
                Drag::Track(grabbed) => {
                    let shift = [ground[0] - grabbed[0], ground[1] - grabbed[1]];
                    for curve in 0..=track.islands.len() {
                        for point in track.curve_mut(curve).unwrap() {
                            point[0] += shift[0];
                            point[1] += shift[1];
                        }
                    }
                    track.start.position[0] += shift[0];
                    track.start.position[1] += shift[1];
                    self.drag = Some(Drag::Track(ground));
                }
            }
        }
        // The drag followed the cursor to where it was let go of.
        if !input.response.held {
            self.drag = None;
        }

        // The curves' control points, and the lines between them.
        for (index, points) in track.curves().enumerate() {
            let selected = index == self.curve;
            let color = if selected { theme.accent } else { theme.text };
            editing::outline(frame, viewport, points, if selected { 1.5 } else { 1.0 }, color);
            for &point in points.iter() {
                let center = viewport.screen(frame, point);
                editing::dot(frame, viewport, center, if selected { 5.0 } else { 4.0 }, color);
                editing::dot(frame, viewport, center, 2.0, theme.card);
            }
        }
        let [start, end] = [viewport.screen(frame, track.start.position), viewport.screen(frame, handle(track))];
        editing::line(frame, viewport, start, end, 2.0, theme.text);
        editing::dot(frame, viewport, end, 7.0, theme.text);
        editing::dot(frame, viewport, end, 4.0, theme.card);
        if let Some(cursor) = input.cursor
            && !input.panning
        {
            let under = track
                .curves()
                .find_map(|points| editing::nearest(frame, viewport, cursor, points, REACH).map(|index| points[index]));
            if let Some(point) = under {
                let center = viewport.screen(frame, point);
                editing::ring(frame, viewport, center, 9.0, 2.0, theme.accent);
            }
        }
    }

    pub fn state(&self, track: &Track, problems: &[String]) -> Value {
        json::object([
            ("Tool", TOOLS[self.tool].into()),
            ("Curve", curve_name(self.curve).into()),
            ("ControlPoints", Value::Array(track.curves().map(|points| points.len().into()).collect())),
            ("Start", track.start.to_value()),
            ("Problems", problems.to_vec().into()),
        ])
    }
}
