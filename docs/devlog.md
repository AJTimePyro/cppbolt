# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppwasm`.

---

## 2026-10-28: C++17 Lambda Expressions & Recursive operator-> Chaining

### Context & Motivation
Following the implementation of foundational operator overloads, modern C++ patterns require two major language capabilities: C++17 lambda expressions (anonymous closures, variable captures, return type deduction, functor invocation, and function pointer conversions) and full recursive chaining of `operator->()` (smart pointer wrappers delegating to other smart pointers until reaching a concrete target). Additionally, unassigned class prvalue expressions (`Value().get()`) exposed a missing constructor synthesis path for classes without explicit constructors.

### Work Executed
1. **Recursive `operator->()` Chaining & Cycle Detection**:
   - Updated `src/sema/expr_member.rs` to iteratively resolve `operator->()` calls across intermediate wrapper classes until reaching a pointer to a struct or class.
   - Enforced cycle detection (`visited` set tracking) and a 128-hop delegation budget, producing clear diagnostics (`circular operator-> delegation on '<class>'` and `operator-> delegation nesting exceeds limit`).
   - Propagated receiver `const`-qualification across delegation hops and handled pointer reference returns (`Type::Reference(Type::Pointer(...))`).
   - Emitted recursive `OP_CALL` instructions in WebAssembly codegen (`src/codegen/expr_call_member.rs:emit_arrow_chain`) for field accesses and member method calls.
2. **Implicit Default Constructor Synthesis for Prvalues**:
   - Adjusted `src/sema/analyze.rs` to synthesize default constructors for classes without explicit constructors even when member fields do not strictly require non-trivial construction.
   - Enabled direct invocation of member functions on unassigned prvalue instances (`Value().get()`).
3. **C++17 Lambda Expressions AST & Parsing**:
   - Added `Expr::Lambda` AST variant and dedicated parser in `src/parser/expr_lambda.rs`.
   - Supported capture lists (`[]`, `[x]`, `[&x]`, `[=]`, `[&]`), parameters, optional trailing return types (`-> T`), and the `mutable` specifier.
4. **Closure Class Synthesis & Semantic Analysis**:
   - Lowered lambda expressions in `src/sema/expr_lambda.rs` by generating unique anonymous closure structs (`__lambda_N`).
   - Added capture members stored as by-value values or references, synthesized member initializers, and generated `operator()` with `const`-qualification by default (or non-const if `mutable`).
   - Implemented automatic return type deduction from return statements in lambda bodies when trailing return type is omitted.
   - Supported storing lambdas in `auto` variables and direct invocation expressions.
5. **Captureless Lambda Conversion to C Function Pointers**:
   - For captureless lambdas (`[]`), synthesized static `__invoke` trampoline methods and member conversion operators `__to_fn_ptr`.
   - Enabled implicit conversion from captureless lambdas to standard C function pointer types (`R (*)(P...)`) via WebAssembly function table dispatch, including return statement target conversion and unary `+` pointer decay (`+[](){}`).
6. **Semantic Conformance & Bug Fixes**:
   - Wrapped conversion calls in `analyze_return` when returning captureless lambdas to function pointer targets to avoid runtime table index traps.
   - Extended free-variable traversal (`FreeVarCollector`) to cover `RangeFor`, `StructuredBinding`, and nested `Lambda`/`New` expressions, with array by-value capture copying.
   - Enforced `const` qualification on fields accessed inside `const` methods, diagnosing mutation of by-value captures in non-mutable lambdas.
   - Enforced capture-default ordering in the parser (must appear first) and aligned reference field dereferencing in class member codegen.

### Verification
- 13 dedicated test suites added across `tests/classes_oop/` and `tests/negative/`:
  - `tests/classes_oop/operators/operator_arrow_chaining.cpp`
  - `tests/classes_oop/constructors_destructors/prvalue_default_constructor.cpp`
  - `tests/classes_oop/lambdas/lambda_basic.cpp`
  - `tests/classes_oop/lambdas/lambda_default_captures.cpp`
  - `tests/classes_oop/lambdas/lambda_function_pointer.cpp`
  - `tests/classes_oop/lambdas/lambda_return_fn_ptr.cpp`
  - `tests/classes_oop/lambdas/lambda_range_for_capture.cpp`
  - `tests/classes_oop/lambdas/lambda_unary_plus.cpp`
  - `tests/negative/operator_arrow_cycle.cpp`
  - `tests/negative/operator_arrow_self_cycle.cpp`
  - `tests/negative/operator_arrow_non_pointer.cpp`
  - `tests/negative/lambda_const_capture_mutation.cpp`
  - `tests/negative/lambda_out_of_order_default_capture.cpp`
- Full corpus expanded to 300 passing tests with zero regressions.
- Verified all census limits (functions <= 80 lines, files <= 500 lines) and clippy checks (`-D warnings`).

---

## 2026-10-28: C++17 Compatibility Fixes & Feature Completeness

