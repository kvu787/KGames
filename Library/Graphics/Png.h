// Writes images as PNGs, with the Windows Imaging Component.

#pragma once

#include "Graphics/Frame.h"

#include <filesystem>

namespace graphics {

// Without alpha, like a screenshot of a window.
void write_png(const std::filesystem::path& path, const Image& image);

} // namespace graphics
