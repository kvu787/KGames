// Library/Base's text: UTF-8 as Rust reads it, and numbers as Rust writes
// them, which the files and the scripts' states that the programs write
// depend on.

#include "Base/Text.h"
#include "Testing/Testing.h"

#include <bit>
#include <cmath>
#include <cstdint>
#include <limits>
#include <string>
#include <vector>

namespace {

std::vector<char32_t> characters(std::string_view text) {
    std::vector<char32_t> characters;
    for (char32_t character : base::chars(text)) {
        characters.push_back(character);
    }
    return characters;
}

} // namespace

TEST(characters_decode_from_one_to_four_bytes) {
    CHECK_EQ(characters("a\xC3\xA9\xE2\x82\xAC\xF0\x9F\x98\x80"), (std::vector<char32_t>{U'a', 0xE9, 0x20AC, 0x1F600}));
    CHECK_EQ(base::char_count("a\xC3\xA9\xE2\x82\xAC\xF0\x9F\x98\x80"), 4);
    std::string text;
    for (char32_t character : {char32_t{U'a'}, char32_t{0xE9}, char32_t{0x20AC}, char32_t{0x1F600}}) {
        base::push_char(text, character);
    }
    CHECK_EQ(text, "a\xC3\xA9\xE2\x82\xAC\xF0\x9F\x98\x80");
    CHECK_EQ(base::len_utf8(0x7F), 1);
    CHECK_EQ(base::len_utf8(0x80), 2);
    CHECK_EQ(base::len_utf8(0x800), 3);
    CHECK_EQ(base::len_utf8(0x10000), 4);
}

// One U+FFFD for each run of bytes that starts a character and goes wrong,
// as String::from_utf8_lossy gives, so text that isn't UTF-8 reads the same
// in both languages.
TEST(bytes_that_arent_utf8_are_replaced_as_rust_replaces_them) {
    // A continuation byte on its own, and a byte that starts nothing.
    CHECK_EQ(base::from_utf8_lossy("a\x80z"), "a\xEF\xBF\xBDz");
    CHECK_EQ(base::from_utf8_lossy("a\xFFz"), "a\xEF\xBF\xBDz");
    // The start of a three-byte character, cut short: one replacement for
    // its two bytes.
    CHECK_EQ(base::from_utf8_lossy("a\xE2\x82z"), "a\xEF\xBF\xBDz");
    // The start of a four-byte character, cut short after three.
    CHECK_EQ(base::from_utf8_lossy("\xF0\x9F\x98"), "\xEF\xBF\xBD");
    // The long form of '/', which isn't UTF-8: each byte on its own.
    CHECK_EQ(base::from_utf8_lossy("\xC0\xAF"), "\xEF\xBF\xBD\xEF\xBF\xBD");
    // A surrogate, which isn't a character: ED A0 80 is three replacements.
    CHECK_EQ(base::from_utf8_lossy("\xED\xA0\x80"), "\xEF\xBF\xBD\xEF\xBF\xBD\xEF\xBF\xBD");
    // Past U+10FFFF.
    CHECK_EQ(base::from_utf8_lossy("\xF4\x90\x80\x80"), "\xEF\xBF\xBD\xEF\xBF\xBD\xEF\xBF\xBD\xEF\xBF\xBD");
    CHECK(!base::is_utf8("a\x80z"));
    CHECK(!base::is_utf8("\xED\xA0\x80"));
    CHECK(!base::is_utf8("\xE2\x82"));
    // U+FFFD itself is a character like any other.
    CHECK(base::is_utf8("\xEF\xBF\xBD"));
    CHECK(base::is_utf8("a\xC3\xA9\xF0\x9F\x98\x80"));
    CHECK(base::is_utf8(""));
}

TEST(trim_takes_unicodes_whitespace_off_both_ends) {
    CHECK_EQ(base::trim("  \t a b \r\n"), "a b");
    // A no-break space and an ideographic space, as str::trim takes.
    CHECK_EQ(base::trim("\xC2\xA0#102030\xE3\x80\x80"), "#102030");
    CHECK_EQ(base::trim(" \xC3\xA9 "), "\xC3\xA9");
    CHECK_EQ(base::trim("   "), "");
    CHECK_EQ(base::trim(""), "");
    // Bytes that aren't UTF-8 aren't whitespace.
    CHECK_EQ(base::trim(" \x80 "), "\x80");
}

