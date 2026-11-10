#include "Platformer/Cards.h"

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "Base/Vec.h"
#include "Platformer/Level.h"
#include "Platformer/Physics.h"
#include "Platformer/Platformer.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/Card.h"
#include "UserInterface/Widgets/ComboBox.h"
#include "UserInterface/Widgets/Layout.h"
#include "UserInterface/Widgets/Shell.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <format>
#include <numbers>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace platformer::cards {

using game::Effect;
using game::EffectKind;
using game::Game;
using game::RunState;
using level::Level;
using level::LevelColor;
using level::Patrol;
using level::Zone;
using physics::Half;
using program::Message;
using user_interface::Align;
using user_interface::AppBar;
using user_interface::Border;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Color;
using user_interface::ComboBox;
using user_interface::Draw;
using user_interface::Field;
using user_interface::Form;
using user_interface::Icon;
using user_interface::Insets;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Sense;
using user_interface::Shell;
using user_interface::Size;
using user_interface::Status;
using user_interface::Text;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::Widget;
namespace tokens = user_interface::tokens;

namespace {

// The cards' width, and the rows of events the Events card shows.
constexpr float CardWidth = 380.0f;
constexpr size_t EventRows = 14;
// How wide the app bar's combo box of levels is.
constexpr float LevelWidth = 180.0f;
// The HUD's field labels.
constexpr float LabelWidth = 96.0f;

std::vector<Status> status_items(const Game& game) {
    std::vector<Status> items = base::vec(Status::shortcut("Arrows", "Run"), Status::shortcut("Space", "Jump, held jumps higher"), Status::shortcut("R", "Start over"),
                                          Status::shortcut("PageUp", "Next level"), Status::shortcut("F1", game.show_cards ? "Hide the cards" : "Show the cards"),
                                          Status::shortcut("F9", "Theme"), Status::shortcut("F10", "VSync"), Status::shortcut("F11", "Fullscreen"), Status(Status::Spacer{}));
    if (!game.status.empty()) {
        items.push_back(Status::text(game.status));
    }
    return items;
}

// The app bar's commands: the unlocked levels, and starting the run over.
void commands(const Game& game, Ui& ui, Tree<Message>& tree, Rect bar) {
    Button<Message> restart = Button<Message>("Start over").icon(Icon::Refresh).on_press(Message::Restart{});
    Rect restart_area = bar.cut_right(restart.width(ui), tokens::SpaceS);
    Rect level_area = bar.cut_right(LevelWidth, tokens::SpaceS);
    Rect label_area = bar.cut_right(std::ceil(ui.measure("Level", tokens::Body).width), tokens::SpaceL);
    tree.add("Restart", restart_area, std::move(restart)).set_tooltip("Starts the run over (R)");
    size_t unlocked = std::min(game.progress.unlocked, game.levels.size());
    std::vector<std::string_view> names;
    names.reserve(unlocked);
    for (size_t index = 0; index < unlocked; index++) {
        names.push_back(game.levels[index].name);
    }
    tree.add("Level", level_area, ComboBox<Message>(game.level, names).on_select(base::make<Message::GoLevel>));
    tree.draw(label_area, Text::label("Level").secondary());
}

// The Run card: where the run stands.
Rect hud_card(const Game& game, Ui& ui, Tree<Message>& tree, Rect rect) {
    std::string level = std::format("{}, {} of {}", game.current().name, game.level + 1, game.levels.size());
    std::optional<double> best_time = game.progress.best[game.level];
    std::string best = best_time ? std::format("{:.2f} s", *best_time) : std::string("None yet");
    std::string_view door = std::ranges::all_of(game.doors_open, [](bool open) { return open; }) ? "Open" : "Closed until every coin";
    std::array<std::pair<std::string_view, std::string>, 6> rows = {{
        {"Level", std::move(level)},
        {"Coins", std::format("{} of {}", game.coins, game.total_coins())},
        {"Lives", std::format("{}", game.lives)},
        {"Time", std::format("{:.2f} s", game.run_seconds())},
        {"Best", std::move(best)},
        {"Door", std::string(door)},
    }};
    return tree.sized_card(ui, "Run", rect, Card("Run").floating(), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form = Form(area.content).gap(tokens::SpaceXs);
        for (auto& [label, value] : rows) {
            card.draw(form.take(tokens::Body.line_height), Field(std::string(label), LabelWidth, std::move(value)));
        }
        return form.bottom();
    });
}

