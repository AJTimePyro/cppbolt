# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppbolt`.

---

## 2026-11-10: C++17 Aggregate Brace Elision and Upstream LLVM libc++ std::array

Implemented compiler-level aggregate brace elision for nested array and aggregate members, and integrated the upstream LLVM libc++ 23.1.5 `<array>` component preserving standard aggregate semantics.

### Summary of Work

1. Implemented C++17 aggregate brace elision in `src/sema/init.rs`, `src/codegen/stmt_var_init.rs`, and `src/codegen/function_global_init.rs`. Flat initializer lists unpack elements sequentially into subaggregate and array members across local variables, globals, and nested structs.
2. Extended brace elision to top-level multidimensional arrays and raw struct arrays in `src/sema/init.rs` and `src/codegen/stmt_var.rs`.
3. Handled base class subobjects and braced scalar initializers in global aggregate initialization (`src/codegen/function_global_init.rs`), aligning global initialization layout with local variable initialization.
4. Preserved aggregate semantics without constructor compatibility shims. Added diagnostics for excess initializer elements in `tests/negative/aggregate_excess_elision.cpp`.
5. Adapted upstream LLVM libc++ 23.1.5 `<array>` into `stdlib/array`, providing the zero-sized specialization `std::array<T, 0>`, forward and reverse iterators, `at()`, `data()`, `fill()`, move-safe `swap()`, relational comparisons, and tuple helpers (`tuple_size`, `tuple_element`, `std::get`).
6. Added reverse iterator support via `stdlib/detail/reverse_iterator.h` with pointer return type for `operator->` and registered it in the embedded header table (`src/preprocessor/include.rs`).
7. Added standard algorithm templates `std::equal`, `std::fill_n`, and `std::lexicographical_compare` to `stdlib/algorithm`.
8. Added compatibility macros in `stdlib/compat/compiler/compiler.h` and out-of-range diagnostics via `std::__throw_out_of_range` in `stdlib/compat/runtime/runtime.h`.
9. Fixed free binary operator overload scoring in `src/sema/expr_ops.rs` to validate candidate signatures before bypassing function template resolution.
10. Resolved all Clippy `-D warnings` lints across the codebase, eliminating direct `.unwrap()` calls on initializer cursors and collapsing redundant conditionals.
11. Added regression test coverage in `tests/classes_oop/structs/` and differential test suites against native Clang in `tests/mini_stl/array_comprehensive.cpp` and `tests/mini_stl_variant/array_comprehensive_variant.cpp`.

### Verification Results

- Unit and integration tests: 506 corpus tests on disk, 506 executed, 506 passed, 0 failures. All negative tests passed.
- Oracle gate: 100% byte-for-byte output match against host Clang and GCC.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-10: Template Specialization, Non-Type Parameters, and Alias Templates

Implemented class template explicit and partial specialization, non-type template parameters (NTTP), compile-time constant expression evaluation, alias templates, and standard `<array>` and `<type_traits>` headers.

### Summary of Work

1. Implemented class template explicit (full) specialization (`template <> struct S<T> { ... }`) in `src/sema/template_spec.rs` and `src/sema/template_registry.rs`, prioritizing exact specialized definitions over primary templates during instantiation.
2. Implemented class template partial specialization (`template <Params> struct S<Pattern> { ... }`) with pattern matching and specificity scoring in `src/sema/template_spec.rs`, registering nested structs and member aliases into the specialization scope.
3. Added non-type template parameters (`template <typename T, int N>`) in `src/ast/template.rs` and `src/parser/template.rs`, supporting default parameter expressions and rejecting non-type arguments passed to type parameters.
4. Built a compile-time constant expression evaluator in `src/sema/constexpr_eval.rs` supporting integral and boolean literals, unary, binary arithmetic, bitwise operations, shifts, comparisons, conditionals, and `sizeof`, with division-by-zero, shift-count range checks, and source diagnostic locations.
5. Added alias templates (`template <Params> using Name = Target;`) in `src/parser/template.rs` and `src/sema/template_registry.rs`, substituting type arguments and resolving dependent aliases during type lookup.
6. Hardened template angle-bracket parsing in `src/parser/template.rs` and `src/parser/expr.rs`, tracking nested argument depths, splitting `>>` tokens, and preserving token state across speculative parsing backtracks.
7. Added `<array>` (`std::array<T, N>`) and `<type_traits>` (`std::integral_constant`, `std::true_type`, `std::false_type`, `std::is_same`, `std::remove_reference`, `std::remove_reference_t`) in `stdlib/`.
8. Added unit, integration, variant, and negative diagnostic tests across `tests/templates_operators/`, `tests/mini_stl/`, and `tests/negative/`.