TEST(splits_lines_and_cases) {
    CHECK_EQ(base::split("a,,b", ','), (std::vector<std::string_view>{"a", "", "b"}));
    CHECK_EQ(base::split("", ','), (std::vector<std::string_view>{""}));
    CHECK_EQ(base::lines("a\r\nb\n\nc"), (std::vector<std::string_view>{"a", "b", "", "c"}));
    CHECK_EQ(base::lines("a\n"), (std::vector<std::string_view>{"a"}));
    CHECK_EQ(base::to_ascii_lowercase("AbC\xC3\x89"), "abc\xC3\x89");
    CHECK_EQ(base::to_ascii_uppercase("AbC"), "ABC");
    CHECK_EQ(base::join(std::vector<std::string>{"a", "b", "c"}, ", "), "a, b, c");
    CHECK_EQ(base::join(std::vector<std::string_view>{}, ", "), "");
    CHECK_EQ(base::join(std::vector<std::string_view>{"one"}, ", "), "one");
    CHECK_EQ(base::trim_end(" a b \t\n"), " a b");
    CHECK_EQ(base::trim_end("a\xC2\xA0"), "a");
    CHECK(base::eq_ignore_ascii_case("Ctrl", "cTRL"));
    CHECK(!base::eq_ignore_ascii_case("Ctrl", "Ctr"));
    CHECK_EQ(base::replace("a.b.c", ".", "::"), "a::b::c");
    CHECK_EQ(base::replace("abc", "", "x"), "abc");
    CHECK(base::is_char_boundary("a\xC3\xA9", 1));
    CHECK(!base::is_char_boundary("a\xC3\xA9", 2));
    CHECK(base::is_char_boundary("a\xC3\xA9", 3));
    CHECK(!base::is_char_boundary("a\xC3\xA9", 4));
}

// What Rust's {} writes: the shortest digits that read back, never an
// exponent, and no ".0".
TEST(display_writes_numbers_as_rust_does) {
    CHECK_EQ(base::display(1.0), "1");
    CHECK_EQ(base::display(0.1f), "0.1");
    CHECK_EQ(base::display(-0.0), "-0");
    CHECK_EQ(base::display(1e21), "1000000000000000000000");
    CHECK_EQ(base::display(1e-7), "0.0000001");
    CHECK_EQ(base::display(16777216.0f), "16777216");
    CHECK_EQ(base::display(1.0 / 3.0), "0.3333333333333333");
    CHECK_EQ(base::display(std::numeric_limits<double>::infinity()), "inf");
    CHECK_EQ(base::display(-std::numeric_limits<float>::infinity()), "-inf");
    CHECK_EQ(base::display(std::numeric_limits<double>::quiet_NaN()), "NaN");
}

// What Rust's {:?} writes: ".0" for a whole number, and an exponent below
// 0.0001 and from 1e16.
TEST(debug_writes_numbers_as_rust_does) {
    CHECK_EQ(base::debug(1.0), "1.0");
    CHECK_EQ(base::debug(0.0), "0.0");
    CHECK_EQ(base::debug(-0.0), "-0.0");
    CHECK_EQ(base::debug(0.1f), "0.1");
    CHECK_EQ(base::debug(0.0001), "0.0001");
    CHECK_EQ(base::debug(0.00001), "1e-5");
    CHECK_EQ(base::debug(1.5e-7), "1.5e-7");
    CHECK_EQ(base::debug(9999999999999998.0), "9999999999999998.0");
    CHECK_EQ(base::debug(1e16), "1e16");
    CHECK_EQ(base::debug(1.5e300), "1.5e300");
    CHECK_EQ(base::debug(-2.5e-10f), "-2.5e-10");
    CHECK_EQ(base::debug(3.4028235e38f), "3.4028235e38");
    CHECK_EQ(base::debug(std::numeric_limits<float>::quiet_NaN()), "NaN");
    CHECK_EQ(base::debug(std::string_view("a\"b\\c\n\t")), "\"a\\\"b\\\\c\\n\\t\"");
}