// The Events card: the newest events, with their times on the simulation's
// clock, newest first. The whole cascade of a moment shares one time.
void events_card(const Game& game, Ui& ui, Tree<Message>& tree, Rect rect) {
    Card card = Card("Events").subtitle("Newest first, on the game's clock").floating();
    tree.sized_card(ui, "Events", rect, std::move(card), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        Form form = Form(area.content).gap(tokens::SpaceXs);
        size_t shown = std::min(game.log.size(), EventRows);
        for (size_t index = 0; index < shown; index++) {
            const auto& [seconds, label] = game.log[index];
            Rect line = form.take(tokens::Mono.line_height);
            Text text = Text::label(std::format("{:8.3f}  {}", seconds, label)).style(tokens::Mono);
            if (index == 0) {
                card.draw(line, std::move(text));
            } else {
                card.draw(line, std::move(text).secondary());
            }
        }
        if (game.log.empty()) {
            form.caption(ui, card, "Nothing yet.");
        }
        return form.bottom();
    });
}

// The banner's face: a card-colored plate with the outcome and what R does,
// a target for scripts, which see it the frame the run ends.
class Banner : public Widget<Message> {
public:
    Banner(std::string title, std::string line) : title(std::move(title)), line(std::move(line)) {}
    Banner(const Banner&) = delete;
    Banner& operator=(const Banner&) = delete;
    Banner(Banner&&) = default;
    Banner& operator=(Banner&&) = default;

    std::string_view kind() const override {
        return "Banner";
    }

    std::optional<std::string> label() const override {
        return title;
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        Painter painter = ctx.painter();
        const Theme& theme = painter.theme();
        painter.shape(rect, 8.0f, theme.card_opaque(), Border(theme.card_stroke, 1.0f));
        Rect content = rect.pad(Insets::all(tokens::SpaceM));
        painter.text_in(content.cut_top(tokens::Title.line_height, tokens::SpaceXs), title, tokens::Title, theme.text_primary, Align::Center);
        painter.text_in(content, line, tokens::Subtitle, theme.text_secondary, Align::Center);
    }

private:
    std::string title;
    std::string line;
};

// The banner over the middle of the view when the run has ended, painted
// with the theme's roles.
void banner(const Game& game, Tree<Message>& tree, Rect view) {
    std::string title;
    std::string line;
    if (base::is<RunState::Running>(game.state_)) {
        return;
    } else if (const RunState::Finished* finished = base::as<RunState::Finished>(game.state_)) {
        double seconds = finished->seconds;
        std::optional<double> best = game.progress.best[game.level];
        std::string note;
        if (best && seconds <= *best) {
            note = "A new record, saved.";
        } else if (best) {
            note = std::format("The best is {:.2f} s.", *best);
        }
        title = std::format("Finished in {:.2f} s", seconds);
        line = std::string(base::trim(std::format("{} R starts again.", note)));
    } else {
        title = "Out of lives";
        line = "R starts the run over.";
    }
    Rect rect = Rect::from_center(view.center(), Size(620.0f, 128.0f));
    tree.add("Banner", rect, Banner(std::move(title), std::move(line)));
}

// Where the camera looks, in world units: the player, a little above the
// feet, kept inside the level's bounds, or the bounds' middle where the
// level is smaller than the view.
std::array<double, 2> camera(const Game& game, const Level& level, Rect view) {
    std::array<double, 2> half = {static_cast<double>(view.width) / 2.0 / static_cast<double>(Scale), static_cast<double>(view.height) / 2.0 / static_cast<double>(Scale)};
    auto clamp = [](double center, double low, double high, double half_view) {
        if (high - low <= half_view * 2.0) {
            return (low + high) / 2.0;
        }
        return std::clamp(center, low + half_view, high - half_view);
    };
    Zone bounds = level.bounds;
    return {
        clamp(game.player.position[0], bounds.x, bounds.right(), half[0]),
        clamp(game.player.position[1] + 1.5, bounds.y, bounds.top(), half[1]),
    };
}

