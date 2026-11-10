#include "Benchmarks/Interface.h"

#include "Base/Numbers.h"
#include "GameInput/GameInput.h"
#include "Graphics/Graphics.h"
#include "MinimalSceneTest/Settings.h"
#include "PresentTiming/PresentTiming.h"
#include "SimplePaint/Material.h"
#include "SpinningCarTest/Settings.h"
#include "UserInterface/UserInterface.h"
#include "Json/Json.h"

#include <array>
#include <chrono>
#include <cmath>
#include <cstddef>
#include <format>
#include <span>
#include <string>
#include <tuple>
#include <utility>

namespace benchmarks::interface_ {

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::GpuTimer;
using graphics::Offscreen;
using user_interface::Event;
using user_interface::FocusChange;
using user_interface::Font;
using user_interface::Fonts;
using user_interface::GlyphQuad;
using user_interface::Headless;
using user_interface::InputReader;
using user_interface::Point;
using user_interface::Reading;
using user_interface::Startup;
using user_interface::TextStyle;
using user_interface::Weight;
using user_interface::WindowState;
namespace clock = user_interface::clock;
namespace tokens = user_interface::tokens;

std::filesystem::path scratch(std::string_view name) {
    return std::filesystem::temp_directory_path() / "KGamesBenchmarks" / name;
}

std::string_view Showcase::name() const {
    return "UiShowcase";
}

Headless<Showcase::P> Showcase::make() const {
    return Headless(ui_showcase::showcase::Showcase{});
}

std::string_view SimplePaint::name() const {
    return "SimplePaintDemo";
}

Headless<SimplePaint::P> SimplePaint::make() const {
    return Headless(simple_paint_demo::demo::Demo(simple_paint::Parameters{}, scratch("Paint.json")));
}

std::string_view Modeler::name() const {
    return levels == 0 ? "MeshModeler" : "MeshModeler/subdivided";
}

Headless<Modeler::P> Modeler::make() const {
    Headless app{mesh_modeler::modeler::Modeler(scratch("Models"))};
    app.program.levels = levels;
    app.frames(2);
    return app;
}

std::string_view GameInput::name() const {
    return "GameInputDemo";
}

Headless<GameInput::P> GameInput::make() const {
    game_input_demo::Demo demo = game_input_demo::Demo::scripted(0.1f);
    size_t index = 0;
    for (std::string_view name : {"Xbox Controller", "DualSense"}) {
        std::string step = std::format(R"({{ "Connect": {{ "Id": {}, "Name": "{}", "Input": ["Gamepad"] }} }})", index + 1, name);
        demo.script(json::parse(step).value()).value();
        index += 1;
    }
    Headless app(std::move(demo));
    app.frames(2);
    return app;
}

std::string_view Racing::name() const {
    return "Veehiicuul";
}

Headless<Racing::P> Racing::make() const {
    Headless<P> app = Headless<P>::start(Startup::for_tests("{}", scratch("Veehiicuul")), veehiicuul::program::Veehiicuul::start);
    app.frames(2);
    return app;
}

std::string_view Sphere::name() const {
    return "MinimalSceneTest";
}

Headless<Sphere::P> Sphere::make() const {
    return Headless(minimal_scene_test::moving_sphere::MovingSphere(minimal_scene_test::settings::SceneSettings{}));
}

std::string_view Car::name() const {
    return "SpinningCarTest";
}

Headless<Car::P> Car::make() const {
    return Headless(spinning_car_test::spinning_car::SpinningCar(spinning_car_test::settings::SceneSettings{}));
}

namespace {

// A frame's update with nothing changing, which is what most frames are:
// the layouts are cached, and the widgets draw what they drew.
template <Subject S> Measurement steady(const S& subject, Budget budget) {
    Headless<typename S::P> app = subject.make();
    Measurement measurement = measure(std::format("ui/{}/frame/steady", subject.name()), "frame", budget, 1.0, [&] { app.frame(); });
    return std::move(measurement).with_note(std::format("{} targets, {} instances", app.ui.targets().size(), app.ui.instance_count()));
}

// A frame's update with the pointer moving across the page, which changes
// what's hovered, and so what's drawn, and can show a tooltip.
template <Subject S> Measurement moving(const S& subject, Budget budget) {
    Headless<typename S::P> app = subject.make();
    size_t step = 0;
    return measure(std::format("ui/{}/frame/moving", subject.name()), "frame", budget, 1.0, [&] {
        step += 1;
        float t = static_cast<float>(step) * 0.017f;
        float x = (std::sin(t) * 0.5f + 0.5f) * tokens::WindowWidth;
        float y = (std::cos(t * 0.7f) * 0.5f + 0.5f) * tokens::MaximizedHeight;
        app.send({{Event::PointerMoved{Point(x, y)}}});
    });
}

// The first frame: a program made from nothing, with every layout and
// glyph still to make.
template <Subject S> Measurement first(const S& subject, Budget budget) {
    return measure(std::format("ui/{}/start", subject.name()), "start", budget, 1.0, [&] {
        Headless<typename S::P> app = subject.make();
        black_box(app.frame_number());
    });
}

// What the GPU takes to draw the last frame into a target the size of the
// maximized window's client area, and what the CPU takes to record it.
template <Subject S> std::vector<Measurement> draw(const S& subject, Gpu& gpu, Budget budget) {
    Headless<typename S::P> app = subject.make();
    Offscreen target(gpu, base::saturating_cast<std::uint32_t>(tokens::WindowWidth), base::saturating_cast<std::uint32_t>(tokens::MaximizedHeight), {0.0f, 0.0f, 0.0f, 0.0f});
    GpuTimer timer(gpu, 1);
    user_interface::Color background = app.ui.theme().background;
    std::array<float, 4> clear{background.r, background.g, background.b, 1.0f};
    std::vector<double> record_times;
    Measurement gpu_time = measure_timed(std::format("gpu/{}/draw", subject.name()), "frame", budget, 1.0, [&] {
        graphics::Frame frame = target.begin();
        frame.clear(clear);
        graphics::Span span = timer.begin(frame);
        std::chrono::steady_clock::time_point before = std::chrono::steady_clock::now();
        app.record(gpu, frame);
        record_times.push_back(seconds_since(before) * 1000.0);
        timer.end(frame, span);
        timer.resolve(frame);
        std::uint64_t fence = target.submit(gpu, std::move(frame));
        return timer.read(gpu, fence)[0];
    });
    std::string note = std::format("{} instances", app.ui.instance_count());
    size_t count = gpu_time.iterations;
    Measurement record = Measurement::from_times(std::format("cpu/{}/record", subject.name()), "frame", std::span<const double>(record_times).last(count), 1.0);
    std::vector<Measurement> both;
    both.reserve(2);
    both.push_back(std::move(gpu_time).with_note(note));
    both.push_back(std::move(record).with_note(std::move(note)));
    return both;
}

void extend(std::vector<Measurement>& all, std::vector<Measurement> more) {
    all.insert(all.end(), std::make_move_iterator(more.begin()), std::make_move_iterator(more.end()));
}

} // namespace

std::vector<Measurement> frames(Budget budget) {
    std::vector<Measurement> all;
    all.reserve(12);
    all.push_back(steady(Showcase{}, budget));
    all.push_back(moving(Showcase{}, budget));
    all.push_back(first(Showcase{}, budget));
    all.push_back(steady(SimplePaint{}, budget));
    all.push_back(steady(Modeler{.levels = 0}, budget));
    all.push_back(steady(Modeler{.levels = 4}, budget));
    all.push_back(moving(Modeler{.levels = 4}, budget));
    all.push_back(steady(GameInput{}, budget));
    all.push_back(steady(Racing{}, budget));
    all.push_back(first(Racing{}, budget));
    all.push_back(steady(Sphere{}, budget));
    all.push_back(steady(Car{}, budget));
    return all;
}

std::vector<Measurement> draws(Budget budget) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    std::vector<Measurement> all;
    extend(all, draw(Showcase{}, gpu, budget));
    extend(all, draw(SimplePaint{}, gpu, budget));
    extend(all, draw(Modeler{.levels = 4}, gpu, budget));
    extend(all, draw(GameInput{}, gpu, budget));
    extend(all, draw(Racing{}, gpu, budget));
    extend(all, draw(Sphere{}, gpu, budget));
    extend(all, draw(Car{}, gpu, budget));
    return all;
}