### Verification Results

- Unit and integration tests: 501 corpus tests on disk, 501 executed, 501 passed, 0 failures. All negative tests passed.
- Golden baseline: all 359 entries match `tests/golden.txt`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-09: Move Semantics, Value Categories, and Smart Pointers

Implemented rvalue references (`T&&`), C++17 value category tracking, implicit move constructor and assignment synthesis, copy suppression for move-only types, forwarding references, reference collapsing, and standard library headers `<memory>` and `<utility>`.

### Summary of Work

1. Added `Type::RvalueReference` in `src/ast/types.rs` and parser support for `&&` type declarators in `src/parser/types.rs`.
2. Implemented value category tracking (prvalue, xvalue, lvalue) across expression analysis in `src/sema/`, classifying identifiers and named rvalue reference parameters as lvalues, and temporary returns and casts as xvalues or prvalues.
3. Added overload resolution ranking in `src/sema/overload.rs` favoring rvalue references for rvalue arguments, while keeping `const T&` bindings viable for lvalues and rvalues.
4. Implemented implicit move constructor and move assignment operator synthesis in `src/sema/synthesize.rs` and `src/sema/synthesize_ops.rs`, and suppressed implicit copy operations for classes declaring move operations.
5. Added forwarding reference deduction (`template <typename T> void f(T&&)`) in `src/sema/template_deduce.rs` and reference collapsing rules (`&` + `&&` -> `&`, `&&` + `&&` -> `&&`).
6. Enforced copy suppression in `src/sema/constructor.rs`, `src/sema/assign.rs`, `src/sema/stmt_return.rs`, and `src/codegen/` across direct initialization, assignment, call arguments, and return statements.
7. Added `<memory>` providing `std::unique_ptr<T>` with single-owner semantics, custom deleters, `reset`, `release`, self-reset protection, and `std::make_unique`.
8. Added `std::move`, `std::forward`, and `std::swap` to `<utility>`, and added move constructor, move assignment, `push_back(T&&)`, and move-based buffer reallocation to `<vector>`.
9. Added corpus tests across `tests/c_basics/types/`, `tests/classes_oop/`, `tests/templates_operators/`, `tests/mini_stl/`, and `tests/negative/`.

### Verification Results

- Unit and integration tests: 469 corpus tests executed, 469 passed, 0 failures.
- Golden baseline: 339 entries match `tests/golden.txt`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-08: Member Structured Bindings, Range-For Loops, and Template Array Decay

Fixed structured binding and range-based for lowering in non-template member functions, added array-to-pointer decay for function template argument deduction, and fixed array member initialization from initializer lists.

### Summary of Work

