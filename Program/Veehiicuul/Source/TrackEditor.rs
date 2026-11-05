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
// | Move track     | Moves every curve, and the start                           |

use json::Value;
use user_interface::{Button, Icon, Rect, Ui};

use crate::curve::{self, FEWEST};
use crate::editing::{self, Form, Overlay, SliderRange, Viewport, nearest, subtitle, window_size};
use crate::track::Track;

pub const TOOLS: [&str; 4] = ["Move points", "Add points", "Delete points", "Move track"];
const TOOL_ICONS: [Icon; 4] = [Icon::Pin, Icon::Add, Icon::Delete, Icon::Move];
// How near the pointer a point counts as under it, in pixels.
const REACH: f32 = 12.0;
// How far ahead of the start its handle is, in units.
const HANDLE: f64 = 8.0;
// The most problems the card lists.
const MOST_PROBLEMS: usize = 4;

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

// Where the start's handle is, which turns it.
fn handle(track: &Track) -> [f64; 2] {
    let [sin, cos] = track.start.ahead();
    [track.start.position[0] + sin * HANDLE, track.start.position[1] + cos * HANDLE]
}

impl TrackEditor {
    // The view, where the tool edits the curves, then the card, over it.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, viewport: &mut Viewport, track: &mut Track, problems: &[String]) {
        let input = viewport.interact(ui, "Track/View", view);
        let size = window_size(ui);
        let eye = viewport.eye;
        let point_under = |track: &Track, cursor: [f32; 2]| {
            track
                .curves()
                .enumerate()
                .find_map(|(curve, points)| nearest(&eye, size, cursor, points, REACH).map(|index| (curve, index)))
        };
        if input.response.pressed
            && !input.panning
            && let (Some(cursor), Some(ground)) = (input.cursor, input.ground)
        {
            match self.tool {
                0 => {
                    self.drag = if nearest(&eye, size, cursor, &[handle(track)], REACH).is_some() {
                        Some(Drag::Heading)
                    } else if nearest(&eye, size, cursor, &[track.start.position], REACH * 2.0).is_some() {
                        Some(Drag::Start)
                    } else {
                        point_under(track, cursor).map(|(curve, index)| Drag::Point(curve, index))
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
                    if let Some((curve, index)) = point_under(track, cursor)
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
        // The drag followed the pointer to where it was let go of.
        if !input.response.held {
            self.drag = None;
        }
        let under = input
            .cursor
            .filter(|_| !input.panning)
            .and_then(|cursor| point_under(track, cursor))
            .and_then(|(curve, index)| track.curves().nth(curve).map(|points| points[index]));
        self.overlay(ui, view, viewport, track, under);
        self.card(ui, view, viewport, track, problems);
    }

    // The curves' control points, and the lines between them, the start's
    // handle, and a ring around the point under the pointer.
    fn overlay(&self, ui: &mut Ui, view: Rect, viewport: &Viewport, track: &Track, under: Option<[f64; 2]>) {
        let theme = ui.theme().clone();
        let mut overlay = Overlay::new(ui, viewport.eye, view);
        for (index, points) in track.curves().enumerate() {
            let selected = index == self.curve;
            let color = if selected { theme.accent } else { theme.text_primary };
            overlay.outline(points, if selected { 1.5 } else { 1.0 }, color);
            for &point in points.iter() {
                let center = overlay.screen(point);
                overlay.dot(center, if selected { 5.0 } else { 4.0 }, color);
                overlay.dot(center, 2.0, theme.flyout);
            }
        }
        let (start, end) = (overlay.screen(track.start.position), overlay.screen(handle(track)));
        overlay.line(start, end, 2.0, theme.text_primary);
        overlay.dot(end, 7.0, theme.text_primary);
        overlay.dot(end, 4.0, theme.flyout);
        if let Some(point) = under {
            let center = overlay.screen(point);
            overlay.ring(center, 9.0, 2.0, theme.accent);
        }
    }

    fn card(&mut self, ui: &mut Ui, view: Rect, viewport: &Viewport, track: &mut Track, problems: &[String]) {
        let theme = ui.theme().clone();
        let page = ui.page();
        let card = ui.floating_card(editing::left_card(page), "Track");
        subtitle(ui, card.header, "Curves that make the road and its barriers");
        let mut form = Form::new(card.content);
        let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
        form.tools(ui, "Track/Tool", &tools, &mut self.tool);

        form.heading(ui, "Curves");
        self.curve = self.curve.min(track.islands.len());
        let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
        form.combo(ui, "Track/Curve", "Curve", &names, &mut self.curve);
        let add = Button::new("Add an island").icon(Icon::Add);
        let delete = Button::new("Delete the island").icon(Icon::Delete);
        match form.buttons(ui, &[("Track/AddIsland", add), ("Track/DeleteIsland", delete)]) {
            Some(0) => {
                let middle = editing::visible(view).center();
                let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
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
        form.caption(ui, &format!("{} has {count} control points.", curve_name(self.curve)));

        form.heading(ui, "Start");
        let degrees = SliderRange::new(-180.0, 180.0, 10.0);
        form.slider(ui, "Track/Heading", "Heading", &mut track.start.heading, degrees, |value| format!("{value:.1}°"));
        form.caption(ui, "Drag the car, or its handle, to move or turn it.");

        form.heading(ui, "Barriers");
        let width = SliderRange::new(0.1, 5.0, 100.0);
        form.slider(ui, "Track/BarrierWidth", "Width", &mut track.barrier_width, width, |value| format!("{value:.2}"));
        let length = SliderRange::new(1.0, 50.0, 10.0);
        form.slider(ui, "Track/BarrierLength", "Length", &mut track.barrier_length, length, |value| format!("{value:.1}"));

        form.heading(ui, "Colors");
        form.color(ui, "Track/RoadColor", "Road", &mut self.road_text, &mut track.road_color);
        let [first, second] = &mut track.barrier_colors;
        let [first_text, second_text] = &mut self.barrier_texts;
        form.color(ui, "Track/BarrierColor0", "Barriers", first_text, first);
        form.color(ui, "Track/BarrierColor1", "", second_text, second);

        form.heading(ui, "Problems");
        if problems.is_empty() {
            form.text(ui, "None. The track can be driven.", theme.success);
        }
        for (index, problem) in problems.iter().take(MOST_PROBLEMS).enumerate() {
            form.warning(ui, &format!("Track/Problem/{index}"), "", problem);
        }
        if problems.len() > MOST_PROBLEMS {
            form.caption(ui, &format!("And {} more.", problems.len() - MOST_PROBLEMS));
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
