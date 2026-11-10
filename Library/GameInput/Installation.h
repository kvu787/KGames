// The GameInput runtime that KGames needs installed: version 4.0.306, which
// the user installs, once, with InstallCommand. GameInputRedist.msi
// installs it into C:\Program Files\Microsoft GameInput\x64, then runs its
// service with /serviceconfig, which copies its DLL into System32, where
// GameInput.lib's loader finds it (see README.md). Nothing in KGames installs
// it.
//
// installed_runtime checks that the DLL is there, and that its version is
// Version, with Library/Base alone, so that InstallationCheck can compile
// Installation.cpp and fail the build without it, and every program checks
// it again as it starts, so that it says how to install it, instead of
// failing to start GameInput.

#pragma once

#include <array>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <string_view>

namespace game_input::installation {

// What the user runs, from an elevated prompt, to install the runtime.
inline constexpr std::string_view InstallCommand = R"(winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306")";

// The installed x64 runtime's DLL, in System32, where GameInput's service
// copies it.
inline constexpr std::string_view Runtime = R"(C:\Windows\System32\GameInputRedist.dll)";

// The runtime's version, as its DLL's file version says it.
inline constexpr std::array<std::uint16_t, 4> Version = {4, 0, 306, 0};

// The runtime's DLL, once it's there and its version is Version, or what's
// wrong, and how to install it.
std::expected<std::filesystem::path, std::string> installed_runtime();

// The DLL at `path`, once its version is Version. It's read without letting
// others write it, so a DLL that's being written isn't checked.
std::expected<std::filesystem::path, std::string> check(const std::filesystem::path& path);

// The file version in a DLL's version resource: its VS_FIXEDFILEINFO starts
// with the signature 0xFEEF04BD and the structure's version, 1.0, then has the
// file version's most and least significant halves, all little-endian. It's
// searched for from the end, where a DLL's resources are.
std::optional<std::array<std::uint16_t, 4>> version_in(std::span<const std::uint8_t> bytes);

// What a build or a program says when the runtime's DLL isn't there.
std::string not_installed_message(const std::filesystem::path& path);

// What a build or a program says when the runtime's version isn't Version,
// given its version, if it has one.
std::string wrong_version_message(const std::filesystem::path& path, std::optional<std::array<std::uint16_t, 4>> version);

} // namespace game_input::installation
