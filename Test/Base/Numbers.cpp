// Library/Base's numbers, sum types, files and errors: what Rust does that
// C++ doesn't, which the rest of the repository relies on.

#include "Base/Numbers.h"
#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <cstdint>
#include <filesystem>
#include <limits>
#include <string>

namespace {

struct Shape {
    struct Circle {
        float radius;
        bool operator==(const Circle&) const = default;
    };
    struct Rectangle {
        float width;
        float height;
        bool operator==(const Rectangle&) const = default;
    };
    struct Nothing {
        bool operator==(const Nothing&) const = default;
    };
    std::variant<Circle, Rectangle, Nothing> value;
    template <base::AlternativeOf<Shape> T> Shape(T&& alternative) : value(std::forward<T>(alternative)) {}
    bool operator==(const Shape&) const = default;
};

float area(const Shape& shape) {
    return base::match(
        shape, [](const Shape::Circle& circle) { return 3.0f * circle.radius * circle.radius; },
        [](const Shape::Rectangle& rectangle) { return rectangle.width * rectangle.height; }, [](const Shape::Nothing&) { return 0.0f; });
}

} // namespace

// What `as` gives in Rust: toward zero, the nearest the type holds when it's
// out of range, and 0 for NaN. In C++ the plain cast is undefined there.
TEST(a_float_becomes_a_whole_number_as_rusts_as_does) {
    CHECK_EQ(base::saturating_cast<std::uint8_t>(255.9f), 255);
    CHECK_EQ(base::saturating_cast<std::uint8_t>(256.0f), 255);
    CHECK_EQ(base::saturating_cast<std::uint8_t>(-0.9f), 0);
    CHECK_EQ(base::saturating_cast<std::uint8_t>(-5.0f), 0);
    CHECK_EQ(base::saturating_cast<std::uint8_t>(1e30f), 255);
    CHECK_EQ(base::saturating_cast<std::uint8_t>(std::numeric_limits<float>::quiet_NaN()), 0);
    CHECK_EQ(base::saturating_cast<std::int32_t>(-1.9), -1);
    CHECK_EQ(base::saturating_cast<std::int32_t>(1e12), std::numeric_limits<std::int32_t>::max());
    CHECK_EQ(base::saturating_cast<std::int32_t>(-1e12), std::numeric_limits<std::int32_t>::min());
    CHECK_EQ(base::saturating_cast<std::int64_t>(9.3e18), std::numeric_limits<std::int64_t>::max());
    CHECK_EQ(base::saturating_cast<std::uint64_t>(1.9e19), std::numeric_limits<std::uint64_t>::max());
    CHECK_EQ(base::saturating_cast<std::uint64_t>(std::numeric_limits<double>::infinity()), std::numeric_limits<std::uint64_t>::max());
    CHECK_EQ(base::saturating_cast<size_t>(41.99f), 41);
}

TEST(whole_numbers_stop_where_rusts_do) {
    CHECK_EQ(base::saturating_sub(size_t{3}, size_t{5}), 0);
    CHECK_EQ(base::saturating_sub(size_t{5}, size_t{3}), 2);
    CHECK_EQ(base::checked_sub(size_t{5}, size_t{3}), size_t{2});
    CHECK(!base::checked_sub(size_t{3}, size_t{5}).has_value());
    CHECK_EQ(base::abs_diff(size_t{3}, size_t{5}), 2);
    CHECK_EQ(base::abs_diff(std::numeric_limits<std::int32_t>::min(), std::numeric_limits<std::int32_t>::max()), std::numeric_limits<std::uint32_t>::max());
    CHECK_EQ(base::rem_euclid(-1, 4), 3);
    CHECK_EQ(base::rem_euclid(-1, -4), 3);
    CHECK_EQ(base::rem_euclid(7, 4), 3);
    CHECK_EQ(base::rem_euclid(-0.5f, 2.0f), 1.5f);
    CHECK_EQ(base::rem_euclid(5.5, 2.0), 1.5);
}