1. Preserved analyzed method bodies and deduced return types in `analyze_declarations` (`src/sema/analyze.rs`) by writing `func_decl.body` and `func_decl.return_type` back to `m`. This keeps in-place desugaring of `StmtKind::StructuredBinding` and `StmtKind::RangeFor`, and synchronizes deduced `auto` return types for non-template member functions.
2. Added array-to-pointer decay for function template argument deduction in `deduce_template_arguments` (`src/sema/template_deduce.rs`) when parameter types are non-references (C++17 [temp.deduct.call] p2). This allows `std::sort(arr, arr + n)` and user-defined iterator templates to deduce pointer types from array arguments.
3. Added recursive `Type::Array` matching in `deduce_type` (`src/sema/template_deduce.rs`) to retain array dimensions for explicitly array-typed template parameters.
4. Handled scalar and struct elements in `compile_member_init_expr` and `compile_array_member_init` (`src/codegen/function_lifecycle.rs`) for array members by writing elements sequentially at member offsets instead of emitting an invalid local assignment on an empty WebAssembly stack.
5. Added regression tests in `tests/cxx17_regressions.rs` for by-value, by-reference, and const-reference structured bindings in member functions, member `auto` return type deduction, `std::sort` array decay, user-defined template iterator functions, array reference retention, member range-for with NSDMI and constructor initialization, container iterators, and array members with struct element types.

---

## 2026-11-08: Reference Binding and Const Method Diagnostics

Fixed non-const lvalue reference binding to temporaries, preserved reference categories for call arguments, and improved diagnostics for non-const method calls on const objects.

### Summary of Work

1. Enforced value-category checks during reference binding in `can_assign_expr` (`src/sema/assign.rs`) and variable initializers (`src/sema/stmt_decl.rs`), ensuring non-const references reject temporaries and rvalues while const references continue to bind.
2. Updated function argument overload resolution in `score_arguments` (`src/sema/overload.rs`) to reject rvalues passed to non-const reference parameters, and select const reference overloads when available.
3. Preserved reference types during call argument analysis in `analyze_call` (`src/sema/expr_call.rs`) and in namespace-scope and struct-field initializers (`src/sema/analyze.rs`), allowing reference-returning functions to bind to non-const reference parameters and globals.
4. Aligned codegen overload parameter scoring in `src/codegen/overload.rs` with semantic analysis, checking reference compatibility and allowing convertible types to bind to const references.
5. Added diagnostic in `check_member_arguments` (`src/sema/expr_call_member.rs`) when calling a non-const method on a const receiver, reporting that the method exists but cannot be called on a const object instead of falling through to codegen's missing method error.
6. Added positive regression tests in `tests/c_basics/functions/reference_overload.cpp` and negative regression tests in `tests/negative/reference_temporary_bind.cpp`, `tests/negative/reference_call_temporary.cpp`, and `tests/negative/const_method_call_on_const.cpp`.

---

## 2026-11-07: npm Worker Isolation and Timeouts

Added worker-isolated execution with timeout and cancellation support to the npm package.

### Summary of Work

1. Added `AsyncCppBoltRunner` and `compileAndRunAsync` to `npm/src/api.ts`.
2. Bundled worker runtimes at build time with esbuild: Node uses `dist/worker.node.js` (`worker_threads`), and browser builds embed the worker IIFE as `__WORKER_BUNDLE__` for `new Blob(...)` construction.
3. Added `TimeoutError` and `RunOptions` (`timeoutMs`, `signal`, `stdin`, `onStdout`), preserving any stdout emitted before worker termination.
4. Corrected worker execution semantics: WebAssembly instantiation failures reject with the underlying error (`CompileError`/`LinkError`) rather than masking as SIGABRT exit code 134; program traps are restricted to `RuntimeError` and stack overflow `RangeError`.
5. Forwarded `stdin` input to worker execution, enabled streaming `onStdout` callbacks, made the `AsyncCppBoltRunner` host parameter optional, and guarded worker termination against multiple invocations.
6. Added test coverage in `tests/test_npm_node_worker.js`, `tests/test_npm_browser_build.js`, `tests/test_npm_runner_options.js`, `tests/test_npm_compile_and_run_async.js`, and `tests/test_browser_worker.mjs` (Chromium).

---

## 2026-11-03: `std::initializer_list` Review Fixes

Fixed five findings in the initializer-list and brace-initialization support added earlier the same day.

