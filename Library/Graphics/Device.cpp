#include "Graphics/Device.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Graphics/Error.h"

#include <algorithm>
#include <cstdio>
#include <format>
#include <iterator>
#include <new>
#include <print>
#include <tuple>
#include <vector>

// NTSTATUS, which d3dkmthk.h needs before it.
#include <winternl.h>

#include <d3dkmthk.h>

#pragma comment(lib, "d3d12.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "gdi32.lib")

namespace graphics {

namespace {

D3DKMT_SCHEDULINGPRIORITYCLASS class_(GpuPriority priority) {
    switch (priority) {
    case GpuPriority::Idle:
        return D3DKMT_SCHEDULINGPRIORITYCLASS_IDLE;
    case GpuPriority::BelowNormal:
        return D3DKMT_SCHEDULINGPRIORITYCLASS_BELOW_NORMAL;
    case GpuPriority::Normal:
        return D3DKMT_SCHEDULINGPRIORITYCLASS_NORMAL;
    case GpuPriority::AboveNormal:
        return D3DKMT_SCHEDULINGPRIORITYCLASS_ABOVE_NORMAL;
    case GpuPriority::High:
        return D3DKMT_SCHEDULINGPRIORITYCLASS_HIGH;
    }
    base::unreachable();
}

// Throws if the NTSTATUS is a failure, as the HRESULT that it is.
void check_status(NTSTATUS status) {
    if (status < 0) {
        throw base::WindowsError(HRESULT_FROM_NT(status));
    }
}

// Whether the text is UTF-16 that a string can hold: without a surrogate
// that lacks its pair.
bool is_unicode(std::wstring_view text) {
    for (size_t index = 0; index < text.size(); index++) {
        wchar_t unit = text[index];
        if (unit >= 0xD800 && unit <= 0xDBFF) {
            if (index + 1 >= text.size() || text[index + 1] < 0xDC00 || text[index + 1] > 0xDFFF) {
                return false;
            }
            index++;
        } else if (unit >= 0xDC00 && unit <= 0xDFFF) {
            return false;
        }
    }
    return true;
}

// The environment variable's value, as std::env::var gives it in Rust:
// nothing if it's unset, or if its value isn't Unicode.
std::optional<std::string> environment_variable(const wchar_t* name) {
    std::wstring value(128, L'\0');
    while (true) {
        SetLastError(ERROR_SUCCESS);
        DWORD length = GetEnvironmentVariableW(name, value.data(), static_cast<DWORD>(value.size()));
        if (length == 0 && GetLastError() != ERROR_SUCCESS) {
            return std::nullopt;
        }
        if (length < value.size()) {
            value.resize(length);
            break;
        }
        // Too small: the length is what the value takes, with its end.
        value.resize(length);
    }
    if (!is_unicode(value)) {
        return std::nullopt;
    }
    return base::narrow(value);
}

#ifndef NDEBUG
// Turns on the D3D12 debug layer, if Windows' optional "Graphics Tools"
// feature is installed.
bool enable_debug_layer() {
    ComPtr<ID3D12Debug> debug;
    if (SUCCEEDED(D3D12GetDebugInterface(IID_PPV_ARGS(&debug))) && debug) {
        debug->EnableDebugLayer();
        return true;
    }
    return false;
}
#endif

bool drives_monitor(IDXGIAdapter1* adapter, HMONITOR monitor) {
    UINT index = 0;
    while (true) {
        ComPtr<IDXGIOutput> output;
        if (FAILED(adapter->EnumOutputs(index, &output))) {
            return false;
        }
        index += 1;
        if (!output) {
            return false;
        }
        DXGI_OUTPUT_DESC description{};
        if (SUCCEEDED(output->GetDesc(&description)) && description.Monitor == monitor) {
            return true;
        }
    }
}

std::pair<ComPtr<IDXGIAdapter1>, ComPtr<ID3D12Device>> create_device(IDXGIFactory6* factory, AdapterChoice choice) {
    std::vector<ComPtr<IDXGIAdapter1>> hardware;
    UINT index = 0;
    while (true) {
        ComPtr<IDXGIAdapter1> adapter;
        if (FAILED(factory->EnumAdapterByGpuPreference(index, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, IID_PPV_ARGS(&adapter)))) {
            break;
        }
        index += 1;
        DXGI_ADAPTER_DESC1 description{};
        if (FAILED(adapter->GetDesc1(&description)) || (description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) != 0) {
            continue;
        }
        hardware.push_back(std::move(adapter));
    }

    std::vector<ComPtr<IDXGIAdapter1>> candidates;
    base::match(
        choice,
        [&](const AdapterChoice::Monitor& monitor) {
            // The monitor's own GPU first, then the others.
            std::vector<ComPtr<IDXGIAdapter1>> outputs_monitor;
            std::vector<ComPtr<IDXGIAdapter1>> others;
            for (ComPtr<IDXGIAdapter1>& adapter : hardware) {
                if (drives_monitor(adapter.Get(), monitor.monitor)) {
                    outputs_monitor.push_back(std::move(adapter));
                } else {
                    others.push_back(std::move(adapter));
                }
            }
            std::ranges::move(outputs_monitor, std::back_inserter(candidates));
            std::ranges::move(others, std::back_inserter(candidates));
        },
        [&](const AdapterChoice::HighPerformance&) { std::ranges::move(hardware, std::back_inserter(candidates)); }, [&](const AdapterChoice::Warp&) {});
    ComPtr<IDXGIAdapter1> warp;
    base::check(factory->EnumWarpAdapter(IID_PPV_ARGS(&warp)));
    candidates.push_back(std::move(warp));

    // The last adapter's error, if any had one.
    std::optional<HRESULT> last_error;
    for (ComPtr<IDXGIAdapter1>& adapter : candidates) {
        ComPtr<ID3D12Device> device;
        HRESULT result = D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device));
        if (SUCCEEDED(result)) {
            if (device) {
                return {std::move(adapter), std::move(device)};
            }
        } else {
            last_error = result;
        }
    }
    if (last_error) {
        throw base::WindowsError(*last_error);
    }
    throw Error(E_FAIL, "no GPU supports Direct3D 12");
}

} // namespace

