// MeshModeler's frames, drawn offscreen with the GPU, as the window would
// draw them: the mesh in the view's card, painted with SimplePaint or lit
// flat, subdivided, and with what's selected tinted in the accent.

#include "Base/Numbers.h"
#include "Graphics/Graphics.h"
#include "MeshModeler/MeshModeler.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

#include <array>
#include <cstdint>
#include <filesystem>

using mesh_modeler::modeler::Modeler;
using mesh_modeler::viewport::Shader;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Point;
using user_interface::Rect;

namespace {

std::array<std::uint8_t, 3> pixel(const graphics::Image& image, Point point) {
    std::array<std::uint8_t, 4> color = image.pixel(base::saturating_cast<std::uint32_t>(point.x), base::saturating_cast<std::uint32_t>(point.y));
    return {color[0], color[1], color[2]};
}

bool differs(std::array<std::uint8_t, 3> a, std::array<std::uint8_t, 3> b) {
    for (size_t channel = 0; channel < 3; channel++) {
        if (base::abs_diff(a[channel], b[channel]) > 8) {
            return true;
        }
    }
    return false;
}

} // namespace

TEST(the_view_draws_the_mesh_with_either_shader_subdivided_and_selected) {
    graphics::Gpu gpu(graphics::AdapterChoice::HighPerformance{});
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "MeshModelerRenderingTests";
    Headless<Modeler> app{Modeler(folder)};
    app.frames(2);
    std::array<std::uint8_t, 4> bytes = app.ui.theme().card_opaque().to_bytes();
    std::array<std::uint8_t, 3> card{bytes[0], bytes[1], bytes[2]};

    graphics::Image image = app.render(gpu);
    Rect view = app.program.view_;
    Point corner = view.origin() + Point(4.0f, 4.0f);
    CHECK(!differs(pixel(image, corner), card), "{}", testing::describe(pixel(image, corner)));
    // The cube's +Z face is in the middle of the view, in the first paint, a
    // blue.
    Point center = view.center() + Point(-30.0f, 30.0f);
    std::array<std::uint8_t, 3> painted = pixel(image, center);
    CHECK(painted[2] > painted[0] + 40 && painted[2] > painted[1], "{}", testing::describe(painted));

    // The flat shader colors it differently.
    app.program.shader = Shader::Flat;
    app.frame();
    std::array<std::uint8_t, 3> flat = pixel(app.render(gpu), center);
    CHECK(differs(flat, painted), "{} {}", testing::describe(flat), testing::describe(painted));
    CHECK(flat[2] > flat[0], "{}", testing::describe(flat));

    // Subdivided, the cube rounds into a ball, so the top face's corner at
    // the back of the outline, away from the cage's edges, is no longer
    // drawn.
    Point top_corner = app.program.camera.to_screen({-0.85f, 1.0f, -0.85f}, view).first;
    std::array<std::uint8_t, 3> before = pixel(app.render(gpu), top_corner);
    CHECK(differs(before, card), "{}", testing::describe(before));
    app.key_with(Modifiers::Control, Key::Digit3);
    std::array<std::uint8_t, 3> rounded = pixel(app.render(gpu), top_corner);
    CHECK(!differs(rounded, card), "{}", testing::describe(rounded));

    // A selected face is tinted with the accent.
    app.key_with(Modifiers::Control, Key::Digit0);
    std::array<std::uint8_t, 3> unselected = pixel(app.render(gpu), center);
    app.click_point(view.center());
    app.move_point(view.origin() + Point(10.0f, 10.0f));
    std::array<std::uint8_t, 3> selected = pixel(app.render(gpu), center);
    CHECK(differs(selected, unselected), "{} {}", testing::describe(selected), testing::describe(unselected));

    // At fullscreen's height, the view's taller, and the mesh still in it.
    app.set_fullscreen(true).frames(2);
    image = app.render(gpu);
    CHECK(app.program.view_.height > view.height);
    CHECK(differs(pixel(image, app.program.view_.center()), card));
}
