#include "UserInterface/Drawing/Renderer.h"

#include "InterfacePixel.h"
#include "InterfaceVertex.h"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <span>
#include <vector>

namespace user_interface {

namespace {

using Microsoft::WRL::ComPtr;

// Where each part of the upload buffer starts: copies of texture rows need
// 512-byte alignment.
constexpr size_t Alignment = 512;

// The smallest multiple of `multiple` that isn't less than the value:
// usize::next_multiple_of.
constexpr size_t next_multiple_of(size_t value, size_t multiple) {
    size_t remainder = value % multiple;
    return remainder == 0 ? value : value + (multiple - remainder);
}

size_t pitch(std::uint32_t width) {
    return next_multiple_of(width, D3D12_TEXTURE_DATA_PITCH_ALIGNMENT);
}

ComPtr<ID3D12RootSignature> create_root_signature(ID3D12Device* device) {
    ComPtr<ID3D12RootSignature> root_signature;
    base::check(device->CreateRootSignature(0, InterfaceVertex, sizeof(InterfaceVertex), IID_PPV_ARGS(&root_signature)));
    return root_signature;
}

ComPtr<ID3D12PipelineState> create_pipeline(ID3D12Device* device, ID3D12RootSignature* root_signature) {
    // Premultiplied alpha.
    D3D12_RENDER_TARGET_BLEND_DESC blend{
        .BlendEnable = TRUE,
        .SrcBlend = D3D12_BLEND_ONE,
        .DestBlend = D3D12_BLEND_INV_SRC_ALPHA,
        .BlendOp = D3D12_BLEND_OP_ADD,
        .SrcBlendAlpha = D3D12_BLEND_ONE,
        .DestBlendAlpha = D3D12_BLEND_INV_SRC_ALPHA,
        .BlendOpAlpha = D3D12_BLEND_OP_ADD,
        .RenderTargetWriteMask = D3D12_COLOR_WRITE_ENABLE_ALL,
    };
    D3D12_GRAPHICS_PIPELINE_STATE_DESC description{
        .pRootSignature = root_signature,
        .VS = graphics::bytecode(InterfaceVertex),
        .PS = graphics::bytecode(InterfacePixel),
        .SampleMask = std::numeric_limits<UINT>::max(),
        .RasterizerState =
            {
                .FillMode = D3D12_FILL_MODE_SOLID,
                .CullMode = D3D12_CULL_MODE_NONE,
                .DepthClipEnable = TRUE,
            },
        .PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        .NumRenderTargets = 1,
        .SampleDesc = {.Count = 1, .Quality = 0},
    };
    for (D3D12_RENDER_TARGET_BLEND_DESC& render_target : description.BlendState.RenderTarget) {
        render_target = blend;
    }
    description.RTVFormats[0] = graphics::TargetFormat;
    ComPtr<ID3D12PipelineState> pipeline;
    base::check(device->CreateGraphicsPipelineState(&description, IID_PPV_ARGS(&pipeline)));
    return pipeline;
}

} // namespace

Renderer::Renderer(const graphics::Gpu& gpu) :
    root_signature(create_root_signature(gpu.device.Get())),
    pipeline(create_pipeline(gpu.device.Get(), root_signature.Get())),
    atlas(graphics::create_texture(gpu.device.Get(), AtlasSize, AtlasSize, DXGI_FORMAT_R8_UNORM, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE, nullptr, 1)),
    descriptors(gpu.device.Get(), D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV, 1, true) {
    D3D12_SHADER_RESOURCE_VIEW_DESC view{
        .Format = DXGI_FORMAT_R8_UNORM,
        .ViewDimension = D3D12_SRV_DIMENSION_TEXTURE2D,
        .Shader4ComponentMapping = D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        .Texture2D = {.MipLevels = 1},
    };
    gpu.device->CreateShaderResourceView(atlas.Get(), &view, descriptors.cpu(0));
}

void Renderer::draw(const graphics::Gpu& gpu, const graphics::Frame& frame, const DrawList& list, Fonts& fonts) {
    std::vector<GlyphUpload> uploads = fonts.take_uploads();
    size_t instance_bytes = std::max(list.instance_count() * sizeof(Instance), sizeof(Instance));
    size_t clip_bytes = list.clips.size() * sizeof(std::array<float, 4>);
    size_t point_bytes = std::max<size_t>(list.points.size(), 1) * sizeof(std::array<float, 2>);
    size_t glyph_bytes = 0;
    for (const GlyphUpload& glyph : uploads) {
        glyph_bytes += next_multiple_of(pitch(glyph.width) * glyph.height, Alignment);
    }
    size_t needed = next_multiple_of(instance_bytes, Alignment) + next_multiple_of(clip_bytes, Alignment) + next_multiple_of(point_bytes, Alignment) + glyph_bytes;
    if (!upload || upload->size < needed) {
        // Half again as big, so that a growing interface doesn't make a
        // new buffer every frame.
        upload = graphics::UploadBuffer(gpu.device.Get(), std::max<size_t>(needed * 3 / 2, size_t{1} << 20));
    }
    graphics::UploadBuffer& buffer = *upload;

    // The instances, layer by layer, then the clips, then the corners.
    size_t offset = 0;
    for (Layer layer : AllLayers) {
        std::span<const Instance> instances = list.layer_instances(layer);
        buffer.write(offset, instances);
        offset += instances.size_bytes();
    }
    size_t clips_at = next_multiple_of(instance_bytes, Alignment);
    buffer.write(clips_at, list.clips);
    size_t points_at = clips_at + next_multiple_of(clip_bytes, Alignment);
    if (list.points.empty()) {
        buffer.write(points_at, {std::array<float, 2>{0.0f, 0.0f}});
    } else {
        buffer.write(points_at, list.points);
    }
    offset = points_at + next_multiple_of(point_bytes, Alignment);

    ID3D12GraphicsCommandList* commands = frame.list.Get();
    if (!uploads.empty()) {
        D3D12_RESOURCE_BARRIER to_copy_into = graphics::transition(atlas.Get(), D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_COPY_DEST);
        commands->ResourceBarrier(1, &to_copy_into);
        for (const GlyphUpload& glyph : uploads) {
            size_t row_pitch = pitch(glyph.width);
            std::span<std::uint8_t> bytes = buffer.bytes_mut();
            for (size_t row = 0; row < glyph.height; row++) {
                std::span<const std::uint8_t> pixels = std::span(glyph.pixels).subspan(row * glyph.width, glyph.width);
                size_t at = offset + row * row_pitch;
                std::ranges::copy(pixels, bytes.subspan(at, pixels.size()).begin());
            }
            D3D12_TEXTURE_COPY_LOCATION source{
                .pResource = buffer.resource.Get(),
                .Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                .PlacedFootprint =
                    {
                        .Offset = offset,
                        .Footprint =
                            {
                                .Format = DXGI_FORMAT_R8_UNORM,
                                .Width = glyph.width,
                                .Height = glyph.height,
                                .Depth = 1,
                                .RowPitch = static_cast<UINT>(row_pitch),
                            },
                    },
            };
            D3D12_TEXTURE_COPY_LOCATION destination{
                .pResource = atlas.Get(),
                .Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                .SubresourceIndex = 0,
            };
            commands->CopyTextureRegion(&destination, glyph.x, glyph.y, 0, &source, nullptr);
            offset += next_multiple_of(row_pitch * glyph.height, Alignment);
        }
        D3D12_RESOURCE_BARRIER to_read = graphics::transition(atlas.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE);
        commands->ResourceBarrier(1, &to_read);
    }

    std::uint32_t count = static_cast<std::uint32_t>(list.instance_count());
    if (count == 0) {
        return;
    }
    frame.bind(false, std::nullopt);
    commands->SetPipelineState(pipeline.Get());
    commands->SetGraphicsRootSignature(root_signature.Get());
    ID3D12DescriptorHeap* heaps[] = {descriptors.heap.Get()};
    commands->SetDescriptorHeaps(1, heaps);
    std::array<float, 4> constants{static_cast<float>(frame.width), static_cast<float>(frame.height), 0.0f, 0.0f};
    commands->SetGraphicsRoot32BitConstants(0, 4, constants.data(), 0);
    commands->SetGraphicsRootShaderResourceView(1, buffer.gpu_address(0));
    commands->SetGraphicsRootShaderResourceView(2, buffer.gpu_address(clips_at));
    commands->SetGraphicsRootShaderResourceView(3, buffer.gpu_address(points_at));
    commands->SetGraphicsRootDescriptorTable(4, descriptors.gpu(0));
    commands->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    commands->DrawInstanced(4, count, 0, 0);
}

} // namespace user_interface