Color color(LevelColor bytes) {
    return Color::from_bytes({bytes[0], bytes[1], bytes[2], 255});
}

// What the canvas draws, in screen pixels, built in view so the canvas's
// function owns plain data.
struct Shape {
    struct Fill {
        Rect rect;
        Color color;
        float radius;
    };
    struct Circle {
        Point center;
        float radius;
        Color color;
    };
    struct Ring {
        Point center;
        float radius;
        float width;
        Color color;
    };
    struct Polygon {
        std::vector<Point> points;
        Color color;
    };
    struct Line {
        Point from;
        Point to;
        float width;
        Color color;
    };
    struct Icon {
        Point center;
        user_interface::Icon icon;
        float size;
        Color color;
    };
    std::variant<Fill, Circle, Ring, Polygon, Line, Icon> value;

    template <base::AlternativeOf<Shape> T> Shape(T&& alternative) : value(std::forward<T>(alternative)) {}

    void draw(Painter& painter) const {
        base::match(
            *this, [&](const Fill& fill) { painter.rounded(fill.rect, fill.radius, fill.color); },
            [&](const Circle& circle) { painter.circle(circle.center, circle.radius, circle.color); },
            [&](const Ring& ring) { painter.ring(ring.center, ring.radius, ring.width, ring.color); },
            [&](const Polygon& polygon) { painter.polygon(polygon.points, polygon.color); }, [&](const Line& line) { painter.line(line.from, line.to, line.width, line.color); },
            [&](const Icon& icon) { painter.icon(icon.center, icon.icon, icon.size, icon.color); });
    }
};

// The world's own colors, beyond the level's.
constexpr Color Coin = Color::hex(0xF5C542);
constexpr Color Spring = Color::hex(0xE2574C);
constexpr Color Door = Color::hex(0x7A5CC6);
constexpr Color FlagUp = Color::hex(0x3BAA5C);
constexpr Color FlagDown = Color::hex(0x9AA3AF);
constexpr Color Player = Color::hex(0x2D7FF9);
constexpr Color Spike = Color::hex(0x39404E);
constexpr std::array<Color, 5> Confetti = {
    Color::hex(0xF5C542), Color::hex(0xE2574C), Color::hex(0x3BAA5C), Color::hex(0x2D7FF9), Color::hex(0xB05CC6),
};

class Scene {
public:
    Scene(Rect view, std::array<double, 2> camera, Color sky, Color rock) : view(view), camera(camera), sky(sky), rock(rock) {}
    Scene(const Scene&) = delete;
    Scene& operator=(const Scene&) = delete;
    Scene(Scene&&) = default;
    Scene& operator=(Scene&&) = default;

    Point point(std::array<double, 2> world) const {
        return Point(view.center().x + static_cast<float>((world[0] - camera[0]) * static_cast<double>(Scale)),
                     view.center().y - static_cast<float>((world[1] - camera[1]) * static_cast<double>(Scale)));
    }

    Rect zone(Zone zone) const {
        Point top_left = point({zone.x, zone.top()});
        return Rect(top_left.x, top_left.y, static_cast<float>(zone.width * static_cast<double>(Scale)), static_cast<float>(zone.height * static_cast<double>(Scale)));
    }

