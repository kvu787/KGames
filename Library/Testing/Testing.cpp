#include "Testing/Testing.h"

#include "Base/Panic.h"

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <exception>
#include <filesystem>
#include <print>
#include <vector>

namespace testing {

namespace {

struct Test {
    std::string name;
    void (*body)();
};

// A function's static, so that it's there for the registrations, which run
// before main in no order that the language gives.
std::vector<Test>& tests() {
    static std::vector<Test> tests;
    return tests;
}

std::string at(const std::source_location& where) {
    return std::format("{}:{}", std::filesystem::path(where.file_name()).filename().string(), where.line());
}

} // namespace

void fail(std::string message, std::source_location where) {
    throw Failure{std::move(message), where};
}

Registration::Registration(const char* file, const char* name, void (*body)()) {
    tests().push_back({std::filesystem::path(file).stem().string() + "::" + name, body});
}

namespace detail {

void fail_check(std::string_view what, std::string message, std::source_location where) {
    std::string text = std::format("check failed: {}", what);
    if (!message.empty()) {
        text += "\n  " + message;
    }
    throw Failure{std::move(text), where};
}

void fail_compare(std::string_view comparison, std::string_view left_text, std::string_view right_text, std::string left, std::string right, std::string message,
                  std::source_location where) {
    std::string text = std::format("check failed: {} {} {}\n   left: {}\n  right: {}", left_text, comparison, right_text, left, right);
    if (!message.empty()) {
        text += "\n  " + message;
    }
    throw Failure{std::move(text), where};
}

int run() {
    base::fail_quietly();
    std::vector<std::string> filters;
    bool list = false;
    for (const std::string& argument : base::arguments()) {
        if (argument == "--list") {
            list = true;
        } else {
            filters.push_back(argument);
        }
    }

    std::vector<Test> selected;
    for (const Test& test : tests()) {
        bool wanted = filters.empty() || std::ranges::any_of(filters, [&](const std::string& filter) { return test.name.contains(filter); });
        if (wanted) {
            selected.push_back(test);
        }
    }
    std::ranges::sort(selected, {}, &Test::name);
    if (list) {
        for (const Test& test : selected) {
            std::println("{}", test.name);
        }
        return 0;
    }

    std::vector<std::string> failed;
    auto started = std::chrono::steady_clock::now();
    for (const Test& test : selected) {
        std::string problem;
        try {
            test.body();
        } catch (const Failure& failure) {
            problem = std::format("{} at {}", failure.message, at(failure.where));
        } catch (const std::exception& exception) {
            problem = std::format("threw: {}", exception.what());
        } catch (...) {
            problem = "threw something that isn't a std::exception";
        }
        if (problem.empty()) {
            std::println("test {} ... ok", test.name);
        } else {
            std::println("test {} ... FAILED\n  {}", test.name, problem);
            failed.push_back(test.name);
        }
        std::fflush(stdout);
    }

    double seconds = std::chrono::duration<double>(std::chrono::steady_clock::now() - started).count();
    std::println("\ntest result: {}. {} passed; {} failed; {} filtered out; finished in {:.2f}s", failed.empty() ? "ok" : "FAILED", selected.size() - failed.size(), failed.size(),
                 tests().size() - selected.size(), seconds);
    for (const std::string& name : failed) {
        std::println("  failed: {}", name);
    }
    return failed.empty() ? 0 : 1;
}

} // namespace detail

} // namespace testing

int main() {
    return testing::detail::run();
}
