#include "Benchmarks/Footprints.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Benchmarks/Interface.h"
#include "Gltf/Gltf.h"
#include "Graphics/Graphics.h"
#include "PolygonMesh/PolygonMesh.h"
#include "UserInterface/UserInterface.h"
#include "Json/Json.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <iterator>
#include <numbers>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace benchmarks::footprints {

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::Offscreen;
using interface_::Car;
using interface_::GameInput;
using interface_::Modeler;
using interface_::Racing;
using interface_::Showcase;
using interface_::SimplePaint;
using interface_::Sphere;
using interface_::Subject;
using memory::Counts;
using memory::Report;
using memory::Usage;
using memory::Video;
using polygon_mesh::Mesh;
using polygon_mesh::Shading;
using user_interface::Event;
using user_interface::Font;
using user_interface::Fonts;
using user_interface::GlyphQuad;
using user_interface::Headless;
using user_interface::Point;
using user_interface::Rect;
using user_interface::TextStyle;
using user_interface::Tree;
using user_interface::Weight;
namespace primitives = polygon_mesh::primitives;
namespace tokens = user_interface::tokens;

namespace {

// How many frames or runs a benchmark counts at most. A memory benchmark
// needs only enough runs for the per-run numbers to be whole; it isn't
// looking for a minimum the way a timing benchmark is.
constexpr size_t Frames = 200;
constexpr size_t Runs = 3;

// Veehiicuul's folder, whose models and scenes are the inputs, as in
// Libraries.cpp.
std::filesystem::path veehiicuul() {
    return base::path(std::string(folders::Project) + "/../Veehiicuul");
}

// A torus of 1,152 quads, like a model a modeler would subdivide.
Mesh torus() {
    return primitives::torus(1.0f, 0.3f, 48, 24);
}

// What a program holds once it has started and drawn its first frame, and
// what each steady frame allocates and frees.
template <Subject S> std::vector<Usage> program(const S& subject, Budget budget) {
    auto [start, app] = memory::held(std::format("memory/{}/start", subject.name()), "program", [&] { return subject.make(); });
    std::string note = std::format("{} targets, {} instances", app.ui.targets().size(), app.ui.instance_count());
    Usage steady = memory::measure(std::format("memory/{}/frame/steady", subject.name()), "frame", budget, Frames, [&] { app.frame(); });
    std::vector<Usage> both;
    both.reserve(2);
    both.push_back(std::move(start).with_note(note));
    both.push_back(std::move(steady).with_note(std::move(note)));
    return both;
}

// Frames with the pointer crossing the page, which changes what's hovered,
// lays out new text, and shows tooltips.
template <Subject S> Usage moving(const S& subject, Budget budget) {
    Headless<typename S::P> app = subject.make();
    size_t step = 0;
    return memory::measure(std::format("memory/{}/frame/moving", subject.name()), "frame", budget, Frames, [&] {
        step += 1;
        float t = static_cast<float>(step) * 0.017f;
        float x = (std::sin(t) * 0.5f + 0.5f) * tokens::WindowWidth;
        float y = (std::cos(t * 0.7f) * 0.5f + 0.5f) * tokens::MaximizedHeight;
        app.send({{Event::PointerMoved{Point(x, y)}}});
    });
}

// Frames in a row, counted only after the program has run for long enough
// to fill its caches and cross the text cache's purge, every 600 frames.
// What's kept here is what a program that has been up for a while still
// grows by a frame: a leak, if it isn't nothing.
Usage long_(Budget budget) {
    auto [warmup, frames] = budget.iterations <= 1 ? std::pair<size_t, size_t>(2, 2) : std::pair<size_t, size_t>(800, 1500);
    Headless app(ui_showcase::showcase::Showcase{});
    app.frames(warmup);
    memory::start();
    app.frames(frames);
    Counts counted = memory::stop();
    return Usage::from_counts("memory/UiShowcase/frames/long", "frame", counted, static_cast<double>(frames))
        .with_note(std::format("{} frames after the first {}", frames, warmup));
}

// A widget that keeps a number between frames, like a scroll offset.
class Remember : public user_interface::Widget<> {
public:
    explicit Remember(float value) : value(value) {}

