// Library/GameInput: that the DLLs that the GameInput runtime imports are
// looked for beside the exe first. GameInput.lib's loader, which create starts
// GameInput with, loads the runtime with LoadLibraryW, and GameInputRedist.dll
// imports HID.DLL, which isn't a known DLL, so Windows takes a hid.dll beside
// the exe over System32's: here, one that isn't a DLL at all, which fails to
// load, so GameInput doesn't start.
//
// The test runs in a copy of this program, in a folder of its own, which runs
// this one test, so that nothing loads the runtime, or HID.DLL, before the
// file beside the exe is there, and so that the file is beside no other
// program.

#include "GameInput/GameInput.h"
#include "Testing/Testing.h"

#include "Modules.h"
#include "Processes.h"

#include "Base/Files.h"

#include <format>

namespace {

// A file or folder that a test made, removed as the test ends.
struct RemovedOnDrop {
    std::filesystem::path path;

    ~RemovedOnDrop() {
        std::error_code error;
        std::filesystem::remove_all(path, error);
    }
};

} // namespace

TEST(the_runtimes_imports_are_looked_for_beside_the_exe_first) {
    constexpr std::string_view InTheCopy = "KGAMES_GAME_INPUT_TEST_IN_THE_COPY";
    constexpr std::string_view Name = "RuntimeImports::the_runtimes_imports_are_looked_for_beside_the_exe_first";
    if (!processes::has_variable(InTheCopy)) {
        std::filesystem::path exe = processes::current_exe();
        std::filesystem::path folder = std::filesystem::temp_directory_path() / std::format("KGamesRuntimeImports{}", GetCurrentProcessId());
        std::expected<void, std::string> created = base::create_dir_all(folder);
        CHECK(created.has_value(), "Couldn't make {}: {}", base::display(folder), created.error());
        RemovedOnDrop removed{folder};
        std::filesystem::path copy = folder / exe.filename();
        std::filesystem::copy_file(exe, copy, std::filesystem::copy_options::overwrite_existing);

        processes::Output output = processes::run(copy, {Name}, InTheCopy);
        CHECK(output.exit_code == 0, "{}", output.printed);
        // The copy ran the test, rather than none.
        CHECK(output.printed.contains(std::format("test {} ... ok", Name)), "{}", output.printed);
        return;
    }

    std::filesystem::path beside = processes::current_exe().replace_filename("hid.dll");
    std::expected<void, std::string> written = base::write(beside, "Not a DLL");
    CHECK(written.has_value(), "Couldn't write {}: {}", base::display(beside), written.error());
    RemovedOnDrop removed{beside};
    CHECK(modules::named("hid.dll").empty(), "{}", modules::debug(modules::named("hid.dll")));

    // Windows' ERROR_BAD_EXE_FORMAT, as an HRESULT.
    auto created = game_input::create();
    CHECK(!created.has_value());
    const std::string& error = created.error();
    CHECK(error.contains("0x800700C1"), "{}", error);
    CHECK(modules::named("hid.dll").empty(), "{}", modules::debug(modules::named("hid.dll")));
}