### Summary of Work

1. Resolved non-template method parameter and return types in `register_struct_declarations` (`src/sema/analyze.rs`), so a constructor such as `X(std::initializer_list<int>)` or `X(std::vector<int>)` matches an argument whose template type was already instantiated (`c78aa25`).
2. Ranked every initializer-list constructor instead of taking the first whose elements were convertible: exact conversions beat integral promotions, which beat other standard conversions. A narrowing candidate is skipped, and its diagnostic is reported only when no candidate is viable (`fb4a983`).
3. Added empty initializer-list constructor fallback for value-initialization (`T x{}`) when the class declares no default constructor (`fb4a983`).
4. Normalized extracted template element types to their registered struct names and resolved nested brace lists against the chosen element type, enabling `std::vector<std::vector<int>> v{{1, 2}, {3, 4}}` (`cadf45c`).
5. Routed a namespace-scope brace initializer through the same list resolution as locals, and lowered a namespace-scope `std::initializer_list<T>` by building its backing array in `main`'s frame and storing the pointer and length in the global object (`366bad8`).

### Resolved Issue

Range-based `for` loops and structured bindings inside non-template method bodies were previously unanalyzed because `analyze_declarations` type-checked a cloned body and discarded it. Resolved on 2026-11-08 by writing back the analyzed method body. Free functions and template method bodies were unaffected.

### Verification Results

- Corpus: 428 tests on disk, 428 passed, 0 unexpected failures; all 118 negative tests passed.
- Golden baseline: all 310 entries match `tests/golden.txt`, with only the two new tests added.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

## 2026-11-03: `std::initializer_list` and Brace Initialization

Implemented `std::initializer_list<T>` and container brace initialization.

### Summary of Work

1. Added the vector count-and-value constructor so `vector<int>(3, 4)` remains distinct from `vector<int>{3, 4}`.
2. Implemented semantic initializer-list resolution with constructor preference, `auto` deduction, homogeneous element checks, narrowing diagnostics, and support for parenthesized braced arguments.
3. Lowered each list to a frame-backed contiguous element array plus an eight-byte `{pointer, length}` `std::initializer_list<T>` value. Temporary backing elements are destroyed at full-expression cleanup; local list elements are destroyed with their containing scope.
4. Added positive, variant, and negative corpus coverage for list arguments, `auto` deduction, vector brace initialization, mismatched types, empty `auto`, and narrowing.

### Verification Results

- Corpus: 416 tests on disk, 416 passed, 0 unexpected failures; all 114 negative tests passed.
- New positive coverage includes `std::vector<std::string>` brace initialization and `vector({1, 2, 3, 4})`.

## 2026-11-02: Template Method Defaults, Lifecycle Synthesis, and Out-of-Class Qualified Attachment

Resolved review findings in member template default argument mapping, same-arity template candidate deduplication, parameter shadowing, out-of-class qualified constructor parsing, and base subobject copying.

### Summary of Work

1. Corrected member template default argument indexing in `src/sema/constructor.rs`, mapping default argument expressions across the full signature parameter slice including the leading implicit instance receiver.
2. Deduplicated function template candidates in `src/sema/template_registry.rs` using exact parameter types rather than arity alone, preserving coexisting same-arity overloads and updating bodies on forward declarations.
3. Aligned out-of-class template method attachment with parameter types and constness, preventing same-arity overloads from overwriting existing definitions.
4. Extended constructor and destructor detection in `src/parser/template.rs` to traverse multi-part namespace and class qualifier paths (`Outer::Inner::Class::Class()`).
5. Added in-place out-of-class non-template method attachment in `src/sema/analyze.rs`, attaching method and constructor bodies to target struct declarations prior to layout and semantic checking.
6. Enforced member template parameter shadowing during outer class template substitution in `src/sema/template_subst.rs`, preserving inner template parameters.
7. Refined functor dispatch in `src/sema/expr_call.rs` to verify that member templates match `operator()` before rewriting instance calls.
8. Included base class initializers in synthesized copy constructors and prepended base assignments in copy assignment operators in `src/sema/synthesize.rs`, with base subobject offset adjustments in `src/codegen/call_args.rs` and `src/codegen/lvalue.rs`.
9. Added dynamic destructor and copy operation synthesis for instantiated template classes containing members with non-trivial lifecycles.
10. Added 8 unit tests in `src/sema/tests.rs` covering all resolved cases, and updated golden baseline hashes across 13 affected tests.

