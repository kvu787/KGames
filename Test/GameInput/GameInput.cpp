// Library/GameInput: that KGames starts GameInput with GameInputInitialize
// from the NuGet package's GameInput.lib, whose loader takes GameInput
// 4.0.306's installed runtime, from System32, unless one is beside the exe,
// and that it says how to install it where it isn't installed. KGames doesn't
// build without it, so these run where it's installed.

#include "GameInput/GameInput.h"
#include "Testing/Testing.h"

#include "Modules.h"
#include "Processes.h"

#include "Base/Files.h"
#include "Base/Text.h"
#include "Folders.h"

#include <algorithm>
#include <array>
#include <atomic>
#include <barrier>
#include <chrono>
#include <format>
#include <optional>
#include <print>
#include <thread>
#include <vector>

namespace installation = game_input::installation;
using game_input::GameInputCallbackToken;
using game_input::GameInputDeviceEvent;
using game_input::GameInputKindGamepad;
using game_input::GameInputKindKeyboard;
using game_input::GameInputKindMouse;
using game_input::installation::InstallCommand;
using game_input::installation::Runtime;
using game_input::installation::Version;

namespace {

// The command that the user installs GameInput with, which is theirs to run.
constexpr std::string_view UsersCommand = R"(winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306")";

// A file or folder that a test made, removed as the test ends.
struct RemovedOnDrop {
    std::filesystem::path path;

    ~RemovedOnDrop() {
        std::error_code error;
        std::filesystem::remove_all(path, error);
    }
};

// A file that a test opened, closed as the test ends, or sooner.
struct Opened {
    HANDLE handle;

    ~Opened() {
        if (handle != INVALID_HANDLE_VALUE) {
            CloseHandle(handle);
        }
    }
};

// A folder of a test's own, for what it writes, which this process's ID keeps
// apart from another run's.
std::filesystem::path folder_of_its_own(std::string_view name) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / std::format("KGames{}{}", name, GetCurrentProcessId());
    std::expected<void, std::string> created = base::create_dir_all(folder);
    CHECK(created.has_value(), "Couldn't make {}: {}", base::display(folder), created.error());
    return folder;
}

// Whether GameInput has reported a device to one_copy_of_the_runtime_runs.
std::atomic<bool> Reported = false;

void CALLBACK reported(GameInputCallbackToken, void*, GameInputDeviceEvent, game_input::IGameInputDevice*) {
    Reported.store(true, std::memory_order_release);
}

} // namespace

TEST(the_runtime_is_4_0_306_in_system32) {
    CHECK_EQ(Version, (std::array<std::uint16_t, 4>{4, 0, 306, 0}));
    CHECK_EQ(Runtime, R"(C:\Windows\System32\GameInputRedist.dll)");
    CHECK_EQ(base::display(installation::installed_runtime().value()), Runtime);

    // With nothing beside the exe, the loader takes it.
    auto [game_input, runtime] = game_input::create().value();
    CHECK(modules::same(runtime.path, base::path(Runtime)), "{}", base::debug(base::display(runtime.path)));
    CHECK_EQ(runtime.version, Version);
}

// The version is read from the DLL's bytes, with Library/Base alone, which
// InstallationCheck has: from its VS_FIXEDFILEINFO, which starts with the
// signature 0xFEEF04BD and the structure's version, 1.0.
TEST(the_version_is_read_from_the_dll_itself) {
    CHECK_EQ(installation::version_in(base::read(base::path(Runtime)).value()), std::optional(Version));
    CHECK_EQ(installation::version_in({}), std::nullopt);

    // The signature alone, without the structure's version, isn't one.
    std::vector<std::uint8_t> bytes(64);
    std::ranges::copy(std::array<std::uint8_t, 8>{0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x02, 0x00}, bytes.begin() + 4);
    CHECK_EQ(installation::version_in(bytes), std::nullopt);

    // 4.1.2.3, as its most significant half, 4 and 1, and its least, 2 and 3.
    std::ranges::copy(std::array<std::uint8_t, 16>{0xBD, 0x04, 0xEF, 0xFE, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x04, 0x00, 0x03, 0x00, 0x02, 0x00}, bytes.begin() + 32);
    CHECK_EQ(installation::version_in(bytes), std::optional(std::array<std::uint16_t, 4>{4, 1, 2, 3}));
}

