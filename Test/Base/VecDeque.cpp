// Library/Base's VecDeque: a ring that keeps its items' order whichever end
// they go in and come out of, and moves them without copying.

#include "Base/VecDeque.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

std::vector<int> items(const base::VecDeque<int>& queue) {
    return std::vector<int>(queue.begin(), queue.end());
}

} // namespace

TEST(a_queue_keeps_its_order_from_both_ends) {
    base::VecDeque<int> queue;
    CHECK(queue.empty());
    CHECK(queue.front() == nullptr && queue.back() == nullptr);
    CHECK(!queue.pop_front().has_value() && !queue.pop_back().has_value());
    queue.push_back(2);
    queue.push_back(3);
    queue.push_front(1);
    queue.push_front(0);
    CHECK_EQ(items(queue), (std::vector<int>{0, 1, 2, 3}));
    CHECK_EQ(*queue.front(), 0);
    CHECK_EQ(*queue.back(), 3);
    CHECK_EQ(queue[2], 2);
    CHECK_EQ(queue.pop_front(), 0);
    CHECK_EQ(queue.pop_back(), 3);
    CHECK_EQ(items(queue), (std::vector<int>{1, 2}));
    CHECK_EQ(queue.size(), 2);
}

// The ring wraps around the end of its buffer, and grows from there with
// its items in order.
TEST(a_queue_wraps_and_grows_in_order) {
    base::VecDeque<int> queue;
    for (int item = 0; item < 4; item++) {
        queue.push_back(item);
    }
    CHECK_EQ(queue.capacity(), 4);
    // Two out of the front, two more in at the back: these wrap.
    CHECK_EQ(queue.pop_front(), 0);
    CHECK_EQ(queue.pop_front(), 1);
    queue.push_back(4);
    queue.push_back(5);
    CHECK_EQ(queue.capacity(), 4);
    CHECK_EQ(items(queue), (std::vector<int>{2, 3, 4, 5}));
    // Full and wrapped: the next one grows it.
    queue.push_back(6);
    CHECK_EQ(queue.capacity(), 8);
    CHECK_EQ(items(queue), (std::vector<int>{2, 3, 4, 5, 6}));
    queue.push_front(1);
    CHECK_EQ(items(queue), (std::vector<int>{1, 2, 3, 4, 5, 6}));
    for (int expected = 1; expected <= 6; expected++) {
        CHECK_EQ(queue.pop_front(), expected);
    }
    CHECK(queue.empty());
}

TEST(a_queue_made_from_a_vector_allocates_once) {
    base::VecDeque<std::string> queue(std::vector<std::string>{"a", "b", "c"});
    CHECK_EQ(queue.capacity(), 3);
    CHECK_EQ(queue.pop_front(), std::string("a"));
    queue.push_back("d");
    CHECK_EQ(queue.capacity(), 3);
    CHECK_EQ(std::vector<std::string>(queue.begin(), queue.end()), (std::vector<std::string>{"b", "c", "d"}));
    queue.truncate(1);
    CHECK_EQ(queue.size(), 1);
    CHECK_EQ(*queue.back(), "b");
    queue.clear();
    CHECK(queue.empty());
    CHECK_EQ(queue.capacity(), 3);
}

// Items move in and out, and each is destroyed once.
TEST(a_queue_owns_its_items) {
    std::shared_ptr<int> shared = std::make_shared<int>(7);
    {
        base::VecDeque<std::shared_ptr<int>> queue;
        for (int count = 0; count < 9; count++) {
            queue.push_back(shared);
            queue.push_front(shared);
        }
        CHECK_EQ(shared.use_count(), 19);
        std::optional<std::shared_ptr<int>> taken = queue.pop_front();
        CHECK_EQ(shared.use_count(), 19);
        taken.reset();
        CHECK_EQ(shared.use_count(), 18);

        base::VecDeque<std::shared_ptr<int>> copy = queue;
        CHECK_EQ(shared.use_count(), 35);
        base::VecDeque<std::shared_ptr<int>> moved = std::move(queue);
        CHECK_EQ(shared.use_count(), 35);
        CHECK(queue.empty());
        copy = std::move(moved);
        CHECK_EQ(shared.use_count(), 18);

        base::VecDeque<std::unique_ptr<int>> only_moves;
        only_moves.push_back(std::make_unique<int>(1));
        only_moves.push_front(std::make_unique<int>(0));
        for (int count = 0; count < 8; count++) {
            only_moves.push_back(std::make_unique<int>(count));
        }
        CHECK_EQ(**only_moves.front(), 0);
        CHECK_EQ(*only_moves.pop_back().value(), 7);
    }
    CHECK_EQ(shared.use_count(), 1);
}

TEST(queues_are_equal_by_their_items) {
    base::VecDeque<int> pushed;
    pushed.push_back(2);
    pushed.push_back(3);
    pushed.push_front(1);
    CHECK(pushed == (base::VecDeque<int>{1, 2, 3}));
    CHECK(!(pushed == (base::VecDeque<int>{1, 2})));
    CHECK(!(pushed == (base::VecDeque<int>{1, 2, 4})));
    CHECK(base::VecDeque<int>() == base::VecDeque<int>());
}

TEST(a_queues_items_work_with_the_standard_algorithms) {
    base::VecDeque<int> queue{5, 1, 4};
    queue.push_front(9);
    CHECK(std::ranges::find(queue, 4) != queue.end());
    CHECK(std::ranges::find(queue, 7) == queue.end());
    CHECK_EQ(std::ranges::max(queue), 9);
    std::ranges::sort(queue);
    CHECK_EQ(items(queue), (std::vector<int>{1, 4, 5, 9}));
    for (int& item : queue) {
        item *= 2;
    }
    CHECK_EQ(items(queue), (std::vector<int>{2, 8, 10, 18}));
    CHECK_EQ(queue.end() - queue.begin(), 4);
}

namespace {

// Counts the ones alive, and throws when the one marked is moved.
struct Fragile {
    static inline int alive = 0;
    int number;
    bool throws;

    Fragile(int number, bool throws) : number(number), throws(throws) {
        alive += 1;
    }

    Fragile(Fragile&& other) : number(other.number), throws(other.throws) {
        if (throws) {
            throw std::runtime_error("this one doesn't move");
        }
        alive += 1;
    }

    ~Fragile() {
        alive -= 1;
    }
};

} // namespace

// When a move throws as the queue grows, the new buffer goes with what was
// moved into it, and the queue keeps what it had: nothing is destroyed
// twice, and nothing is left behind.
TEST(a_queue_survives_a_move_that_throws_as_it_grows) {
    Fragile::alive = 0;
    {
        base::VecDeque<Fragile> queue;
        queue.reserve(4);
        for (int number = 0; number < 4; number++) {
            queue.push_back(Fragile(number, false));
        }
        CHECK_EQ(Fragile::alive, 4);
        // The third one throws when the queue moves it to a bigger buffer.
        queue[2].throws = true;
        CHECK_THROWS(queue.reserve(16));
        CHECK_EQ(queue.size(), 4);
        CHECK_EQ(queue.capacity(), 4);
        CHECK_EQ(Fragile::alive, 4);
        CHECK_EQ(queue[3].number, 3);
        queue[2].throws = false;
        queue.reserve(16);
        CHECK_EQ(queue.capacity(), 16);
        CHECK_EQ(Fragile::alive, 4);
    }
    CHECK_EQ(Fragile::alive, 0);
}