std::string describe_value(GpuPriority priority) {
    switch (priority) {
    case GpuPriority::Idle:
        return "Idle";
    case GpuPriority::BelowNormal:
        return "BelowNormal";
    case GpuPriority::Normal:
        return "Normal";
    case GpuPriority::AboveNormal:
        return "AboveNormal";
    case GpuPriority::High:
        return "High";
    }
    base::unreachable();
}

std::optional<GpuPriority> gpu_priority_from_name(std::string_view name) {
    if (name == "Idle") {
        return GpuPriority::Idle;
    }
    if (name == "BelowNormal") {
        return GpuPriority::BelowNormal;
    }
    if (name == "Normal") {
        return GpuPriority::Normal;
    }
    if (name == "AboveNormal") {
        return GpuPriority::AboveNormal;
    }
    if (name == "High") {
        return GpuPriority::High;
    }
    return std::nullopt;
}

GpuPriority gpu_priority_from_environment() {
    std::optional<std::string> value = environment_variable(L"KGAMES_GPU_PRIORITY");
    if (!value) {
        return GpuPriority::Normal;
    }
    std::optional<GpuPriority> priority = gpu_priority_from_name(*value);
    if (!priority) {
        throw Error(E_INVALIDARG, std::format("KGAMES_GPU_PRIORITY must be Idle, BelowNormal, Normal, AboveNormal or High, not {}", *value));
    }
    return *priority;
}

void set_gpu_priority(GpuPriority priority) {
    check_status(D3DKMTSetProcessSchedulingPriorityClass(GetCurrentProcess(), class_(priority)));
}

GpuPriority gpu_priority() {
    D3DKMT_SCHEDULINGPRIORITYCLASS current = D3DKMT_SCHEDULINGPRIORITYCLASS_NORMAL;
    check_status(D3DKMTGetProcessSchedulingPriorityClass(GetCurrentProcess(), &current));
    for (GpuPriority priority : {GpuPriority::Idle, GpuPriority::BelowNormal, GpuPriority::Normal, GpuPriority::AboveNormal, GpuPriority::High}) {
        if (class_(priority) == current) {
            return priority;
        }
    }
    return GpuPriority::Normal;
}

Gpu::Gpu(AdapterChoice choice) {
    // Before the device, so that the GPU's scheduler has the priority
    // for all of the process's devices.
    GpuPriority priority = gpu_priority_from_environment();
    if (priority != GpuPriority::Normal) {
        set_gpu_priority(priority);
    }
#ifndef NDEBUG
    bool debug = enable_debug_layer();
#else
    bool debug = false;
#endif
    UINT flags = debug ? DXGI_CREATE_FACTORY_DEBUG : 0;
    base::check(CreateDXGIFactory2(flags, IID_PPV_ARGS(&factory)));
    std::tie(adapter, device) = create_device(factory.Get(), choice);
    DXGI_ADAPTER_DESC1 description{};
    base::check(adapter->GetDesc1(&description));
    std::wstring_view wide_name(description.Description, std::size(description.Description));
    while (wide_name.ends_with(L'\0')) {
        wide_name.remove_suffix(1);
    }
    name = base::narrow(wide_name);

    D3D12_COMMAND_QUEUE_DESC queue_description{.Type = D3D12_COMMAND_LIST_TYPE_DIRECT};
    base::check(device->CreateCommandQueue(&queue_description, IID_PPV_ARGS(&queue)));
    base::check(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence_)));
    fence_event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (fence_event == nullptr) {
        throw base::last_error();
    }
    base::check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&setup_allocator)));
    base::check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, setup_allocator.Get(), nullptr, IID_PPV_ARGS(&setup_list)));
    base::check(setup_list->Close());

    BOOL tearing = FALSE;
    tearing_supported = SUCCEEDED(factory->CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING, &tearing, sizeof(BOOL))) && tearing != FALSE;

    if (debug) {
        // Without the queue if the device has none.
        device.As(&info_queue);
    }
}

