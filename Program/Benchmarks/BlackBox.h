// black_box, which Rust has as std::hint::black_box and C++ doesn't: a value
// that the optimizer must make, whole, in memory, and can't know anything
// about afterwards, so that it can't drop the work that made it, or work the
// answer out as it compiles. Every benchmark's result goes through it.
//
// The value's address goes into a volatile pointer, and comes back out of it.
// The compiler has to write and read a volatile each time, and can't tell
// that what it reads is what it wrote: so the value must be in memory, where
// the pointer that comes back may point, and whatever uses the value
// afterwards reads it from there. That's two moves of a pointer, and no call,
// like Rust's, which hands the value's address to an empty block of assembly.
// A call of a function that the compiler can't see into would do too, but
// Release optimizes the whole program as it links, so there's no such
// function short of one behind a pointer, and a call clobbers registers that
// a short loop's body keeps its values in, which Rust's doesn't.

#pragma once

#include <memory>
#include <type_traits>

namespace benchmarks {

namespace detail {

// Where black_box leaves a value's address, and takes it back from.
inline const volatile void* volatile black_box_slot = nullptr;

} // namespace detail

// The value, as far as the optimizer knows changed into any other. It's the
// same object that comes back, not a copy: a temporary lasts until the end
// of its statement, as it would without this.
template <class T> T&& black_box(T&& value) {
    detail::black_box_slot = std::addressof(value);
    return static_cast<T&&>(*static_cast<std::remove_reference_t<T>*>(const_cast<void*>(detail::black_box_slot)));
}

} // namespace benchmarks
