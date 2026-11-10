#include "GameInput/GameInput.h"

#include "Base/Files.h"
#include "Base/Text.h"

#include <format>
#include <mutex>
#include <vector>

namespace game_input {

// The API version of the GameInput.h on the include path, and the sizes that
// the v4 header gives these structs, as MSVC compiles it for x64, so that
// another version's header fails the build.
static_assert(GAMEINPUT_API_VERSION == 4);
static_assert(sizeof(GameInputKeyState) == 12);
static_assert(sizeof(GameInputMouseState) == 56);
static_assert(sizeof(GameInputGamepadState) == 28);
static_assert(sizeof(GameInputVersion) == 8);
static_assert(sizeof(GameInputUsage) == 4);
static_assert(sizeof(GameInputKeyboardInfo) == 28);
static_assert(sizeof(GameInputMouseInfo) == 12);
static_assert(sizeof(GameInputGamepadInfo) == 116);
static_assert(sizeof(GameInputRumbleParams) == 16);

namespace {

// What the system says a failed call's HRESULT means, and the HRESULT, like
// "The parameter is incorrect. (0x80070057)".
std::string describe(HRESULT code) {
    wchar_t* buffer = nullptr;
    DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr, static_cast<DWORD>(code), 0,
                                  reinterpret_cast<wchar_t*>(&buffer), 0, nullptr);
    std::string message;
    if (buffer != nullptr) {
        message = base::trim(base::narrow(std::wstring_view(buffer, length)));
        LocalFree(buffer);
    }
    return std::format("{} (0x{:08X})", message, static_cast<unsigned long>(code));
}

// The runtime whose code an object of GameInput's is: the DLL that holds its
// vtable, which is the first thing in a COM object, and that DLL's version.
std::expected<Runtime, std::string> runtime_of(IUnknown* object) {
    const void* vtable = *reinterpret_cast<const void* const*>(object);
    HMODULE module_ = nullptr;
    DWORD flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
    if (!GetModuleHandleExW(flags, static_cast<const wchar_t*>(vtable), &module_)) {
        return std::unexpected(std::format("Couldn't find GameInput's runtime: {}", describe(HRESULT_FROM_WIN32(GetLastError()))));
    }
    std::vector<wchar_t> name(32768);
    size_t length = GetModuleFileNameW(module_, name.data(), static_cast<DWORD>(name.size()));
    if (length == 0 || length == name.size()) {
        return std::unexpected(std::format("Couldn't find GameInput's runtime: {}", describe(HRESULT_FROM_WIN32(GetLastError()))));
    }
    std::filesystem::path path(std::wstring_view(name.data(), length));
    std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
    if (!bytes) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(path), bytes.error()));
    }
    std::optional<std::array<std::uint16_t, 4>> version = installation::version_in(*bytes);
    if (!version) {
        return std::unexpected(std::format("{} has no version", base::display(path)));
    }
    return Runtime{std::move(path), *version};
}

} // namespace

// This calls GameInput.lib's loader, GameInputInitialize, from the
// Microsoft.GameInput NuGet package, whose source is native/src/GameInput.cpp
// there. Program\Build\Build.ps1 restores the package, and GameInput.vcxproj
// takes the library into this one. It's what the header's inline
// GameInputCreate calls, with the IID of the header's IGameInput.
std::expected<std::pair<Microsoft::WRL::ComPtr<IGameInput>, Runtime>, std::string> create() {
    static std::once_flag kept;
    static std::mutex one_at_a_time;
    std::expected<std::filesystem::path, std::string> installed = installation::installed_runtime();
    if (!installed) {
        return std::unexpected(std::move(installed.error()));
    }
    Microsoft::WRL::ComPtr<IGameInput> game_input;
    {
        std::lock_guard turn(one_at_a_time);
        HRESULT result = GameInputInitialize(IID_IGameInput, reinterpret_cast<void**>(game_input.GetAddressOf()));
        if (FAILED(result)) {
            return std::unexpected(std::format("Couldn't start GameInput: {}", describe(result)));
        }
        // A success without an IGameInput is an error without a code.
        if (game_input == nullptr) {
            return std::unexpected(std::format("Couldn't start GameInput: {}", describe(S_OK)));
        }
    }
    // The first one is kept with a reference that nothing releases. GameInput's
    // objects are free-threaded, so whichever thread made it, every thread can
    // go on using the runtime.
    std::call_once(kept, [&] { game_input->AddRef(); });
    std::expected<Runtime, std::string> runtime = runtime_of(game_input.Get());
    if (!runtime) {
        return std::unexpected(std::move(runtime.error()));
    }
    return std::pair(std::move(game_input), std::move(*runtime));
}

} // namespace game_input