Gpu::~Gpu() {
    // A GPU that was moved from holds nothing.
    if (fence_event == nullptr) {
        return;
    }
    // Nothing may still use what the GPU's objects hold.
    try {
        flush();
    } catch (const base::WindowsError&) {}
    CloseHandle(fence_event);
}

Gpu::Gpu(Gpu&& other) noexcept :
    factory(std::move(other.factory)),
    adapter(std::move(other.adapter)),
    name(std::move(other.name)),
    device(std::move(other.device)),
    queue(std::move(other.queue)),
    tearing_supported(other.tearing_supported),
    fence_(std::move(other.fence_)),
    fence_event(std::exchange(other.fence_event, nullptr)),
    last_signal(other.last_signal),
    setup_allocator(std::move(other.setup_allocator)),
    setup_list(std::move(other.setup_list)),
    info_queue(std::move(other.info_queue)) {}

Gpu& Gpu::operator=(Gpu&& other) noexcept {
    if (this != &other) {
        this->~Gpu();
        new (this) Gpu(std::move(other));
    }
    return *this;
}

std::array<VideoMemory, 2> Gpu::video_memory() const {
    ComPtr<IDXGIAdapter3> adapter3;
    base::check(adapter.As(&adapter3));
    std::array<VideoMemory, 2> segments{};
    size_t segment = 0;
    for (DXGI_MEMORY_SEGMENT_GROUP group : {DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL}) {
        DXGI_QUERY_VIDEO_MEMORY_INFO info{};
        base::check(adapter3->QueryVideoMemoryInfo(0, group, &info));
        segments[segment] = VideoMemory{
            .used = info.CurrentUsage,
            .budget = info.Budget,
        };
        segment++;
    }
    return segments;
}

ComPtr<ID3D12Fence> Gpu::fence() const {
    return fence_;
}

std::uint64_t Gpu::signal() {
    last_signal += 1;
    base::check(queue->Signal(fence_.Get(), last_signal));
    return last_signal;
}

std::uint64_t Gpu::completed() const {
    return fence_->GetCompletedValue();
}

bool Gpu::is_complete(std::uint64_t value) const {
    return completed() >= value;
}

void Gpu::wait_for(std::uint64_t value) const {
    if (is_complete(value)) {
        return;
    }
    base::check(fence_->SetEventOnCompletion(value, fence_event));
    WaitForSingleObject(fence_event, INFINITE);
}

void Gpu::flush() {
    std::uint64_t value = signal();
    wait_for(value);
}

size_t Gpu::report_messages() const {
    if (!info_queue) {
        return 0;
    }
    size_t errors = 0;
    UINT64 count = info_queue->GetNumStoredMessagesAllowedByRetrievalFilter();
    for (UINT64 index = 0; index < count; index++) {
        SIZE_T length = 0;
        if (FAILED(info_queue->GetMessage(index, nullptr, &length)) || length == 0) {
            continue;
        }
        // D3D12_MESSAGE is followed by its description in one allocation.
        std::vector<std::uint64_t> bytes((length + 7) / 8);
        D3D12_MESSAGE* message = reinterpret_cast<D3D12_MESSAGE*>(bytes.data());
        if (FAILED(info_queue->GetMessage(index, message, &length))) {
            continue;
        }
        std::string_view text(message->pDescription, base::saturating_sub<size_t>(message->DescriptionByteLength, 1));
        const char* severity = "message";
        switch (message->Severity) {
        case D3D12_MESSAGE_SEVERITY_CORRUPTION:
            severity = "corruption";
            break;
        case D3D12_MESSAGE_SEVERITY_ERROR:
            severity = "error";
            break;
        case D3D12_MESSAGE_SEVERITY_WARNING:
            severity = "warning";
            break;
        case D3D12_MESSAGE_SEVERITY_INFO:
            severity = "info";
            break;
        default:
            break;
        }
        if (message->Severity <= D3D12_MESSAGE_SEVERITY_ERROR) {
            errors += 1;
        }
        std::println(stderr, "D3D12 {}: {}", severity, text);
    }
    info_queue->ClearStoredMessages();
    return errors;
}

} // namespace graphics