// The check reads the DLL without letting others write it, so a DLL that's
// being written isn't checked. System32 takes an administrator to change at
// all, so this checks a copy.
TEST(a_dll_thats_being_written_isnt_checked) {
    std::filesystem::path folder = folder_of_its_own("CheckedRuntime");
    RemovedOnDrop removed{folder};
    std::filesystem::path dll = folder / "GameInputRedist.dll";
    std::filesystem::copy_file(base::path(Runtime), dll, std::filesystem::copy_options::overwrite_existing);
    CHECK_EQ(base::display(installation::check(dll).value()), base::display(dll));

    {
        Opened writing{CreateFileW(dll.c_str(), GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)};
        CHECK(writing.handle != INVALID_HANDLE_VALUE, "Couldn't open {}: {}", base::display(dll), base::os_error(GetLastError()));
        CHECK(!installation::check(dll).has_value());
    }
    std::filesystem::remove(dll);
    std::expected<std::filesystem::path, std::string> checked = installation::check(dll);
    CHECK(!checked.has_value());
    CHECK(checked.error().contains("isn't installed"), "{}", checked.error());
}

// GameInput.lib's loader, which create starts GameInput with, takes a
// GameInputRedist.dll beside the exe over the installed one of the same
// version, as Microsoft's documentation of sideloading says. So nothing in
// the repo may put one beside an exe.
//
// The loader picks a runtime once a process, and a loaded DLL can't be
// deleted, so a copy of this test's exe, in a folder of its own, with a copy
// of the runtime beside it, runs this test again, and says which runtime it
// loaded.
TEST(a_runtime_beside_the_exe_of_the_same_version_is_loaded) {
    constexpr std::string_view InTheCopy = "KGAMES_GAME_INPUT_TEST_IN_THE_COPY";
    if (processes::has_variable(InTheCopy)) {
        auto [game_input, runtime] = game_input::create().value();
        std::println("Loaded {}", base::display(runtime.path));
        return;
    }

    std::filesystem::path exe = processes::current_exe();
    std::filesystem::path folder = folder_of_its_own("SideloadedRuntime");
    RemovedOnDrop removed{folder};
    std::filesystem::path copy = folder / exe.filename();
    std::filesystem::copy_file(exe, copy, std::filesystem::copy_options::overwrite_existing);
    std::filesystem::path beside = folder / "GameInputRedist.dll";
    std::filesystem::copy_file(base::path(Runtime), beside, std::filesystem::copy_options::overwrite_existing);

    processes::Output output = processes::run(copy, {"GameInput::a_runtime_beside_the_exe_of_the_same_version_is_loaded"}, InTheCopy);
    CHECK(output.exit_code == 0, "{}", output.printed);
    std::optional<std::string_view> loaded;
    for (std::string_view line : base::lines(output.printed)) {
        if (line.starts_with("Loaded ")) {
            loaded = line.substr(7);
            break;
        }
    }
    CHECK(loaded.has_value(), "{}", output.printed);
    CHECK(modules::same(base::path(*loaded), beside), "{}", *loaded);
}

// The runtime's DLL is also what COM loads to carry calls to GameInput's
// service, from System32, where the installer registers it. The runtime
// connects to its service on a thread of its own as it starts, which loads
// that copy, about 5 ms in, and reports devices after that, about 600 ms in,
// so this waits for the first device. With nothing beside the exe,
// GameInput.lib's loader takes System32's copy too, so the process runs one.
TEST(one_copy_of_the_runtime_runs) {
    auto [game_input, runtime] = game_input::create().value();
    game_input::GameInputKind kinds = GameInputKindGamepad | GameInputKindKeyboard | GameInputKindMouse;
    GameInputCallbackToken token = 0;
    base::check(game_input->RegisterDeviceCallback(nullptr, kinds, nullptr, reported, &token));
    std::chrono::steady_clock::time_point deadline = std::chrono::steady_clock::now() + std::chrono::seconds(10);
    while (!Reported.load(std::memory_order_acquire)) {
        CHECK(std::chrono::steady_clock::now() < deadline, "GameInput reported no keyboard, mouse or gamepad");
        std::this_thread::sleep_for(std::chrono::milliseconds(1));
    }
    CHECK(game_input->UnregisterCallback(token));

    std::vector<std::filesystem::path> loaded = modules::named("GameInputRedist.dll");
    CHECK_EQ(loaded.size(), 1, "{}", modules::debug(loaded));
    CHECK(modules::same(loaded[0], base::path(Runtime)), "{}", modules::debug(loaded));
}

// GameInput 4.0.306's runtime crashes the process when one thread releases
// the last IGameInput while another creates one. create keeps the first one
// that it creates, so threads can create and release them at once.
TEST(threads_can_create_and_release_game_input_at_once) {
    for (int round = 0; round < 20; round++) {
        std::barrier barrier(2);
        // What went wrong on each thread, if anything did.
        std::array<std::string, 2> problems;
        std::vector<std::thread> threads;
        for (size_t index = 0; index < 2; index++) {
            threads.emplace_back([&barrier, &problems, index] {
                barrier.arrive_and_wait();
                // The second thread creates while the first releases.
                if (index == 1) {
                    std::this_thread::sleep_for(std::chrono::microseconds(200));
                }
                auto created = game_input::create();
                if (!created) {
                    problems[index] = created.error();
                }
            });
        }
        for (std::thread& thread : threads) {
            thread.join();
        }
        for (const std::string& problem : problems) {
            CHECK(problem.empty(), "{}", problem);
        }
    }
}

