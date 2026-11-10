// Draws a frame's draw list with Direct3D 12 in one draw call: the instances,
// clips and polygon corners go into an upload buffer, the glyphs rasterized
// since the last frame go into the atlas texture, and one instanced draw of a
// 4-vertex strip draws everything, layer by layer.
//
// It binds the frame's target, viewport and scissor itself, so a program's
// 3D may leave anything bound, or draw into a target of its own and copy it
// into the frame.

#pragma once

#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"

#include <d3d12.h>
#include <wrl/client.h>

#include <optional>

namespace user_interface {

class Renderer {
public:
    // Throws a base::WindowsError if Direct3D can't make what it draws with.
    explicit Renderer(const graphics::Gpu& gpu);
    Renderer(const Renderer&) = delete;
    Renderer& operator=(const Renderer&) = delete;
    Renderer(Renderer&&) = default;
    Renderer& operator=(Renderer&&) = default;

    // Records the draw list's drawing into the frame, over what it has. The
    // GPU must be done with the last frame this renderer drew, which the main
    // loop's pacing makes sure of. Throws a base::WindowsError if Direct3D
    // can't make the buffer it draws from.
    void draw(const graphics::Gpu& gpu, const graphics::Frame& frame, const DrawList& list, Fonts& fonts);

private:
    Microsoft::WRL::ComPtr<ID3D12RootSignature> root_signature;
    Microsoft::WRL::ComPtr<ID3D12PipelineState> pipeline;
    Microsoft::WRL::ComPtr<ID3D12Resource> atlas;
    graphics::DescriptorHeap descriptors;
    std::optional<graphics::UploadBuffer> upload;
};

} // namespace user_interface
