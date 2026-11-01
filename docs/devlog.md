# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppbolt`.

---

## 2026-11-01: Milestone 0 Baseline Snapshot and Scalar Prerequisite Probes

### Context & Motivation
Milestone 0 establishes the regression baseline for template instantiation and std::vector support. Before implementing template parameter substitution, we reconciled test runner manifests and probed compiler behavior on typedef stand-in types representing scalar type parameters (`typedef int T;`).

### Work Executed
1. Golden manifest reconciliation:
   - Synchronized `tests/golden.txt` against commit 13051c384029eb2f4b05a4e958b8ca893234c23f.
   - Resolved 49 stale entries that differed due to previous qualification conversions, numeric literals, and brace-initialization updates.
   - Re-verified clean golden output across all 247 recorded test entries with `cargo run --bin cppbolt-testrunner -- --check-golden tests/golden.txt target/release/cppbolt-cli`.
2. Scalar prerequisite probes:
   - Value-initialization `T a = T();`: failed with `Undefined function 'T'`. The parser interprets `T()` as a function call instead of a scalar functional cast.
   - Direct-initialization `T b(5);`: passed.
   - Size evaluation and allocation `sizeof(T)` and `(T*)malloc(...)`: passed.
   - Scalar placement new `new (buf) T(7);`: passed.
   - Scalar pseudo-destructor `p->~T()`: failed with `Arrow member access on non-struct pointer`. Member access rejects non-struct pointers and lacks pseudo-destructor grammar.
   - Class destructor call via typedef `sp->~ClassT()`: failed with `Method '~ClassT' not found in class 'S'`. Destructor lookup does not resolve typedef aliases to target class names.

### Verification
- Synchronized golden manifest passes: 247 entries matched, 19 uncompilable tests skipped.
- Corpus test suite execution: 360 total tests on disk, 341 passed, 19 expected failures, 0 unexpected failures. All 94 negative tests passed.

---

## 2026-11-01: Type Qualification, Pointer Conversions, and String Literal Semantics

### Context & Motivation
Work on this date addressed C++17 pointer conversion semantics, qualification conversions, and string literals:
1. The compiler allowed arbitrary integers and variables to convert to pointers instead of restricting null pointer constants to literal zero and `nullptr`.
2. `nullptr` was represented internally as `void*`, collapsing `std::nullptr_t` type identity and distorting overload ranking.
3. Multi-level pointer qualification conversions did not enforce C++17 rules, incorrectly allowing `int**` to convert to `const int**`.
4. String literals were typed as pointers rather than `const char[N]`, causing incorrect `sizeof` results and permitting assignment to `char*`.
5. Implicit conversion from `void*` to typed pointers was accepted without an explicit `static_cast`.

### Work Executed
1. `nullptr` type identity:
   - Added `Type::Nullptr` (mangled `Dn`) for `nullptr`, `std::nullptr_t`, and `decltype(nullptr)`, with implicit conversions to pointers and `bool`.
   - Mapped `KwNullptr` and `nullptr_t` tokens to `Type::Nullptr` across lexer and parser.
2. Pointer qualification and null pointer constants:
   - Implemented null pointer constant validation in `src/sema/assign.rs`, accepting only literal zeroes (`0`, `0u`) and `nullptr`.
   - Implemented `check_pointer_conversion` in `src/sema/assign.rs` for multi-level qualification conversions, requiring `const` on all preceding levels when introducing `const` at an inner level.
   - Allowed object-pointer-to-`void*` conversion; rejected implicit `void*`-to-typed-pointer conversions.
   - Updated overload resolution (`score_arguments`) and constructor ranking (`score_constructor_candidate`) to score qualification conversions and null pointer constant conversions.
3. String literal semantics:
   - Represented string literals as `Type::Array(Type::Char(const), len + 1)` in `src/sema/expr.rs` and codegen.
   - Preserved array typing through `sizeof` (`sizeof("hello") == 6`) and decayed to `const char*` for call arguments and pointer assignments.
   - Rejected assignments of string literals to mutable `char*`.

### Verification
- Added positive tests in `tests/c_basics/types/pointer_conversions.cpp` for null pointer constants, `nullptr`, multi-level qualification, `void*`, and string literals.
- Added 7 negative diagnostic tests in `tests/negative/`.
- 69 library and binary tests pass (`cargo test --lib --bins`).
- 337 corpus tests pass without regressions (`cargo run --bin cppbolt-testrunner`).
- Census metrics and clippy pass with zero warnings.