### Verification Results

- Unit and integration tests: 394 corpus tests on disk, 390 passed, 4 expected failures, 0 unexpected failures. All 16 unit tests in `src/sema/tests.rs` and 88 library unit tests passed.
- Golden baseline: all 283 entries match `tests/golden.txt`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-02: Member Function Templates, Standard Vector, and Algorithm Overloads

Implemented member function templates, out-of-class template method definitions, dependent member type aliases, and multi-arity function template overloads. These additions unlock `<vector>`, `<utility>` (`std::pair`), and `<algorithm>` (`std::sort`, `std::reverse`, `std::min`, `std::max`).

### Summary of Work

1. Added parsing and registration for out-of-class template method definitions (`template <typename T> void Class<T>::method()`) in `src/parser/decl.rs` and `src/sema/template_registry.rs`.
2. Supported member function templates inside class templates and concrete structs in `src/parser/structs.rs`, `src/sema/constructor.rs`, and `src/sema/template_registry.rs`. This enables templated constructors and call operators such as `pair(U1&&, U2&&)` and functor invocations.
3. Enabled dependent member type aliases on instantiated template classes (such as `std::vector<int>::iterator` to `int*`) in `src/parser/types.rs` and `src/sema/template_registry.rs`. Range-based for loops over template containers now resolve iterators and dereferenced elements directly.
4. Expanded function template registration in `src/sema/template_registry.rs` to track candidate lists rather than single declarations. Candidate filtering by arity and default arguments enables overloads such as `std::sort(first, last)` and `std::sort(first, last, comp)` to coexist.
5. Handled member template instantiations for plain structs (such as `DefaultLess::operator()<T>`) by synthesizing concrete method declarations into the AST for WebAssembly codegen.
6. Added constructor conversion for by-value struct arguments in `src/codegen/call_args.rs`, creating managed temporaries when passing arguments such as string literals to struct parameters.
7. Synthesized copy constructors and copy assignment operators for classes containing non-trivial subobjects in `src/sema/synthesize.rs`, and updated placement new and member initializers in `src/codegen/expr_new.rs` and `src/codegen/function_lifecycle.rs` to copy struct bytes when explicit copy constructors are absent.
8. Extracted `src/sema/template_func.rs`, `src/sema/synthesize.rs`, and `src/sema/expr_call_member.rs` to maintain repository census compliance.
9. Added 5 variant tests: `tests/templates_operators_variant/template_class_methods_variant.cpp`, `tests/mini_stl_variant/pair_basic.cpp`, `tests/mini_stl_variant/vector_basic.cpp`, `tests/mini_stl_variant/iterators_range_for_variant.cpp`, `tests/mini_stl_variant/algorithm_sort_variant.cpp`, and `tests/mini_stl_variant/combined_integration_variant.cpp`.

### Verification Results

- Unit and integration tests: 394 corpus tests on disk, 390 passed, 4 expected failures (remaining associative container tests), 0 unexpected failures. All 107 negative tests passed.
- Host oracle gate: matched native compiler behavior on all expected files (82.2s).
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-01: Template Integration Bugfixes and Lifecycle Lowering

Resolved edge cases across template type resolution, field layout calculation, lazy operator lowering, RAII destructor lifecycles, and function argument deduction.

### Summary of Work

