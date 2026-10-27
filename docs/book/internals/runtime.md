# Runtime & Embedded Standard Library

`cppwasm` provides a self-contained runtime environment comprising an ILP32 linear memory layout, an in-module WebAssembly free-list allocator, an embedded `printf` formatting engine, and compile-time embedded standard library headers.

---

## 1. Linear Memory Model & ABI

Programs compiled with `cppwasm` use 32-bit ILP32 addresses. Linear memory is partitioned into four distinct segments defined in `src/runtime/memory_layout.rs`:

```text
 +--------------------------+ 0x000000 (0)
 | Reserved Guard Region    | 64 KiB
 | - Address 0 is Null ptr  |
 +--------------------------+ 0x010000 (65,536)
 |                          |
 | Downward-Growing Stack   | 8 MiB
 |                          |
 |                          |
 +--------------------------+ 0x810000 (8,454,144 = STACK_TOP / DATA_BASE)
 | Static Data Segment      |
 | - String literals        | Variable size
 | - Global/static vars     |
 | - Vtables                |
 +--------------------------+
 |                          |
 | Upward-Growing Heap      | Expands via memory.grow
 | (Managed by allocator)   | up to max_memory_mb (default 256 MiB)
 |                          |
 +--------------------------+
```

### Safety & Invariants
- **Stack Overflow Guard**: Frame allocation in function prologues verifies that the decremented stack pointer remains strictly greater than `65,536` (`GUARD_PAGE_END`). Entering the guard region causes an immediate WebAssembly `unreachable` trap (`tests/runtime_trap/stack_overflow.cpp`).
- **Function Table Slot 0**: Index 0 of the WebAssembly indirect call table (`funcref`) is explicitly reserved for a trapping stub. Indirect function calls through a null function pointer trap deterministically (`tests/runtime_trap/null_function_pointer.cpp`).
- **Virtual Method Dispatch**: Objects of polymorphic classes place their vtable pointer (`vptr`) at byte offset 0. Table slots strictly follow base-prefix order.

---

## 2. In-Module Free-List Memory Allocator

Dynamic memory allocation is managed by an in-module free-list allocator implemented directly in WebAssembly instructions (`src/runtime/allocator.rs`):

- **Block Structure**: Free and allocated memory chunks are prefixed with an 8-byte aligned header storing the block size and allocation flag.
- **Coalescing**: Adjacent free blocks are automatically merged on `free` to prevent heap fragmentation.
- **Heap Growth**: When existing free blocks cannot satisfy a request, the allocator issues `memory.grow` to expand linear memory up to the configured limit.
- **Allocation Failure Semantics**:
  - `malloc(size)` returns `0` (null pointer) on allocation failure.
  - `operator new` checks the result of `malloc` and triggers an `unreachable` WebAssembly trap if null is returned (since exceptions like `std::bad_alloc` are excluded in v1 per ADR 0005).

Verified by `tests/memory_model/allocator_correctness.cpp`, `tests/memory_model/free_list_reuse.cpp`, and `tests/runtime_trap/malloc_beyond_max.cpp`.

---

## 3. Embedded Standard Library (`stdlib/`)

Headers under `stdlib/` are embedded into the compiler binary at compile time using Rust's `include_str!` macro ([ADR 0004](../adr/0004-embedded-stdlib.md)).

```text
stdlib/
├── iostream          # std::cout, std::cin, streams
├── string            # std::string implementation
├── cstdio, stdio.h   # printf, sprintf, snprintf, puts
├── cstdlib, stdlib.h # malloc, free, abs, exit
├── cmath             # sqrt, sin, cos, abs, etc.
├── iomanip           # std::setprecision, std::fixed, std::setw
├── new               # operator new, placement new
├── utility           # std::move, std::forward
├── initializer_list  # std::initializer_list
└── cstddef           # size_t, ptrdiff_t, NULL
```

### Present vs. Supported Split
- **Supported Headers**: Streams (`<iostream>`), string manipulation (`<string>`), C stdio (`<cstdio>`), math (`<cmath>`), manipulators (`<iomanip>`), and memory allocation (`<new>`, `<cstdlib>`). Fully covered by corpus tests.
- **Partial Headers**: `<vector>`, `<map>`, `<set>`, and `<algorithm>`. These files are present in `stdlib/` to compile specific mini-STL test fixtures, but full template container functionality is outside the v1 subset (tracked as Planned).

### Independent Native Header Verification
All embedded headers are compiled and verified against host `g++` and `clang++` using `-nostdinc++` and AddressSanitizer/UndefinedBehaviorSanitizer via `tools/verify_mini_stl_gxx.py`.

### Standard I/O Contract
- **Shared Lookahead**: C stdio (`scanf`, `getchar`) and C++ streams (`cin >>`) share an internal character lookahead buffer to allow interleaved input.
- **Limitations**: Mixing C and C++ input after `std::ios::sync_with_stdio(false)` is outside the supported contract. Stream `unget()` is not supported.