---

## 2026-10-30: C++17 Numeric Literals, Macro Undef, and Raw Token Pasting

### Context & Motivation
Work on this date expanded literal syntax and fixed macro token pasting:
1. The lexer did not support binary literals (`0b1010`), digit separators (`1'000'000`), or octal digit checks.
2. The preprocessor could not undefine macros (`#undef`).
3. Macro token pasting (`##`) expanded operand macros before pasting instead of concatenating raw argument tokens.

### Work Executed
1. Numeric literals:
   - Parsed binary literals with `0b` and `0B` prefixes.
   - Supported single-quote digit separators, rejecting consecutive quotes or quotes at literal boundaries.
   - Rejected digits `8` and `9` in octal literals.
2. Preprocessor directives and macro expansion:
   - Added the `#undef` directive in `src/preprocessor/directive_line.rs`.
   - Updated `src/preprocessor/macros.rs` to paste raw tokens directly on `##` without prior expansion.

### Verification
- Added positive tests in `tests/c_basics/types/numeric_literals.cpp` and `tests/c_basics/preprocessor/macro_token_paste_raw.cpp`.
- Added negative diagnostic tests in `tests/negative/` for invalid binary digits, misplaced digit separators, and invalid octal digits.
- Formatting and clippy clean.

---

## 2026-10-29: List Initialization, Array Decay, and Constructor Resolution

### Context & Motivation
Work on this date addressed list initialization and several compiler edge cases exposed during testing:
1. Braced initializer lists were previously limited to array declarations and struct assignments. Direct initialization (`Type x{42}`), value initialization (`Type x{}`), C++17 aggregate subobjects, narrowing conversions, typed temporaries (`Type{}.get()`), and braced `new` allocations were unsupported.
2. Multi-dimensional pointer dereferencing (`*(*(matrix + 1) + 2)`) mistakenly emitted scalar loads on intermediate array addresses rather than decaying to row base pointers.
3. Namespace-scope global variables with parenthesized constructor initializers (`Item global_item(1);`) conflicted with function declaration grammar.
4. User-defined conversion operators and pointer constructors were improperly matching copy constructor lookups during by-value argument passing and returns.
5. Free `operator*` overloads were asymmetric between rvalue reading and lvalue assignment (`*it = 42;`).

### Work Executed
1. List initialization:
   - Added `Expr::TypedInitList { ty, elements, pos }` to the AST and attached source positions to `Expr::InitializerList`.
   - Updated Pratt expression parsing to handle typed initializers and braced lists across variable declarations, member initializers, return statements, arguments, assignments, and `new` expressions. Extracted `Parser::parse_brace_expr_list` in `src/parser/mod.rs` to deduplicate grammar loops.
   - Implemented `resolve_list_init` in `src/sema/init.rs` driven by `InitContext`, allowing explicit constructors in direct-list contexts while rejecting them in copy-list contexts.
   - Enforced narrowing conversion checks per C++17 [dcl.init.list] p7 for floats, doubles, and integer types, with compile-time range evaluation for constants and negative literals.
   - Added C++17 aggregate classification (`is_cxx17_aggregate`) rejecting classes with user-declared constructors (`layout.has_user_declared_constructors`), private members, or virtual functions, while supporting public non-virtual base subobjects and flattened brace elision.
   - Extended code generation in `src/codegen/stmt_var_init.rs` for empty brace zeroing, added dynamic array allocation with trailing element value-initialization in `src/codegen/expr_new_array.rs`, and allocated frame slots for typed temporaries.
2. Pointer dereference and array decay:
   - In `src/codegen/expr_unary.rs`, updated dereference emission so that when the dereferenced element is an array (such as `int (*)[3]`), the base address stays on the stack instead of emitting a scalar load, allowing subsequent pointer arithmetic to compute proper offsets.
3. Global variable constructor disambiguation:
   - In `src/parser/decl.rs`, introduced `parse_ambiguous_global_or_func` with lookahead for function qualifiers and bodies, falling back to parenthesized constructor expressions for global variable definitions.
4. Copy constructor resolution:
   - In `src/codegen/construct.rs`, introduced `find_copy_constructor` requiring the parameter to be a reference, preventing conversion operators or pointer constructors from being picked as copy constructors.