    std::string_view kind() const override {
        return "Remember";
    }

    void draw(user_interface::Draw& ctx) const override {
        ctx.memory<float>() = value;
    }

private:
    float value;
};

// A program whose widgets have a new name every frame, as a program's would
// if it named them after what they show, like a row's text. The interface
// keeps widgets' state by name, so this is what it never lets go of.
class Churner : public user_interface::Program<std::monostate> {
public:
    std::uint32_t frame;
    // Widgets a frame.
    std::uint32_t widgets;

    Churner(std::uint32_t frame, std::uint32_t widgets) : frame(frame), widgets(widgets) {}

    std::string title() const {
        return "Churner";
    }

    std::vector<std::monostate> input(const user_interface::Ui&) const {
        // One message a frame, to count the frame.
        return {std::monostate{}};
    }

    void update(std::monostate, user_interface::Context<std::monostate>&) {
        frame += 1;
    }

    Tree<std::monostate> view(user_interface::Ui&) const {
        Tree<std::monostate> tree;
        for (std::uint32_t index = 0; index < widgets; index++) {
            std::string name = std::format("Row {} {}", frame, index);
            tree.add(name, Rect(static_cast<float>(index), 0.0f, 1.0f, 1.0f), Remember(static_cast<float>(index)));
        }
        return tree;
    }
};

Usage churn(std::string_view name, Budget budget) {
    size_t frames = budget.iterations <= 1 ? 2 : 1500;
    std::uint32_t widgets = 20;
    Headless app{Churner(0, widgets)};
    app.frames(10);
    memory::start();
    app.frames(frames);
    Counts counted = memory::stop();
    return Usage::from_counts(name, "frame", counted, static_cast<double>(frames)).with_note(std::format("{} new names a frame, {} frames", widgets, frames));
}

void extend(std::vector<Usage>& all, std::vector<Usage> more) {
    all.insert(all.end(), std::make_move_iterator(more.begin()), std::make_move_iterator(more.end()));
}

// One program's GPU memory: what a frame of it, drawn into the target, adds
// to the device's usage, and what it leaves behind once it's gone.
template <Subject S> Video drawn(const S& subject, Gpu& gpu, Offscreen& target) {
    gpu.flush();
    std::array<graphics::VideoMemory, 2> before = gpu.video_memory();
    std::array<graphics::VideoMemory, 2> after;
    {
        Headless<typename S::P> app = subject.make();
        graphics::Frame frame = target.begin();
        app.record(gpu, frame);
        std::uint64_t fence = target.submit(gpu, std::move(frame));
        gpu.wait_for(fence);
        after = gpu.video_memory();
    }
    gpu.flush();
    std::uint64_t freed = base::saturating_sub(after[0].used, gpu.video_memory()[0].used);
    return Video{
        .name = std::format("memory/gpu/{}", subject.name()),
        .local = base::saturating_sub(after[0].used, before[0].used),
        .non_local = base::saturating_sub(after[1].used, before[1].used),
        .note = std::format("{:.1f} MiB given back when it went", static_cast<double>(freed) / (1024.0 * 1024.0)),
    };
}

} // namespace

Report frames(Budget budget) {
    std::vector<Usage> all;
    extend(all, program(Showcase{}, budget));
    all.push_back(moving(Showcase{}, budget));
    extend(all, program(SimplePaint{}, budget));
    extend(all, program(Modeler{.levels = 0}, budget));
    extend(all, program(Modeler{.levels = 4}, budget));
    all.push_back(moving(Modeler{.levels = 4}, budget));
    extend(all, program(GameInput{}, budget));
    extend(all, program(Racing{}, budget));
    extend(all, program(Sphere{}, budget));
    extend(all, program(Car{}, budget));
    all.push_back(long_(budget));
    return Report::of(std::move(all));
}