std::vector<Measurement> text(Budget budget) {
    std::vector<Measurement> all;
    Fonts fonts;
    std::vector<std::string> labels;
    labels.reserve(64);
    for (int label = 0; label < 64; label++) {
        labels.push_back(std::format("Label {}", label));
    }
    size_t index = 0;
    all.push_back(measure("text/layout/cached", "measure", budget, 64.0, [&] {
        fonts.begin_frame();
        for (const std::string& label : labels) {
            black_box(fonts.measure(label, tokens::Body));
        }
    }));
    all.push_back(measure("text/layout/new", "layout", budget, 1.0, [&] {
        index += 1;
        fonts.begin_frame();
        black_box(fonts.measure(std::format("Never seen before {}", index), tokens::Body));
    }));
    std::string_view paragraph = "The quick brown fox jumps over the lazy dog, and the five boxing wizards jump quickly, while pack my box with five dozen liquor jugs.";
    all.push_back(measure("text/layout/wrapped-new", "layout", budget, 1.0, [&] {
        index += 1;
        fonts.begin_frame();
        black_box(fonts.measure_wrapped(std::format("{} {}", paragraph, index), tokens::Body, 400.0f));
    }));
    size_t quads = 0;
    all.push_back(measure("text/quads/cached", "quad", budget, 64.0 * 8.0, [&] {
        fonts.begin_frame();
        quads = 0;
        for (const std::string& label : labels) {
            fonts.quads(label, tokens::Body, std::nullopt, Point(10.5f, 20.0f), [&](const GlyphQuad&) { quads += 1; });
        }
    }));
    std::uint32_t size = 0;
    all.push_back(measure("text/rasterize/new", "glyph", budget, 10.0, [&] {
        size += 1;
        fonts.begin_frame();
        // Ten digits at a size no glyph has been rasterized at yet.
        TextStyle style(Font::Text, Weight::Regular, 8.0f + static_cast<float>(size) * 0.125f, 20.0f);
        fonts.quads("0123456789", style, std::nullopt, Point::Zero, [](const GlyphQuad&) {});
    }));
    all.push_back(measure("text/layout/caption-cached", "measure", budget, 64.0, [&] {
        fonts.begin_frame();
        for (const std::string& label : labels) {
            black_box(fonts.measure(label, tokens::Caption));
        }
    }));
    return all;
}

