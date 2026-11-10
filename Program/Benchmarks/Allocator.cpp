// The heap counts of every benchmark that asks for them, from one allocator
// for the whole program: the global operator new and operator delete, in
// every form that a program may replace, which is what #[global_allocator] is
// in Rust. Each does what the C runtime's own does, malloc and free, which
// in Release are HeapAlloc and HeapFree on the process's heap, as Rust's
// System allocator is, and tells Memory.cpp of each block. It counts nothing
// until a memory benchmark turns it on, so the timing benchmarks pay only a
// relaxed load an allocation.
//
// This file isn't in BenchmarksLibrary. A replacement must be among a
// program's own objects: the linker takes an object out of a library only
// for a name that nothing has defined yet, and the C runtime's library
// defines these too. So Benchmarks.vcxproj and BenchmarksTests.vcxproj each
// compile it, and no other program gets it.
//
// A block's bytes are the bytes that were asked for. Rust's allocator is
// told them again when the block is freed. C++'s is only where the compiler
// knows them, which is the operator delete with a size: for the rest, like
// the delete[] of an array of numbers, the C runtime's heap says how many
// were asked for, with _msize, and only while counting is on.
//
// What counts is what the heap is asked for, which isn't always what the
// program asked a container for: the standard library asks for 39 bytes
// more than a vector or a string of 4096 bytes or more holds, to align it
// to 32 itself, and in a Debug build for 47, and each container also
// allocates a proxy for its checked iterators there. So the counts mean
// something in Release.

#include "Benchmarks/Memory.h"

#include <crtdbg.h>
#include <malloc.h>

#include <cstddef>
#include <cstdlib>
#include <new>

namespace {

namespace memory = benchmarks::memory;

// A block from malloc, or from _aligned_malloc for an alignment that malloc
// doesn't give. Null if there's no memory and no new handler to ask for
// some, or the handler throws, as it does to say that there's none.
void* allocate(std::size_t size) {
    while (true) {
        void* pointer = std::malloc(size);
        if (pointer != nullptr) {
            memory::allocated(size);
            return pointer;
        }
        std::new_handler handler = std::get_new_handler();
        if (handler == nullptr) {
            return nullptr;
        }
        handler();
    }
}

void* allocate(std::size_t size, std::align_val_t alignment) {
    while (true) {
        void* pointer = _aligned_malloc(size, static_cast<std::size_t>(alignment));
        if (pointer != nullptr) {
            memory::allocated(size);
            return pointer;
        }
        std::new_handler handler = std::get_new_handler();
        if (handler == nullptr) {
            return nullptr;
        }
        handler();
    }
}

void* allocate_or_throw(std::size_t size) {
    void* pointer = allocate(size);
    if (pointer == nullptr) {
        throw std::bad_alloc();
    }
    return pointer;
}

void* allocate_or_throw(std::size_t size, std::align_val_t alignment) {
    void* pointer = allocate(size, alignment);
    if (pointer == nullptr) {
        throw std::bad_alloc();
    }
    return pointer;
}

void* allocate_or_null(std::size_t size) noexcept {
    try {
        return allocate(size);
    } catch (...) {
        return nullptr;
    }
}

void* allocate_or_null(std::size_t size, std::align_val_t alignment) noexcept {
    try {
        return allocate(size, alignment);
    } catch (...) {
        return nullptr;
    }
}

// Gives a block back to the heap that malloc took it from.
void give_back(void* pointer) noexcept {
#ifdef _DEBUG
    // As the debug C runtime's own operator delete frees, so that a block
    // that the runtime's libraries allocated may come back here.
    _free_dbg(pointer, _UNKNOWN_BLOCK);
#else
    std::free(pointer);
#endif
}

// Frees a block of a size that the compiler said.
void release(void* pointer, std::size_t size) noexcept {
    if (pointer == nullptr) {
        return;
    }
    memory::freed(size);
    give_back(pointer);
}

// Frees a block of the size that the heap says it was asked for.
void release(void* pointer) noexcept {
    if (pointer == nullptr) {
        return;
    }
    if (memory::is_counting()) {
        memory::freed(_msize(pointer));
    }
    give_back(pointer);
}

void release(void* pointer, std::size_t size, std::align_val_t) noexcept {
    if (pointer == nullptr) {
        return;
    }
    memory::freed(size);
    _aligned_free(pointer);
}

void release(void* pointer, std::align_val_t alignment) noexcept {
    if (pointer == nullptr) {
        return;
    }
    if (memory::is_counting()) {
        memory::freed(_aligned_msize(pointer, static_cast<std::size_t>(alignment), 0));
    }
    _aligned_free(pointer);
}

} // namespace

void* operator new(std::size_t size) {
    return allocate_or_throw(size);
}

void* operator new[](std::size_t size) {
    return allocate_or_throw(size);
}

void* operator new(std::size_t size, std::align_val_t alignment) {
    return allocate_or_throw(size, alignment);
}

void* operator new[](std::size_t size, std::align_val_t alignment) {
    return allocate_or_throw(size, alignment);
}

void* operator new(std::size_t size, const std::nothrow_t&) noexcept {
    return allocate_or_null(size);
}

void* operator new[](std::size_t size, const std::nothrow_t&) noexcept {
    return allocate_or_null(size);
}

void* operator new(std::size_t size, std::align_val_t alignment, const std::nothrow_t&) noexcept {
    return allocate_or_null(size, alignment);
}

void* operator new[](std::size_t size, std::align_val_t alignment, const std::nothrow_t&) noexcept {
    return allocate_or_null(size, alignment);
}

void operator delete(void* pointer) noexcept {
    release(pointer);
}

void operator delete[](void* pointer) noexcept {
    release(pointer);
}

void operator delete(void* pointer, std::size_t size) noexcept {
    release(pointer, size);
}

void operator delete[](void* pointer, std::size_t size) noexcept {
    release(pointer, size);
}

void operator delete(void* pointer, std::align_val_t alignment) noexcept {
    release(pointer, alignment);
}

void operator delete[](void* pointer, std::align_val_t alignment) noexcept {
    release(pointer, alignment);
}

void operator delete(void* pointer, std::size_t size, std::align_val_t alignment) noexcept {
    release(pointer, size, alignment);
}

void operator delete[](void* pointer, std::size_t size, std::align_val_t alignment) noexcept {
    release(pointer, size, alignment);
}

// What a new expression calls when the constructor of what it made throws.
void operator delete(void* pointer, const std::nothrow_t&) noexcept {
    release(pointer);
}

void operator delete[](void* pointer, const std::nothrow_t&) noexcept {
    release(pointer);
}

void operator delete(void* pointer, std::align_val_t alignment, const std::nothrow_t&) noexcept {
    release(pointer, alignment);
}

void operator delete[](void* pointer, std::align_val_t alignment, const std::nothrow_t&) noexcept {
    release(pointer, alignment);
}
