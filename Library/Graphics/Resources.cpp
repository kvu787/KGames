#include "Graphics/Resources.h"

#include "Base/Panic.h"
#include "Graphics/Graphics.h"

#include <algorithm>
#include <cstring>

namespace graphics {

D3D12_HEAP_PROPERTIES heap_properties(D3D12_HEAP_TYPE kind) {
    return D3D12_HEAP_PROPERTIES{
        .Type = kind,
        .CreationNodeMask = 1,
        .VisibleNodeMask = 1,
    };
}

ComPtr<ID3D12Resource> create_buffer(ID3D12Device* device, std::uint64_t size, D3D12_HEAP_TYPE heap, D3D12_RESOURCE_STATES state) {
    D3D12_RESOURCE_DESC description{
        .Dimension = D3D12_RESOURCE_DIMENSION_BUFFER,
        .Width = std::max<std::uint64_t>(size, 1),
        .Height = 1,
        .DepthOrArraySize = 1,
        .MipLevels = 1,
        .SampleDesc = {.Count = 1, .Quality = 0},
        .Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
    };
    D3D12_HEAP_PROPERTIES properties = heap_properties(heap);
    ComPtr<ID3D12Resource> resource;
    base::check(device->CreateCommittedResource(&properties, D3D12_HEAP_FLAG_NONE, &description, state, nullptr, IID_PPV_ARGS(&resource)));
    return resource;
}

ComPtr<ID3D12Resource> create_texture(ID3D12Device* device, std::uint32_t width, std::uint32_t height, DXGI_FORMAT format, D3D12_RESOURCE_FLAGS flags, D3D12_RESOURCE_STATES state,
                                      const D3D12_CLEAR_VALUE* clear, std::uint32_t sample_count) {
    D3D12_RESOURCE_DESC description{
        .Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        .Width = width,
        .Height = height,
        .DepthOrArraySize = 1,
        .MipLevels = 1,
        .Format = format,
        .SampleDesc = {.Count = sample_count, .Quality = 0},
        .Flags = flags,
    };
    D3D12_HEAP_PROPERTIES properties = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
    ComPtr<ID3D12Resource> resource;
    base::check(device->CreateCommittedResource(&properties, D3D12_HEAP_FLAG_NONE, &description, state, clear, IID_PPV_ARGS(&resource)));
    return resource;
}

D3D12_RESOURCE_BARRIER transition(ID3D12Resource* resource, D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after) {
    return D3D12_RESOURCE_BARRIER{
        .Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        .Flags = D3D12_RESOURCE_BARRIER_FLAG_NONE,
        .Transition =
            {
                // Borrowed without touching its reference count.
                .pResource = resource,
                .Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                .StateBefore = before,
                .StateAfter = after,
            },
    };
}

D3D12_SHADER_BYTECODE bytecode(std::span<const std::uint8_t> shader) {
    return D3D12_SHADER_BYTECODE{
        .pShaderBytecode = shader.data(),
        .BytecodeLength = shader.size(),
    };
}

UploadBuffer::UploadBuffer(ID3D12Device* device, size_t size) :
    resource(create_buffer(device, size, D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ)), size(size), mapped(nullptr) {
    void* address = nullptr;
    // The CPU never reads it.
    D3D12_RANGE read{.Begin = 0, .End = 0};
    base::check(resource->Map(0, &read, &address));
    mapped = static_cast<std::uint8_t*>(address);
}

void UploadBuffer::write_bytes(size_t offset, const void* values, size_t bytes) const {
    if (offset + bytes > size) {
        base::panic("write past the end of an upload buffer");
    }
    std::memcpy(mapped + offset, values, bytes);
}

std::span<std::uint8_t> UploadBuffer::bytes_mut() {
    return std::span<std::uint8_t>(mapped, size);
}

D3D12_GPU_VIRTUAL_ADDRESS UploadBuffer::gpu_address(size_t offset) const {
    return resource->GetGPUVirtualAddress() + offset;
}

DescriptorHeap::DescriptorHeap(ID3D12Device* device, D3D12_DESCRIPTOR_HEAP_TYPE kind, std::uint32_t count, bool shader_visible) {
    D3D12_DESCRIPTOR_HEAP_DESC description{
        .Type = kind,
        .NumDescriptors = count,
        .Flags = shader_visible ? D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE : D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        .NodeMask = 0,
    };
    base::check(device->CreateDescriptorHeap(&description, IID_PPV_ARGS(&heap)));
    cpu_start = heap->GetCPUDescriptorHandleForHeapStart();
    gpu_start = shader_visible ? heap->GetGPUDescriptorHandleForHeapStart() : D3D12_GPU_DESCRIPTOR_HANDLE{};
    increment = device->GetDescriptorHandleIncrementSize(kind);
}

D3D12_CPU_DESCRIPTOR_HANDLE DescriptorHeap::cpu(size_t index) const {
    return D3D12_CPU_DESCRIPTOR_HANDLE{
        .ptr = cpu_start.ptr + index * increment,
    };
}

D3D12_GPU_DESCRIPTOR_HANDLE DescriptorHeap::gpu(size_t index) const {
    return D3D12_GPU_DESCRIPTOR_HANDLE{
        .ptr = gpu_start.ptr + static_cast<std::uint64_t>(index * increment),
    };
}

void create_target_views(ID3D12Device* device, ID3D12Resource* target, const DescriptorHeap& heap, size_t first) {
    for (auto [index, format] : {std::pair{first, TargetFormat}, std::pair{first + 1, TargetSrgbFormat}}) {
        D3D12_RENDER_TARGET_VIEW_DESC description{
            .Format = format,
            .ViewDimension = D3D12_RTV_DIMENSION_TEXTURE2D,
        };
        device->CreateRenderTargetView(target, &description, heap.cpu(index));
    }
}

std::pair<ComPtr<ID3D12CommandAllocator>, ComPtr<ID3D12GraphicsCommandList>> command_list(ID3D12Device* device) {
    ComPtr<ID3D12CommandAllocator> allocator;
    base::check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)));
    ComPtr<ID3D12GraphicsCommandList> list;
    base::check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), nullptr, IID_PPV_ARGS(&list)));
    base::check(list->Close());
    return {std::move(allocator), std::move(list)};
}

void execute(ID3D12CommandQueue* queue, ID3D12GraphicsCommandList* list) {
    ID3D12CommandList* lists[] = {list};
    queue->ExecuteCommandLists(1, lists);
}

} // namespace graphics
