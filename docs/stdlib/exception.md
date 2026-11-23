# `<exception>`: LLVM libc++ 23.1.5 Integration

## Upstream source

- Tag: `llvmorg-23.1.5` (libc++ 23.1.5)
- Umbrella header: `libcxx/include/exception`
- Implementation: `libcxx/include/__exception/exception.h`
- License: Apache License v2.0 with LLVM Exceptions (retained in the imported header; `SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception`).

## Integration approach

The header provides C++17 exception facilities adapted for cppbolt's WebAssembly exception runtime:

1. `stdlib/exception` provides `std::exception`, `std::bad_exception`, terminate handler controls, and uncaught exception queries.
2. `stdlib/compat/compiler/compiler.h` provides visibility and export macros (`_LIBCPP_EXPORTED_FROM_ABI`, `_LIBCPP_HIDDEN`, `_LIBCPP_NORETURN`).
3. `stdlib/compat/runtime/runtime.h` provides runtime process abort (`abort()` invoking `exit(134)`).

### Supported facilities

Verified against native Clang and GCC using automated test suites:

- `std::exception`: Polymorphic base class with a virtual destructor and virtual `what()` method returning `"std::exception"`.
- `std::bad_exception`: Derived from `std::exception` with virtual `what()` returning `"std::bad_exception"`.
- `std::terminate_handler`: Function pointer type `void (*)()`.
- `std::terminate()`: Invokes the active terminate handler or calls `abort()` directly. Runtime terminations (`noexcept` violations, `throw;` with no active exception, a destructor throwing during unwinding, and exceptions escaping `main`) route through the same handler via `__cppbolt_terminate`.
- `std::set_terminate(handler)`: Installs a custom terminate handler and returns the previous handler. Passing `nullptr` restores the default handler.
- `std::get_terminate()`: Returns the currently installed terminate handler.
- `std::uncaught_exceptions()`: Returns the number of exceptions thrown but not yet caught. A mutable WebAssembly global is incremented at `throw` and `throw;` (rethrow), and decremented at catch-handler entry, so destructors running during unwinding observe the active count.
- `std::uncaught_exception()`: Returns whether `std::uncaught_exceptions()` is nonzero (deprecated in C++17).

### Compiler requirements

Semantic analysis and WebAssembly code generation enforce the following rules:

- Canonical type identification: Semantic layout assigns canonical unqualified type IDs to classes declared in `namespace std`, ensuring `exception` and `std::exception` resolve to the exact same numeric type ID.
- Dynamic polymorphic catch matching: The WebAssembly runtime evaluates base inheritance via `__cppbolt_can_catch`, matching derived exceptions against `catch (const std::exception&)` handlers.
- Copy construction and slicing: Catching `std::exception` by value invokes copy constructors and slices derived state cleanly. Catching by reference preserves derived object state and virtual method dispatch.

### Deferred facilities

- `std::exception_ptr`, `std::make_exception_ptr`, `std::current_exception`, and `std::rethrow_exception` remain deferred because they require reference-counted `exnref` transport or RTTI (excluded under ADR 0005).
- `std::nested_exception`, `std::throw_with_nested`, and `std::rethrow_if_nested` remain deferred because `dynamic_cast` is excluded under ADR 0005.