TEST(a_sum_type_holds_one_alternative) {
    Shape circle = Shape::Circle{2.0f};
    CHECK(base::is<Shape::Circle>(circle));
    CHECK(!base::is<Shape::Rectangle>(circle));
    CHECK(base::as<Shape::Rectangle>(circle) == nullptr);
    CHECK_EQ(base::as<Shape::Circle>(circle)->radius, 2.0f);
    CHECK_EQ(area(circle), 12.0f);
    CHECK_EQ(area(Shape::Rectangle{2.0f, 3.0f}), 6.0f);
    CHECK_EQ(area(Shape::Nothing{}), 0.0f);
    CHECK(circle == Shape(Shape::Circle{2.0f}));
    CHECK(circle != Shape(Shape::Circle{3.0f}));
    CHECK(circle != Shape(Shape::Nothing{}));
    // base::make is the function that an enum's variant is in Rust.
    Shape made = base::make<Shape::Rectangle>(4.0f, 5.0f);
    CHECK_EQ(area(made), 20.0f);
    base::as<Shape::Rectangle>(made)->width = 1.0f;
    CHECK_EQ(area(made), 5.0f);
}

TEST(a_panic_is_an_exception_that_a_test_catches) {
    CHECK_THROWS(base::panic("the cascade didn't settle"));
    CHECK_THROWS(base::unreachable());
    std::optional<int> nothing;
    CHECK_THROWS(base::expect(nothing, "there must be one"));
    std::optional<int> seven = 7;
    CHECK_EQ(base::expect(seven, "there must be one"), 7);
    try {
        base::panic("the message");
    } catch (const base::Panic& panic) {
        CHECK_EQ(std::string_view(panic.what()), "the message");
    }
}

TEST(a_failed_call_says_what_and_its_code) {
    try {
        base::check(E_INVALIDARG);
        CHECK(false, "E_INVALIDARG is a failure");
    } catch (const base::WindowsError& error) {
        CHECK_EQ(error.code(), E_INVALIDARG);
        CHECK(std::string_view(error.what()).ends_with("(0x80070057)"), "{}", error.what());
    }
    base::check(S_OK);
    base::check(S_FALSE);
    base::WindowsError own(E_FAIL, "No GPU supports Direct3D 12.");
    CHECK_EQ(std::string_view(own.what()), "No GPU supports Direct3D 12. (0x80004005)");
    CHECK_EQ(own.code(), E_FAIL);
}

TEST(files_write_and_read_back_and_say_what_went_wrong) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "KGamesBaseTests" / "Files";
    std::filesystem::remove_all(folder);
    // The folders it's in aren't there yet.
    std::filesystem::path file = folder / "Deeper" / "Text \xC3\xA9.txt";
    CHECK(!base::read_to_string(file).has_value());
    CHECK(base::read_to_string(file).error().ends_with("(os error 3)"), "{}", base::read_to_string(file).error());
    CHECK(base::create_dir_all(file.parent_path()).has_value());
    CHECK(base::read_to_string(file).error().ends_with("(os error 2)"), "{}", base::read_to_string(file).error());

    std::string text = "line\n\xC3\xA9\xF0\x9F\x98\x80";
    CHECK(base::write(file, text).has_value());
    CHECK_EQ(base::read_to_string(file).value(), text);
    CHECK_EQ(base::read(file).value().size(), text.size());
    // Writing replaces what's there.
    CHECK(base::write(file, std::string_view("x")).has_value());
    CHECK_EQ(base::read_to_string(file).value(), "x");

    // Text that isn't UTF-8 isn't text, as in Rust.
    CHECK(base::write(file, std::string_view("a\xFFz")).has_value());
    CHECK_EQ(base::read_to_string(file).error(), "stream did not contain valid UTF-8");
    CHECK_EQ(base::read(file).value().size(), 3);

    // A path is UTF-8 on the way in and out.
    CHECK_EQ(base::display(base::path("C:\\a\\\xC3\xA9")), "C:\\a\\\xC3\xA9");
    CHECK(base::current_exe().filename() == std::filesystem::path("BaseTests.exe"));
    std::filesystem::remove_all(folder);
}
