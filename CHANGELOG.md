# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project does not yet have tagged semantic releases; all historical work is grouped by development milestone under `[Unreleased]`.

---

## [Unreleased]

### Overloaded Operators: Postfix `++`/`--`, Arrow `->`, Dereference `*`, and Functor `()`
- **Feature**: Implemented support for overloaded `operator*()`, `operator->`, `operator()`, and postfix `operator++(int)`/`operator--(int)`.
- **Semantics**:
  - Dereference `operator*()` allows classes to behave as smart pointers and iterators with lvalue reference writes (`*ptr = val`) and rvalue loads.
  - Arrow member access `operator->()` delegates field access and method calls to the pointed-to object.
  - Functor call `operator()` desugars `obj(...)` invocations directly into member call dispatches with overload resolution on const and non-const receivers.
  - Postfix `operator++(int)` and `operator--(int)` distinguish postfix from prefix overloads via dummy `int` parameter resolution.
- **Diagnostics**: Added `Arrow member access on non-pointer` diagnostic when `->` is invoked on a class instance that does not define `operator->()`.
- **Tests**: Added comprehensive test cases in `tests/classes_oop/operators/operator_smart_pointer.cpp`, `tests/classes_oop/operators/operator_call_functor.cpp`, `tests/classes_oop/operators/operator_postfix_incdec.cpp`, and negative test `tests/negative/operator_arrow_invalid.cpp`.

### Range-Based `for` Loops
- **Feature**: Implemented C++11 range-based `for` loops (`for (range_decl : range_expr) body`).
- **Semantics**: Added semantic desugaring for fixed-size 1D and multi-dimensional C-style arrays into pointer interval bounds (`__begin`, `__end`).
- **Standard Library**: Supported container iteration via member `.begin()` and `.end()` calls (e.g. `std::string`).
- **Type Deduction**: Extended local variable type deduction for `auto`, `const auto`, `auto&`, and `const auto&` loop bindings.
- **Diagnostics**: Added validation error `"range expression must be an array or class with begin() and end()"` for non-iterable types.
- **Tests**: Added tests for array value iteration and reference mutation (`tests/c_basics/pointers_arrays/range_for_array.cpp`), string iteration (`tests/mini_stl/range_for_string.cpp`), RAII cleanups on `break`/`continue`/`return` (`tests/classes_oop/memory_raii/range_for_raii.cpp`), and non-iterable negative diagnostics (`tests/negative/range_for_non_iterable.cpp`).

### Comma-Separated Multi-Declarators & Parser Deduplication
- **Feature**: Added support for comma-separated multi-declarator declarations sharing base specifiers (e.g. `int *p, q = 2;`, `Pair one(1), two(5);`, `static int a = 1, b = 2;`).
- **Refactor**: Deduplicated parser grammar rules across function-pointer declarators, parenthesized expression lists, name qualification, and qualified-name tails.
- **Tests**: Cleaned up redundant parser nesting and memory layout unit tests.

### Code Quality ("Slop") Cleanup & Invariant Hardening
- **Refactor**: Cleaned up dead branches, redundant defensive checks, leaked field prose in comments, and duplicate dispatch across parser, sema, and codegen.
- **Robustness**: Replaced fallback layout assumptions at guaranteed compile-time sites; converted operator lookup to use unified `find_call_candidates`.
- **Naming**: Updated `SwitchSlots` with explicit `any_match_slot` fields so names and values align.
- **Standard Library**: Cleaned up dead stdlib helpers; deduplicated integer stream input parsing via helper functions; centralized string reallocation using `_ensure_cap`; extracted AVL rebalancing.
- **Diagnostics**: Restructured error positions to report the exact offending expressions and tokens rather than statement starts.

### Browser Playground & WebAssembly Embedding
- **Feature**: Added an in-browser playground (`web/playground.html`) that compiles and runs C++ directly in the browser.
- **Architecture**: Decoupled compilation into a dedicated Web Worker (`web/playground-worker.js`) to prevent blocking the UI thread.
- **Telemetry**: Added execution timing telemetry to playground runs (compiler load time, compilation time, instantiation and execution time).
- **Deployment**: Added GitHub Actions workflow (`.github/workflows/pages.yml`) deploying static playground assets to GitHub Pages.
- **Refactor**: Simplified web deployment assets to serve the raw WebAssembly module directly without extraneous packaging.

