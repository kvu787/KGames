// Compiles HLSL to DXIL with dxc.exe from the newest installed Windows SDK,
// for tests, which compile shaders while they run, so that no library is
// needed for it. A project's own shaders are compiled as it builds instead,
// by the Shader items that KGames.targets describes.

#pragma once

#include <expected>
#include <filesystem>
#include <span>
#include <string>
#include <string_view>

namespace shader_compiler {

// Picks dxc.exe from the newest installed Windows SDK.
std::filesystem::path find_dxc();

// Compiles one entry point of `source` into `output`. `arguments` go to dxc
// before the source, like {"-D", "COUNT=3", "-O3"}. Fails with what dxc
// wrote, like the line of a mistake in the shader.
std::expected<void, std::string> compile(const std::filesystem::path& source, std::string_view entry_point, std::string_view profile, std::span<const std::string_view> arguments,
                                         const std::filesystem::path& output);

} // namespace shader_compiler