TEST(parses_numbers_as_rust_does) {
    CHECK_EQ(base::parse_f64("1.5"), 1.5);
    CHECK_EQ(base::parse_f64("+1.5"), 1.5);
    CHECK_EQ(base::parse_f64("-1e3"), -1000.0);
    CHECK_EQ(base::parse_f64(".5"), 0.5);
    CHECK_EQ(base::parse_f64("5."), 5.0);
    CHECK_EQ(base::parse_f64("1e400"), std::numeric_limits<double>::infinity());
    CHECK_EQ(base::parse_f64("-1e400"), -std::numeric_limits<double>::infinity());
    CHECK_EQ(base::parse_f64("1e-400"), 0.0);
    CHECK_EQ(base::parse_f64("inf"), std::numeric_limits<double>::infinity());
    CHECK(base::parse_f64("NaN").value() != base::parse_f64("NaN").value());
    CHECK_EQ(base::parse_f32("0.1"), 0.1f);
    for (std::string_view wrong : {"", " 1", "1 ", "1x", "+-1", "++1", "e5", "+", "-", "."}) {
        CHECK(!base::parse_f64(wrong).has_value(), "{}", testing::describe(wrong));
    }
    CHECK_EQ(base::parse_i64("-12"), -12);
    CHECK_EQ(base::parse_i64("+12"), 12);
    CHECK(!base::parse_i64("1.0").has_value());
    CHECK(!base::parse_i64("9223372036854775808").has_value());
    CHECK_EQ(base::parse_u64("18446744073709551615"), std::numeric_limits<std::uint64_t>::max());
    CHECK(!base::parse_u64("-1").has_value());
    CHECK(!base::parse_u64("").has_value());
    CHECK_EQ(base::parse_u64("ff", 16), 255);
    CHECK_EQ(base::parse_u64("+7F", 16), 127);
    CHECK(!base::parse_u64("g", 16).has_value());
    CHECK(!base::parse_u64("-1", 16).has_value());
}

TEST(utf16_goes_there_and_back) {
    std::string text = "a\xC3\xA9\xE2\x82\xAC\xF0\x9F\x98\x80";
    std::wstring wide = base::widen(text);
    // The emoji takes two UTF-16 units.
    CHECK_EQ(wide.size(), 5);
    CHECK_EQ(base::narrow(wide), text);
    CHECK_EQ(base::widen(""), std::wstring());
}

TEST(an_environment_variable_that_isnt_set_is_nothing) {
    CHECK(!base::environment_variable("KGAMES_BASE_TESTS_NO_SUCH_VARIABLE").has_value());
    CHECK(base::environment_variable("SystemRoot").has_value());
}

// The iterator decodes each character once, and stops at the text's end
// whatever the last character's length.
TEST(characters_iterate_to_the_end_of_any_text) {
    CHECK_EQ(characters(""), std::vector<char32_t>{});
    CHECK_EQ(characters("a"), std::vector<char32_t>{U'a'});
    CHECK_EQ(characters("\xF0\x9F\x98\x80"), std::vector<char32_t>{0x1F600});
    CHECK_EQ(characters("\xF0\x9F\x98\x80z"), (std::vector<char32_t>{0x1F600, U'z'}));
    // Cut short at the end: one replacement, and no read past the end.
    CHECK_EQ(characters("z\xF0\x9F"), (std::vector<char32_t>{U'z', base::Replacement}));
    base::Chars both = base::chars("ab");
    base::Chars::Iterator first = both.begin();
    base::Chars::Iterator second = first++;
    CHECK_EQ(*second, U'a');
    CHECK_EQ(*first, U'b');
    CHECK(++first == both.end());
}

