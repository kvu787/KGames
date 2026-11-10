// The process's GPU scheduling priority, which RunLightly lowers through the
// environment, so that a game the user is playing keeps its frames while
// coding agents' tests draw.

#include "Graphics/Graphics.h"
#include "Testing/Testing.h"

#include <exception>
#include <mutex>
#include <optional>
#include <string>
#include <string_view>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::gpu_priority;
using graphics::gpu_priority_from_name;
using graphics::GpuPriority;
using graphics::set_gpu_priority;

namespace {

const wchar_t* const Variable = L"KGAMES_GPU_PRIORITY";

// The variable's value in the process's environment, or nothing if it's unset.
std::optional<std::wstring> variable() {
    std::wstring value(64, L'\0');
    while (true) {
        SetLastError(ERROR_SUCCESS);
        DWORD length = GetEnvironmentVariableW(Variable, value.data(), static_cast<DWORD>(value.size()));
        if (length == 0 && GetLastError() != ERROR_SUCCESS) {
            return std::nullopt;
        }
        if (length < value.size()) {
            value.resize(length);
            return value;
        }
        value.resize(length);
    }
}

// The priority and the environment are the process's, so these tests run
// one at a time. Every test of this project is in the one process, so each
// of these also leaves the environment as it found it, for the tests after
// it to draw at the priority that RunLightly asked for.
class Process {
public:
    Process() : lock(mutex()), found(variable()) {}

    ~Process() {
        SetEnvironmentVariableW(Variable, found ? found->c_str() : nullptr);
    }

    Process(const Process&) = delete;
    Process& operator=(const Process&) = delete;

private:
    static std::mutex& mutex() {
        static std::mutex process;
        return process;
    }

    std::lock_guard<std::mutex> lock;
    std::optional<std::wstring> found;
};

} // namespace

TEST(names_are_the_scheduler_s_classes) {
    CHECK_EQ(gpu_priority_from_name("Idle"), std::optional(GpuPriority::Idle));
    CHECK_EQ(gpu_priority_from_name("BelowNormal"), std::optional(GpuPriority::BelowNormal));
    CHECK_EQ(gpu_priority_from_name("Normal"), std::optional(GpuPriority::Normal));
    CHECK_EQ(gpu_priority_from_name("AboveNormal"), std::optional(GpuPriority::AboveNormal));
    CHECK_EQ(gpu_priority_from_name("High"), std::optional(GpuPriority::High));
    CHECK_EQ(gpu_priority_from_name("idle"), std::nullopt);
    CHECK_EQ(gpu_priority_from_name(""), std::nullopt);
}

TEST(the_process_s_priority_can_be_lowered_and_put_back) {
    Process process;
    set_gpu_priority(GpuPriority::Idle);
    CHECK_EQ(gpu_priority(), GpuPriority::Idle);
    set_gpu_priority(GpuPriority::BelowNormal);
    CHECK_EQ(gpu_priority(), GpuPriority::BelowNormal);
    set_gpu_priority(GpuPriority::Normal);
    CHECK_EQ(gpu_priority(), GpuPriority::Normal);
}

TEST(a_device_takes_the_environment_s_priority) {
    Process process;
    SetEnvironmentVariableW(Variable, L"Idle");
    std::optional<Gpu> gpu;
    std::exception_ptr error;
    try {
        gpu.emplace(AdapterChoice::HighPerformance{});
    } catch (...) {
        error = std::current_exception();
    }
    SetEnvironmentVariableW(Variable, nullptr);
    if (error) {
        std::rethrow_exception(error);
    }
    gpu.reset();
    CHECK_EQ(gpu_priority(), GpuPriority::Idle);
    set_gpu_priority(GpuPriority::Normal);
}

TEST(without_the_environment_a_device_leaves_the_priority_alone) {
    Process process;
    SetEnvironmentVariableW(Variable, nullptr);
    set_gpu_priority(GpuPriority::BelowNormal);
    {
        Gpu gpu(AdapterChoice::HighPerformance{});
        CHECK_EQ(gpu_priority(), GpuPriority::BelowNormal);
    }
    set_gpu_priority(GpuPriority::Normal);
}

TEST(a_wrong_value_in_the_environment_is_an_error) {
    Process process;
    SetEnvironmentVariableW(Variable, L"Lowest");
    std::optional<std::string> error;
    try {
        Gpu gpu(AdapterChoice::HighPerformance{});
    } catch (const base::WindowsError& failure) {
        error = failure.what();
    }
    SetEnvironmentVariableW(Variable, nullptr);
    if (!error) {
        testing::fail("the device shouldn't be made");
    }
    CHECK(std::string_view(*error).contains("KGAMES_GPU_PRIORITY must be"), "{}", *error);
}