Report caches(Budget budget) {
    std::vector<Usage> all;
    {
        auto [fonts, fonts_cache] = memory::held("memory/text/fonts", "cache", [] { return Fonts(); });
        all.push_back(std::move(fonts).with_note("DirectWrite's factory and the system font collection"));

        // Each text in each style is a layout of its own in the cache.
        size_t count = budget.iterations <= 1 ? 4 : 2000;
        std::vector<std::string> labels;
        labels.reserve(count);
        for (size_t index = 0; index < count; index++) {
            labels.push_back(std::format("Label number {}", index));
        }
        // DirectWrite's layouts are in its own heap, which the allocator here
        // can't see, so what a layout costs there is the slope of the process's
        // private commit: the commit over the last three quarters of the
        // layouts, which leaves out the font files that the first ones load.
        size_t quarter = std::max<size_t>(count / 4, 1);
        memory::start();
        for (const std::string& label : std::span<const std::string>(labels).first(quarter)) {
            black_box(fonts_cache.measure(label, tokens::Body));
        }
        memory::ProcessMemory first = memory::process_memory();
        for (const std::string& label : std::span<const std::string>(labels).subspan(quarter)) {
            black_box(fonts_cache.measure(label, tokens::Body));
        }
        Counts layouts = memory::stop();
        std::uint64_t committed = base::saturating_sub(memory::process_memory().private_commit, first.private_commit);
        all.push_back(
            Usage::from_counts("memory/text/layouts", "layout", layouts, static_cast<double>(count))
                .with_note(std::format("{} B a layout of private commit, mostly DirectWrite's own", committed / static_cast<std::uint64_t>(std::max<size_t>(count - quarter, 1)))));

        // Glyphs that the atlas doesn't have are rasterized into it, and their
        // pixels wait on the heap until the renderer copies them in. A frame
        // that's drawn takes them every frame; these are never taken.
        size_t sizes = budget.iterations <= 1 ? 2 : 40;
        size_t glyphs = 0;
        memory::start();
        for (size_t size = 0; size < sizes; size++) {
            TextStyle style(Font::Text, Weight::Regular, 8.0f + static_cast<float>(size) * 0.125f, 20.0f);
            fonts_cache.quads("0123456789", style, std::nullopt, Point::Zero, [&](const GlyphQuad&) { glyphs += 1; });
        }
        Counts rasterized = memory::stop();
        all.push_back(Usage::from_counts("memory/text/glyphs", "glyph", rasterized, static_cast<double>(glyphs))
                          .with_note(std::format("{} glyphs at {} sizes, with each size's layout, their pixels waiting for the renderer", glyphs, sizes)));

        // Layouts unused for 600 frames go, which `begin_frame` does on the
        // 600th, so what's kept here is what the purge gave back.
        size_t purges = budget.iterations <= 1 ? 1 : 601;
        memory::start();
        for (size_t purge = 0; purge < purges; purge++) {
            fonts_cache.begin_frame();
        }
        Counts purged = memory::stop();
        all.push_back(Usage::from_counts("memory/text/purge", "purge", purged, 1.0).with_note(std::format("{} layouts let go of after 600 frames", count)));
    }

    all.push_back(churn("memory/ui/memory/new-names", budget));
    return Report::of(std::move(all));
}

