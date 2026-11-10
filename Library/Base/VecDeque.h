// A queue with two ends in one allocation, a ring that doubles when it's
// full: what VecDeque is in Rust. std::deque isn't that in MSVC's library,
// which allocates a block for every 16 bytes of elements, so one for each
// element that's larger than 8 bytes, where a frame's queue of messages
// should allocate once, or not at all.
//
// As in Rust, taking from an end gives an optional, and looking at one gives
// a pointer, null when the queue is empty:
//
//     base::VecDeque<Message> queue(std::move(messages));
//     while (std::optional<Message> message = queue.pop_front()) { ... }

#pragma once

#include <cstddef>
#include <initializer_list>
#include <iterator>
#include <memory>
#include <optional>
#include <utility>
#include <vector>

namespace base {

template <class T> class VecDeque {
public:
    VecDeque() = default;

    VecDeque(std::initializer_list<T> items) {
        reserve(items.size());
        for (const T& item : items) {
            push_back(item);
        }
    }

    // The vector's items, in their order: VecDeque::from(vec).
    explicit VecDeque(std::vector<T> items) {
        reserve(items.size());
        for (T& item : items) {
            push_back(std::move(item));
        }
    }

    VecDeque(const VecDeque& other) {
        reserve(other.size_);
        for (const T& item : other) {
            push_back(item);
        }
    }

    VecDeque(VecDeque&& other) noexcept : buffer_(other.buffer_), capacity_(other.capacity_), head_(other.head_), size_(other.size_) {
        other.buffer_ = nullptr;
        other.capacity_ = 0;
        other.head_ = 0;
        other.size_ = 0;
    }

    VecDeque& operator=(VecDeque other) noexcept {
        std::swap(buffer_, other.buffer_);
        std::swap(capacity_, other.capacity_);
        std::swap(head_, other.head_);
        std::swap(size_, other.size_);
        return *this;
    }

    ~VecDeque() {
        clear();
        if (buffer_ != nullptr) {
            std::allocator<T>().deallocate(buffer_, capacity_);
        }
    }

    size_t size() const {
        return size_;
    }

    bool empty() const {
        return size_ == 0;
    }

    size_t capacity() const {
        return capacity_;
    }

    // Room for this many items in all, without another allocation.
    void reserve(size_t wanted) {
        if (wanted > capacity_) {
            grow_to(wanted);
        }
    }

    // Drops every item, and keeps the allocation.
    void clear() {
        truncate(0);
        head_ = 0;
    }

    // Drops the items from the back until this many are left.
    void truncate(size_t length) {
        while (size_ > length) {
            std::destroy_at(slot(size_ - 1));
            size_ -= 1;
        }
    }

    void push_back(T item) {
        if (size_ == capacity_) {
            grow();
        }
        std::construct_at(slot(size_), std::move(item));
        size_ += 1;
    }

    void push_front(T item) {
        if (size_ == capacity_) {
            grow();
        }
        head_ = head_ == 0 ? capacity_ - 1 : head_ - 1;
        std::construct_at(buffer_ + head_, std::move(item));
        size_ += 1;
    }

    // The first item, taken out, or nothing if there's none.
    std::optional<T> pop_front() {
        if (size_ == 0) {
            return std::nullopt;
        }
        T* first = buffer_ + head_;
        std::optional<T> item(std::move(*first));
        std::destroy_at(first);
        head_ = head_ + 1 == capacity_ ? 0 : head_ + 1;
        size_ -= 1;
        return item;
    }

    // The last item, taken out, or nothing if there's none.
    std::optional<T> pop_back() {
        if (size_ == 0) {
            return std::nullopt;
        }
        T* last = slot(size_ - 1);
        std::optional<T> item(std::move(*last));
        std::destroy_at(last);
        size_ -= 1;
        return item;
    }

    // The first and the last item, or null if there's none.
    T* front() {
        return size_ == 0 ? nullptr : buffer_ + head_;
    }

    const T* front() const {
        return size_ == 0 ? nullptr : buffer_ + head_;
    }

    T* back() {
        return size_ == 0 ? nullptr : slot(size_ - 1);
    }

    const T* back() const {
        return size_ == 0 ? nullptr : slot(size_ - 1);
    }

    // The item at the index, from the front, which must be there.
    T& operator[](size_t index) {
        return *slot(index);
    }

    const T& operator[](size_t index) const {
        return *slot(index);
    }

    template <class Queue, class Item> class Iterator {
    public:
        using iterator_category = std::random_access_iterator_tag;
        using difference_type = std::ptrdiff_t;
        using value_type = T;
        using reference = Item&;
        using pointer = Item*;

        Iterator() = default;
        Iterator(Queue* queue, size_t index) : queue_(queue), index_(index) {}

        reference operator*() const {
            return (*queue_)[index_];
        }

        pointer operator->() const {
            return &(*queue_)[index_];
        }

        reference operator[](difference_type offset) const {
            return (*queue_)[index_ + offset];
        }

        Iterator& operator++() {
            index_ += 1;
            return *this;
        }

        Iterator operator++(int) {
            Iterator before = *this;
            index_ += 1;
            return before;
        }

        Iterator& operator--() {
            index_ -= 1;
            return *this;
        }

        Iterator operator--(int) {
            Iterator before = *this;
            index_ -= 1;
            return before;
        }

        Iterator& operator+=(difference_type offset) {
            index_ += offset;
            return *this;
        }

        Iterator& operator-=(difference_type offset) {
            index_ -= offset;
            return *this;
        }

        friend Iterator operator+(Iterator iterator, difference_type offset) {
            return iterator += offset;
        }

        friend Iterator operator+(difference_type offset, Iterator iterator) {
            return iterator += offset;
        }

        friend Iterator operator-(Iterator iterator, difference_type offset) {
            return iterator -= offset;
        }

        friend difference_type operator-(const Iterator& a, const Iterator& b) {
            return static_cast<difference_type>(a.index_) - static_cast<difference_type>(b.index_);
        }

        friend bool operator==(const Iterator& a, const Iterator& b) {
            return a.index_ == b.index_;
        }

        friend auto operator<=>(const Iterator& a, const Iterator& b) {
            return a.index_ <=> b.index_;
        }

    private:
        Queue* queue_ = nullptr;
        size_t index_ = 0;
    };

    // The same items in the same order, wherever each ring starts.
    bool operator==(const VecDeque& other) const {
        if (size_ != other.size_) {
            return false;
        }
        for (size_t index = 0; index < size_; index++) {
            if (!((*this)[index] == other[index])) {
                return false;
            }
        }
        return true;
    }

    using iterator = Iterator<VecDeque, T>;
    using const_iterator = Iterator<const VecDeque, const T>;

    // From the front to the back.
    iterator begin() {
        return iterator(this, 0);
    }

    iterator end() {
        return iterator(this, size_);
    }

    const_iterator begin() const {
        return const_iterator(this, 0);
    }

    const_iterator end() const {
        return const_iterator(this, size_);
    }

private:
    T* slot(size_t index) const {
        size_t at = head_ + index;
        return buffer_ + (at >= capacity_ ? at - capacity_ : at);
    }

    // Four items at first, then twice as many each time, as Rust's does.
    void grow() {
        grow_to(capacity_ == 0 ? 4 : capacity_ * 2);
    }

    void grow_to(size_t capacity) {
        T* bigger = std::allocator<T>().allocate(capacity);
        for (size_t index = 0; index < size_; index++) {
            T* item = slot(index);
            std::construct_at(bigger + index, std::move(*item));
            std::destroy_at(item);
        }
        if (buffer_ != nullptr) {
            std::allocator<T>().deallocate(buffer_, capacity_);
        }
        buffer_ = bigger;
        capacity_ = capacity;
        head_ = 0;
    }

    T* buffer_ = nullptr;
    size_t capacity_ = 0;
    // Where the first item is in the buffer.
    size_t head_ = 0;
    size_t size_ = 0;
};

} // namespace base
