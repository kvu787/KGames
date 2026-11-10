#include "SimplePaint/SimplePaint.h"

#include "Folders.h"

#include <array>
#include <cstddef>
#include <string>

namespace simple_paint {

namespace {

// The Shaders folder in this project's folder, which folders::Project is
// here, as the build made it for this file: the two as one text, made as
// this compiles.
constexpr std::string_view Folder = R"(\Shaders)";
constexpr size_t Length = std::char_traits<char>::length(folders::Project) + Folder.size();
constexpr std::array<char, Length + 1> Text = [] {
    std::array<char, Length + 1> text{};
    size_t at = 0;
    for (const char* character = folders::Project; *character != '\0'; character++) {
        text[at++] = *character;
    }
    for (char character : Folder) {
        text[at++] = character;
    }
    return text;
}();

} // namespace

constexpr std::string_view ShaderDirectory(Text.data(), Length);

} // namespace simple_paint
