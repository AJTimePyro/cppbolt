# Project Roadmap

This roadmap outlines active priorities, medium-term enhancements, and planned language extensions for `cppwasm`. Items are tracked against the test suite and feature matrix in [Supported C++](docs/book/user/supported-cpp.md).

For current baseline results, see [`tests/PASSING.txt`](tests/PASSING.txt) and run the test runner (`cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli`).

---

## 1. Compiler Core & Diagnostics

- [ ] **Arena-Based Semantic State**: Replace string-keyed semantic symbol tables with typed arena IDs (`FileId`, `DeclId`, `TypeId`) to eliminate redundant allocations and accelerate symbol resolution.
- [ ] **Macro Source-Map Fidelity**: Complete source-map span tracking through nested macro expansions so compiler diagnostics point to expansion invocation sites as well as definition locations.
- [ ] **Per-Function Stack Frame Layouts**: Replace conservative frame/scratch allocations in [`src/codegen/function_context.rs`](src/codegen/function_context.rs) with measured per-function layouts based on live variable analysis.
- [ ] **Value Category Normalization**: Formally represent C++17 value categories (prvalue, xvalue, lvalue) in the typed AST to further refine temporary object lifecycles and move semantics.

---

## 2. Language Feature Expansion (v1 Subset Additions)

These features build naturally on existing language constructs without requiring a full template engine:

- [x] **Range-Based `for` Loops**: Lower `for (decl : range)` into standard iterator loops using `begin()` / `end()` or pointer intervals across fixed-size 1D/multidimensional arrays and container classes (e.g. `std::string`).
- [x] **C++11/C++17 List-Initialization (Brace Initialization)**: Direct-list and copy-list-initialization, empty brace value initialization (`{}`), C++17 aggregates with base subobjects and brace elision, narrowing conversion rejection ([dcl.init.list] p7), dynamic allocation (`new T{...}`, `new T[n]{...}` with loop value-initialization of omitted elements), typed temporaries (`Type{...}.get()`), and explicit constructor enforcement.
- [x] **Additional Operator Overloads**:
  - Overloaded postfix `operator++(int)` and `operator--(int)` distinguishing from prefix via dummy `int` parameter.
  - Overloaded pointer member access `operator->`.
  - Overloaded dereference `operator*()`.
  - Overloaded function call `operator()` (functors).
- [ ] **Namespace Aliases**: Parse and resolve `namespace NewName = OriginalName;` declarations (currently rejected in `src/parser/unsupported.rs`).
- [ ] **Extended Casts**: Support `reinterpret_cast` across scalar and pointer types within the ILP32 model.

---

## 3. Standard Library & Templates (Post-v1)

Templates and standard containers remain the primary boundary between the v1 compiler subset and full C++17 support:

- [ ] **Template Support**:
  - Function templates and class templates with type parameters (`template <typename T>`).
  - Template specialization and method instantiation.
  - `std::initializer_list<T>` constructors and overload resolution priority.
  - *Target*: Resolve expected failures currently tracked in [`tests/templates_operators`](tests/templates_operators/).
- [ ] **Standard Containers & Algorithms**:
  - Full templated implementations of `<vector>`, `<map>`, `<set>`, and `<utility>` (`std::pair`).
  - `<algorithm>` support (`std::sort`, `std::find`, `std::min`, `std::max`).
  - *Target*: Resolve expected container failures currently tracked in [`tests/mini_stl`](tests/mini_stl/).

---

## 4. WebAssembly & Tooling Ecosystem

- [ ] **Source Maps for Web Debugging**: Emit WebAssembly DWARF or source-map sections mapping browser execution back to original C++ source lines in DevTools.
- [ ] **WebAssembly SIMD (128-bit)**: Support WebAssembly vector instructions (`v128`) for vectorizable numeric loops.
- [ ] **Continuous Performance Benchmarks**: Automate tracking of execution time and binary size using the existing benchmarks in [`benchmarks/`](benchmarks/).
