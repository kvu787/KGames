#include "SimplePaint/Direct3D12.h"

#include "Graphics/Error.h"

#include <iterator>
#include <limits>

namespace simple_paint::direct3d12 {

using Microsoft::WRL::ComPtr;

Pipeline::Pipeline(ID3D12Device* device, std::span<const std::uint8_t> vertex_shader, std::span<const std::uint8_t> pixel_shader, const Target& target) {
    base::check(device->CreateRootSignature(0, vertex_shader.data(), vertex_shader.size(), IID_PPV_ARGS(&root_signature)));
    // Matches Vertex.
    D3D12_INPUT_ELEMENT_DESC input_elements[] = {
        {
            .SemanticName = "POSITION",
            .Format = DXGI_FORMAT_R32G32B32_FLOAT,
            .AlignedByteOffset = 0,
            .InputSlotClass = D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
        },
        {
            .SemanticName = "NORMAL",
            .Format = DXGI_FORMAT_R32G32B32_FLOAT,
            .AlignedByteOffset = 12,
            .InputSlotClass = D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
        },
        {
            .SemanticName = "MATERIAL",
            .Format = DXGI_FORMAT_R32_UINT,
            .AlignedByteOffset = 24,
            .InputSlotClass = D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
        },
    };
    D3D12_GRAPHICS_PIPELINE_STATE_DESC description{
        // Borrow the root signature without touching its reference count.
        .pRootSignature = root_signature.Get(),
        .VS = graphics::bytecode(vertex_shader),
        .PS = graphics::bytecode(pixel_shader),
        .SampleMask = std::numeric_limits<UINT>::max(),
        .RasterizerState =
            {
                .FillMode = D3D12_FILL_MODE_SOLID,
                .CullMode = target.cull_mode,
                .FrontCounterClockwise = TRUE,
                .DepthClipEnable = TRUE,
                .MultisampleEnable = target.sample_count > 1,
            },
        .DepthStencilState =
            {
                .DepthEnable = target.depth_format.has_value(),
                .DepthWriteMask = D3D12_DEPTH_WRITE_MASK_ALL,
                .DepthFunc = D3D12_COMPARISON_FUNC_LESS,
            },
        .InputLayout =
            {
                .pInputElementDescs = input_elements,
                .NumElements = static_cast<UINT>(std::size(input_elements)),
            },
        .PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        .NumRenderTargets = 1,
        .DSVFormat = target.depth_format.value_or(DXGI_FORMAT_UNKNOWN),
        .SampleDesc =
            {
                .Count = target.sample_count,
                .Quality = 0,
            },
    };
    for (D3D12_RENDER_TARGET_BLEND_DESC& render_target : description.BlendState.RenderTarget) {
        render_target.RenderTargetWriteMask = D3D12_COLOR_WRITE_ENABLE_ALL;
    }
    description.RTVFormats[0] = target.format;
    base::check(device->CreateGraphicsPipelineState(&description, IID_PPV_ARGS(&state)));
}

void Pipeline::bind(ID3D12GraphicsCommandList* list) const {
    list->SetPipelineState(state.Get());
    list->SetGraphicsRootSignature(root_signature.Get());
    list->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
}

Mesh::Mesh(graphics::Gpu& gpu, std::span<const Vertex> vertices, std::span<const std::uint32_t> indices, std::uint32_t material_count) {
    if (std::expected<void, Error> valid = validate_mesh(vertices, indices, material_count); !valid) {
        throw graphics::Error(E_INVALIDARG, valid.error().message);
    }
    ComPtr<ID3D12Device> device = gpu.device;
    size_t vertex_bytes = vertices.size_bytes();
    size_t index_bytes = indices.size_bytes();
    size_t size = vertex_bytes + index_bytes;
    graphics::UploadBuffer upload(device.Get(), size);
    upload.write(0, vertices);
    upload.write(vertex_bytes, indices);
    // Buffers are created in the common state, and the copy promotes
    // it to COPY_DEST.
    buffer = graphics::create_buffer(device.Get(), size, D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_STATE_COMMON);
    // The upload buffer must outlive the copy, which run waits for.
    gpu.run([&](ID3D12GraphicsCommandList* list) {
        list->CopyBufferRegion(buffer.Get(), 0, upload.resource.Get(), 0, size);
        D3D12_RESOURCE_BARRIER barrier =
            graphics::transition(buffer.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER | D3D12_RESOURCE_STATE_INDEX_BUFFER);
        list->ResourceBarrier(1, &barrier);
    });
    D3D12_GPU_VIRTUAL_ADDRESS start = buffer->GetGPUVirtualAddress();
    vertex_view = D3D12_VERTEX_BUFFER_VIEW{
        .BufferLocation = start,
        .SizeInBytes = static_cast<UINT>(vertex_bytes),
        .StrideInBytes = static_cast<UINT>(sizeof(Vertex)),
    };
    index_view = D3D12_INDEX_BUFFER_VIEW{
        .BufferLocation = start + vertex_bytes,
        .SizeInBytes = static_cast<UINT>(index_bytes),
        .Format = DXGI_FORMAT_R32_UINT,
    };
    index_count = static_cast<std::uint32_t>(indices.size());
}

void Mesh::draw(ID3D12GraphicsCommandList* list, D3D12_GPU_VIRTUAL_ADDRESS object, D3D12_GPU_VIRTUAL_ADDRESS materials) const {
    list->SetGraphicsRootConstantBufferView(0, object);
    list->SetGraphicsRootConstantBufferView(1, materials);
    list->IASetVertexBuffers(0, 1, &vertex_view);
    list->IASetIndexBuffer(&index_view);
    list->DrawIndexedInstanced(index_count, 1, 0, 0, 0);
}

} // namespace simple_paint::direct3d12
