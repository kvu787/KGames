// The programs' frames, as Library/UserInterface runs them: what a frame's
// update costs the CPU, without a window, through Headless, steady and with
// the pointer moving; what recording the frame's drawing costs the CPU; and
// what drawing it costs the GPU, timed with timestamp queries, into a target
// the size of the maximized window's client area. And the parts of a frame
// that every program pays for: text layout and the glyph atlas, the input
// reader, and the PerfBar's bookkeeping.
//
// The namespace is interface_, with an underscore: interface is one of the
// Windows SDK's macros.

#pragma once

#include "Benchmarks/Harness.h"
#include "GameInputDemo/Demo.h"
#include "MeshModeler/Modeler.h"
#include "MinimalSceneTest/MovingSphere.h"
#include "SimplePaintDemo/Demo.h"
#include "SpinningCarTest/SpinningCar.h"
#include "UiShowcase/Showcase.h"
#include "UserInterface/Testing/Headless.h"
#include "Veehiicuul/Program.h"

#include <concepts>
#include <cstdint>
#include <filesystem>
#include <string_view>
#include <vector>

namespace benchmarks::interface_ {

// A program with its frames ready to time, made fresh for each benchmark:
// its name, a string literal's text, and `make`, which gives the program in
// a Headless.
template <class S>
concept Subject = requires(const S& subject) {
    { subject.name() } -> std::convertible_to<std::string_view>;
    { subject.make() } -> std::same_as<user_interface::Headless<typename S::P>>;
};

struct Showcase {
    using P = ui_showcase::showcase::Showcase;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct SimplePaint {
    using P = simple_paint_demo::demo::Demo;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct Modeler {
    using P = mesh_modeler::modeler::Modeler;
    std::uint32_t levels;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct GameInput {
    using P = game_input_demo::Demo;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct Racing {
    using P = veehiicuul::program::Veehiicuul;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct Sphere {
    using P = minimal_scene_test::moving_sphere::MovingSphere;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

struct Car {
    using P = spinning_car_test::spinning_car::SpinningCar;
    std::string_view name() const;
    user_interface::Headless<P> make() const;
};

// Where programs that would save files would save them, which none of these
// benchmarks make them do.
std::filesystem::path scratch(std::string_view name);

// Every program's CPU frame benchmarks.
std::vector<Measurement> frames(Budget budget);

// Every program's GPU draw benchmarks, on the fastest GPU.
std::vector<Measurement> draws(Budget budget);

// Text through DirectWrite: a layout that the cache has, one that it
// doesn't, and glyphs that the atlas doesn't have yet.
std::vector<Measurement> text(Budget budget);

// The input reader with a frame's readings from a 1000 Hz mouse at 60
// frames a second, and a keyboard that holds a key.
std::vector<Measurement> input(Budget budget);

// The PerfBar's bookkeeping: a frame presented, its present known, and the
// stats refreshed, at 240 frames a second.
std::vector<Measurement> perf_bar(Budget budget);

} // namespace benchmarks::interface_
