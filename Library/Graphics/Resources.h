// Buffers, textures, barriers and descriptor heaps.

#pragma once

#include "Base/Windows.h"

#include <cstddef>
#include <cstdint>
#include <d3d12.h>
#include <initializer_list>
#include <ranges>
#include <span>
#include <type_traits>
#include <utility>
#include <wrl/client.h>

namespace graphics {

using Microsoft::WRL::ComPtr;

D3D12_HEAP_PROPERTIES heap_properties(D3D12_HEAP_TYPE kind);

// A buffer of `size` bytes in a heap of the kind.
ComPtr<ID3D12Resource> create_buffer(ID3D12Device* device, std::uint64_t size, D3D12_HEAP_TYPE heap, D3D12_RESOURCE_STATES state);

// A 2D texture in GPU memory, with one mip level.
ComPtr<ID3D12Resource> create_texture(ID3D12Device* device, std::uint32_t width, std::uint32_t height, DXGI_FORMAT format, D3D12_RESOURCE_FLAGS flags, D3D12_RESOURCE_STATES state,
                                      const D3D12_CLEAR_VALUE* clear, std::uint32_t sample_count);

// A barrier that moves every subresource of the resource from one state to
// another. It borrows the resource, which must outlive it.
D3D12_RESOURCE_BARRIER transition(ID3D12Resource* resource, D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after);

D3D12_SHADER_BYTECODE bytecode(std::span<const std::uint8_t> shader);

// A buffer in an upload heap that stays mapped, which the CPU writes and the
// GPU reads across PCIe. The writer must keep the GPU from reading what it
// overwrites, like by waiting for the frame that read it.
class UploadBuffer {
public:
    ComPtr<ID3D12Resource> resource;
    size_t size;

    UploadBuffer(ID3D12Device* device, size_t size);
    UploadBuffer(const UploadBuffer&) = delete;
    UploadBuffer& operator=(const UploadBuffer&) = delete;
    UploadBuffer(UploadBuffer&&) = default;
    UploadBuffer& operator=(UploadBuffer&&) = default;

    // Copies the values to `offset` bytes into the buffer: a span, a vector,
    // an array, or anything else whose values are one after another.
    template <std::ranges::contiguous_range Values>
        requires std::is_trivially_copyable_v<std::ranges::range_value_t<Values>>
    void write(size_t offset, const Values& values) const {
        write_bytes(offset, std::ranges::data(values), std::ranges::size(values) * sizeof(std::ranges::range_value_t<Values>));
    }

    // The same, for values in braces, like write(0, {constants}).
    template <class T>
        requires std::is_trivially_copyable_v<T>
    void write(size_t offset, std::initializer_list<T> values) const {
        write_bytes(offset, values.begin(), values.size() * sizeof(T));
    }

    std::span<std::uint8_t> bytes_mut();

    D3D12_GPU_VIRTUAL_ADDRESS gpu_address(size_t offset) const;

private:
    // Copies the bytes to `offset` bytes into the buffer.
    void write_bytes(size_t offset, const void* values, size_t bytes) const;

    std::uint8_t* mapped;
};

// A descriptor heap of one kind, with its descriptors' handles.
class DescriptorHeap {
public:
    ComPtr<ID3D12DescriptorHeap> heap;

    DescriptorHeap(ID3D12Device* device, D3D12_DESCRIPTOR_HEAP_TYPE kind, std::uint32_t count, bool shader_visible);
    DescriptorHeap(const DescriptorHeap&) = delete;
    DescriptorHeap& operator=(const DescriptorHeap&) = delete;
    DescriptorHeap(DescriptorHeap&&) = default;
    DescriptorHeap& operator=(DescriptorHeap&&) = default;

    D3D12_CPU_DESCRIPTOR_HANDLE cpu(size_t index) const;

    D3D12_GPU_DESCRIPTOR_HANDLE gpu(size_t index) const;

private:
    D3D12_CPU_DESCRIPTOR_HANDLE cpu_start;
    D3D12_GPU_DESCRIPTOR_HANDLE gpu_start;
    size_t increment;
};

// Creates the render target views of a target: one that stores colors as
// they are, and one that encodes linear colors to sRGB.
void create_target_views(ID3D12Device* device, ID3D12Resource* target, const DescriptorHeap& heap, size_t first);

std::pair<ComPtr<ID3D12CommandAllocator>, ComPtr<ID3D12GraphicsCommandList>> command_list(ID3D12Device* device);

void execute(ID3D12CommandQueue* queue, ID3D12GraphicsCommandList* list);

} // namespace graphics
