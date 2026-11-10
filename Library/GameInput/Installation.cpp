#include "GameInput/Installation.h"

#include "Base/Files.h"
#include "Base/Windows.h"

#include <algorithm>
#include <format>
#include <vector>

namespace game_input::installation {

namespace {

// Windows' FILE_SHARE_READ: others may read the file while it's read, but not
// write it.
constexpr DWORD ShareReadOnly = FILE_SHARE_READ;

// Closes the file when it goes out of scope.
struct File {
    HANDLE handle;

    ~File() {
        if (handle != INVALID_HANDLE_VALUE) {
            CloseHandle(handle);
        }
    }
};

// Whether the system's error says that the file isn't there: the errors that
// are std::io::ErrorKind::NotFound in Rust.
bool is_not_found(DWORD error) {
    return error == ERROR_FILE_NOT_FOUND || error == ERROR_PATH_NOT_FOUND || error == ERROR_INVALID_DRIVE || error == ERROR_BAD_NETPATH || error == ERROR_BAD_NET_NAME;
}

std::string couldnt_read_message(const std::filesystem::path& path, DWORD error) {
    return std::format("Couldn't read {}: {}", base::display(path), base::os_error(error));
}

std::string dotted(const std::array<std::uint16_t, 4>& version) {
    return std::format("{}.{}.{}.{}", version[0], version[1], version[2], version[3]);
}

} // namespace

std::expected<std::filesystem::path, std::string> installed_runtime() {
    return check(base::path(Runtime));
}

std::expected<std::filesystem::path, std::string> check(const std::filesystem::path& path) {
    File file{CreateFileW(path.c_str(), GENERIC_READ, ShareReadOnly, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)};
    if (file.handle == INVALID_HANDLE_VALUE) {
        DWORD error = GetLastError();
        if (is_not_found(error)) {
            return std::unexpected(not_installed_message(path));
        }
        return std::unexpected(couldnt_read_message(path, error));
    }
    LARGE_INTEGER size{};
    if (!GetFileSizeEx(file.handle, &size)) {
        return std::unexpected(couldnt_read_message(path, GetLastError()));
    }
    std::vector<std::uint8_t> bytes(static_cast<size_t>(size.QuadPart));
    size_t done = 0;
    while (done < bytes.size()) {
        DWORD wanted = static_cast<DWORD>(std::min<size_t>(bytes.size() - done, 1u << 30));
        DWORD got = 0;
        if (!ReadFile(file.handle, bytes.data() + done, wanted, &got, nullptr)) {
            return std::unexpected(couldnt_read_message(path, GetLastError()));
        }
        if (got == 0) {
            bytes.resize(done);
            break;
        }
        done += got;
    }
    std::optional<std::array<std::uint16_t, 4>> version = version_in(bytes);
    if (version && *version == Version) {
        return path;
    }
    return std::unexpected(wrong_version_message(path, version));
}

std::optional<std::array<std::uint16_t, 4>> version_in(std::span<const std::uint8_t> bytes) {
    constexpr std::array<std::uint8_t, 8> Start = {0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x01, 0x00};
    for (size_t end = bytes.size(); end >= 16; end--) {
        std::span<const std::uint8_t> info = bytes.subspan(end - 16, 16);
        if (!std::ranges::equal(info.first(8), Start)) {
            continue;
        }
        auto half = [&](size_t at) {
            return static_cast<std::uint32_t>(info[at]) | static_cast<std::uint32_t>(info[at + 1]) << 8 | static_cast<std::uint32_t>(info[at + 2]) << 16 |
                   static_cast<std::uint32_t>(info[at + 3]) << 24;
        };
        std::uint32_t most = half(8);
        std::uint32_t least = half(12);
        return std::array<std::uint16_t, 4>{static_cast<std::uint16_t>(most >> 16), static_cast<std::uint16_t>(most), static_cast<std::uint16_t>(least >> 16),
                                            static_cast<std::uint16_t>(least)};
    }
    return std::nullopt;
}

std::string not_installed_message(const std::filesystem::path& path) {
    return std::format("GameInput 4.0.306 isn't installed: {} isn't there, and KGames needs it. "
                       "It never installs it itself. Install it, from an elevated prompt, with: {}",
                       base::display(path), InstallCommand);
}

std::string wrong_version_message(const std::filesystem::path& path, std::optional<std::array<std::uint16_t, 4>> version) {
    std::string displayed = base::display(path);
    std::string needed = dotted(Version);
    if (version && *version > Version) {
        return std::format("{} is GameInput {}, but KGames needs exactly {}. GameInputRedist.msi doesn't "
                           "replace a newer version, so uninstall Microsoft GameInput first, then install {}, from an "
                           "elevated prompt, with: {}",
                           displayed, dotted(*version), needed, needed, InstallCommand);
    }
    if (version) {
        return std::format("{} is GameInput {}, but KGames needs exactly {}. Install it, from an elevated "
                           "prompt, with: {}",
                           displayed, dotted(*version), needed, InstallCommand);
    }
    return std::format("{} has no version, but KGames needs GameInput {}. Uninstall Microsoft GameInput, "
                       "then install it again, from an elevated prompt, with: {}",
                       displayed, needed, InstallCommand);
}

} // namespace game_input::installation
