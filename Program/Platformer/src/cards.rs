// Describes the platformer's interface from the game's state: the world,
// drawn into a canvas that fills the window between the bars, the Run card
// and the Events card floating over it, and a banner when the run ends. The
// world's look is the level's own colors; everything else colors with the
// theme's roles, as the design system says.

use user_interface::tokens::{APP_BAR_HEIGHT, BODY, CARD_GAP, MONO, SPACE_L, SPACE_M, SPACE_S, SPACE_XS, STATUS_BAR_HEIGHT, SUBTITLE, TITLE};
use user_interface::{
    Align, AppBar, Border, Button, Canvas, Card, Color, ComboBox, Draw, Field, Form, Icon, Insets, Painter, Point, Rect, Sense, Size,
    Status, Text, Tree, Ui, Widget,
};

use crate::NAME;
use crate::game::{Effect, EffectKind, Game, RunState};
use crate::level::{Level, LevelColor, Zone};
use crate::physics::HALF;
use crate::program::Message;

// Pixels per world unit.
pub const SCALE: f32 = 48.0;
// The cards' width, and the rows of events the Events card shows.
const CARD_WIDTH: f32 = 380.0;
const EVENT_ROWS: usize = 14;
// How wide the app bar's combo box of levels is.
const LEVEL_WIDTH: f32 = 180.0;
// The HUD's field labels.
const LABEL_WIDTH: f32 = 96.0;

// Where the world's canvas is: the window between the app bar and the
// status bar.
pub fn view_area(ui: &Ui) -> Rect {
    let size = ui.size();
    let top = ui.content_top() + APP_BAR_HEIGHT;
    Rect::new(0.0, top, size.width, size.height - top - STATUS_BAR_HEIGHT)
}

pub fn view(game: &Game, ui: &mut Ui) -> Tree<Message> {
    let mut tree = Tree::new();
    // Nothing takes keyboard focus, so the keys stay the game's.
    tree.set_focusable(false);
    let view = view_area(ui);
    world(game, &mut tree, view);

    let shell = tree.shell(ui, AppBar::new(Icon::Flag, NAME), false, Some(status_items(game)));
    commands(game, ui, &mut tree, shell.commands);
    if game.show_cards {
        let mut column = shell.page;
        let hud = hud_card(game, ui, &mut tree, column.left_strip(CARD_WIDTH));
        column.cut_top(hud.height + CARD_GAP, 0.0);
        events_card(game, ui, &mut tree, column.left_strip(CARD_WIDTH));
    }
    banner(game, &mut tree, view);
    tree
}

fn status_items(game: &Game) -> Vec<Status> {
    let mut items = vec![
        Status::shortcut("Arrows", "Run"),
        Status::shortcut("Space", "Jump, held jumps higher"),
        Status::shortcut("R", "Start over"),
        Status::shortcut("PageUp", "Next level"),
        Status::shortcut("F1", if game.show_cards { "Hide the cards" } else { "Show the cards" }),
        Status::shortcut("F9", "Theme"),
        Status::shortcut("F10", "VSync"),
        Status::shortcut("F11", "Fullscreen"),
        Status::Spacer,
    ];
    if !game.status.is_empty() {
        items.push(Status::text(&game.status));
    }
    items
}

// The app bar's commands: the unlocked levels, and starting the run over.
fn commands(game: &Game, ui: &mut Ui, tree: &mut Tree<Message>, mut bar: Rect) {
    let restart = Button::new("Start over").icon(Icon::Refresh).on_press(Message::Restart);
    let restart_area = bar.cut_right(restart.width(ui), SPACE_S);
    let level_area = bar.cut_right(LEVEL_WIDTH, SPACE_S);
    let label_area = bar.cut_right(ui.measure("Level", BODY).width.ceil(), SPACE_L);
    tree.add("Restart", restart_area, restart).set_tooltip("Starts the run over (R)");
    let unlocked = game.progress.unlocked.min(game.levels.len());
    let names: Vec<&str> = game.levels[..unlocked].iter().map(|level| level.name).collect();
    tree.add("Level", level_area, ComboBox::new(game.level, &names).on_select(Message::GoLevel));
    tree.draw(label_area, Text::label("Level").secondary());
}

