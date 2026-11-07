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
use user_interface::{Button, Card, ComboBox, Form, Icon, InfoBar, Rect, Segmented, Severity, Ui};

use crate::curve::{self, FEWEST};
use crate::editing::{self, HEADED_HEIGHT, Projection, SliderRange, Viewport, header, nearest, slider_height, window_size};
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
        if !input.held {
            self.drag = None;
        }
        let under = input
            .cursor
            .filter(|_| !input.panning)
            .and_then(|cursor| point_under(track, cursor))
            .and_then(|(curve, index)| track.curves().nth(curve).map(|points| points[index]));
        self.overlay(ui, view, viewport, track, under);
        let page = ui.page();
        self.card(ui, page, view, viewport, track, problems);
    }

    // The curves' control points, and the lines between them, the start's
    // handle, and a ring around the point under the pointer.
    fn overlay(&self, ui: &mut Ui, view: Rect, viewport: &Viewport, track: &Track, under: Option<[f64; 2]>) {
        let projection = Projection::new(ui, viewport.eye);
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.push_clip(view);
        for (index, points) in track.curves().enumerate() {
            let selected = index == self.curve;
            let color = if selected { theme.accent } else { theme.text_primary };
            projection.outline(&mut painter, points, if selected { 1.5 } else { 1.0 }, color);
            for &point in points.iter() {
                let center = projection.point(point);
                painter.circle(center, if selected { 5.0 } else { 4.0 }, color);
                painter.circle(center, 2.0, theme.acrylic);
            }
        }
        let (start, end) = (projection.point(track.start.position), projection.point(handle(track)));
        painter.line(start, end, 2.0, theme.text_primary);
        painter.circle(end, 7.0, theme.text_primary);
        painter.circle(end, 4.0, theme.acrylic);
        if let Some(point) = under {
            painter.ring(projection.point(point), 9.0, 2.0, theme.accent);
        }
        painter.pop_clip();
    }

    fn card(&mut self, ui: &mut Ui, page: Rect, view: Rect, viewport: &Viewport, track: &mut Track, problems: &[String]) {
        let card = Card::new("Track").subtitle("Curves that make the road and its barriers").floating();
        let open = ui.begin_card(editing::left_card(page), card);
        let mut form = Form::new(open.area.content);
        let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
        ui.add("Track/Tool", form.take(CONTROL_HEIGHT), Segmented::new(&mut self.tool, &tools));

        form.heading(ui, "Curves");
        self.curve = self.curve.min(track.islands.len());
        let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let area = form.control(ui, "Curve");
        ui.add("Track/Curve", area, ComboBox::new(&mut self.curve, &names));
        let [add, delete] = form.columns::<2>(CONTROL_HEIGHT);
        let add = ui.add("Track/AddIsland", add, Button::new("Add an island").icon(Icon::Add));
        ui.tooltip(&add, "Adds a circle of 8 points in the middle of the view");
        let delete = ui.add_enabled(self.curve > 0, "Track/DeleteIsland", delete, Button::new("Delete the island").icon(Icon::Delete));
        ui.tooltip(&delete, "Deletes the chosen island");
        if add.clicked {
            let middle = editing::visible(view).center();
            let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
            track.islands.push(curve::circle(center, viewport.eye.height / 12.0, 8));
            self.curve = track.islands.len();
        }
        if delete.clicked && self.curve > 0 {
            track.islands.remove(self.curve - 1);
            self.curve -= 1;
        }
        let count = track.curves().nth(self.curve).map_or(0, Vec::len);
        form.caption(ui, &format!("{} has {count} control points.", curve_name(self.curve)));

        form.heading(ui, "Start");
        let degrees = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
        editing::slider_row(&mut form, ui, "Track/Heading", "Heading", &mut track.start.heading, degrees);
        form.caption(ui, "Drag the car, or its handle, to move or turn it.");

        form.heading(ui, "Barriers");
        let [width, length] = form.columns::<2>(slider_height());
        let widths = SliderRange::new(0.1, 5.0, 100.0, 2);
        editing::slider(ui, "Track/BarrierWidth", width, "Width", &mut track.barrier_width, widths);
        let lengths = SliderRange::new(1.0, 50.0, 10.0, 1);
        editing::slider(ui, "Track/BarrierLength", length, "Length", &mut track.barrier_length, lengths);

        form.heading(ui, "Colors");
        let [road, first, second] = form.columns::<3>(HEADED_HEIGHT);
        let (road, first) = (header(ui, road, "Road"), header(ui, first, "Barriers"));
        editing::color_field(ui, "Track/RoadColor", road, &mut track.road_color);
        let [first_color, second_color] = &mut track.barrier_colors;
        editing::color_field(ui, "Track/BarrierColor0", first, first_color);
        editing::color_field(ui, "Track/BarrierColor1", second.bottom_strip(CONTROL_HEIGHT), second_color);

        form.heading(ui, "Problems");
        if problems.is_empty() {
            let bar = InfoBar::new(Severity::Success, "", "None. The track can be driven.");
            let height = ui.info_bar_height(form.width(), &bar);
            ui.add("Track/Drivable", form.take(height), bar);
        }
        for (index, problem) in problems.iter().take(MOST_PROBLEMS).enumerate() {
            let bar = InfoBar::new(Severity::Warning, "", problem);
            let height = ui.info_bar_height(form.width(), &bar);
            ui.add(&format!("Track/Problem/{index}"), form.take(height), bar);
        }
        if problems.len() > MOST_PROBLEMS {
            form.caption(ui, &format!("And {} more.", problems.len() - MOST_PROBLEMS));
        }
        ui.end_card(open, form.bottom());
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