### Diagnostics Precision, Name Lookup & Overload Resolution
- **Diagnostics**: Attached exact source coordinates to syntax and semantic diagnostics, avoiding fabricated line/column numbers.
- **Semantics**: Added ambiguity diagnostics when names are ambiguous across `using` directives or when overload candidate scores tie.
- **Type Checking**: Added strict parameter conversion checks for constructor invocations and function calls.
- **Optimization**: Implemented C++17 guaranteed copy elision (in-place construction from prvalues for direct initializations and returns).
- **Lookup**: Fixed member lookup to properly prioritize derived class fields when base class fields are shadowed.

### Preprocessor Expansion, Nesting Depth & Safety
- **Feature**: Implemented full macro expansion with standard painted-blue hide sets, object and function macro replacement rescanning, and `#` stringizing / `##` token pasting.
- **Feature**: Implemented integer constant expression evaluation for `#if` and `#elif` directives, including `defined()`, arithmetic, comparison, shift, and ternary `?:` operations.
- **Safety**: Replaced global recursion limits with per-construct depth counters (`src/depth.rs`) for expressions, parentheses, call arguments, statements, compound blocks, declarations, type expressions, and preprocessor conditionals.
- **Limits**: Calibrated nesting limits against WebAssembly engine shadow stack boundaries on Node/V8 and verified parity with the native CLI.
- **Line Splicing**: Implemented preprocessor backslash-newline continuation splicing (`\`).
- **Directives**: Implemented `#error` (emits error diagnostic) and `#warning` (emits stderr warning without failing build).

### Printf Formatting & Mini Standard Library Runtime
- **Runtime**: Implemented WebAssembly-lowered `printf` runtime supporting `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, and `%G`.
- **Flags**: Added support for `h`, `hh`, `l`, `ll`, and `z` length modifiers and flags `0`, `-`, `+`, space, and `#`.
- **Differential Verification**: Generated comprehensive test matrices verified against native host `g++` and `clang++` output.
- **Memory**: Verified allocator heap growth and free-list block reuse up to 64 MiB linear memory limits.

### C ABI & Host Interface Lifecycle
- **C ABI**: Implemented explicit C ABI buffer lifecycle functions (`alloc_buffer`, `dealloc_buffer`, `compile_wasm`, `get_result_len`, `free_result`, `get_error_ptr`, `get_error_len`).
- **Safety**: Added lifecycle tests proving repeated compilations keep linear memory usage strictly bounded without leaks.
- **Host Shim**: Unified browser and Node.js host import environment in `web/host.js` (`print_char`, `print_int`, `print_double`, `read_char`, `exit`).

### Modular Architecture & Function Right-Sizing
- **Refactor**: Split oversized functions across the compiler to ensure no function exceeds 80 lines and no file exceeds 500 lines.
- **Context Structs**: Grouped loose parameter lists into structured contexts (`CallSite`, `MemberAccess`, `NamedCall`, `SwitchSlots`, `CtorCall`).
- **Census Scanner**: Added `cppwasm-census` tool to automatically check repository code metrics (lines per function, parameter counts, file sizes).
- **Module Tree**: Added integration test ensuring every `.rs` file in `src/` is reachable from the crate module tree.

### Test Harness, Variant Gates & Golden Baselines
- **Harness**: Developed `cppwasm-testrunner` executing tests grouped by feature directories with expected output and error checks.
- **Golden Baseline**: Implemented `tests/golden.txt` cryptographic SHA-256 baseline pinning emitted WebAssembly modules and stdout across positive tests.
- **Anti-Overfitting Variants**: Created `_variant` suites with structural divergence gates preventing trivial identifier-only copy-paste tests.
- **Native Oracle**: Implemented native oracle gate compiling corpus tests through `g++` and `clang++` to guarantee standards conformance.
- **Cleanroom Independent Corpus**: Established `tests/independent` cleanroom test harness pinned by cryptographic manifest.

### Initial Compiler Pipeline & Single Cargo Package
- **Architecture**: Flattened the initial multi-crate workspace into a single, cohesive Cargo package with a core library and CLI/test runner binaries.
- **Pipeline**: Established direct WebAssembly emission pipeline: `preprocessor -> lexer -> parser -> sema -> wasm codegen`.
- **Target ABI**: Defined ILP32 target model (32-bit pointers, 64 KiB guard page, 8 MiB downward stack, upward heap).
