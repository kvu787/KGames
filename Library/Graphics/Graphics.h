// Direct3D 12 for KGames' programs: the GPU that draws, a swap chain paced
// for the lowest input latency, frames to draw into, offscreen targets for
// tests, readback of what was drawn, and the small helpers that every user of
// the device needs, like buffers, textures and barriers.
//
// Library/UserInterface draws its widgets with it, and programs draw their 3D
// views into the same frames, before the widgets go on top.

#pragma once

#include "Graphics/Depth.h"
#include "Graphics/Device.h"
#include "Graphics/Frame.h"
#include "Graphics/Png.h"
#include "Graphics/Presenter.h"
#include "Graphics/Resources.h"
#include "Graphics/Timer.h"

#include <cstddef>
#include <dxgiformat.h>

namespace graphics {

// The format of every frame's target, swap chain or offscreen. Flip model
// swap chains can't be sRGB, so frames have two views: this one, which
// stores colors as they are, for the interface, whose colors are sRGB
// already, and an sRGB one, which encodes linear colors, for 3D shading.
inline constexpr DXGI_FORMAT TargetFormat = DXGI_FORMAT_B8G8R8A8_UNORM;
inline constexpr DXGI_FORMAT TargetSrgbFormat = DXGI_FORMAT_B8G8R8A8_UNORM_SRGB;
// For the depth buffers of 3D views.
inline constexpr DXGI_FORMAT DepthFormat = DXGI_FORMAT_D32_FLOAT;
// Where constant buffers may start in a buffer.
inline constexpr size_t ConstantAlignment = 256;

} // namespace graphics