5. Free `operator*` lvalue assignment:
   - In `src/codegen/lvalue.rs`, routed assignments through free `operator*` overloads returning references (`T& operator*(Iter)`).

### Verification
- Added positive tests in `tests/classes_oop/structs/` for scalar initialization, class constructors, aggregates, brace elision, temporaries, and dynamic `new`.
- Added negative tests in `tests/negative/` for narrowing conversions, explicit constructor rejection, and excess initializer counts.
- Added regression tests for multi-dimensional pointer dereferencing, global constructor arguments, and conversion operator parameter passing.
- 51 library tests pass (`cargo test --lib`).
- 324 corpus tests pass without manifest violations (`cargo run --bin cppbolt-testrunner`).
- Formatting and clippy clean.

---

## 2026-10-28: Lambdas, Operator Overloading, and Language Conformance

### Context & Motivation
Work on this date added core C++17 language constructs, expanded operator overloading, and implemented lambda expressions:
1. Smart pointer, iterator, and functor abstractions required `operator*()`, pointer member access `operator->`, function call `operator()`, and postfix `operator++(int)`/`operator--(int)`.
2. Chained smart pointer delegation required recursive `operator->` resolution with cycle detection.
3. C++17 lambdas (captures, return type deduction, direct call, and conversion to function pointers) were needed for modern functional patterns.
4. Compiler compatibility audits revealed gaps in array parameter decay, constexpr global initializer reachability, delegating constructors, switch case constant expressions, scoped enums (`enum class`), user conversion operators, ref-qualified member methods, structured bindings, and explicit `throw` rejection diagnostics.

### Work Executed
1. Operator overloading:
   - Enabled parsing and semantic resolution for unary dereference `operator*()`, pointer member access `operator->`, functor `operator()`, and postfix `operator++`/`--` (distinguished via dummy `int` parameter).
   - In `src/sema/expr_member.rs`, implemented recursive `operator->` resolution across intermediate wrapper objects with cycle detection and a 128-hop recursion budget.
   - In `src/codegen/expr_unary.rs`, `lvalue.rs`, and `expr_call_member.rs`, lowered operator calls and routed receiver evaluations through overloaded operators.
2. Lambda expressions:
   - Added `Expr::Lambda` AST node and parsing for capture lists (`[]`, `[x]`, `[&x]`, `[=]`, `[&]`), parameters, trailing return types (`-> T`), and `mutable`.
   - In `src/sema/expr_lambda.rs`, synthesized anonymous closure structs (`__lambda_N`) with capture member fields, member initializers, and `operator()` methods (`const` by default).
   - Added automatic return type deduction and static `__invoke` trampolines for captureless lambdas, enabling implicit conversion to C function pointers (`R (*)(P...)`) and unary `+` decay (`+[](){}`).
3. Language conformance:
   - Adjusted function parameter array syntax (`int arr[]`, `int arr[N]`) to pointer types (`int* arr`) in `src/parser/decl_func.rs`.
   - Preserved global variable initializer call targets through dead-code elimination in `src/codegen/program_prepare.rs`.
   - Supported delegating constructors (`Point() : Point(5) {}`) and enforced mutual exclusivity against member initializers.
   - Allowed arbitrary constant expressions and scoped enum variants in switch case labels.
   - Added scoped enums (`enum class`) with qualified lookup (`Scope::Val`) and distinct type identity.
   - Supported user-defined conversion operators (`operator T() const`), respecting `explicit` qualification.
   - Added `&` and `&&` ref-qualified member methods with value-category receiver dispatch and distinct symbol mangling (`_R`, `_O`).
   - Implemented structured bindings (`auto [a, b] = p;`, `auto&`, `const auto&`) for aggregate structs.
   - Diagnosed `throw` with a clear message noting that exceptions are not supported on this WebAssembly target.

### Verification
- Added positive and negative test suites across operators, lambdas, structured bindings, conversion operators, and delegating constructors.
- Test corpus reached 300 passing tests with zero unexpected failures.
- Formatting and clippy clean.

---

## 2026-10-27: Range-Based for Loops and Documentation Architecture

### Context & Motivation
Following completion of the initial compiler subset, work focused on adding high-impact iteration syntax and restructuring project documentation into a standardized reference system.

