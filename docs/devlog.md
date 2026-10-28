# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppwasm`.

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
   - Added positive test cases in `tests/classes_oop/operator_smart_pointer.cpp`, `tests/classes_oop/operator_call_functor.cpp`, and `tests/classes_oop/operator_postfix_incdec.cpp`.
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
   - Added automated tests: `tests/c_basics/range_for_array.cpp`, `tests/mini_stl/range_for_string.cpp`, `tests/classes_oop/range_for_raii.cpp`, and `tests/negative/range_for_non_iterable.cpp`.

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