1. Made type resolution recursive across pointers, references, const qualifiers, and arrays in `src/sema/template_registry.rs`, generating concrete struct layouts before field lookup. Added a cycle guard in `src/sema/lookup.rs` to stop infinite alias looping.
2. Resolved template instance types in struct fields before computing struct layouts in `src/sema/analyze.rs`, setting correct field offsets and total struct sizes. Resolved template types in non-template function parameters and return types during declaration registration and function analysis.
3. Lowered lazy template operator methods (`operator[]`, unary operators) upon match in `src/sema/expr_ops.rs`. Eagerly instantiated non-virtual template class destructors in `src/sema/template_registry.rs` so destructor synthesis registers cleanup calls on scope exit.
4. Filled default arguments on function template calls via `self.full_arguments(&sig, args)` in `src/sema/expr_call.rs`, providing all required stack values in WebAssembly bytecode.
5. Passed known class templates into explicit template argument list parsing in `src/sema/template_deduce.rs` to support nested types like `func<Inner<int>>(x)`.
6. Stripped top-level const during pointer argument deduction in `src/sema/template_deduce.rs` to deduce `T` from `T* const`.
7. Refined template name substitution in `src/sema/template_subst.rs` to prevent accidental replacement across scope qualifiers.
8. Added regression tests for pointer access, const-reference binding, struct template fields, non-template functions with template arguments, overloaded subscript operators, destructor scope cleanups, function default arguments, nested explicit types, and const pointer deduction (`template_ptr_const_ref.cpp`, `template_field_and_func.cpp`, `template_ops_dtor.cpp`, `template_default_arg_deduce.cpp`).

### Verification Results

- Unit and integration tests: 381 corpus tests on disk, 364 passed, 17 expected failures, 0 unexpected failures. All 107 negative tests passed.
- Golden baseline: updated to 267 entries in `tests/golden.txt`, all matching `target/debug/cppbolt-cli`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-01: Template Core Implementation

We implemented class and function templates with type parameters, single-phase name lookup, lazy member instantiation, and function template argument deduction.

### Summary of Work

1. Added template AST definitions in `src/ast/template.rs`, including `TemplateDecl`, `TemplateParam`, and `Type::TemplateInstance`.
2. Implemented parsing for template declarations, template argument lists (`<T, U>`), and template-ids in `src/parser/template.rs`. Added diagnostics rejecting unsupported constructs: explicit specialization, explicit instantiation, template template parameters, variadic templates, and non-type template parameters.
3. Implemented AST substitution in `src/sema/template_subst.rs`, replacing type parameters with concrete types across expressions, statements, types, and declarations.
4. Implemented template registry in `src/sema/template_registry.rs` to lay out struct fields, assign canonical specialization names, cache instantiated declarations, and lower member methods lazily on call.
5. Implemented function template argument deduction in `src/sema/template_deduce.rs`, deducing concrete types from call argument types, handling reference and const qualifiers, and checking type consistency across repeated parameters.
6. Integrated function templates into overload resolution in `src/sema/expr_call.rs`. The compiler prefers exact matches, falls back to function templates when available, and respects defaulted arguments during candidate selection.
7. Added positive regression tests covering class templates, function templates, multi-parameter templates, reference deduction, and distinct instantiations (`template_class_basic`, `template_func_basic`, `template_multi_param`, `template_pair`, `template_func_multi`, `template_lazy_member`, `template_multi_instance`, `template_deduce_ref`).
8. Added negative tests verifying rejection of template recursion exceeding the 500 depth limit and deduction conflicts (`template_instantiation_depth.cpp`, `template_deduction_conflict.cpp`).

### Verification Results

- Unit and integration tests: 377 corpus tests on disk, 360 passed, 17 expected failures, 0 unexpected failures. All 107 negative tests passed.
- Host oracle gate: matched native compiler behavior on all expected files (73.6s).
- Golden baseline: updated to 263 entries in `tests/golden.txt`, all matching `target/debug/cppbolt-cli`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-01: Scalar Operations and Baseline Synchronization

