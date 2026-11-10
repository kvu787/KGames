// The persistent store: round trips, fresh starts, wrong files, and the
// dirty flag that makes one cascade save once.

#include "Platformer/Progress.h"
#include "Base/Files.h"
#include "Testing/Testing.h"

#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <vector>

using platformer::progress::Progress;

namespace {

std::filesystem::path folder(std::string_view test) {
    std::filesystem::path path = std::filesystem::temp_directory_path() / "PlatformerProgressTests" / test;
    std::error_code ignored;
    std::filesystem::remove_all(path, ignored);
    return path;
}

} // namespace

TEST(progress_round_trips) {
    std::filesystem::path path = folder("RoundTrip") / "Save.json";
    Progress progress(3);
    CHECK(progress.record(0, 12.345).has_value());
    CHECK(progress.unlock(1));
    (void)progress.save(path).value();
    Progress loaded = Progress::load(path, 3).value();
    CHECK_EQ(loaded.unlocked, 2);
    CHECK_EQ(loaded.best, (std::vector<std::optional<double>>{12.345, std::nullopt, std::nullopt}));
    CHECK(!loaded.dirty);
}

TEST(a_missing_file_is_a_fresh_start) {
    Progress progress = Progress::load(folder("Missing") / "Save.json", 2).value();
    CHECK_EQ(progress.unlocked, 1);
    CHECK_EQ(progress.best, (std::vector<std::optional<double>>{std::nullopt, std::nullopt}));
}

TEST(a_wrong_file_says_what_is_wrong) {
    std::filesystem::path path = folder("Wrong") / "Save.json";
    base::create_dir_all(path.parent_path()).value();
    base::write(path, std::string_view("{ \"Unlocked\": \"many\" }")).value();
    std::string problem = Progress::load(path, 2).error();
    CHECK(problem.contains("Unlocked"), "{}", problem);
    base::write(path, std::string_view("{ \"Best\": [true] }")).value();
    problem = Progress::load(path, 2).error();
    CHECK(problem.contains("Best[0]"), "{}", problem);
    base::write(path, std::string_view("not json")).value();
    CHECK(!Progress::load(path, 2).has_value());
}

TEST(only_a_better_time_records) {
    using Recorded = std::optional<std::optional<double>>;
    Progress progress(1);
    CHECK_EQ(progress.record(0, 10.0), Recorded(std::in_place, std::nullopt));
    CHECK_EQ(progress.record(0, 12.0), Recorded(), "slower doesn't record");
    CHECK_EQ(progress.record(0, 8.0), Recorded(std::in_place, 10.0), "faster does, and says what it beat");
    CHECK_EQ(progress.best[0], std::optional<double>(8.0));
}

TEST(saving_clears_the_dirty_flag_and_unlock_is_idempotent) {
    std::filesystem::path path = folder("Dirty") / "Save.json";
    Progress progress(2);
    CHECK(progress.unlock(1));
    CHECK(progress.dirty);
    (void)progress.save(path).value();
    CHECK(!progress.dirty);
    CHECK(!progress.unlock(1), "already unlocked");
    CHECK(!progress.dirty);
    CHECK(!progress.unlock(5), "past the levels");
}
