// SimplePaintDemo's frames, drawn offscreen with the GPU, as the window would
// draw them: the sphere fills the preview card's view, around the card's
// color, at both window heights, and is drawn again when the material changes.

#include "Base/Numbers.h"
#include "Graphics/Graphics.h"
#include "SimplePaint/Material.h"
#include "SimplePaintDemo/Demo.h"
#include "Testing/Testing.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Testing/Headless.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <filesystem>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::Image;
using simple_paint::Parameters;
using simple_paint_demo::demo::Demo;
using user_interface::Headless;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Theme;

namespace {

std::array<std::uint8_t, 3> pixel(const Image& image, Point point) {
    auto [r, g, b, a] = image.pixel(base::saturating_cast<std::uint32_t>(point.x), base::saturating_cast<std::uint32_t>(point.y));
    (void)a;
    return {r, g, b};
}

bool near_(std::array<std::uint8_t, 3> actual, std::array<std::uint8_t, 3> expected) {
    for (size_t channel = 0; channel < 3; channel++) {
        if (base::abs_diff(actual[channel], expected[channel]) > 1) {
            return false;
        }
    }
    return true;
}

// What the frame shows at the points: the card's color around the sphere,
// inside the view and outside it, and the sphere, right of its middle.
std::array<std::uint8_t, 3> check(const Image& image, Rect view, std::array<std::uint8_t, 3> card) {
    Point middle = view.center();
    for (Point point : {
             view.origin() + Point(8.0f, 8.0f),
             Point(view.right() - 8.0f, view.bottom() - 8.0f),
             Point(view.x - 8.0f, middle.y),
             Point(view.x + 8.0f, middle.y),
         }) {
        CHECK(near_(pixel(image, point), card), "{} at {} isn't the card's {}", testing::describe(pixel(image, point)), testing::describe(point), testing::describe(card));
    }
    std::array<std::uint8_t, 3> sphere = pixel(image, middle + Point(300.0f, 0.0f));
    CHECK(!near_(sphere, card), "the sphere isn't in the view");
    return sphere;
}

} // namespace

TEST(the_sphere_fills_the_preview_card_s_view_at_both_heights) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless app(Demo(Parameters{}, std::filesystem::path(R"(C:\Nowhere\Paint.json)")));
    const Theme& theme = app.ui.theme();
    auto [r, g, b, a] = theme.card_opaque().to_bytes();
    (void)a;
    std::array<std::uint8_t, 3> card{r, g, b};

    Image image = app.render(gpu);
    Rect view = app.program.view_.value();
    std::array<std::uint8_t, 3> blue = check(image, view, card);
    // The default material is a blue.
    CHECK(blue[2] > blue[0] + 64 && blue[2] > blue[1], "{}", testing::describe(blue));

    // In fullscreen, the view is taller, and the sphere is still in its middle.
    app.set_fullscreen(true).frame();
    image = app.render(gpu);
    Rect tall = app.program.view_.value();
    CHECK(tall.height > view.height);
    CHECK_EQ(check(image, tall, card), blue);

    // A new material draws the sphere again.
    app.program.editor.set(0, 0.9);
    app.program.editor.set(2, 0.1);
    app.frame();
    image = app.render(gpu);
    std::array<std::uint8_t, 3> red = check(image, tall, card);
    CHECK(red[0] > red[2] + 64, "{}", testing::describe(red));
}
