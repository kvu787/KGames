// SimplePaint's production vertex and pixel shaders on the GPU, against the
// independent binary64 reference: every case draws one pixel, through the
// real root signature and material array, on the preferred hardware adapter
// and on WARP, which works on machines without a GPU. It fails on D3D12
// debug-layer warnings or errors when the debug layer is installed.
//
// With --benchmark, it instead times 128 warmed 1024 x 1024 draws for Shift 0
// and Shift 0.6 on the hardware adapter, with GPU timestamps.

#include "Base/Text.h"
#include "Base/Windows.h"
#include "Graphics/Graphics.h"
#include "Math/Math.h"
#include "SimplePaint/SimplePaint.h"
#include "Testing/Testing.h"

#include "Reference.h"

// The shaders, which SimplePaintTests.vcxproj names: SimplePaintVertex and
// SimplePaintPixel.
#include "SimplePaintPixel.h"
#include "SimplePaintVertex.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cwchar>
#include <d3d12.h>
#include <d3d12sdklayers.h>
#include <dxgi1_6.h>
#include <iterator>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <vector>
#include <wrl/client.h>

#pragma comment(lib, "d3d12.lib")
#pragma comment(lib, "dxgi.lib")

using math::Matrix;
using Microsoft::WRL::ComPtr;
using reference::Case;
using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint::Vertex;
using simple_paint::direct3d12::Pipeline;
using simple_paint::direct3d12::Target;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;
using simple_paint::orthographic::ObjectTransforms;

