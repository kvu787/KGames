#include "ShaderCompiler/ShaderCompiler.h"

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"

#include <algorithm>
#include <format>
#include <system_error>
#include <thread>
#include <utility>
#include <vector>

namespace shader_compiler {

namespace {

// Running a program and taking what it wrote, which is what
// std::process::Command and its output() are in Rust.

// A handle of Windows', closed when it goes out of scope.
class Handle {
public:
    Handle() = default;
    explicit Handle(HANDLE handle) : handle_(handle) {}
    Handle(const Handle&) = delete;
    Handle& operator=(const Handle&) = delete;

    ~Handle() {
        close();
    }

    void close() {
        if (handle_ != nullptr && handle_ != INVALID_HANDLE_VALUE) {
            CloseHandle(handle_);
        }
        handle_ = nullptr;
    }

    HANDLE get() const {
        return handle_;
    }

    HANDLE* put() {
        return &handle_;
    }

private:
    HANDLE handle_ = nullptr;
};

// What a program that ran wrote, and the code it exited with.
struct Output {
    DWORD exit_code = 0;
    std::string standard_output;
    std::string standard_error;
};

// Adds an argument to a command line, quoted and escaped the way that the
// C runtime of the program that's run takes its arguments apart, as Rust's
// Command does it.
void append_argument(std::wstring& command, std::wstring_view argument, bool force_quotes) {
    bool quote = force_quotes || argument.empty() || argument.find(L' ') != std::wstring_view::npos || argument.find(L'\t') != std::wstring_view::npos;
    if (quote) {
        command.push_back(L'"');
    }
    size_t backslashes = 0;
    for (wchar_t character : argument) {
        if (character == L'\\') {
            backslashes += 1;
        } else {
            if (character == L'"') {
                // Add n+1 backslashes to total 2n+1 before internal '"'.
                command.append(backslashes + 1, L'\\');
            }
            backslashes = 0;
        }
        command.push_back(character);
    }
    if (quote) {
        // Add n backslashes to total 2n before ending '"'.
        command.append(backslashes, L'\\');
        command.push_back(L'"');
    }
}

// Everything that the pipe's other end writes, until it closes.
std::string read_to_end(HANDLE pipe) {
    std::string text;
    char buffer[4096];
    while (true) {
        DWORD got = 0;
        if (!ReadFile(pipe, buffer, sizeof(buffer), &got, nullptr) || got == 0) {
            return text;
        }
        text.append(buffer, got);
    }
}

// The bytes as text, with U+FFFD where they aren't UTF-8:
// String::from_utf8_lossy.
std::string from_utf8_lossy(std::string_view bytes) {
    std::string text;
    text.reserve(bytes.size());
    for (char32_t character : base::chars(bytes)) {
        base::push_char(text, character);
    }
    return text;
}

// Runs the program with the arguments, without input, waits for it to end,
// and returns what it wrote. Fails with what the system says if the program
// can't start.
std::expected<Output, std::string> run(const std::filesystem::path& program, std::span<const std::wstring> arguments) {
    std::wstring command;
    append_argument(command, program.native(), true);
    for (const std::wstring& argument : arguments) {
        command.push_back(L' ');
        append_argument(command, argument, false);
    }

    // The program inherits its ends of the pipes, and nothing to read.
    SECURITY_ATTRIBUTES inherited{.nLength = sizeof(SECURITY_ATTRIBUTES), .lpSecurityDescriptor = nullptr, .bInheritHandle = TRUE};
    Handle output_read;
    Handle output_write;
    Handle error_read;
    Handle error_write;
    if (!CreatePipe(output_read.put(), output_write.put(), &inherited, 0) || !CreatePipe(error_read.put(), error_write.put(), &inherited, 0)) {
        return std::unexpected(base::os_error(GetLastError()));
    }
    SetHandleInformation(output_read.get(), HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(error_read.get(), HANDLE_FLAG_INHERIT, 0);
    Handle input(CreateFileW(L"NUL", GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, &inherited, OPEN_EXISTING, 0, nullptr));
    if (input.get() == INVALID_HANDLE_VALUE) {
        return std::unexpected(base::os_error(GetLastError()));
    }

    STARTUPINFOW startup{};
    startup.cb = sizeof(startup);
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = input.get();
    startup.hStdOutput = output_write.get();
    startup.hStdError = error_write.get();
    PROCESS_INFORMATION information{};
    if (!CreateProcessW(program.c_str(), command.data(), nullptr, nullptr, TRUE, CREATE_UNICODE_ENVIRONMENT, nullptr, nullptr, &startup, &information)) {
        return std::unexpected(base::os_error(GetLastError()));
    }
    Handle process(information.hProcess);
    Handle thread(information.hThread);
    // Only the program has the pipes' other ends now, so that they close
    // when it ends.
    output_write.close();
    error_write.close();
    input.close();

    // Both at once, so that the program never waits for one to be read
    // while this waits for the other.
    Output output;
    std::thread error_reader([&] { output.standard_error = read_to_end(error_read.get()); });
    output.standard_output = read_to_end(output_read.get());
    error_reader.join();
    WaitForSingleObject(process.get(), INFINITE);
    if (!GetExitCodeProcess(process.get(), &output.exit_code)) {
        return std::unexpected(base::os_error(GetLastError()));
    }
    return output;
}

} // namespace

std::filesystem::path find_dxc() {
    std::filesystem::path bin = LR"(C:\Program Files (x86)\Windows Kits\10\bin)";
    std::error_code error;
    std::filesystem::directory_iterator entries(bin, error);
    if (error) {
        base::panic(std::format("Windows SDK not found: {}", base::os_error(static_cast<unsigned long>(error.value()))));
    }
    std::vector<std::filesystem::path> candidates;
    for (std::filesystem::directory_iterator end; !error && entries != end; entries.increment(error)) {
        std::filesystem::path path = entries->path() / LR"(x64\dxc.exe)";
        std::error_code missing;
        if (std::filesystem::exists(path, missing)) {
            candidates.push_back(std::move(path));
        }
    }
    std::ranges::stable_sort(candidates);
    if (candidates.empty()) {
        base::panic("dxc.exe not found; install the Windows SDK");
    }
    return std::move(candidates.back());
}

std::expected<void, std::string> compile(const std::filesystem::path& source, std::string_view entry_point, std::string_view profile, std::span<const std::string_view> arguments,
                                         const std::filesystem::path& output) {
    std::vector<std::wstring> command{L"-T", base::widen(profile), L"-E", base::widen(entry_point)};
    for (std::string_view argument : arguments) {
        command.push_back(base::widen(argument));
    }
    command.push_back(L"-Fo");
    command.push_back(output.native());
    command.push_back(source.native());
    std::expected<Output, std::string> result = run(find_dxc(), command);
    if (!result) {
        return std::unexpected(std::format("failed to run dxc.exe: {}", result.error()));
    }
    if (result->exit_code == 0) {
        return {};
    }
    return std::unexpected(std::format("dxc.exe failed to compile {} in {}:\n{}{}", entry_point, base::display(source), from_utf8_lossy(result->standard_output),
                                       from_utf8_lossy(result->standard_error)));
}

} // namespace shader_compiler