// The Run card: where the run stands.
fn hud_card(game: &Game, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) -> Rect {
    let level = format!("{}, {} of {}", game.current().name, game.level + 1, game.levels.len());
    let best = match game.progress.best[game.level] {
        Some(best) => format!("{best:.2} s"),
        None => "None yet".to_string(),
    };
    let door = if game.doors_open.iter().all(|&open| open) { "Open" } else { "Closed until every coin" };
    let rows = [
        ("Level", level),
        ("Coins", format!("{} of {}", game.coins, game.total_coins())),
        ("Lives", game.lives.to_string()),
        ("Time", format!("{:.2} s", game.run_seconds())),
        ("Best", best),
        ("Door", door.to_string()),
    ];
    tree.sized_card(ui, Some("Run"), rect, Card::new("Run").floating(), |_, card, area| {
        let mut form = Form::new(area.content).gap(SPACE_XS);
        for (label, value) in rows {
            card.draw(form.take(BODY.line_height), Field::new(label, LABEL_WIDTH, value));
        }
        form.bottom()
    })
}

// The Events card: the newest events, with their times on the simulation's
// clock, newest first. The whole cascade of a moment shares one time.
fn events_card(game: &Game, ui: &mut Ui, tree: &mut Tree<Message>, rect: Rect) {
    let card = Card::new("Events").subtitle("Newest first, on the game's clock").floating();
    tree.sized_card(ui, Some("Events"), rect, card, |ui, card, area| {
        let mut form = Form::new(area.content).gap(SPACE_XS);
        for (index, (seconds, label)) in game.log.iter().take(EVENT_ROWS).enumerate() {
            let line = form.take(MONO.line_height);
            let text = Text::label(format!("{seconds:8.3}  {label}")).style(MONO);
            card.draw(line, if index == 0 { text } else { text.secondary() });
        }
        if game.log.is_empty() {
            form.caption(ui, card, "Nothing yet.");
        }
        form.bottom()
    });
}

// The banner over the middle of the view when the run has ended, painted
// with the theme's roles.
fn banner(game: &Game, tree: &mut Tree<Message>, view: Rect) {
    let (title, line) = match game.state {
        RunState::Running => return,
        RunState::Finished(seconds) => {
            let best = game.progress.best[game.level];
            let note = match best {
                Some(best) if seconds <= best => "A new record, saved.".to_string(),
                Some(best) => format!("The best is {best:.2} s."),
                None => String::new(),
            };
            (format!("Finished in {seconds:.2} s"), format!("{note} R starts again.").trim().to_string())
        }
        RunState::Failed => ("Out of lives".to_string(), "R starts the run over.".to_string()),
    };
    let rect = Rect::from_center(view.center(), Size::new(620.0, 128.0));
    tree.add("Banner", rect, Banner { title, line });
}

// The banner's face: a card-colored plate with the outcome and what R does,
// a target for scripts, which see it the frame the run ends.
struct Banner {
    title: String,
    line: String,
}

impl Widget<Message> for Banner {
    fn kind(&self) -> &'static str {
        "Banner"
    }

    fn label(&self) -> Option<String> {
        Some(self.title.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme().clone();
        painter.shape(rect, 8.0_f32, theme.card_opaque(), Some(Border::new(theme.card_stroke, 1.0)));
        let mut content = rect.pad(Insets::all(SPACE_M));
        painter.text_in(content.cut_top(TITLE.line_height, SPACE_XS), &self.title, TITLE, theme.text_primary, Align::Center);
        painter.text_in(content, &self.line, SUBTITLE, theme.text_secondary, Align::Center);
    }
}

// The world, drawn from the level and the run: the camera follows the
// player inside the level's bounds.
fn world(game: &Game, tree: &mut Tree<Message>, view: Rect) {
    let level = game.current();
    let camera = camera(game, level, view);
    let mut scene = Scene {
        view,
        camera,
        sky: color(level.sky),
        rock: color(level.rock),
        shapes: Vec::new(),
    };
    scene.level(game, level);
    scene.player(game);
    for effect in &game.effects {
        scene.effect(effect, game.simulated, view);
    }
    let Scene { sky, shapes, .. } = scene;
    tree.draw(
        view,
        Canvas::new(move |painter, rect| {
            painter.push_clip(rect);
            painter.fill(rect, sky);
            for shape in &shapes {
                shape.draw(painter);
            }
            painter.pop_clip();
        }),
    );
}

// Where the camera looks, in world units: the player, a little above the
// feet, kept inside the level's bounds, or the bounds' middle where the
// level is smaller than the view.
fn camera(game: &Game, level: &Level, view: Rect) -> [f64; 2] {
    let half = [f64::from(view.width) / 2.0 / f64::from(SCALE), f64::from(view.height) / 2.0 / f64::from(SCALE)];
    let clamp = |center: f64, low: f64, high: f64, half: f64| {
        if high - low <= half * 2.0 {
            (low + high) / 2.0
        } else {
            center.clamp(low + half, high - half)
        }
    };
    let bounds = level.bounds;
    [
        clamp(game.player.position[0], bounds.x, bounds.right(), half[0]),
        clamp(game.player.position[1] + 1.5, bounds.y, bounds.top(), half[1]),
    ]
}

