// The counting allocator, and the usage that `measure` and `held` make of
// its counts.
//
// The counters are the whole process's, so these tests are one test, which
// runs them one after another, as the Rust's ran them without its test
// harness: the tests of a project all run on the main thread, one at a time,
// so nothing else allocates while they count, and counting is off whenever
// the harness itself runs, between tests. A CHECK that holds allocates
// nothing. A CHECK_EQ makes its message's text before it compares, even an
// empty one, which allocates in a Debug build: so none is made while counting
// is on, nor has `stop()` in it. Every allocation goes through `black_box`,
// so that the optimizer can't drop one whose block is never read.
//
// The blocks that are counted to the byte are arrays from new, and not
// std::vectors as they're Vecs in Rust: in a Debug build, where the tests
// run, each standard container also allocates a proxy for its checked
// iterators, which Release doesn't have.

#include "Benchmarks/Benchmarks.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <iterator>
#include <memory>
#include <new>
#include <print>
#include <string_view>
#include <utility>
#include <vector>

using benchmarks::black_box;
using benchmarks::Budget;
using benchmarks::Usage;
using benchmarks::memory::Counts;
using benchmarks::memory::counts;
using benchmarks::memory::held;
using benchmarks::memory::start;
using benchmarks::memory::stop;

namespace {

void the_allocator_counts_the_bytes_it_hands_out_and_takes_back() {
    start();
    {
        std::vector<std::uint8_t> block = black_box(std::vector<std::uint8_t>(4096, 7));
        Counts after = counts();
        CHECK(after.allocations >= 1, "{}", describe_value(after));
        CHECK(after.bytes >= 4096, "{}", describe_value(after));
        CHECK(after.live >= 4096, "{}", describe_value(after));
    }
    Counts done = stop();
    CHECK(done.frees >= 1, "{}", describe_value(done));
    CHECK(done.live <= 0, "{}", describe_value(done));
    CHECK(done.peak >= 4096, "{}", describe_value(done));
}

void it_counts_nothing_until_a_benchmark_turns_it_on() {
    start();
    Counts quiet = stop();
    std::vector<std::uint8_t> block = black_box(std::vector<std::uint8_t>(8192, 7));
    black_box(block);
    CHECK_EQ(counts(), quiet);
}

void a_measurement_is_of_one_run_of_the_work() {
    size_t runs_done = 0;
    Usage usage = benchmarks::memory::measure("memory/test/vector", "run", Budget{.seconds = 0.0, .iterations = 100, .warmup = 2}, 4, [&] {
        runs_done += 1;
        std::unique_ptr<std::uint64_t[]> block = black_box(std::unique_ptr<std::uint64_t[]>(new std::uint64_t[128]));
        std::fill_n(block.get(), 128, std::uint64_t{1});
        black_box(block);
    });
    // Two warm-up runs that aren't counted, and four that are.
    CHECK_EQ(runs_done, 6);
    CHECK_EQ(usage.units, 4.0);
    CHECK_EQ(usage.allocations, 1.0);
    CHECK_EQ(usage.bytes, 1024.0);
    // It frees what it allocates, and never holds more than one at a time.
    CHECK_EQ(usage.kept, 0);
    CHECK_EQ(usage.peak, 1024);
}

void what_is_held_is_counted_while_it_is_alive() {
    auto [usage, held_block] = held("memory/test/held", "block", [] { return black_box(std::make_unique<std::array<std::uint8_t, 2048>>()); });
    CHECK_EQ(held_block->size(), 2048);
    black_box(held_block);
    CHECK_EQ(usage.units, 1.0);
    CHECK_EQ(usage.kept, 2048);
    CHECK_EQ(usage.peak, 2048);
    // Counting is off again, so the drop isn't counted.
    Counts before = counts();
    held_block.reset();
    CHECK_EQ(counts(), before);
}

// A block's size and alignment for the test below, and one of the ways that
// C++ has of allocating it and freeing it.
constexpr size_t Size = 300;
constexpr std::align_val_t Alignment{64};

struct Form {
    std::string_view name;
    void* (*allocate)();
    void (*release)(void*);
};

// Not in the Rust, whose allocator has one way in and one way out: every
// form of operator new counts the bytes that it was asked for, and every
// form of operator delete the same bytes, whether the compiler told it them
// or the heap did.
void every_form_of_new_and_delete_counts_the_same_bytes() {
    const Form forms[] = {
        {"new, delete", [] { return ::operator new(Size); }, [](void* block) { ::operator delete(block); }},
        {"new, delete with a size", [] { return ::operator new(Size); }, [](void* block) { ::operator delete(block, Size); }},
        {"new[], delete[]", [] { return ::operator new[](Size); }, [](void* block) { ::operator delete[](block); }},
        {"new[], delete[] with a size", [] { return ::operator new[](Size); }, [](void* block) { ::operator delete[](block, Size); }},
        {"aligned new, delete", [] { return ::operator new(Size, Alignment); }, [](void* block) { ::operator delete(block, Alignment); }},
        {"aligned new, delete with a size", [] { return ::operator new(Size, Alignment); }, [](void* block) { ::operator delete(block, Size, Alignment); }},
        {"aligned new[], delete[]", [] { return ::operator new[](Size, Alignment); }, [](void* block) { ::operator delete[](block, Alignment); }},
        {"aligned new[], delete[] with a size", [] { return ::operator new[](Size, Alignment); }, [](void* block) { ::operator delete[](block, Size, Alignment); }},
        {"nothrow new, delete", [] { return ::operator new(Size, std::nothrow); }, [](void* block) { ::operator delete(block, std::nothrow); }},
        {"nothrow new[], delete[]", [] { return ::operator new[](Size, std::nothrow); }, [](void* block) { ::operator delete[](block, std::nothrow); }},
        {"aligned nothrow new, delete", [] { return ::operator new(Size, Alignment, std::nothrow); }, [](void* block) { ::operator delete(block, Alignment, std::nothrow); }},
        {"aligned nothrow new[], delete[]", [] { return ::operator new[](Size, Alignment, std::nothrow); },
         [](void* block) { ::operator delete[](block, Alignment, std::nothrow); }},
    };
    for (const Form& form : forms) {
        start();
        void* block = black_box(form.allocate());
        Counts after = counts();
        form.release(block);
        Counts done = stop();
        CHECK(block != nullptr, "{}", form.name);
        CHECK_EQ(after, (Counts{.allocations = 1, .frees = 0, .bytes = Size, .live = Size, .peak = Size}), "{}", form.name);
        CHECK_EQ(done, (Counts{.allocations = 1, .frees = 1, .bytes = Size, .live = 0, .peak = Size}), "{}", form.name);
    }
    // Nothing is counted for the null that a delete may be given.
    start();
    ::operator delete(nullptr);
    ::operator delete[](nullptr, Size);
    ::operator delete(nullptr, Alignment);
    Counts nothing = stop();
    CHECK_EQ(nothing, Counts{});
}

struct Test {
    std::string_view name;
    void (*body)();
};

constexpr Test Tests[] = {
    {"the_allocator_counts_the_bytes_it_hands_out_and_takes_back", the_allocator_counts_the_bytes_it_hands_out_and_takes_back},
    {"it_counts_nothing_until_a_benchmark_turns_it_on", it_counts_nothing_until_a_benchmark_turns_it_on},
    {"a_measurement_is_of_one_run_of_the_work", a_measurement_is_of_one_run_of_the_work},
    {"what_is_held_is_counted_while_it_is_alive", what_is_held_is_counted_while_it_is_alive},
    {"every_form_of_new_and_delete_counts_the_same_bytes", every_form_of_new_and_delete_counts_the_same_bytes},
};

} // namespace

// Runs every test, and stops at the first that fails, whose failed check
// fails this one. Counting is off between them, so what this prints isn't
// counted.
TEST(counting) {
    std::println("running {} tests", std::size(Tests));
    for (const Test& test : Tests) {
        std::print("test {} ... ", test.name);
        std::fflush(stdout);
        test.body();
        std::println("ok");
    }
    std::println("test result: ok. {} passed", std::size(Tests));
}