    void level(const Game& game, const Level& level) {
        for (const Zone& solid : level.solids) {
            shapes.push_back(Shape::Fill{zone(solid), rock, 4.0f});
        }
        for (const Patrol& patrol : level.patrols) {
            Zone moved = patrol.at(game.simulated);
            shapes.push_back(Shape::Fill{zone(moved), rock, 6.0f});
            shapes.push_back(Shape::Fill{zone(moved).top_strip(4.0f), Coin.fade(0.6f), 2.0f});
        }
        for (const Zone& spikes : level.spikes) {
            // A row of teeth across the zone.
            size_t teeth = base::saturating_cast<size_t>(std::max(std::round(spikes.width / 0.5), 1.0));
            double tooth = spikes.width / static_cast<double>(teeth);
            for (size_t index = 0; index < teeth; index++) {
                double x = spikes.x + tooth * static_cast<double>(index);
                std::vector<Point> points = {
                    point({x, spikes.y}),
                    point({x + tooth / 2.0, spikes.top()}),
                    point({x + tooth, spikes.y}),
                };
                shapes.push_back(Shape::Polygon{std::move(points), Spike});
            }
        }
        for (const Zone& spring : level.springs) {
            Rect rect = zone(spring);
            shapes.push_back(Shape::Fill{rect, Spring, 3.0f});
            shapes.push_back(Shape::Fill{rect.top_strip(4.0f), Color::White.fade(0.7f), 2.0f});
        }
        for (size_t index = 0; index < level.checkpoints.size(); index++) {
            const Zone& checkpoint = level.checkpoints[index];
            bool reached = game.reached[index];
            Point bottom = point({checkpoint.center()[0], checkpoint.y});
            Point top = point({checkpoint.center()[0], checkpoint.top()});
            shapes.push_back(Shape::Line{bottom, top, 3.0f, Spike});
            Color flag = reached ? FlagUp : FlagDown;
            Point tip(top.x + 18.0f, top.y + 7.0f);
            shapes.push_back(Shape::Polygon{{top, tip, Point(top.x, top.y + 14.0f)}, flag});
        }
        for (size_t index = 0; index < level.doors.size(); index++) {
            const Zone& door = level.doors[index];
            Rect rect = zone(door);
            if (game.doors_open[index]) {
                // The frame of the opened door.
                shapes.push_back(Shape::Fill{rect.left_strip(3.0f), Door.fade(0.5f), 0.0f});
                shapes.push_back(Shape::Fill{rect.right_strip(3.0f), Door.fade(0.5f), 0.0f});
            } else {
                shapes.push_back(Shape::Fill{rect, Door, 3.0f});
                Point keyhole = point({door.center()[0], door.center()[1]});
                shapes.push_back(Shape::Circle{keyhole, 4.0f, sky.fade(0.8f)});
            }
        }
        // The goal: a taller pole with the goal flag.
        Zone goal = level.goal;
        Point bottom = point({goal.center()[0], goal.y});
        Point top = point({goal.center()[0], goal.top()});
        shapes.push_back(Shape::Line{bottom, top, 3.0f, Spike});
        Point tip(top.x + 26.0f, top.y + 10.0f);
        shapes.push_back(Shape::Polygon{{top, tip, Point(top.x, top.y + 20.0f)}, FlagUp});
        for (size_t index = 0; index < level.coins.size(); index++) {
            if (!game.collected[index]) {
                Point center = point(level.coins[index]);
                shapes.push_back(Shape::Circle{center, 9.0f, Coin});
                shapes.push_back(Shape::Circle{Point(center.x - 2.0f, center.y - 2.0f), 3.0f, Color::White.fade(0.8f)});
            }
        }
    }

    void player(const Game& game) {
        if (base::is<RunState::Failed>(game.state_)) {
            return;
        }
        Point center = point(game.player.position);
        Rect body = Rect::from_center(center, Size(static_cast<float>(Half[0] * 2.0 * static_cast<double>(Scale)), static_cast<float>(Half[1] * 2.0 * static_cast<double>(Scale))));
        shapes.push_back(Shape::Fill{body, Player, 8.0f});
        // An eye toward where the player runs.
        float facing = game.player.velocity[0] < 0.0 ? -1.0f : 1.0f;
        Point eye(center.x + 7.0f * facing, center.y - 14.0f);
        shapes.push_back(Shape::Circle{eye, 4.0f, Color::White});
        shapes.push_back(Shape::Circle{Point(eye.x + 1.5f * facing, eye.y), 2.0f, Spike});
    }

