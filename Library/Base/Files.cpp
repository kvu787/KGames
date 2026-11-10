#include "Base/Files.h"

#include "Base/Text.h"
#include "Base/Windows.h"

#include <format>

namespace base {

std::filesystem::path path(std::string_view text) {
    return std::filesystem::path(widen(text));
}

std::string display(const std::filesystem::path& path) {
    return narrow(path.native());
}

std::string os_error(unsigned long code) {
    wchar_t* buffer = nullptr;
    DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr, code, 0,
                                  reinterpret_cast<wchar_t*>(&buffer), 0, nullptr);
    std::string text;
    if (buffer != nullptr) {
        text = trim(narrow(std::wstring_view(buffer, length)));
        LocalFree(buffer);
    }
    return std::format("{} (os error {})", text, code);
}

namespace {

// Closes the file when it goes out of scope.
struct File {
    HANDLE handle;

    ~File() {
        if (handle != INVALID_HANDLE_VALUE) {
            CloseHandle(handle);
        }
    }
};

} // namespace

std::expected<std::vector<std::uint8_t>, std::string> read(const std::filesystem::path& path) {
    File file{CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)};
    if (file.handle == INVALID_HANDLE_VALUE) {
        return std::unexpected(os_error(GetLastError()));
    }
    LARGE_INTEGER size{};
    if (!GetFileSizeEx(file.handle, &size)) {
        return std::unexpected(os_error(GetLastError()));
    }
    std::vector<std::uint8_t> bytes(static_cast<size_t>(size.QuadPart));
    size_t done = 0;
    while (done < bytes.size()) {
        DWORD wanted = static_cast<DWORD>(std::min<size_t>(bytes.size() - done, 1u << 30));
        DWORD got = 0;
        if (!ReadFile(file.handle, bytes.data() + done, wanted, &got, nullptr)) {
            return std::unexpected(os_error(GetLastError()));
        }
        if (got == 0) {
            bytes.resize(done);
            break;
        }
        done += got;
    }
    return bytes;
}

std::expected<std::string, std::string> read_to_string(const std::filesystem::path& path) {
    auto bytes = read(path);
    if (!bytes) {
        return std::unexpected(bytes.error());
    }
    std::string text(reinterpret_cast<const char*>(bytes->data()), bytes->size());
    if (!is_utf8(text)) {
        return std::unexpected(std::string("stream did not contain valid UTF-8"));
    }
    return text;
}

std::expected<void, std::string> write(const std::filesystem::path& path, std::span<const std::uint8_t> bytes) {
    File file{CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr)};
    if (file.handle == INVALID_HANDLE_VALUE) {
        return std::unexpected(os_error(GetLastError()));
    }
    size_t done = 0;
    while (done < bytes.size()) {
        DWORD wanted = static_cast<DWORD>(std::min<size_t>(bytes.size() - done, 1u << 30));
        DWORD wrote = 0;
        if (!WriteFile(file.handle, bytes.data() + done, wanted, &wrote, nullptr)) {
            return std::unexpected(os_error(GetLastError()));
        }
        done += wrote;
    }
    return {};
}

std::expected<void, std::string> write(const std::filesystem::path& path, std::string_view text) {
    return write(path, std::span(reinterpret_cast<const std::uint8_t*>(text.data()), text.size()));
}

std::filesystem::path current_exe() {
    std::wstring path(MAX_PATH, L'\0');
    while (true) {
        DWORD length = GetModuleFileNameW(nullptr, path.data(), static_cast<DWORD>(path.size()));
        if (length < path.size()) {
            path.resize(length);
            return std::filesystem::path(path);
        }
        // It didn't fit.
        path.resize(path.size() * 2);
    }
}

bool is_not_found(std::string_view error) {
    // The codes that Rust's standard library reads as NotFound:
    // ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_INVALID_DRIVE,
    // ERROR_BAD_NETPATH and ERROR_BAD_NET_NAME.
    for (unsigned long code : {ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_INVALID_DRIVE, ERROR_BAD_NETPATH, ERROR_BAD_NET_NAME}) {
        if (error.ends_with(std::format("(os error {})", code))) {
            return true;
        }
    }
    return false;
}

std::expected<void, std::string> create_dir_all(const std::filesystem::path& path) {
    if (path.empty()) {
        return {};
    }
    std::error_code error;
    std::filesystem::create_directories(path, error);
    if (error) {
        return std::unexpected(os_error(static_cast<unsigned long>(error.value())));
    }
    return {};
}

} // namespace base