// Where std::to_chars and Rust part ways, found by writing 1.2 million
// numbers with both. A number exactly halfway between two candidates of the
// same length takes the upper one in Rust, and a large number is its
// shortest digits and then zeros, never every digit it has.
TEST(numbers_are_written_as_rust_writes_them_at_the_edges) {
    // 254.6f as a double is 254.600006103515625: halfway at 17 digits.
    double widened = static_cast<double>(254.6f);
    CHECK_EQ(base::display(widened), "254.60000610351563");
    CHECK_EQ(base::debug(widened), "254.60000610351563");
    CHECK_EQ(base::parse_f64(base::display(widened)), widened);

    float large = std::bit_cast<float>(std::uint32_t{0xD6E8F2CF});
    CHECK_EQ(base::display(large), "-128064780000000");
    CHECK_EQ(base::debug(large), "-128064780000000.0");
    CHECK_EQ(base::display(std::bit_cast<float>(std::uint32_t{0xCDBDF613})), "-398377570");
    CHECK_EQ(base::debug(std::bit_cast<float>(std::uint32_t{0xCDBDF613})), "-398377570.0");

    double huge = std::bit_cast<double>(std::uint64_t{0xC78BA87FE0000000});
    CHECK_EQ(base::display(huge), "-4595506921444331000000000000000000000");
    CHECK_EQ(base::debug(huge), "-4.595506921444331e36");
    CHECK_EQ(base::display(std::bit_cast<double>(std::uint64_t{0x456E2969E0000000})), "291706757306122680000000000");

    double tiny = std::bit_cast<double>(std::uint64_t{0x397E69B5A0000000});
    CHECK_EQ(base::display(tiny), "0.00000000000000000000000000000009371706882305017");
    CHECK_EQ(base::debug(tiny), "9.371706882305017e-32");

    CHECK_EQ(base::display(5e-324), "0." + std::string(323, '0') + "5");
    CHECK_EQ(base::debug(5e-324), "5e-324");
    CHECK_EQ(base::debug(1.7976931348623157e308), "1.7976931348623157e308");
    CHECK_EQ(base::display(123456789.0f), "123456790");
    CHECK_EQ(base::debug(0.3f), "0.3");
    CHECK_EQ(base::debug(static_cast<double>(0.3f)), "0.30000001192092896");
}

// The start of a text is a character's boundary whatever its first byte is,
// as in Rust. A search back for a boundary, like a text box's, ends there:
// it didn't, in text that started with a byte that continues a character.
TEST(the_start_of_any_text_is_a_boundary) {
    CHECK(base::is_char_boundary("\x80"
                                 "abc",
                                 0));
    CHECK(base::is_char_boundary("", 0));
    CHECK(!base::is_char_boundary("a\x80", 1));
}

// A number too large to hold is infinity, and one too small is zero, as
// Rust reads them, whichever way it's written: with its size in its digits,
// or in its exponent.
TEST(numbers_out_of_range_go_to_the_right_end) {
    std::string tiny = "0." + std::string(400, '0') + "1";
    CHECK_EQ(base::parse_f64(tiny), 0.0);
    std::string huge = "1" + std::string(400, '0');
    CHECK_EQ(base::parse_f64(huge), std::numeric_limits<double>::infinity());
    // Its digits are huge and its exponent is negative: still huge.
    CHECK_EQ(base::parse_f64(huge + "e-5"), std::numeric_limits<double>::infinity());
    // Its digits are tiny and its exponent is positive: still tiny.
    CHECK_EQ(base::parse_f64(tiny + "e5"), 0.0);
    CHECK_EQ(base::parse_f64("-" + huge), -std::numeric_limits<double>::infinity());
    CHECK(std::signbit(base::parse_f64("-" + tiny).value()));
    CHECK_EQ(base::parse_f64("1e400"), std::numeric_limits<double>::infinity());
    CHECK_EQ(base::parse_f64("1e-400"), 0.0);
    CHECK_EQ(base::parse_f64("1e99999999999999999999999"), std::numeric_limits<double>::infinity());
    CHECK_EQ(base::parse_f64("1e-99999999999999999999999"), 0.0);
    CHECK_EQ(base::parse_f32("1e39"), std::numeric_limits<float>::infinity());
    CHECK_EQ(base::parse_f32("1e-50"), 0.0f);
    // In range, close to the ends.
    CHECK_EQ(base::parse_f64("1.7976931348623157e308"), std::numeric_limits<double>::max());
    CHECK_EQ(base::parse_f64("5e-324"), std::numeric_limits<double>::denorm_min());
}