### Work Executed
1. Range-based `for` loops:
   - Added `StmtKind::RangeFor { decl, range, body }` to the AST and parser.
   - In `src/sema/stmt_range_for.rs`, validated range expressions and desugared loops into scoped standard `for` loops.
   - For fixed-size 1D and multi-dimensional arrays (`int[N]`, `int[M][N]`), lowered iteration to pointer bounds `[__begin, __end)`.
   - For classes with `.begin()` and `.end()` methods (such as `std::string`), synthesized member call bounds.
   - Supported by-value (`auto x`), by-reference (`auto& x`), and const-reference (`const auto& x`) bindings.
   - Verified RAII destructor cleanups on early `break`, `continue`, and `return`.
   - Added validation diagnostics for non-iterable types.
2. Documentation architecture:
   - Organized documentation into an mdBook hierarchy (`docs/book/`) with User Guides and Internals chapters.
   - Standardized Architecture Decision Records in `docs/adr/`.
   - Consolidated development and testing workflows into `docs/contributing.md` and `docs/book/internals/testing.md`.
   - Established `CHANGELOG.md` and moved `ROADMAP.md` to the repository root.
   - Added `#![warn(missing_docs)]` to `src/lib.rs` and documented public pipeline entry points.

### Verification
- Added test coverage in `tests/c_basics/pointers_arrays/range_for_array.cpp`, `tests/mini_stl/range_for_string.cpp`, `tests/classes_oop/memory_raii/range_for_raii.cpp`, and `tests/negative/range_for_non_iterable.cpp`.
- Verified automated mdBook, `cargo doc`, and playground builds in CI.

---

## 2026-11-01: C++17 Namespace Aliases

### Context & Motivation
Section 2 of the roadmap tracked namespace aliases (`namespace NewName = OriginalName;`), which were rejected upfront by the parser unsupported validator.

### Work Executed
1. Removed the parser rejection rule for namespace aliases.
2. Added `Decl::NamespaceAlias` and `StmtKind::NamespaceAlias` to AST definitions.
3. Parsed namespace alias declarations at global, namespace, and block scopes.
4. Added transitive alias resolution in `Parser` and `Sema` across types, functions, variables, and `using namespace` directives.
5. Diagnosed invalid alias redefinitions in semantic analysis.

### Verification
- Added test coverage in `tests/classes_oop/statics_namespaces/namespace_aliases.cpp`.
- Updated negative test coverage in `tests/negative/namespace_alias.cpp`.
- Verified clean suite with 338 passing tests in `cppbolt-testrunner`.

---

## 2026-11-01: C++17 Extended Casts (reinterpret_cast)

### Context & Motivation
Section 2 of the roadmap tracked `reinterpret_cast` across scalar and pointer types within the ILP32 model. Previously, only `static_cast` and C-style casts were recognized, while `reinterpret_cast` was rejected at parse time.

### Work Executed
1. Added `TokenKind::KwReinterpretCast` to the lexer and parser.
2. Introduced `CastKind` (`Static`, `Reinterpret`, `CStyle`) and unified cast expressions into `Expr::Cast { kind, target_type, expr }`.
3. Extended the parser to handle `reinterpret_cast<Type>(expr)` with template bracket matching.
4. Implemented semantic validation for `reinterpret_cast` in `src/sema/expr.rs`:
   - Allowed conversions between integer types and pointer types without loss under 32-bit ILP32 semantics.
   - Allowed conversions between arbitrary pointer types.
   - Handled lvalue reference reinterpretations (`reinterpret_cast<T&>(expr)`).
   - Rejected scalar floating-point value conversions (direct float-to-int or float-to-pointer casts) per C++17 rules.
   - Bypassed user-defined conversion operators (`operator T()`) and constructors.
5. Implemented WebAssembly code generation lowering in `src/codegen/expr.rs` and `src/codegen/memory.rs`, generating bit-preserving no-ops or width adjustments (wrapping or extending for 64-bit integers).
6. Documented all additions and marked Roadmap Section 2 complete.

### Verification
- Added end-to-end integration tests in `tests/c_basics/types/reinterpret_cast.cpp`.
- Added unit tests across lexer, parser, sema, and codegen in `src/`.
- Verified clean suite with 339 passing tests in `cppbolt-testrunner`.
- Verified zero warnings with `cargo clippy` and satisfied all census constraints.