namespace {

// The shaders' SIMPLE_PAINT_MATERIAL_COUNT, from SimplePaintTests.vcxproj.
constexpr size_t MaterialCount = 6;
constexpr DXGI_FORMAT Format = DXGI_FORMAT_R32G32B32A32_FLOAT;
constexpr std::uint32_t BenchmarkRepetitions = 128;

// One case's constant buffers: the object's transforms at b0, then the
// materials at b1, each at a 256-byte boundary.
struct alignas(256) Constants {
    ObjectTransforms object;
    std::array<std::uint8_t, 160> object_padding;
    std::array<GpuMaterial, MaterialCount> materials;
    std::array<std::uint8_t, 32> tail_padding;
};

static_assert(sizeof(Constants) == 768);

enum class Adapter {
    Hardware,
    Warp,
};

// The timestamps' heap, and the buffer that they're read back from.
struct Timestamps {
    ComPtr<ID3D12QueryHeap> heap;
    ComPtr<ID3D12Resource> readback;
};

void run(Adapter choice, bool benchmark) {
    ComPtr<IDXGIFactory6> factory;
    base::check(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory)));
    ComPtr<IDXGIAdapter1> adapter;
    switch (choice) {
    case Adapter::Hardware:
        base::check(factory->EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, IID_PPV_ARGS(&adapter)));
        break;
    case Adapter::Warp: {
        ComPtr<IDXGIFactory4> factory4;
        base::check(factory.As(&factory4));
        base::check(factory4->EnumWarpAdapter(IID_PPV_ARGS(&adapter)));
        break;
    }
    }
    DXGI_ADAPTER_DESC1 description{};
    base::check(adapter->GetDesc1(&description));
    std::string name = base::narrow(std::wstring_view(description.Description, wcsnlen(description.Description, std::size(description.Description))));
    ComPtr<ID3D12Device> device;
    base::check(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)));
    // Null without the debug layer.
    ComPtr<ID3D12InfoQueue> info;
    device.As(&info);

    Pipeline pipeline(device.Get(), SimplePaintVertex, SimplePaintPixel,
                      Target{
                          .format = Format,
                          .depth_format = std::nullopt,
                          .sample_count = 1,
                          .cull_mode = D3D12_CULL_MODE_NONE,
                      });

    std::vector<Case> cases;
    if (benchmark) {
        reference::Normal normal{0.6f, 0.2f, 0.8f};
        cases = {
            Case{
                .parameters = Parameters{},
                .normal = normal,
            },
            Case{
                .parameters = Parameters{.shift = 0.6},
                .normal = normal,
            },
        };
    } else {
        cases = reference::cases();
    }
    ObjectTransforms object = build_object_transforms(Matrix::Identity, make_projection(2.0f, 2.0f, 0.0f, 1.0f).value()).value();
    std::vector<Constants> constants;
    constants.reserve(cases.size());
    std::vector<Vertex> vertices;
    vertices.reserve(cases.size() * 3);
    size_t count = cases.size();
    for (size_t i = 0; i < count; i++) {
        Case& case_ = cases[i];
        size_t index = i % MaterialCount;
        Constants record{
            .object = object,
            .object_padding = {},
            .materials = {},
            .tail_padding = {},
        };
        record.materials[index] = Material::compile(case_.parameters).value().constants();
        constants.push_back(record);
        // A triangle that covers the pixel, whose center has barycentrics
        // (1/2, 1/4, 1/4).
        reference::Normal n = case_.normal;
        std::array<Vertex, 3> triangle{};
        std::array<std::array<float, 2>, 3> corners{{{-1.0f, -1.0f}, {3.0f, -1.0f}, {-1.0f, 3.0f}}};
        for (size_t corner = 0; corner < 3; corner++) {
            triangle[corner] = Vertex{
                .position = {corners[corner][0], corners[corner][1], 0.0f},
                .normal = n,
                .material_index = static_cast<std::uint32_t>(index),
            };
        }
        if (!benchmark && i >= count - 4096) {
            // Distinct vertex normals exercise interpolation and the
            // vertex stage's rotation together.
            float delta = 0.125f * std::sqrt(n[0] * n[0] + n[1] * n[1] + n[2] * n[2]);
            triangle[0].normal[0] += delta;
            triangle[1].normal[0] -= 2.0f * delta;
            triangle[0].normal[1] -= delta;
            triangle[2].normal[1] += 2.0f * delta;
            auto center = [&](size_t axis) {
                return static_cast<float>(0.5 * static_cast<double>(triangle[0].normal[axis]) + 0.25 * static_cast<double>(triangle[1].normal[axis]) +
                                          0.25 * static_cast<double>(triangle[2].normal[axis]));
            };
            case_.normal = {center(0), center(1), n[2]};
        }
        vertices.append_range(triangle);
    }
    graphics::UploadBuffer constant_buffer(device.Get(), constants.size() * sizeof(Constants));
    constant_buffer.write(0, constants);
    graphics::UploadBuffer vertex_buffer(device.Get(), vertices.size() * sizeof(Vertex));
    vertex_buffer.write(0, vertices);

    std::uint32_t width = benchmark ? 1024 : 256;
    std::uint32_t height = benchmark ? 1024 : static_cast<std::uint32_t>((count + 255) / 256);
    ComPtr<ID3D12Resource> target =
        graphics::create_texture(device.Get(), width, height, Format, D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_STATE_RENDER_TARGET, nullptr, 1);
    D3D12_DESCRIPTOR_HEAP_DESC rtv_heap_description{
        .Type = D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        .NumDescriptors = 1,
    };
    ComPtr<ID3D12DescriptorHeap> rtv_heap;
    base::check(device->CreateDescriptorHeap(&rtv_heap_description, IID_PPV_ARGS(&rtv_heap)));
    D3D12_CPU_DESCRIPTOR_HANDLE rtv = rtv_heap->GetCPUDescriptorHandleForHeapStart();
    device->CreateRenderTargetView(target.Get(), nullptr, rtv);
    D3D12_RESOURCE_DESC target_description = target->GetDesc();
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    std::uint64_t read_size = 0;
    device->GetCopyableFootprints(&target_description, 0, 1, 0, &footprint, nullptr, nullptr, &read_size);
    ComPtr<ID3D12Resource> readback = graphics::create_buffer(device.Get(), read_size, D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);

    D3D12_COMMAND_QUEUE_DESC queue_description{
        .Type = D3D12_COMMAND_LIST_TYPE_DIRECT,
    };
    ComPtr<ID3D12CommandQueue> queue;
    base::check(device->CreateCommandQueue(&queue_description, IID_PPV_ARGS(&queue)));
    ComPtr<ID3D12CommandAllocator> allocator;
    base::check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)));
    ComPtr<ID3D12GraphicsCommandList> list;
    base::check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), nullptr, IID_PPV_ARGS(&list)));
    std::optional<Timestamps> timestamps;
    if (benchmark) {
        D3D12_QUERY_HEAP_DESC heap_description{
            .Type = D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
            .Count = 4,
            .NodeMask = 0,
        };
        ComPtr<ID3D12QueryHeap> heap;
        base::check(device->CreateQueryHeap(&heap_description, IID_PPV_ARGS(&heap)));
        ComPtr<ID3D12Resource> timestamp_readback = graphics::create_buffer(device.Get(), 4 * sizeof(std::uint64_t), D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
        timestamps = Timestamps{heap, timestamp_readback};
    }

    pipeline.bind(list.Get());
    list->OMSetRenderTargets(1, &rtv, FALSE, nullptr);
    D3D12_VERTEX_BUFFER_VIEW vertex_view{
        .BufferLocation = vertex_buffer.gpu_address(0),
        .SizeInBytes = static_cast<UINT>(vertices.size() * sizeof(Vertex)),
        .StrideInBytes = static_cast<UINT>(sizeof(Vertex)),
    };
    list->IASetVertexBuffers(0, 1, &vertex_view);
    for (size_t i = 0; i < count; i++) {
        std::int32_t x = static_cast<std::int32_t>(i % width);
        std::int32_t y = static_cast<std::int32_t>(i / width);
        std::array<float, 4> viewport =
            benchmark ? std::array{0.0f, 0.0f, static_cast<float>(width), static_cast<float>(height)} : std::array{static_cast<float>(x), static_cast<float>(y), 1.0f, 1.0f};
        std::array<std::int32_t, 4> scissor = benchmark ? std::array{0, 0, static_cast<std::int32_t>(width), static_cast<std::int32_t>(height)} : std::array{x, y, x + 1, y + 1};
        D3D12_VIEWPORT viewports[] = {{
            .TopLeftX = viewport[0],
            .TopLeftY = viewport[1],
            .Width = viewport[2],
            .Height = viewport[3],
            .MinDepth = 0.0f,
            .MaxDepth = 1.0f,
        }};
        list->RSSetViewports(1, viewports);
        D3D12_RECT scissors[] = {{
            .left = scissor[0],
            .top = scissor[1],
            .right = scissor[2],
            .bottom = scissor[3],
        }};
        list->RSSetScissorRects(1, scissors);
        size_t offset = i * sizeof(Constants);
        list->SetGraphicsRootConstantBufferView(0, constant_buffer.gpu_address(offset));
        list->SetGraphicsRootConstantBufferView(1, constant_buffer.gpu_address(offset + 256));
        std::uint32_t first_vertex = static_cast<std::uint32_t>(i * 3);
        if (timestamps) {
            // Warm each path, then time production draws on the GPU,
            // without CPU submission, setup, transfers or readback.
            for (int warm = 0; warm < 32; warm++) {
                list->DrawInstanced(3, 1, first_vertex, 0);
            }
            list->EndQuery(timestamps->heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, static_cast<UINT>(i * 2));
            for (std::uint32_t repetition = 0; repetition < BenchmarkRepetitions; repetition++) {
                list->DrawInstanced(3, 1, first_vertex, 0);
            }
            list->EndQuery(timestamps->heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, static_cast<UINT>(i * 2 + 1));
        } else {
            list->DrawInstanced(3, 1, first_vertex, 0);
        }
    }
    if (timestamps) {
        list->ResolveQueryData(timestamps->heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 0, 4, timestamps->readback.Get(), 0);
    }
    D3D12_RESOURCE_BARRIER barrier = graphics::transition(target.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE);
    list->ResourceBarrier(1, &barrier);
    D3D12_TEXTURE_COPY_LOCATION destination{
        .pResource = readback.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
        .PlacedFootprint = footprint,
    };
    D3D12_TEXTURE_COPY_LOCATION source{
        .pResource = target.Get(),
        .Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        .SubresourceIndex = 0,
    };
    list->CopyTextureRegion(&destination, 0, 0, 0, &source, nullptr);
    base::check(list->Close());
    ID3D12CommandList* lists[] = {list.Get()};
    queue->ExecuteCommandLists(1, lists);
    ComPtr<ID3D12Fence> fence;
    base::check(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)));
    base::check(queue->Signal(fence.Get(), 1));
    HANDLE event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (event == nullptr) {
        throw base::last_error();
    }
    base::check(fence->SetEventOnCompletion(1, event));
    DWORD wait = WaitForSingleObject(event, 60'000);
    CloseHandle(event);
    CHECK(wait == WAIT_OBJECT_0, "GPU test fence timeout");

    if (timestamps) {
        void* mapped = nullptr;
        D3D12_RANGE read{.Begin = 0, .End = 4 * sizeof(std::uint64_t)};
        base::check(timestamps->readback->Map(0, &read, &mapped));
        const std::uint64_t* ticks = static_cast<const std::uint64_t*>(mapped);
        std::uint64_t frequency = 0;
        base::check(queue->GetTimestampFrequency(&frequency));
        size_t i = 0;
        for (std::string_view label : {"Zero shift", "Shift 0.6"}) {
            double milliseconds = 1000.0 * static_cast<double>(ticks[i * 2 + 1] - ticks[i * 2]) / static_cast<double>(frequency) / static_cast<double>(BenchmarkRepetitions);
            std::println("{}: {}: {} ms per 1024x1024 draw ({} draws, constant normal, float4 render target).", name, label, base::display(milliseconds), BenchmarkRepetitions);
            i += 1;
        }
        D3D12_RANGE written{.Begin = 0, .End = 0};
        timestamps->readback->Unmap(0, &written);
        return;
    }

    void* mapped = nullptr;
    D3D12_RANGE read{.Begin = 0, .End = static_cast<SIZE_T>(read_size)};
    base::check(readback->Map(0, &read, &mapped));
    double max_linear = 0.0;
    double max_srgb = 0.0;
    for (size_t i = 0; i < cases.size(); i++) {
        const Case& case_ = cases[i];
        size_t row = static_cast<size_t>(footprint.Offset) + (i / width) * static_cast<size_t>(footprint.Footprint.RowPitch);
        const float* pixel = reinterpret_cast<const float*>(static_cast<const std::uint8_t*>(mapped) + row + (i % width) * 16);
        std::array<double, 3> expected = reference::reference(case_);
        CHECK(pixel[3] == 1.0f, "Production pixel shader alpha mismatch or missing draw");
        for (size_t channel = 0; channel < 3; channel++) {
            float value = pixel[channel];
            CHECK(std::isfinite(value) && value >= 0.0f && value <= 1.0f, "GPU output is not finite in [0,1]: case {}, channel {}, value {}", i, channel, base::display(value));
            double linear_error = std::abs(static_cast<double>(value) - expected[channel]);
            double srgb_error = std::abs(reference::srgb(static_cast<double>(value)) - reference::srgb(expected[channel]));
            max_linear = std::fmax(max_linear, linear_error);
            max_srgb = std::fmax(max_srgb, srgb_error);
            CHECK(linear_error <= 0.001 && srgb_error <= 0.001, "GPU reference mismatch at case {}, linear error={}, sRGB error={}", i, base::display(linear_error),
                  base::display(srgb_error));
        }
    }
    D3D12_RANGE written{.Begin = 0, .End = 0};
    readback->Unmap(0, &written);
    if (info) {
        // GetMessage's index runs to the count that the retrieval filter
        // allows.
        UINT64 stored = info->GetNumStoredMessagesAllowedByRetrievalFilter();
        for (UINT64 i = 0; i < stored; i++) {
            SIZE_T size = 0;
            base::check(info->GetMessage(i, nullptr, &size));
            // 64-bit numbers, so the message is aligned for its pointer fields.
            std::vector<std::uint64_t> storage((size + 7) / 8);
            D3D12_MESSAGE* message = reinterpret_cast<D3D12_MESSAGE*>(storage.data());
            base::check(info->GetMessage(i, message, &size));
            CHECK(message->Severity > D3D12_MESSAGE_SEVERITY_WARNING, "DX12 validation: {}", message->pDescription);
        }
    }
    std::println("{}: {} production VS/PS cases passed; max linear error={}, max sRGB error={}; debug layer={}", name, count, base::display(max_linear), base::display(max_srgb),
                 info ? "checked" : "unavailable");
}

} // namespace

TEST(gpu) {
    bool benchmark = std::ranges::contains(base::arguments(), std::string_view("--benchmark"));
    ComPtr<ID3D12Debug> debug;
    if (SUCCEEDED(D3D12GetDebugInterface(IID_PPV_ARGS(&debug)))) {
        debug->EnableDebugLayer();
    }
    if (benchmark) {
        run(Adapter::Hardware, true);
    } else {
        run(Adapter::Hardware, false);
        run(Adapter::Warp, false);
    }
}
