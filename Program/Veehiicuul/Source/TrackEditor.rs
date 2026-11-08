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
use user_interface::tokens::CONTROL_HEIGHT;
use user_interface::{Button, Canvas, Card, ComboBox, Form, Icon, InfoBar, Rect, Segmented, Severity, Tree, Ui};

use crate::camera::Eye;
use crate::curve::{self, FEWEST};
use crate::editing::{self, GroundInput, HEADED_HEIGHT, Projection, SliderRange, Viewport, header, nearest, slider_height};
use crate::files;
use crate::program::Message;
use crate::track::Track;

pub const TOOLS: [&str; 4] = ["Move points", "Add points", "Delete points", "Move track"];
const TOOL_ICONS: [Icon; 4] = [Icon::Pin, Icon::Add, Icon::Delete, Icon::Move];
// How near the pointer a point counts as under it, in pixels.
const REACH: f32 = 12.0;
// How far ahead of the start its handle is, in units.
const HANDLE: f64 = 8.0;
// The most problems the card lists.
const MOST_PROBLEMS: usize = 4;

// What the card's controls do.
#[derive(Clone, Debug)]
pub enum TrackMsg {
    Tool(usize),
    Curve(usize),
    // An island: a circle of 8 points in the middle of the view, this big.
    AddIsland { center: [f64; 2], radius: f64 },
    DeleteIsland,
    Heading(f64),
    BarrierWidth(f64),
    BarrierLength(f64),
    RoadColor(files::Color),
    BarrierColor(usize, files::Color),
}

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
    // The control point under the pointer, for the ring the overlay draws.
    under: Option<[f64; 2]>,
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
    // What the pointer in the view does: the tool, on the curves' points,
    // the start, or the whole track.
    pub fn view_input(&mut self, input: GroundInput, track: &mut Track, eye: Eye, size: [f32; 2]) {
        let point_under = |track: &Track, cursor: [f32; 2]| {
            track
                .curves()
                .enumerate()
                .find_map(|(curve, points)| nearest(&eye, size, cursor, points, REACH).map(|index| (curve, index)))
        };
        if input.pressed
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
        if !input.held {
            self.drag = None;
        }
        self.under = input
            .cursor
            .filter(|_| !input.panning)
            .and_then(|cursor| point_under(track, cursor))
            .and_then(|(curve, index)| track.curves().nth(curve).map(|points| points[index]));
    }

    // A control on the card.
    pub fn apply(&mut self, message: TrackMsg, track: &mut Track) {
        match message {
            TrackMsg::Tool(tool) => self.tool = tool,
            TrackMsg::Curve(curve) => self.curve = curve,
            TrackMsg::AddIsland { center, radius } => {
                track.islands.push(curve::circle(center, radius, 8));
                self.curve = track.islands.len();
            }
            TrackMsg::DeleteIsland => {
                // Only while the choice is still an island: undo can take
                // one away between the frame's messages.
                if self.curve > 0 && self.curve <= track.islands.len() {
                    track.islands.remove(self.curve - 1);
                    self.curve -= 1;
                }
            }
            TrackMsg::Heading(heading) => track.start.heading = heading,
            TrackMsg::BarrierWidth(width) => track.barrier_width = width,
            TrackMsg::BarrierLength(length) => track.barrier_length = length,
            TrackMsg::RoadColor(color) => track.road_color = color,
            TrackMsg::BarrierColor(index, color) => track.barrier_colors[index] = color,
        }
        self.curve = self.curve.min(track.islands.len());
    }

    // The overlay over the view, then the card, over both.
    pub fn view(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, viewport: &Viewport, track: &Track, problems: &[String]) {
        self.overlay(ui, tree, view, viewport, track);
        self.card(ui, tree, page, view, viewport, track, problems);
    }

    // The curves' control points, and the lines between them, the start's
    // handle, and a ring around the point under the pointer.
    fn overlay(&self, ui: &mut Ui, tree: &mut Tree<Message>, view: Rect, viewport: &Viewport, track: &Track) {
        let projection = Projection::new(viewport.eye, editing::window_size(ui));
        let curves: Vec<Vec<[f64; 2]>> = track.curves().cloned().collect();
        let selected = self.curve;
        let (start, handle) = (track.start.position, handle(track));
        let under = self.under;
        tree.draw(
            view,
            Canvas::new(move |painter, rect| {
                let theme = painter.theme().clone();
                painter.push_clip(rect);
                for (index, points) in curves.iter().enumerate() {
                    let picked = index == selected;
                    let color = if picked { theme.accent } else { theme.text_primary };
                    projection.outline(painter, points, if picked { 1.5 } else { 1.0 }, color);
                    for &point in points.iter() {
                        let center = projection.point(point);
                        painter.circle(center, if picked { 5.0 } else { 4.0 }, color);
                        painter.circle(center, 2.0, theme.acrylic);
                    }
                }
                let (start, end) = (projection.point(start), projection.point(handle));
                painter.line(start, end, 2.0, theme.text_primary);
                painter.circle(end, 7.0, theme.text_primary);
                painter.circle(end, 4.0, theme.acrylic);
                if let Some(point) = under {
                    painter.ring(projection.point(point), 9.0, 2.0, theme.accent);
                }
                painter.pop_clip();
            }),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn card(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, viewport: &Viewport, track: &Track, problems: &[String]) {
        let card = Card::new("Track").subtitle("Curves that make the road and its barriers").floating();
        let middle = editing::visible(view).center();
        let island_center = viewport.eye.to_ground([middle.x, middle.y], editing::window_size(ui));
        let island_radius = viewport.eye.height / 12.0;
        tree.sized_card(ui, None, editing::left_card(page), card, |ui, card, area| {
            let mut form = Form::new(area.content);
            let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
            card.add("Track/Tool", form.take(CONTROL_HEIGHT), Segmented::new(self.tool, &tools).on_select(|tool| Message::Track(TrackMsg::Tool(tool))));

            form.heading(card, "Curves");
            let curve = self.curve.min(track.islands.len());
            let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let area = form.control(card, "Curve");
            card.add("Track/Curve", area, ComboBox::new(curve, &names).on_select(|curve| Message::Track(TrackMsg::Curve(curve))));
            let [add, delete] = form.columns::<2>(CONTROL_HEIGHT);
            let island = TrackMsg::AddIsland {
                center: island_center,
                radius: island_radius,
            };
            card.add("Track/AddIsland", add, Button::new("Add an island").icon(Icon::Add).on_press(Message::Track(island)))
                .set_tooltip("Adds a circle of 8 points in the middle of the view");
            card.add("Track/DeleteIsland", delete, Button::new("Delete the island").icon(Icon::Delete).on_press(Message::Track(TrackMsg::DeleteIsland)))
                .set_enabled(curve > 0)
                .set_tooltip("Deletes the chosen island");
            let count = track.curves().nth(curve).map_or(0, Vec::len);
            form.caption(ui, card, &format!("{} has {count} control points.", curve_name(curve)));

            form.heading(card, "Start");
            let degrees = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
            editing::slider_row(&mut form, card, "Track/Heading", "Heading", track.start.heading, degrees, |value| {
                Message::Track(TrackMsg::Heading(value))
            });
            form.caption(ui, card, "Drag the car, or its handle, to move or turn it.");

            form.heading(card, "Barriers");
            let [width, length] = form.columns::<2>(slider_height());
            let widths = SliderRange::new(0.1, 5.0, 100.0, 2);
            card.add("Track/BarrierWidth", width, editing::slider("Width", track.barrier_width, widths, |value| {
                Message::Track(TrackMsg::BarrierWidth(value))
            }));
            let lengths = SliderRange::new(1.0, 50.0, 10.0, 1);
            card.add("Track/BarrierLength", length, editing::slider("Length", track.barrier_length, lengths, |value| {
                Message::Track(TrackMsg::BarrierLength(value))
            }));

            form.heading(card, "Colors");
            let [road, first, second] = form.columns::<3>(HEADED_HEIGHT);
            let (road, first) = (header(card, road, "Road"), header(card, first, "Barriers"));
            editing::color_field(card, "Track/RoadColor", road, track.road_color, |color| Message::Track(TrackMsg::RoadColor(color)));
            editing::color_field(card, "Track/BarrierColor0", first, track.barrier_colors[0], |color| {
                Message::Track(TrackMsg::BarrierColor(0, color))
            });
            editing::color_field(card, "Track/BarrierColor1", second.bottom_strip(CONTROL_HEIGHT), track.barrier_colors[1], |color| {
                Message::Track(TrackMsg::BarrierColor(1, color))
            });

            form.heading(card, "Problems");
            if problems.is_empty() {
                let bar = InfoBar::new(Severity::Success, "", "None. The track can be driven.");
                let height = bar.height(ui, form.rest().width);
                card.add("Track/Drivable", form.take(height), bar);
            }
            for (index, problem) in problems.iter().take(MOST_PROBLEMS).enumerate() {
                let bar = InfoBar::new(Severity::Warning, "", problem);
                let height = bar.height(ui, form.rest().width);
                card.add(&format!("Track/Problem/{index}"), form.take(height), bar);
            }
            if problems.len() > MOST_PROBLEMS {
                form.caption(ui, card, &format!("And {} more.", problems.len() - MOST_PROBLEMS));
            }
            form.bottom()
        });
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