The test suite, clippy, census audit, and golden manifest are green across all targets.

### Summary of Work

1. Synchronized golden baseline: reconciled 49 stale entries in `tests/golden.txt` initially, then added all newly passing tests, bringing the total to 255 verified entries.
2. Added `std::abs(long)` and `std::abs(long long)` overloads in `stdlib/cmath`. Unsigned types now trigger ambiguity errors instead of silently promoting to float.
3. Implemented functional casts and value initialization (`T()`, `Type(val)`) for scalar types and typedef aliases in the parser and sema.
4. Added scalar pseudo-destructor support (`p->~T()`, `x.~T()`, `arr[i].~T()`). The compiler evaluates receiver expressions for side effects and treats the destructor call as a no-op.
5. Resolved struct destructor calls through typedef aliases (`sp->~ClassT()`), checking that the alias resolves to the receiver class before calling the destructor.
6. Corrected `classes_oop/lambdas/lambda_unary_plus` to compare pointer size against `sizeof(void*)`, allowing the test to pass both in 32-bit WebAssembly and under the 64-bit native oracle gate.

### Verification Results

- Unit and integration tests: 370 corpus tests on disk, 351 passed, 19 expected failures, 0 unexpected failures, 0 manifest violations.
- Oracle gate: 544 matched, 2 runtime-trap tests skipped across g++ and clang++.
- Golden manifest: 255 entries recorded in `tests/golden.txt`, all matching `target/release/cppbolt-cli`.
- Census audit: 119 files scanned, 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-01: Baseline Snapshot and Scalar Prerequisite Probes

### Context & Motivation
We reconciled test runner manifests and probed compiler behavior on typedef stand-in types representing scalar type parameters (`typedef int T;`).

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

---

## 2026-11-02: Nested Classes, Dependent Types, and Associative Containers (std::map, std::set)

Implemented member classes within structs and templates, the `typename` keyword for dependent type specifiers, template nested class instantiation, and method overload selection by value category. These additions enabled `std::map`, `std::set`, and their self-balancing binary search tree `detail::AVLTree` in `stdlib/`.

### Summary of Work

1. Parsed inner struct and class declarations in `src/parser/structs.rs` and `src/parser/structs_nested.rs`, registering nested definitions under their outer qualified paths (`Outer::Inner`) and flattening them into top-level declarations.
2. Added `typename` keyword handling in `src/parser/types.rs` and `src/parser/prefix.rs`, requiring qualification (`typename Scope::Type`) and preserving qualified specifiers across function signatures.
3. Implemented `instantiate_nested_structs` and `populate_instantiation_member_aliases` in `src/sema/template_nested.rs`, cloning and specializing inner classes with outer template arguments substituted and computing their record layouts.
4. Added `select_member_overload` in `src/sema/expr_call_member.rs` to score candidate signatures and resolve calls across receiver constness and value categories.
5. Added eager constructor dependency resolution in `src/sema/constructor.rs`, instantiating member subobject and array element constructors before body analysis.
6. Handled scoped destructor lookups in `src/codegen/function_lifecycle.rs` and `src/codegen/raii.rs` via short and qualified names.
7. Enabled `tests/mini_stl/map_basic.cpp`, `tests/mini_stl/set_basic.cpp`, and the 20-seed differential fuzzing suite `tests/mini_stl/differential_fuzz.cpp` in `tests/PASSING.txt`.

### Verification Results

- Unit and integration tests: 401 corpus tests on disk, 401 passed, 1 expected failure (`vector_brace_init`), 0 unexpected failures. All 109 negative tests passed.
- Host oracle gate: matched native compiler behavior on all expected files.
- Golden baseline: updated to 294 entries in `tests/golden.txt`, all matching `target/release/cppbolt-cli`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-02: Nested Class Access Control, Scope Resolution, and Fixed-Point Dependent Aliases