Report libraries(Budget budget) {
    std::vector<Usage> all;
    Mesh torus_mesh = torus();
    auto [mesh, subdivided] = memory::held("memory/mesh/torus/2", "mesh", [&] { return torus_mesh.catmull_clark(2).mesh; });
    size_t faces = subdivided.face_count();
    size_t vertices = subdivided.positions.size();
    auto a_quad = [&](Usage usage, std::string_view what) {
        double bytes = static_cast<double>(usage.kept) / static_cast<double>(faces);
        return std::move(usage).with_note(std::format("{} quads, {}, {:.0f} B a quad", faces, what, bytes));
    };
    all.push_back(a_quad(std::move(mesh), std::format("{} vertices", vertices)));
    all.push_back(memory::measure("memory/mesh/catmull-clark/torus/2", "subdivide", budget, Runs, [&] {
                      black_box(torus_mesh.catmull_clark(2));
                  }).with_note(std::format("{} quads in, {} out", torus_mesh.face_count(), faces)));
    {
        auto [topology, built] = memory::held("memory/mesh/topology", "topology", [&] { return subdivided.topology(); });
        size_t edges = built.edges().size();
        all.push_back(a_quad(std::move(topology), std::format("{} edges", edges)));
    }
    all.push_back(memory::measure("memory/mesh/triangulate/smooth", "triangulate", budget, Runs, [&] {
                      black_box(subdivided.triangulate(Shading::Smooth{std::numbers::pi_v<float>}));
                  }).with_note(std::format("{} quads, with the topology built again inside", faces)));
    {
        auto [triangles, drawn_triangles] = memory::held("memory/mesh/triangles", "triangles", [&] { return subdivided.triangulate(Shading::Flat{}); });
        all.push_back(std::move(triangles).with_note(std::format("{} triangles, {} vertices", drawn_triangles.indices.size() / 3, drawn_triangles.vertices.size())));
    }

    for (std::string_view name : {"SlopeCar", "BigCone"}) {
        std::filesystem::path path = veehiicuul() / "Models" / std::format("{}.glb", name);
        std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
        if (!bytes) {
            base::panic(std::format("Couldn't read {}: {}", base::display(path), bytes.error()));
        }
        auto [usage, model] = memory::held(std::format("memory/gltf/{}", name), "model", [&] { return gltf::parse(*bytes).value(); });
        all.push_back(std::move(usage).with_note(std::format("{} KB on disk, {} triangles", bytes->size() / 1024, model.indices.size() / 3)));
    }

    std::filesystem::path path = veehiicuul() / "Scenes" / "Ribeye.json";
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        base::panic(std::format("Couldn't read {}: {}", base::display(path), text.error()));
    }
    {
        auto [usage, value] = memory::held("memory/json/scene", "parse", [&] { return json::parse(*text).value(); });
        all.push_back(std::move(usage).with_note(std::format("{} KB of text", text->size() / 1024)));
    }
    all.push_back(memory::measure("memory/json/parse/scene", "parse", budget, Runs, [&] {
                      black_box(json::parse(*text).value());
                  }).with_note(std::format("{} KB of text", text->size() / 1024)));
    return Report::of(std::move(all));
}

Report video(Budget) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    std::vector<Video> video;
    std::array<graphics::VideoMemory, 2> empty = gpu.video_memory();
    video.push_back(Video{
        .name = "memory/gpu/device",
        .local = empty[0].used,
        .non_local = empty[1].used,
        .note = std::format("{} MiB of budget", empty[0].budget / (1024 * 1024)),
    });
    std::uint32_t width = base::saturating_cast<std::uint32_t>(tokens::WindowWidth);
    std::uint32_t height = base::saturating_cast<std::uint32_t>(tokens::MaximizedHeight);
    Offscreen target(gpu, width, height, {0.0f, 0.0f, 0.0f, 0.0f});
    std::array<graphics::VideoMemory, 2> idle = gpu.video_memory();
    video.push_back(Video{
        .name = "memory/gpu/target",
        .local = idle[0].used - empty[0].used,
        .non_local = base::saturating_sub(idle[1].used, empty[1].used),
        .note = std::format("{} x {} target, over an idle device", width, height),
    });
    video.push_back(drawn(Showcase{}, gpu, target));
    video.push_back(drawn(SimplePaint{}, gpu, target));
    video.push_back(drawn(Modeler{.levels = 4}, gpu, target));
    video.push_back(drawn(GameInput{}, gpu, target));
    video.push_back(drawn(Racing{}, gpu, target));
    video.push_back(drawn(Sphere{}, gpu, target));
    video.push_back(drawn(Car{}, gpu, target));
    return Report{.heap = {}, .video = std::move(video)};
}

} // namespace benchmarks::footprints
