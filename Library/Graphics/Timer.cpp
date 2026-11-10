#include "Graphics/Timer.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Graphics/Resources.h"

#include <algorithm>
#include <cstring>

namespace graphics {

GpuTimer::GpuTimer(const Gpu& gpu, std::uint32_t spans) : spans(std::max(spans, 1u)) {
    D3D12_QUERY_HEAP_DESC description{
        .Type = D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
        .Count = this->spans * 2,
        .NodeMask = 0,
    };
    base::check(gpu.device->CreateQueryHeap(&description, IID_PPV_ARGS(&heap)));
    if (!heap) {
        // The call succeeded without making the heap, so the error has no
        // code of its own.
        throw base::WindowsError(S_OK);
    }
    readback = create_buffer(gpu.device.Get(), static_cast<std::uint64_t>(this->spans) * 2 * sizeof(std::uint64_t), D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
    base::check(gpu.queue->GetTimestampFrequency(&frequency));
    ended.assign(this->spans, false);
}

Span GpuTimer::begin(const Frame& frame) {
    if (!(begun < spans)) {
        base::panic("more spans than the timer was made for");
    }
    Span span(begun);
    begun += 1;
    frame.list->EndQuery(heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, span.index * 2);
    return span;
}

void GpuTimer::end(const Frame& frame, Span span) {
    if (!(span.index < begun)) {
        base::panic("a span that this frame didn't begin");
    }
    if (ended[span.index]) {
        base::panic("a span ended twice");
    }
    ended[span.index] = true;
    frame.list->EndQuery(heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, span.index * 2 + 1);
}

void GpuTimer::resolve(const Frame& frame) const {
    if (begun == 0) {
        return;
    }
    if (!std::all_of(ended.begin(), ended.begin() + begun, [](bool span_ended) { return span_ended; })) {
        base::panic("a span was begun and never ended, so its end can't be resolved");
    }
    frame.list->ResolveQueryData(heap.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 0, begun * 2, readback.Get(), 0);
}

std::vector<double> GpuTimer::read(const Gpu& gpu, std::uint64_t fence) {
    gpu.wait_for(fence);
    size_t count = begun;
    begun = 0;
    std::fill(ended.begin(), ended.end(), false);
    if (count == 0) {
        return {};
    }
    std::vector<std::uint64_t> stamps(count * 2, 0);
    void* mapped = nullptr;
    D3D12_RANGE range{
        .Begin = 0,
        .End = stamps.size() * sizeof(std::uint64_t),
    };
    base::check(readback->Map(0, &range, &mapped));
    std::memcpy(stamps.data(), mapped, stamps.size() * sizeof(std::uint64_t));
    D3D12_RANGE written{.Begin = 0, .End = 0};
    readback->Unmap(0, &written);
    std::vector<double> lengths;
    lengths.reserve(count);
    for (size_t first = 0; first < stamps.size(); first += 2) {
        lengths.push_back(static_cast<double>(base::saturating_sub(stamps[first + 1], stamps[first])) * 1000.0 / static_cast<double>(frequency));
    }
    return lengths;
}

} // namespace graphics
