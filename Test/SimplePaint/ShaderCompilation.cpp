// The DX12 adapter compiles with the default material count of one and with
// an explicit count, and rejects a count that can't describe a material.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "ShaderCompiler/ShaderCompiler.h"
#include "SimplePaint/SimplePaint.h"
#include "Testing/Testing.h"

#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <vector>

namespace {

std::expected<void, std::string> compile(std::string_view entry_point, std::string_view profile, std::optional<std::uint32_t> count) {
    std::filesystem::path source = base::path(simple_paint::ShaderDirectory) / "SimplePaint.hlsl";
    std::optional<std::string> define = count.transform([](std::uint32_t materials) { return std::format("SIMPLE_PAINT_MATERIAL_COUNT={}", materials); });
    std::vector<std::string_view> arguments{"-O3", "-Ges", "-WX"};
    if (define) {
        arguments.push_back("-D");
        arguments.push_back(*define);
    }
    // The tests' own folder for what they write.
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "KGames" / "SimplePaintTests";
    if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
        return made;
    }
    std::filesystem::path output = folder / std::format("SimplePaint{}{}.dxil", entry_point, count ? std::to_string(*count) : "Default");
    return shader_compiler::compile(source, entry_point, profile, arguments, output);
}

} // namespace

TEST(shader_compilation) {
    for (std::optional<std::uint32_t> count : {std::optional<std::uint32_t>(), std::optional<std::uint32_t>(3)}) {
        if (std::expected<void, std::string> compiled = compile("VSMain", "vs_6_0", count); !compiled) {
            base::panic(compiled.error());
        }
        if (std::expected<void, std::string> compiled = compile("PSMain", "ps_6_0", count); !compiled) {
            base::panic(compiled.error());
        }
    }
    std::expected<void, std::string> zero = compile("PSMain", "ps_6_0", 0);
    CHECK(!zero.has_value(), "a material count of 0 compiled");
    const std::string& error = zero.error();
    CHECK(error.contains("SIMPLE_PAINT_MATERIAL_COUNT must be positive"), "a material count of 0 failed for another reason: {}", error);
    std::println("Both stages compile with the default material count and with 3; 0 is rejected.");
}
