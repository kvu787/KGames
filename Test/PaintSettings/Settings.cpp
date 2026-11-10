// Library/PaintSettings: materials read from JSON with their defaults,
// rejected as SimplePaint would, and written back as text that reads the
// same, nested or not.

#include "PaintSettings/PaintSettings.h"
#include "SimplePaint/Material.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include <expected>
#include <format>
#include <string>
#include <string_view>
#include <utility>

using paint_settings::format;
using paint_settings::parse;
using paint_settings::to_value;
using simple_paint::Parameters;

namespace {

std::string text(const Parameters& parameters) {
    return to_value(parameters).to_json();
}

// The text with every `from` in it replaced by `to`: str::replace.
std::string replace(std::string_view source, std::string_view from, std::string_view to) {
    std::string replaced;
    while (true) {
        size_t at = source.find(from);
        replaced += source.substr(0, at);
        if (at == std::string_view::npos) {
            return replaced;
        }
        replaced += to;
        source.remove_prefix(at + from.size());
    }
}

} // namespace

TEST(a_material_reads_back_as_it_was_written) {
    Parameters material{
        .base_color_srgb = {0.107, 0.223, 0.578},
        .brightness = 0.5,
        .shift = 0.25,
        .rotation_degrees = 30.0,
        .dark_point = 0.1,
        .light_point = 0.9,
    };
    json::Value value = json::parse(text(material)).value();
    CHECK_EQ(parse(value, Parameters{}).value(), material);
}

TEST(a_missing_member_keeps_its_default_and_a_wrong_one_says_so) {
    Parameters defaults{.brightness = 0.75};
    json::Value value = json::parse(R"({ "Shift": 0.5 })").value();
    Parameters parsed = parse(value, defaults).value();
    CHECK_EQ(std::pair(parsed.shift, parsed.brightness), std::pair(0.5, 0.75));
    struct Case {
        std::string_view text;
        std::string_view problem;
    };
    for (const Case& expected : {
             Case{"[]", "must be an object with"},
             Case{R"({ "Shift": "half" })", "Shift must be a number"},
             Case{R"({ "BaseColor": [1, 0] })", "BaseColor must be a list of 3 numbers"},
             Case{R"({ "Gloss": 1 })", "Gloss isn't a SimplePaint setting"},
         }) {
        json::Value wrong = json::parse(expected.text).value();
        std::expected<Parameters, std::string> result = parse(wrong, defaults);
        CHECK(!result.has_value());
        const std::string& error = result.error();
        CHECK(error.contains(expected.problem), "{}: {}", expected.text, error);
    }
    // A value that SimplePaint rejects is rejected here.
    json::Value bright = json::parse(R"({ "Brightness": 7 })").value();
    CHECK(!parse(bright, defaults).has_value());
}

TEST(the_text_nests_by_indenting_the_lines_after_the_first) {
    Parameters material;
    std::string flat = format(material, 0);
    CHECK_EQ(flat, text(material));
    CHECK(flat.starts_with("{\n    \"BaseColor\": ["), "{}", flat);
    CHECK(flat.ends_with("\n}"), "{}", flat);
    std::string nested = format(material, 4);
    CHECK_EQ(nested, replace(flat, "\n", "\n    "));
    json::Value value = json::parse(std::format("{{ \"Paint\": {} }}", nested)).value();
    const json::Value* paint = value.get("Paint");
    CHECK(paint != nullptr);
    CHECK_EQ(parse(*paint, Parameters{}).value(), material);
}
