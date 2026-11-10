// Files: reading and writing whole files, with what went wrong as text for a
// message, which is what std::fs::read_to_string and std::fs::write are in
// Rust. Paths are std::filesystem::path. base::path makes one from UTF-8,
// and base::display writes one as UTF-8.

#pragma once

#include <cstdint>
#include <expected>
#include <filesystem>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace base {

// A path from UTF-8 text, and a path as UTF-8 text.
std::filesystem::path path(std::string_view text);
std::string display(const std::filesystem::path& path);

// The file's bytes, or what the system says is wrong, like
// "The system cannot find the file specified. (os error 2)".
std::expected<std::vector<std::uint8_t>, std::string> read(const std::filesystem::path& path);

// The file's text, which must be UTF-8, as it must be in Rust:
// "stream did not contain valid UTF-8" otherwise.
std::expected<std::string, std::string> read_to_string(const std::filesystem::path& path);

// Writes the file, replacing what's there.
std::expected<void, std::string> write(const std::filesystem::path& path, std::span<const std::uint8_t> bytes);
std::expected<void, std::string> write(const std::filesystem::path& path, std::string_view text);

// Makes the folder, and those it's in: std::fs::create_dir_all.
std::expected<void, std::string> create_dir_all(const std::filesystem::path& path);

// The program's own exe: std::env::current_exe.
std::filesystem::path current_exe();

// What the system says an error code means, as Rust's io::Error shows it.
std::string os_error(unsigned long code);

} // namespace base
