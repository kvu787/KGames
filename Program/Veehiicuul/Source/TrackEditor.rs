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
use user_interface::{Button, Card, ComboBox, Form, Icon, InfoBar, Rect, Segmented, Severity, Text, Ui, UiEventKind};

use crate::curve::{self, FEWEST};
use crate::editing::{self, HEADED_HEIGHT, Projection, Role, Shape, SliderRange, Viewport, header, nearest, slider_height, window_size};
use crate::track::Track;

pub const TOOLS: [&str; 4] = ["Move points", "Add points", "Delete points", "Move track"];
const TOOL_ICONS: [Icon; 4] = [Icon::Pin, Icon::Add, Icon::Delete, Icon::Move];
// How near the pointer a point counts as under it, in pixels.
const REACH: f32 = 12.0;
// How far ahead of the start its handle is, in units.
const HANDLE: f64 = 8.0;
// The most problems the card lists.
const MOST_PROBLEMS: usize = 4;

const DEGREES: SliderRange = SliderRange::new(-180.0, 180.0, 10.0, 1).suffix("°");
const WIDTHS: SliderRange = SliderRange::new(0.1, 5.0, 100.0, 2);
const LENGTHS: SliderRange = SliderRange::new(1.0, 50.0, 10.0, 1);

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
    // The islands and the problems shape the card.
    pub fn fingerprint(&self, track: &Track, problems: &[String]) -> String {
        format!("islands {} problems {:?}", track.islands.len(), problems)
    }

    pub fn build(&self, ui: &mut Ui, view: Rect, track: &Track, problems: &[String]) {
        ui.node("Track/View", view, user_interface::View::new());
        editing::overlay_node(ui, "Track/Overlay", view);
        let page = ui.page();
        self.card(ui, page, track, problems);
    }

    fn card(&self, ui: &mut Ui, page: Rect, track: &Track, problems: &[String]) {
        let card = Card::new("Track").subtitle("Curves that make the road and its barriers").floating();
        let area = card.area(ui, editing::left_card(page));
        let mut bottom = area.content.y;
        ui.group("TrackCard", editing::left_card(page), Card::new("Track").subtitle("Curves that make the road and its barriers").floating(), |ui| {
            let mut form = Form::new("TrackForm", area.content);
            let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
            ui.node("Track/Tool", form.take(CONTROL_HEIGHT), Segmented::new(&tools, self.tool));

            form.heading(ui, "Curves");
            let curve = self.curve.min(track.islands.len());
            let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let area = form.control(ui, "Curve");
            ui.node("Track/Curve", area, ComboBox::new(&names, curve));
            let [add, delete] = form.columns::<2>(CONTROL_HEIGHT);
            ui.node("Track/AddIsland", add, Button::new("Add an island").icon(Icon::Add));
            ui.set_tooltip("Track/AddIsland", "Adds a circle of 8 points in the middle of the view");
            ui.node("Track/DeleteIsland", delete, Button::new("Delete the island").icon(Icon::Delete));
            ui.set_enabled("Track/DeleteIsland", curve > 0);
            ui.set_tooltip("Track/DeleteIsland", "Deletes the chosen island");
            let count = track.curves().nth(curve).map_or(0, Vec::len);
            let line = form.take(user_interface::tokens::CAPTION.line_height);
            ui.node("Track/CurveCaption", line, Text::caption(&format!("{} has {count} control points.", curve_name(curve))));

            form.heading(ui, "Start");
            editing::slider_row(&mut form, ui, "Track/Heading", "Heading", track.start.heading, DEGREES);
            form.caption(ui, "Drag the car, or its handle, to move or turn it.");

            form.heading(ui, "Barriers");
            let [width, length] = form.columns::<2>(slider_height());
            editing::slider_node(ui, "Track/BarrierWidth", width, "Width", track.barrier_width, WIDTHS);
            editing::slider_node(ui, "Track/BarrierLength", length, "Length", track.barrier_length, LENGTHS);

            form.heading(ui, "Colors");
            let [road, first, second] = form.columns::<3>(HEADED_HEIGHT);
            let road = header(ui, "Track/RoadLabel", road, "Road");
            let first = header(ui, "Track/BarriersLabel", first, "Barriers");
            editing::color_field_node(ui, "Track/RoadColor", road, track.road_color);
            editing::color_field_node(ui, "Track/BarrierColor0", first, track.barrier_colors[0]);
            editing::color_field_node(ui, "Track/BarrierColor1", second.bottom_strip(CONTROL_HEIGHT), track.barrier_colors[1]);

            form.heading(ui, "Problems");
            if problems.is_empty() {
                let bar = InfoBar::new(Severity::Success, "", "None. The track can be driven.");
                let height = bar.height(ui, form.width());
                ui.node("Track/Drivable", form.take(height), bar);
            }
            for (index, problem) in problems.iter().take(MOST_PROBLEMS).enumerate() {
                let bar = InfoBar::new(Severity::Warning, "", problem);
                let height = bar.height(ui, form.width());
                ui.node(&format!("Track/Problem/{index}"), form.take(height), bar);
            }
            if problems.len() > MOST_PROBLEMS {
                form.caption(ui, &format!("And {} more.", problems.len() - MOST_PROBLEMS));
            }
            bottom = form.bottom();
        });
        let rect = Rect::new(page.x, page.y, editing::CARD_WIDTH, card.height_around(bottom - area.content.y));
        ui.set_rect("TrackCard", rect);
    }

    // The view, where the tool edits the curves, the frame's events, and
    // what the card shows.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, viewport: &mut Viewport, track: &mut Track, _problems: &[String]) {
        let input = viewport.apply(ui, "Track/View");
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
        self.events(ui, view, viewport, track);
        let under = input
            .cursor
            .filter(|_| !input.panning)
            .and_then(|cursor| point_under(track, cursor))
            .and_then(|(curve, index)| track.curves().nth(curve).map(|points| points[index]));
        self.overlay(ui, viewport, track, under);
        self.sync(ui, track);
    }

    fn events(&mut self, ui: &mut Ui, view: Rect, viewport: &Viewport, track: &mut Track) {
        let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
        for event in events {
            match (event.source.as_str(), &event.kind) {
                ("Track/Tool", UiEventKind::Selected(index)) => self.tool = *index,
                ("Track/Curve", UiEventKind::Selected(index)) => self.curve = *index,
                ("Track/AddIsland", UiEventKind::Clicked) => {
                    let middle = editing::visible(view).center();
                    let center = viewport.eye.to_ground([middle.x, middle.y], window_size(ui));
                    track.islands.push(curve::circle(center, viewport.eye.height / 12.0, 8));
                    self.curve = track.islands.len();
                }
                ("Track/DeleteIsland", UiEventKind::Clicked) => {
                    if self.curve > 0 {
                        track.islands.remove(self.curve - 1);
                        self.curve -= 1;
                    }
                }
                ("Track/Heading", UiEventKind::Changed(value)) => track.start.heading = DEGREES.nearest(*value),
                ("Track/BarrierWidth", UiEventKind::Changed(value)) => track.barrier_width = WIDTHS.nearest(*value),
                ("Track/BarrierLength", UiEventKind::Changed(value)) => track.barrier_length = LENGTHS.nearest(*value),
                ("Track/RoadColor", UiEventKind::ColorChanged(color)) => {
                    let [red, green, blue, _] = color.to_bytes();
                    track.road_color = [red, green, blue];
                }
                ("Track/BarrierColor0", UiEventKind::ColorChanged(color)) => {
                    let [red, green, blue, _] = color.to_bytes();
                    track.barrier_colors[0] = [red, green, blue];
                }
                ("Track/BarrierColor1", UiEventKind::ColorChanged(color)) => {
                    let [red, green, blue, _] = color.to_bytes();
                    track.barrier_colors[1] = [red, green, blue];
                }
                _ => {}
            }
        }
        self.curve = self.curve.min(track.islands.len());
    }

    // The curves' control points, and the lines between them, the start's
    // handle, and a ring around the point under the pointer.
    fn overlay(&self, ui: &mut Ui, viewport: &Viewport, track: &Track, under: Option<[f64; 2]>) {
        let projection = Projection::new(ui, viewport.eye);
        let mut shapes = Vec::new();
        for (index, points) in track.curves().enumerate() {
            let selected = index == self.curve;
            let role = if selected { Role::Accent } else { Role::Primary };
            shapes.push(projection.outline(points, if selected { 1.5 } else { 1.0 }, role));
            for &point in points.iter() {
                let center = projection.point(point);
                shapes.push(Shape::Circle(center, if selected { 5.0 } else { 4.0 }, role));
                shapes.push(Shape::Circle(center, 2.0, Role::Acrylic));
            }
        }
        let (start, end) = (projection.point(track.start.position), projection.point(handle(track)));
        shapes.push(Shape::Line(start, end, 2.0, Role::Primary));
        shapes.push(Shape::Circle(end, 7.0, Role::Primary));
        shapes.push(Shape::Circle(end, 4.0, Role::Acrylic));
        if let Some(point) = under {
            shapes.push(Shape::Ring(projection.point(point), 9.0, 2.0, Role::Accent));
        }
        editing::set_overlay(ui, "Track/Overlay", shapes);
    }

    fn sync(&self, ui: &mut Ui, track: &Track) {
        ui.get_mut::<Segmented>("Track/Tool").set_selected(self.tool);
        let names: Vec<String> = (0..=track.islands.len()).map(curve_name).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let combo = ui.get_mut::<ComboBox>("Track/Curve");
        combo.set_items(&names);
        combo.set_selected(self.curve);
        ui.set_enabled("Track/DeleteIsland", self.curve > 0);
        let count = track.curves().nth(self.curve).map_or(0, Vec::len);
        ui.get_mut::<Text>("Track/CurveCaption").set_text(&format!("{} has {count} control points.", curve_name(self.curve)));
        editing::sync_slider(ui, "Track/Heading", track.start.heading);
        editing::sync_slider(ui, "Track/BarrierWidth", track.barrier_width);
        editing::sync_slider(ui, "Track/BarrierLength", track.barrier_length);
        editing::sync_color_field(ui, "Track/RoadColor", track.road_color);
        editing::sync_color_field(ui, "Track/BarrierColor0", track.barrier_colors[0]);
        editing::sync_color_field(ui, "Track/BarrierColor1", track.barrier_colors[1]);
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
