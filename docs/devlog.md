# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppbolt`.

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
