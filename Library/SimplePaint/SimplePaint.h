// SimplePaint colors 3D models in a way that looks good from any angle. It's
// an unlit, opaque, orthographic-only surface shader that paints a surface
// from its normal relative to the camera. Specification.md defines it, and
// Usage.md explains its controls and how to use it.
//
// - Material.h: the parameters, their validation, and the GPU constants.
// - Geometry.h: validation of meshes before they're uploaded.
// - Orthographic.h: validated transforms for the shader's vertex stage.
// - Direct3D12.h: the shader's pipeline and meshes on the GPU, for hosts
//   that draw with Library/Graphics.
// - Shaders: the HLSL, which hosts compile with their own material count.
//
// The facing-lobe and anchored color-curve mathematical construction is
// credited to Kevin Vu.

#pragma once

#include "SimplePaint/Direct3D12.h"
#include "SimplePaint/Error.h"
#include "SimplePaint/Geometry.h"
#include "SimplePaint/Material.h"
#include "SimplePaint/Orthographic.h"

#include <string_view>

namespace simple_paint {

// The folder of SimplePaint.hlsl and SimplePaintCore.hlsli, for compiling
// them at run time, like the tests do. Projects name SimplePaint.hlsl in a
// Shader item instead, to compile it as they build.
extern const std::string_view ShaderDirectory;

} // namespace simple_paint