    // One effect, by how far through its life it is. `canvas` is the view.
    void effect(const Effect& effect, double now, Rect canvas) {
        double age = std::max(now - effect.started, 0.0);
        double life = game::life(effect.kind);
        float along = static_cast<float>(std::clamp(age / life, 0.0, 1.0));
        float fade = 1.0f - along;
        Point at = point(effect.at);
        switch (effect.kind) {
        case EffectKind::Sparkle:
            for (int index = 0; index < 4; index++) {
                float angle = std::numbers::pi_v<float> / 4.0f + std::numbers::pi_v<float> / 2.0f * static_cast<float>(index);
                float reach = 10.0f + 36.0f * along;
                Point center(at.x + std::cos(angle) * reach, at.y - std::sin(angle) * reach);
                shapes.push_back(Shape::Circle{center, 4.0f * fade, Coin.fade(fade)});
            }
            break;
        case EffectKind::Burst:
        case EffectKind::Flag:
        case EffectKind::Shimmer:
            shapes.push_back(Shape::Ring{at, 12.0f + 60.0f * along, 3.0f, Color::White.fade(fade)});
            break;
        case EffectKind::Bounce:
            shapes.push_back(Shape::Ring{at, 8.0f + 40.0f * along, 3.0f, Spring.fade(fade)});
            break;
        case EffectKind::Poof:
            for (int index = 0; index < 3; index++) {
                float x = at.x + (static_cast<float>(index) - 1.0f) * (10.0f + 14.0f * along);
                float y = at.y - 30.0f * along;
                shapes.push_back(Shape::Circle{Point(x, y), 8.0f * fade + 2.0f, FlagDown.fade(fade * 0.8f)});
            }
            break;
        case EffectKind::Confetti:
            for (size_t index = 0; index < 36; index++) {
                // Deterministic spread from the index alone.
                float angle = static_cast<float>(index) * 2.399963f;
                float speed = 120.0f + 60.0f * static_cast<float>((index * 7) % 5);
                float x = at.x + std::cos(angle) * speed * along;
                float y = at.y - std::abs(std::sin(angle)) * speed * along + 160.0f * along * along;
                Rect piece(x, y, 6.0f, 6.0f);
                shapes.push_back(Shape::Fill{piece, Confetti[index % Confetti.size()].fade(fade), 1.0f});
            }
            break;
        case EffectKind::SaveFlash: {
            // A save light at the view's top right, not in the world.
            float pulse = std::min(fade * 2.0f, 1.0f);
            Point center(canvas.right() - 48.0f, canvas.y + 48.0f);
            shapes.push_back(Shape::Icon{center, Icon::Save, 24.0f, Color::White.fade(pulse)});
            shapes.push_back(Shape::Ring{center, 20.0f, 2.0f, Color::White.fade(pulse * 0.7f)});
            break;
        }
        }
    }

    Rect view;
    std::array<double, 2> camera;
    Color sky;
    Color rock;
    std::vector<Shape> shapes;
};

// The world, drawn from the level and the run: the camera follows the
// player inside the level's bounds.
void world(const Game& game, Tree<Message>& tree, Rect view) {
    const Level& level = game.current();
    Scene scene(view, camera(game, level, view), color(level.sky), color(level.rock));
    scene.level(game, level);
    scene.player(game);
    for (const Effect& effect : game.effects) {
        scene.effect(effect, game.simulated, view);
    }
    tree.draw(view, Canvas([sky = scene.sky, shapes = std::move(scene.shapes)](Painter& painter, Rect rect) {
                  painter.push_clip(rect);
                  painter.fill(rect, sky);
                  for (const Shape& shape : shapes) {
                      shape.draw(painter);
                  }
                  painter.pop_clip();
              }));
}

} // namespace

Rect view_area(const Ui& ui) {
    Size size = ui.size();
    float top = ui.content_top() + tokens::AppBarHeight;
    return Rect(0.0f, top, size.width, size.height - top - tokens::StatusBarHeight);
}

Tree<Message> view(const Game& game, Ui& ui) {
    Tree<Message> tree;
    // Nothing takes keyboard focus, so the keys stay the game's.
    tree.set_focusable(false);
    Rect view = view_area(ui);
    world(game, tree, view);

    Shell shell = tree.shell(ui, AppBar(Icon::Flag, std::string(Name)), false, status_items(game));
    commands(game, ui, tree, shell.commands);
    if (game.show_cards) {
        Rect column = shell.page;
        Rect hud = hud_card(game, ui, tree, column.left_strip(CardWidth));
        column.cut_top(hud.height + tokens::CardGap, 0.0f);
        events_card(game, ui, tree, column.left_strip(CardWidth));
    }
    banner(game, tree, view);
    return tree;
}

} // namespace platformer::cards