Resolved review findings in nested struct scope tracking, global namespace pollution, type specifier access control, and chained dependent alias resolution.

### Summary of Work

1. Pushed qualified names onto `active_struct_scopes` in `src/parser/structs_nested.rs`, ensuring member aliases declared in inner structs bind to the inner scope rather than the outer class.
2. Removed unqualified nested struct names from `known_types` in `src/parser/structs_nested.rs`, scoping nested names to their enclosing struct and resolving them via `resolve_active_struct_scope_type`.
3. Implemented `check_nested_type_access` in `src/sema/lookup.rs` and called it during type resolution in `src/sema/template_registry.rs`, rejecting external references to private nested types while permitting enclosing classes and parameterized templates.
4. Replaced the fixed 3-iteration alias expansion loop in `src/sema/template_nested.rs` with fixed-point iteration (`while changed && iterations < 16`), supporting arbitrary alias chain depths.
5. Guarded member alias injection in `src/sema/template_nested.rs` and `src/sema/template_registry.rs`, injecting `std::` aliases only for classes declared within the `std` namespace.
6. Added positive tests for nested scopes and alias chains (`nested_classes_scopes.cpp`, `template_nested_alias_chain.cpp`), and negative diagnostic tests for unqualified leakage and private access (`nested_class_unqualified_leak.cpp`, `nested_type_private_access.cpp`).

### Verification Results

- Unit and integration tests: 406 corpus tests on disk, 405 passed, 1 expected failure (`vector_brace_init`), 0 unexpected failures. All 111 negative tests passed.
- Golden baseline: all 294 entries match `tests/golden.txt`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-07: C++17 Core Stabilization (Comma Expressions, Selection Initializers, NSDMI & Aggregate Defaults)

Validated and resolved four issues identified during C++17 pre-release testing: comma operator evaluation, selection statement initializers, non-static data member initializers (NSDMI), and aggregate initialization default rules.

### Summary of Work

1. Added `BinaryOp::Comma` to the AST and parser at precedence 1 in `src/ast/expr.rs` and `src/parser/expr.rs`. Restricted argument list, parameter default, and declarator parsing to precedence 2 in `src/parser/mod.rs`, `src/parser/decl_func.rs`, and `src/parser/enums_operators.rs` to keep unparenthesized commas distinct from expression sequence operators.
2. Implemented WebAssembly code generation for comma expressions in `src/codegen/expr_binary.rs`, evaluating the left operand, dropping its result from the value stack, and compiling the right operand.
3. Implemented C++17 selection statement initializers (`if (init; cond)` and `switch (init; cond)`) in `src/parser/stmt_selection.rs`, desugaring initializers into compound scopes enclosing the selection statement while preserving iterative flattening for deep `else if` chains.
4. Added support for non-static data member initializers (`= expr` and `{ ... }`) in struct, class, and template declarations across `src/ast/decl.rs`, `src/ast/template.rs`, and `src/parser/structs.rs`.
5. Updated semantic analysis in `src/sema/analyze.rs` and `src/sema/layout.rs` to validate default member initializer expressions and record `default_init` on `StructField`.
6. Updated constructor lowering in `src/codegen/function_lifecycle.rs` to fall back to `default_init` for fields omitted from constructor member initializer lists.
7. Aligned aggregate initialization with C++17 [dcl.init.aggr] in `src/codegen/stmt_var_init.rs`, `src/codegen/expr_new.rs`, and `src/codegen/expr_call.rs`, initializing omitted trailing aggregate fields from their default member initializers rather than zero-initializing. Confirmed that user-declared constructors correctly disqualify aggregates while NSDMI remains allowed.

### Verification Results

- Unit and integration tests: 438 corpus tests on disk, 438 executed, 438 passed, 0 failures. All 120 negative tests passed.
- Unit tests: 89 passed in `src/lib.rs`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Diagnostic positions test: confirmed all corpus diagnostics name real source locations.