### Context & Motivation
An audit of compiler compatibility against C++17 highlighted 9 critical areas across parser, semantic analysis, and WebAssembly code generation: array function parameter adjustment, constexpr global initializer reachability in dead-code elimination, delegating constructors, enum constants in switch case labels, scoped enums (`enum class`), user-defined conversion operators, ref-qualified member functions, structured bindings, and exception handling diagnostics.

### Work Executed
1. **Array Parameter Decay**:
   - Updated `src/parser/decl_func.rs` to parse array declarator suffixes (`int arr[]`, `int arr[N]`) in function parameter lists and adjust them to pointer types (`int* arr`) according to standard C++ parameter adjustment rules.
2. **Global Initializer Function Reachability**:
   - Re-exported `collect_expr_calls` and incorporated global variable initializer expressions into root call collection in `src/codegen/program_prepare.rs`. Functions evaluated at global initialization time are now marked reachable and preserved through dead-code elimination.
3. **Delegating Constructors**:
   - Added target constructor invocation lowering in `src/codegen/function_lifecycle.rs` and enforced mutual exclusivity between constructor delegation (`Point() : Point(5) {}`) and member initializers in `src/sema/stmt.rs`.
4. **Constant Expression Switch Cases**:
   - Extended `StmtKind::Case` in AST, parser, and sema to accept arbitrary constant expressions and named enum variants, evaluating them at compile time via `eval_const_int_expr`.
5. **Scoped Enums (`enum class`)**:
   - Added scoped enum declarations with distinct type identities, prevented enumerators from polluting enclosing scopes, and enforced qualified resolution (`Status::OK`) without implicit arithmetic conversion.
6. **User-Defined Conversion Operators**:
   - Added parsing for conversion operator declarations (`operator T() const`) in `src/parser/structs.rs`. Wired implicit conversion checks across assignments, initializations, and binary operator operands while respecting `explicit` qualification.
7. **Ref-Qualified Member Functions**:
   - Extended AST and parser to support `&` and `&&` ref-qualifiers on member methods (`get() &`, `get() &&`). Implemented receiver value-category dispatch (`is_lvalue()`) in semantic analysis and codegen overload resolution, mangling distinct symbols with `_R` and `_O`.
8. **Structured Bindings**:
   - Implemented `auto [a, b] = p;`, `auto&`, and `const auto&` for aggregate structs. Desugared into a hidden object/reference variable and individual member references, preserving copy vs reference semantics.
9. **Exception Handling Architectural Specification & Diagnostic**:
   - Analyzed WebAssembly MVP vs Wasm Exception Handling proposal requirements (Tag Section 13, RAII unwinding landing pads, RTTI matching). Formally rejected fake trap implementations, recognized `throw` in lexer and parser, and emitted an explicit target diagnostic.

### Verification
- 11 dedicated test suites added across `tests/c_basics/`, `tests/classes_oop/`, and `tests/negative/`.
- Full corpus expanded to 287 passing tests with zero manifest violations.
- All linter, unit test, and formatting gates passing cleanly.

---

## 2026-10-28: Implementation of Operator Overloading (`*`, `->`, `()`, postfix `++`/`--`)

### Context & Motivation
Following range-based `for` loops, the next priority from Section 2 of `ROADMAP.md` was implementing the missing operator overload quartet: dereference `operator*()`, pointer member access `operator->`, function call `operator()` (functors), and postfix `operator++(int)`/`operator--(int)`. These operators complete the fundamental machinery for smart pointers, iterators, and callable object abstractions in C++.

### Work Executed
1. **Parser Support**:
   - Removed legacy rejection windows in `src/parser/unsupported.rs` for `operator()`, `operator->`, `operator*`, and postfix `operator++(int)`/`operator--(int)`.
   - The token streams were already supported in `enums_operators.rs:68-103` and struct method declaration parsing.
2. **Semantic Analysis**:
   - Added overload resolution in `src/sema/expr_ops.rs` for unary dereference `UnaryOp::Deref` when operand is a class type.
   - Centralized struct class resolution for arrow member access `operator->` in `src/sema/expr_member.rs` and `src/sema/expr_call.rs`, delegating member fields and methods to the returned pointer type.
   - Implemented functor call desugaring in `src/sema/expr_call.rs`: any callable class instance in a call site (`obj(...)`) is rewritten to `obj.operator(...)` and matched against member overloads.
   - Added `find_method_with_param_count` in `src/sema/lookup.rs` to distinguish prefix `operator++()` (1 parameter including `this`) from postfix `operator++(int)` (2 parameters including `this`).
   - Hardened `QualifiedName::implicit_alias` and class/field lookup against infinite recursion on empty class name strings.
3. **WebAssembly Codegen**:
   - In `src/codegen/expr_unary.rs`, lowered overloaded `operator*()` and postfix `operator++`/`operator--` (emitting `i32.const 0` for the dummy int argument and registering temporary struct return values).
   - In `src/codegen/lvalue.rs` and `src/codegen/expr_call_member.rs`, routed arrow access through `operator->()` to evaluate the receiver address.
   - Updated dead code elimination in `src/codegen/analysis.rs` to track usages of `operator->`, `operator*`, and postfix `operator++`/`operator--`.