TEST(without_4_0_306_it_says_how_to_install_it) {
    CHECK_EQ(InstallCommand, UsersCommand);

    std::filesystem::path runtime = base::path(Runtime);
    std::string message = installation::not_installed_message(runtime);
    CHECK(message.contains("GameInput 4.0.306 isn't installed") && message.contains(UsersCommand), "{}", message);

    // A newer version, like one that another program's installer put in,
    // which GameInputRedist.msi 4.0.306 doesn't replace.
    message = installation::wrong_version_message(runtime, std::array<std::uint16_t, 4>{4, 1, 0, 0});
    CHECK(message.contains("is GameInput 4.1.0.0") && message.contains("uninstall Microsoft GameInput first"), "{}", message);
    CHECK(message.contains(UsersCommand), "{}", message);

    // An older one, which it does replace.
    message = installation::wrong_version_message(runtime, std::array<std::uint16_t, 4>{4, 0, 300, 0});
    CHECK(message.contains("is GameInput 4.0.300.0") && !message.contains("uninstall"), "{}", message);
    CHECK(message.contains(UsersCommand), "{}", message);
}

// NuGetPackages.proj asks for exactly the version of GameInput whose runtime
// the user installs, and packages.lock.json pins that version.
TEST(the_package_is_the_runtimes_version) {
    std::string version = std::format("{}.{}.{}", Version[0], Version[1], Version[2]);
    std::filesystem::path folder = base::path(folders::Project) / "../../Library/GameInput";
    std::string project = base::read_to_string(folder / "NuGetPackages.proj").value();
    std::string reference = std::format(R"(<PackageReference Include="Microsoft.GameInput" Version="[{}]")", version);
    CHECK(project.contains(reference), "{}", project);
    std::string lock = base::read_to_string(folder / "packages.lock.json").value();
    CHECK(lock.contains(std::format(R"("requested": "[{0}, {0}]")", version)), "{}", lock);
    CHECK(lock.contains(std::format(R"("resolved": "{}")", version)), "{}", lock);
}

// The documentation that tells people and agents to install GameInput gives
// the command that the code checks for.
TEST(the_documentation_gives_the_command) {
    std::filesystem::path root = base::path(folders::Project) / "../..";
    for (std::string_view file : {"AGENTS.md", "README.md", "Library/GameInput/README.md", "Program/GameInputDemo/README.md"}) {
        std::string text = base::read_to_string(root / file).value();
        CHECK(text.contains(UsersCommand), "{} doesn't give the command", file);
    }
}

namespace {

// Two of a struct that differ in any one byte of its fields, which are its
// first `size` bytes, aren't equal.
template <class T> void check_that_every_byte_is_compared(std::string_view name, size_t size) {
    T zeros{};
    CHECK(zeros == T{}, "{}", name);
    for (size_t offset = 0; offset < size; offset++) {
        T changed{};
        reinterpret_cast<unsigned char*>(&changed)[offset] = 1;
        CHECK(!(zeros == changed), "{} doesn't compare its byte {}", name, offset);
        CHECK(changed != zeros, "{} doesn't compare its byte {}", name, offset);
    }
}

} // namespace

// GameInput.h compares GameInput's structs field by field, with each field
// written out, so this pins that none is left out. GameInputKeyState and
// GameInputMouseInfo end in two bytes of padding.
TEST(structs_that_differ_in_any_field_arent_equal) {
    check_that_every_byte_is_compared<game_input::GameInputKeyState>("GameInputKeyState", sizeof(game_input::GameInputKeyState) - 2);
    check_that_every_byte_is_compared<game_input::GameInputMouseState>("GameInputMouseState", sizeof(game_input::GameInputMouseState));
    check_that_every_byte_is_compared<game_input::GameInputGamepadState>("GameInputGamepadState", sizeof(game_input::GameInputGamepadState));
    check_that_every_byte_is_compared<game_input::GameInputVersion>("GameInputVersion", sizeof(game_input::GameInputVersion));
    check_that_every_byte_is_compared<game_input::GameInputUsage>("GameInputUsage", sizeof(game_input::GameInputUsage));
    check_that_every_byte_is_compared<game_input::GameInputKeyboardInfo>("GameInputKeyboardInfo", sizeof(game_input::GameInputKeyboardInfo));
    check_that_every_byte_is_compared<game_input::GameInputMouseInfo>("GameInputMouseInfo", sizeof(game_input::GameInputMouseInfo) - 2);
    check_that_every_byte_is_compared<game_input::GameInputGamepadInfo>("GameInputGamepadInfo", sizeof(game_input::GameInputGamepadInfo));
    check_that_every_byte_is_compared<game_input::GameInputRumbleParams>("GameInputRumbleParams", sizeof(game_input::GameInputRumbleParams));
}
