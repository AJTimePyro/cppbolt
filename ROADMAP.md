# Project Roadmap

This roadmap outlines active priorities, medium-term enhancements, and planned language extensions for `cppbolt`. Items are tracked against the test suite and feature matrix in [Supported C++](docs/book/user/supported-cpp.md).

For current baseline results, see [`tests/PASSING.txt`](tests/PASSING.txt) and run the test runner (`cargo run --bin cppbolt-testrunner -- target/release/cppbolt-cli`).

---

## 1. Compiler Core & Diagnostics

- [ ] **Arena-Based Semantic State**: Replace string-keyed semantic symbol tables with typed arena IDs (`FileId`, `DeclId`, `TypeId`) to eliminate redundant allocations and accelerate symbol resolution.
- [ ] **Macro Source-Map Fidelity**: Complete source-map span tracking through nested macro expansions so compiler diagnostics point to expansion invocation sites as well as definition locations.
- [ ] **Per-Function Stack Frame Layouts**: Replace conservative frame/scratch allocations in [`src/codegen/function_context.rs`](src/codegen/function_context.rs) with measured per-function layouts based on live variable analysis.
- [x] **Value Category Normalization**: Formally represent C++17 value categories (prvalue, xvalue, lvalue) in expression analysis to refine temporary object lifecycles and move semantics.

---

## 2. Language Feature Expansion (v1 Subset Additions)

These features build naturally on existing language constructs without requiring a full template engine:

- [x] **Move Semantics & Smart Pointers**: Rvalue references (`T&&`), value categories, move constructors, move assignment operators, forwarding references, reference collapsing, `std::move`, `std::forward`, `std::vector` move operations, and `std::unique_ptr` in `<memory>`.
- [x] **Range-Based `for` Loops**: Lower `for (decl : range)` into standard iterator loops using `begin()` / `end()` or pointer intervals across fixed-size 1D/multidimensional arrays and container classes (e.g. `std::string`).
- [x] **List Initialization & Aggregates**: Direct-list and copy-list initialization, value initialization with empty braces (`{}`), aggregate base subobjects and brace elision, narrowing conversion checks, dynamic array allocations (`new T[n]{...}`), and typed temporaries.
- [x] **Additional Operator Overloads**:
  - Overloaded postfix `operator++(int)` and `operator--(int)` distinguishing from prefix via dummy `int` parameter.
  - Overloaded pointer member access `operator->`.
  - Overloaded dereference `operator*()`.
  - Overloaded function call `operator()` (functors).
- [x] **Namespace Aliases**: Parse and resolve `namespace NewName = OriginalName;` declarations at file, namespace, and block scopes with chained and qualified lookup resolution.
- [x] **Extended Casts**: Support `reinterpret_cast` across scalar and pointer types within the ILP32 model.
- [x] **Comma Expressions**: Parse and lower sequence operator `(a, b)` at precedence 1 without conflicting with argument lists or declarators.
- [x] **C++17 Selection Initializers**: Parse and desugar `if (init; cond)` and `switch (init; cond)` with scope isolated to the selection body.
- [x] **Non-Static Data Member Initializers (NSDMI)**: Support in-class field defaults (`= expr`, `{ ... }`) in constructors and aggregate initialization.
- [x] **Pointers to Members & Multiple Inheritance**: C++17 data-member and member-function pointers (`T C::*`, `R (C::*)()`), `.*`/`->*` application and fold expressions, virtual and multiple-inheritance subobject adjustments, base-to-derived conversions, and direct/indirect base layout.

---

## 3. Standard Library & Templates (Post-v1)

Templates and standard containers implemented in cppbolt:

- [x] **Template Core & Instantiation**: Function templates and class templates with type parameters (`template <typename T>`), lazy member instantiation, and default template arguments.
- [x] **Specialization & NTTP**: Class template explicit (full) and partial specialization, non-type template parameters (NTTP), compile-time constant expression folding, and alias templates.
- [x] **Initializer Lists**: `std::initializer_list<T>` constructors, brace initialization, and aggregate deduction.
- [x] **Standard Containers**: Templated implementations of `<vector>`, `<map>`, `<set>`, `<array>`, and `<utility>` (`std::pair`).
- [x] **Algorithms & Traits**: `<algorithm>` (`std::sort`, `std::find`, `std::min`, `std::max`, `std::copy`, `std::reverse`) and `<type_traits>` (C++17 subset adapted from LLVM libc++ 23.1.5, covering `integral_constant`, `is_same`, `enable_if`, `conditional`, `void_t`, cv/reference/pointer/array transforms, and `_v`/`_t` aliases).
- [x] **Substitution Failure (SFINAE) & `decltype`**: Signature substitution failure handling in the immediate context, expression SFINAE with `decltype(expr)`, trailing return types (`auto f(T x) -> decltype(x.member())`), diagnostics listing candidate rejection reasons, and partial-specialization matching with SFINAE fallback.
- [x] **Two-Phase Name Lookup & ADL**: Definition-time binding snapshots for unqualified lookups in templates, argument-dependent lookup (ADL) from associated namespaces and classes at instantiation, dependent call and operator classification, dependent local variable tracking (`auto x = t;`), and friend function templates.
- [x] **Variadic Templates**: Parameter packs (`template <typename... Args>`), pack expansions in call arguments, initializer lists, base classes, constructor initializers, and template arguments, `sizeof...`, forwarding reference packs, partial ordering of function templates with packs, `std::tuple`, and variadic `std::make_unique`.
- [x] **Fold Expressions**: C++17 unary and binary fold expressions (`(... + args)`, `(args + ...)`) over parameter packs.
- [x] **Template Template Parameters**: Templated parameters (`template <template <typename> class C>`).

---

## 4. WebAssembly & Tooling Ecosystem

- [ ] **Source Maps for Web Debugging**: Emit WebAssembly DWARF or source-map sections mapping browser execution back to original C++ source lines in DevTools.
- [ ] **WebAssembly SIMD (128-bit)**: Support WebAssembly vector instructions (`v128`) for vectorizable numeric loops.
- [ ] **Continuous Performance Benchmarks**: Automate tracking of execution time and binary size using the existing benchmarks in [`benchmarks/`](benchmarks/).