4. **Testing & Verification**:
   - Added positive test cases in `tests/classes_oop/operators/operator_smart_pointer.cpp`, `tests/classes_oop/operators/operator_call_functor.cpp`, and `tests/classes_oop/operators/operator_postfix_incdec.cpp`.
   - Added negative test in `tests/negative/operator_arrow_invalid.cpp`.
   - Updated `tests/PASSING.txt` and `tests/golden.txt` (268 passing corpus tests).

---

## 2026-10-27: Implementation of C++11 Range-Based `for` Loops

### Context & Motivation
Following completion of the foundational v1 compiler subset, language feature expansion prioritized high-impact convenience syntax that does not require a full template engine. Range-based `for` loops (`for (decl : range) body`) eliminate repetitive index and iterator boilerplate across arrays and container types.

### Work Executed
1. **Parser & AST**:
   - Added `StmtKind::RangeFor { decl, range, body }` to `src/ast/stmt.rs`.
   - Removed the unsupported feature check in `src/parser/unsupported.rs`.
   - Updated `parse_for_stmt` in `src/parser/stmt.rs` to detect declarations followed by `:` instead of `;`, parsing the range expression and body into `StmtKind::RangeFor`.
2. **Semantic Desugaring**:
   - Implemented `src/sema/stmt_range_for.rs` to validate the range expression and desugar it into a scoped compound statement wrapping a standard `StmtKind::For` loop.
   - For fixed-size 1D and multi-dimensional C-style arrays (`int[N]`, `int[M][N]`), lower to pointer bounds `[__begin, __end)` with element size stride.
   - For classes providing `.begin()` and `.end()` member methods (e.g. `std::string`), synthesize member calls `__range.begin()` and `__range.end()`.
   - Supported binding modes: by-value (`auto x`, `const auto x`, `int x`), by-reference (`auto& x`, `int& x` for in-place mutation), and const-reference (`const auto& x`).
   - Reused existing loop lowering, branch targets, and RAII cleanup mechanics in WebAssembly codegen without requiring new codegen instructions.
3. **Diagnostics & Testing**:
   - Added validation diagnostic `"range expression must be an array or class with begin() and end()"` for non-iterable types.
   - Verified RAII destructor invocations on early `break`, `continue`, and early `return`.
   - Added automated tests: `tests/c_basics/pointers_arrays/range_for_array.cpp`, `tests/mini_stl/range_for_string.cpp`, `tests/classes_oop/memory_raii/range_for_raii.cpp`, and `tests/negative/range_for_non_iterable.cpp`.

---

## 2026-10-27: Comprehensive Documentation Architecture Restructure

### Context & Motivation
As the compiler matured through multiple refactoring phases—flattening from a multi-crate workspace into a single Cargo package, implementing the C++17 subset, adding the browser playground, and enforcing strict census invariants—the project documentation remained split across legacy top-level notes, 4-line decision stubs, and partial design drafts. Furthermore, discrepancies had accumulated (e.g. outdated descriptions of supported operator overloads versus the reality of `src/parser/unsupported.rs`).

### Work Executed
1. **Repository Layout Migration**:
   - Reorganized documentation into an mdBook hierarchy (`docs/book/`) separated into User Guides (`user/`) and Internals Deep Dives (`internals/`).
   - Moved architecture decision records into `docs/adr/`, formatted to a standardized template, and expanded reconstructed ADRs covering memory layout, free-list allocation, string representation, and C ABI interop.
   - Migrated testing documentation to `docs/book/internals/testing.md` and consolidated development workflows into `docs/contributing.md`.
   - Seeded `CHANGELOG.md` from the full 98-commit git history under `[Unreleased]` development milestones.
   - Promoted `ROADMAP.md` to the repository root.
2. **Technical Alignment & Discrepancy Resolution**:
   - Clarified that the compiler pipeline lowers the typed AST directly to WebAssembly bytecode without an intermediate representation (IR) or bytecode optimizer pass.
   - Documented the preprocessor as a full first-class compilation stage handling macro hide sets, recursive expansion, origin mapping, and constant condition evaluation.
   - Audited every claim in language support against the test suite and source code, correcting documentation around operator overloading (clarifying that while primitive pointer `->`, numeric postfix `++`/`--`, and primitive dereference `*` are supported, operator overloads for `->`, `()`, `*()`, and postfix `++`/`--` on class types are explicitly rejected).
3. **In-Code Documentation & Tooling**:
   - Added `#![warn(missing_docs)]` to `src/lib.rs` and documented public pipeline entry points and core data structures.
   - Updated GitHub Actions workflow (`.github/workflows/pages.yml`) to build mdBook, `cargo doc`, and the web playground simultaneously.
