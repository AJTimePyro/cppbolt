# `<stdexcept>`: LLVM libc++ 23.1.5 Integration

## Upstream source

- Tag: `llvmorg-23.1.5` (libc++ 23.1.5)
- Umbrella header: `libcxx/include/stdexcept`
- License: Apache License v2.0 with LLVM Exceptions (retained in the imported header; `SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception`).

## Integration approach

The header implements the full ISO C++17 standard exception hierarchy adapted for cppbolt:

1. `stdlib/stdexcept` declares both `std::logic_error` and `std::runtime_error` subtrees, each rooted in `std::exception`.
2. Message strings are stored in an internal `std::string` buffer.
3. Overridden virtual `what()` methods return `__msg_.c_str()`.

### Supported exception classes

The complete C++17 hierarchy is available:

- `std::logic_error` (derived from `std::exception`)
  - `std::domain_error`
  - `std::invalid_argument`
  - `std::length_error`
  - `std::out_of_range`
- `std::runtime_error` (derived from `std::exception`)
  - `std::range_error`
  - `std::overflow_error`
  - `std::underflow_error`

### Lifetime and dispatch semantics

Verified against native Clang and GCC using automated test suites:

- String and C-string constructors: Every class provides explicit constructors accepting `const std::string&` and `const char*`.
- Virtual method resolution: Base classes define virtual `what() const noexcept`. Overrides in `logic_error` and `runtime_error` dispatch correctly through vtables even when caught through base references (`catch (const std::exception&)`).
- Throw allocation and ownership: When an exception object is thrown, the WebAssembly runtime allocates heap storage and initializes the payload with value or move semantics. Destructors run when the handling catch clause finishes, preventing memory leaks in exception messages.
- Nested propagation: Throwing a new exception inside an active catch block destroys the previous exception object before propagating the new one.