fn color([red, green, blue]: LevelColor) -> Color {
    Color::from_bytes([red, green, blue, 255])
}

// What the canvas draws, in screen pixels, built in view so the closure owns
// plain data.
enum Shape {
    Fill(Rect, Color, f32),
    Circle(Point, f32, Color),
    Ring(Point, f32, f32, Color),
    Polygon(Vec<Point>, Color),
    Line(Point, Point, f32, Color),
    Icon(Point, Icon, f32, Color),
}

impl Shape {
    fn draw(&self, painter: &mut Painter) {
        match self {
            Shape::Fill(rect, color, radius) => painter.rounded(*rect, *radius, *color),
            Shape::Circle(center, radius, color) => painter.circle(*center, *radius, *color),
            Shape::Ring(center, radius, width, color) => painter.ring(*center, *radius, *width, *color),
            Shape::Polygon(points, color) => painter.polygon(points, *color),
            Shape::Line(from, to, width, color) => painter.line(*from, *to, *width, *color),
            Shape::Icon(center, icon, size, color) => painter.icon(*center, *icon, *size, *color),
        }
    }
}

struct Scene {
    view: Rect,
    camera: [f64; 2],
    sky: Color,
    rock: Color,
    shapes: Vec<Shape>,
}

// The world's own colors, beyond the level's.
const COIN: Color = Color::hex(0xF5C542);
const SPRING: Color = Color::hex(0xE2574C);
const DOOR: Color = Color::hex(0x7A5CC6);
const FLAG_UP: Color = Color::hex(0x3BAA5C);
const FLAG_DOWN: Color = Color::hex(0x9AA3AF);
const PLAYER: Color = Color::hex(0x2D7FF9);
const SPIKE: Color = Color::hex(0x39404E);
const CONFETTI: [Color; 5] = [
    Color::hex(0xF5C542),
    Color::hex(0xE2574C),
    Color::hex(0x3BAA5C),
    Color::hex(0x2D7FF9),
    Color::hex(0xB05CC6),
];

impl Scene {
    fn point(&self, world: [f64; 2]) -> Point {
        Point::new(
            self.view.center().x + ((world[0] - self.camera[0]) * f64::from(SCALE)) as f32,
            self.view.center().y - ((world[1] - self.camera[1]) * f64::from(SCALE)) as f32,
        )
    }

    fn zone(&self, zone: Zone) -> Rect {
        let top_left = self.point([zone.x, zone.top()]);
        Rect::new(top_left.x, top_left.y, (zone.width * f64::from(SCALE)) as f32, (zone.height * f64::from(SCALE)) as f32)
    }

    fn level(&mut self, game: &Game, level: &Level) {
        for &solid in &level.solids {
            self.shapes.push(Shape::Fill(self.zone(solid), self.rock, 4.0));
        }
        for patrol in &level.patrols {
            let zone = patrol.at(game.simulated);
            self.shapes.push(Shape::Fill(self.zone(zone), self.rock, 6.0));
            self.shapes.push(Shape::Fill(self.zone(zone).top_strip(4.0), COIN.fade(0.6), 2.0));
        }
        for &spikes in &level.spikes {
            // A row of teeth across the zone.
            let teeth = (spikes.width / 0.5).round().max(1.0) as usize;
            let tooth = spikes.width / teeth as f64;
            for index in 0..teeth {
                let x = spikes.x + tooth * index as f64;
                let points = vec![
                    self.point([x, spikes.y]),
                    self.point([x + tooth / 2.0, spikes.top()]),
                    self.point([x + tooth, spikes.y]),
                ];
                self.shapes.push(Shape::Polygon(points, SPIKE));
            }
        }
        for &spring in &level.springs {
            let rect = self.zone(spring);
            self.shapes.push(Shape::Fill(rect, SPRING, 3.0));
            self.shapes.push(Shape::Fill(rect.top_strip(4.0), Color::WHITE.fade(0.7), 2.0));
        }
        for (index, &zone) in level.checkpoints.iter().enumerate() {
            let reached = game.reached[index];
            let base = self.point([zone.center()[0], zone.y]);
            let top = self.point([zone.center()[0], zone.top()]);
            self.shapes.push(Shape::Line(base, top, 3.0, SPIKE));
            let flag = if reached { FLAG_UP } else { FLAG_DOWN };
            let tip = Point::new(top.x + 18.0, top.y + 7.0);
            self.shapes.push(Shape::Polygon(vec![top, tip, Point::new(top.x, top.y + 14.0)], flag));
        }
        for (index, &door) in level.doors.iter().enumerate() {
            let rect = self.zone(door);
            if game.doors_open[index] {
                // The frame of the opened door.
                self.shapes.push(Shape::Fill(rect.left_strip(3.0), DOOR.fade(0.5), 0.0));
                self.shapes.push(Shape::Fill(rect.right_strip(3.0), DOOR.fade(0.5), 0.0));
            } else {
                self.shapes.push(Shape::Fill(rect, DOOR, 3.0));
                let keyhole = self.point([door.center()[0], door.center()[1]]);
                self.shapes.push(Shape::Circle(keyhole, 4.0, self.sky.fade(0.8)));
            }
        }
        // The goal: a taller pole with the goal flag.
        let goal = level.goal;
        let base = self.point([goal.center()[0], goal.y]);
        let top = self.point([goal.center()[0], goal.top()]);
        self.shapes.push(Shape::Line(base, top, 3.0, SPIKE));
        let tip = Point::new(top.x + 26.0, top.y + 10.0);
        self.shapes.push(Shape::Polygon(vec![top, tip, Point::new(top.x, top.y + 20.0)], FLAG_UP));
        for (index, &coin) in level.coins.iter().enumerate() {
            if !game.collected[index] {
                let center = self.point(coin);
                self.shapes.push(Shape::Circle(center, 9.0, COIN));
                self.shapes.push(Shape::Circle(Point::new(center.x - 2.0, center.y - 2.0), 3.0, Color::WHITE.fade(0.8)));
            }
        }
    }