std::vector<Measurement> input(Budget budget) {
    InputReader reader = InputReader::with_repeat(true, 0.5, 30.0);
    std::int64_t now = clock::now();
    WindowState window{
        .cursor = Point(100.0f, 50.0f),
        .inside = true,
        .focus_changes = {FocusChange{.time = now, .gained = true}},
    };
    reader.read({}, {}, window, now);
    window.focus_changes = {};
    std::int64_t tick = clock::frequency() / 1000;
    std::int64_t time = now;
    std::vector<std::tuple<std::int64_t, game_input::GameInputKind, Reading>> readings;
    for (std::int64_t position = 0; position < 17; position++) {
        time += tick;
        Reading reading;
        reading.device = 2;
        reading.kinds = game_input::GameInputKindMouse;
        reading.mouse.positionX = position;
        reading.mouse.positionY = position * 2;
        readings.emplace_back(time, game_input::GameInputKindMouse, std::move(reading));
    }
    Reading keyboard;
    keyboard.device = 1;
    keyboard.kinds = game_input::GameInputKindKeyboard;
    keyboard.keys = {game_input::GameInputKeyState{
        .scanCode = 0x1E,
        .codePoint = 0,
        .virtualKey = 0x41,
        .isDeadKey = false,
    }};
    readings.emplace_back(time, game_input::GameInputKindKeyboard, std::move(keyboard));
    std::vector<Measurement> all;
    all.push_back(measure("input/reader/frame", "frame", budget, 1.0, [&] {
                      time += tick;
                      black_box(reader.read(readings, {}, window, time));
                  }).with_note("17 mouse readings and 1 keyboard reading"));
    return all;
}

std::vector<Measurement> perf_bar(Budget budget) {
    namespace performance = user_interface::performance;
    performance::Performance bar = performance::Performance::given();
    double frequency = static_cast<double>(clock::frequency());
    std::uint64_t frame = 0;
    std::int64_t start = clock::now();
    std::vector<Measurement> all;
    all.push_back(measure("perfbar/frame", "frame", budget, 1.0, [&] {
        frame += 1;
        double seconds = static_cast<double>(frame) / 240.0;
        std::int64_t present = start + base::saturating_cast<std::int64_t>(seconds * frequency);
        bar.frame_started(seconds);
        bar.presented(performance::Frame{
            .input = present - base::saturating_cast<std::int64_t>(0.002 * frequency),
            .read = present - base::saturating_cast<std::int64_t>(0.001 * frequency),
            .present = present,
        });
        bar.known({present_timing::Present{
            .start = present + 5,
            .thread = 1,
            .mode = present_timing::PresentMode::IndependentFlip,
            .tearing = true,
            .screen = present + base::saturating_cast<std::int64_t>(0.004 * frequency),
            .flip_delay = 0,
        }});
        bar.update(seconds);
    }));
    return all;
}

} // namespace benchmarks::interface_