    fn player(&mut self, game: &Game) {
        if game.state == RunState::Failed {
            return;
        }
        let center = self.point(game.player.position);
        let body = Rect::from_center(center, Size::new((HALF[0] * 2.0 * f64::from(SCALE)) as f32, (HALF[1] * 2.0 * f64::from(SCALE)) as f32));
        self.shapes.push(Shape::Fill(body, PLAYER, 8.0));
        // An eye toward where the player runs.
        let facing = if game.player.velocity[0] < 0.0 { -1.0 } else { 1.0 };
        let eye = Point::new(center.x + 7.0 * facing, center.y - 14.0);
        self.shapes.push(Shape::Circle(eye, 4.0, Color::WHITE));
        self.shapes.push(Shape::Circle(Point::new(eye.x + 1.5 * facing, eye.y), 2.0, SPIKE));
    }

    // One effect, by how far through its life it is.
    fn effect(&mut self, effect: &Effect, now: f64, view: Rect) {
        let age = (now - effect.started).max(0.0);
        let life = effect.kind.life();
        let along = (age / life).clamp(0.0, 1.0) as f32;
        let fade = 1.0 - along;
        let at = self.point(effect.at);
        match effect.kind {
            EffectKind::Sparkle => {
                for index in 0..4 {
                    let angle = std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_2 * index as f32;
                    let reach = 10.0 + 36.0 * along;
                    let center = Point::new(at.x + angle.cos() * reach, at.y - angle.sin() * reach);
                    self.shapes.push(Shape::Circle(center, 4.0 * fade, COIN.fade(fade)));
                }
            }
            EffectKind::Burst | EffectKind::Flag | EffectKind::Shimmer => {
                self.shapes.push(Shape::Ring(at, 12.0 + 60.0 * along, 3.0, Color::WHITE.fade(fade)));
            }
            EffectKind::Bounce => {
                self.shapes.push(Shape::Ring(at, 8.0 + 40.0 * along, 3.0, SPRING.fade(fade)));
            }
            EffectKind::Poof => {
                for index in 0..3 {
                    let x = at.x + (index as f32 - 1.0) * (10.0 + 14.0 * along);
                    let y = at.y - 30.0 * along;
                    self.shapes.push(Shape::Circle(Point::new(x, y), 8.0 * fade + 2.0, FLAG_DOWN.fade(fade * 0.8)));
                }
            }
            EffectKind::Confetti => {
                for index in 0..36_usize {
                    // Deterministic spread from the index alone.
                    let angle = index as f32 * 2.399_963;
                    let speed = 120.0 + 60.0 * ((index * 7) % 5) as f32;
                    let x = at.x + angle.cos() * speed * along;
                    let y = at.y - angle.sin().abs() * speed * along + 160.0 * along * along;
                    let piece = Rect::new(x, y, 6.0, 6.0);
                    self.shapes.push(Shape::Fill(piece, CONFETTI[index % CONFETTI.len()].fade(fade), 1.0));
                }
            }
            EffectKind::SaveFlash => {
                // A save light at the view's top right, not in the world.
                let pulse = (fade * 2.0).min(1.0);
                let center = Point::new(view.right() - 48.0, view.y + 48.0);
                self.shapes.push(Shape::Icon(center, Icon::Save, 24.0, Color::WHITE.fade(pulse)));
                self.shapes.push(Shape::Ring(center, 20.0, 2.0, Color::WHITE.fade(pulse * 0.7)));
            }
        }
    }
}
