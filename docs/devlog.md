# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppbolt`.

---

## 2026-11-23: Typed Exception Handling Correctness Audit and Lifecycle Hardening

Audited, verified, and resolved edge cases across typed C++ exception handling:

### Summary of Work

1. Resolved exception destruction and scratch register corruption (`src/codegen/stmt_throw.rs`):
   - Fixed `emit_destroy_exception_if_not_rethrown` to avoid clobbering `scratch1` when it holds the exception pointer, routing destructor table index loads to `scratch2`.
   - Added null-pointer check before loading metadata or calling `free`.
2. Handled nested exceptions and early catch exits (`src/codegen/stmt_try.rs`, `src/codegen/stmt_return.rs`, `src/codegen/stmt.rs`, `src/codegen/stmt_loop.rs`):
   - Wrapped catch clause bodies in an inner exception handler. When a newly thrown exception exits a catch block, the runtime destroys the handled exception object before propagating the new exception, while preserving it on `throw;`.
   - Tracked active handled exception slots in `FunctionContext::active_exception_slots`.
   - Destroyed active exception objects when exiting catch blocks via `return`, `break`, or `continue`.
   - Maintained `current_block_depth` across try and catch blocks so control flow jumps target the correct block depth.
3. Cleaned up allocations on constructor failure (`src/codegen/stmt_throw.rs`):
   - Enclosed payload construction in a `try_table` guard. If a copy or move constructor throws during exception initialization, the allocated heap buffer is freed before propagating the constructor's exception.
4. Preserved exception destructors during dead-code elimination (`src/codegen/analysis.rs`, `src/codegen/program_prepare.rs`, `src/codegen/program_prepare_runtime.rs`):
   - Retained class destructors and their transitive call trees when programs contain try blocks or throw statements, ensuring indirect calls through the exception destructor table do not trap.
   - Collected lifecycle references for catch-by-value parameters and typed initializer lists.
5. Resolved pointer type collisions and multiple inheritance pointer adjustments (`src/sema/exception_types.rs`, `src/codegen/exception_can_catch.rs`, `src/codegen/stmt_try_param.rs`):
   - Replaced overlapping primitive pointer IDs with distinct type intervals, preventing accidental matches across unrelated pointer types (such as `int*` and `double*`).
   - Supported `nullptr` catch matching for pointer handlers.
   - Adjusted dereferenced pointer values directly for multiple inheritance base conversions rather than adjusting the payload buffer address, preserving null pointers.
6. Implemented move-constructor selection and unnamed catch parameters (`src/codegen/stmt_throw.rs`, `src/codegen/stmt_try_param.rs`):
   - Preferred move constructors when throwing automatic non-const local variables or xvalues per ISO C++17 [class.copy.elision].
   - Constructed temporary parameters, performed slicing, and scheduled destructors for unnamed catch-by-value parameters.
7. Verified architectural boundaries:
   - Confirmed `volatile` qualification is absent from AST `Type` representations and deferred until general cv-qualification improvements.
   - Verified that multi-level pointer qualification conversions correctly reject invalid conversions (such as `T**` to `const T**`) while documenting that `const T* const*` catching `T**` remains unsupported.

### Verification Results

- Exception test suite: all 64 tests in `tests/exceptions.rs` pass.
- Full test suite: unit tests, integration tests, nesting checks, and `tests/oracle_gate.rs` pass.
- Census audit: all 7 tests in `tests/census_fixture.rs` pass (0 files over 500 lines, 0 functions over 80 lines, 0 functions with >5 parameters excluding self).
- Clippy: `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- Documentation checks: `python3 tools/check_feature_matrix.py` and `mdbook build docs/book` pass without error.

---

## 2026-11-23: Exception Handling Phase 4: Typed Exception Objects and Catch Matching

Implemented C++17 typed exception objects, dynamic catch matching, class inheritance conversions, reference and value bindings, and exception object lifecycle management:

### Summary of Work

1. Implemented typed exception object representation and lifecycle:
   - Exception objects allocate a heap buffer containing a 16-byte metadata header: `[offset 0: type_id (u32), offset 4: dtor_table_idx (i32), offset 8: flags (u32), offset 12: size (u32)]`, followed by the payload at offset 16.
   - WebAssembly tag 0 carries the 32-bit pointer to this header.
   - Thrown class objects initialize the payload via copy constructors or byte copies; throwing rvalue temporaries deactivates temporary cleanup and transfers ownership directly to the exception object.
   - Registered class destructor table indices in `src/codegen/program_emit.rs` for indirect destructor invocation upon catch completion.
   - Handlers destroy the exception object exactly once (via `call_indirect` on `dtor_table_idx`) and free the heap buffer when catching finishes normally or returns.
2. Implemented dynamic catch matching helper (`__cppbolt_can_catch` in `src/codegen/exception_can_catch.rs`):
   - Compiles a WebAssembly function testing exact type IDs, pointer conversions (`char*` to `const char*`, object pointers to `void*` or `const void*`), and derived-to-base class inheritance.
   - Returns the non-negative subobject byte offset on a match, or -1 on non-match.
   - Supports single and multiple inheritance hierarchies, adjusting pointer addresses to base subobjects.
3. Implemented catch parameter binding and dispatch in `src/codegen/stmt_try.rs`:
   - Lowered `try_table` clauses to route tag 0 into a typed dispatch block that evaluates each catch clause sequentially.
   - Catch by reference (`catch (T&)` or `catch (const T&)`): binds the adjusted payload address directly to the local reference slot, enabling zero-copy access and in-place mutation without slicing.
   - Catch by value (`catch (T)`): allocates local frame storage and invokes the copy constructor (or performs byte copy) from the adjusted payload address, slicing derived objects to base subobjects and destroying the local copy at catch scope exit.
   - Unmatched clauses in a `try` block propagate the exception by rethrowing tag 0 to the enclosing scope.
   - Preserved rethrow semantics (`throw;`): rethrows the active exception object pointer without premature destruction.
4. Added semantic validation in `src/sema/exception_types.rs`:
   - Validated thrown expressions, rejecting `void`, incomplete types, and abstract classes.
   - Validated catch parameters, rejecting abstract classes caught by value and rejecting rvalue-reference catch handlers (`catch (T&&)`).
5. Added end-to-end test coverage in `tests/exceptions.rs`:
   - Verified throwing and catching classes by value, const reference, and non-const reference with in-place mutation.
   - Verified single and multiple inheritance conversions with non-zero subobject offset adjustments.
   - Verified exact-type handler precedence before base-class handlers.
   - Verified unmatched exception propagation to outer `try` blocks.
   - Verified single destruction of thrown objects on catch exit and preservation across rethrows (`throw;`).
   - Verified compile-time diagnostics for invalid throw and catch types.

### Verification Results

- Exception test suite: all 56 tests in `tests/exceptions.rs` pass.
- Repository test suite: all unittests and integration tests pass, including `tests/oracle_gate.rs` (native GCC/Clang comparisons).
- Census audit: all 7 tests in `tests/census_fixture.rs` pass (0 files over 500 lines, 0 functions over 80 lines, 0 functions with >5 parameters excluding self).
- Clippy: `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- Documentation: `mdbook build docs/book` succeeds with 0 errors, and `python3 tools/check_feature_matrix.py` validates all feature matrix test links.

---

## 2026-11-23: Exception Handling Phases 1–3 Hardening: Multiple Catches, Conditional Temporaries, and 32-Bit Payload ABI

Hardened exception handling across Phase 1 (Infrastructure), Phase 2 (Scope-Aware Unwinding), and Phase 3 (Unwinding Correctness):

### Summary of Work

1. Implemented multiple `catch` clauses per `try` statement:
   - Updated AST representation to `StmtKind::Try { try_body, catches: Vec<CatchClause> }`.
   - Updated the parser to accept arbitrary sequences of catch handlers (`catch (Type param)` and `catch (...)`).
   - Extended semantic analysis and template instantiation to scope and check each catch clause sequentially.
   - Enforced ISO C++17 [except.handle] handler ordering: `catch (...)` must appear as the final clause in any try block.
   - Lowered multiple catches to WebAssembly `try_table` clauses with sequential enclosing target blocks, branching to matching handlers and unwinding only local try-scope cleanups.
2. Guarded normal-path and conditional temporary destruction:
   - Added bitmask and non-null checks in `emit_clean_temporaries` and `emit_destroy_and_free_temp`.
   - Guaranteed that unexecuted branches in ternary expressions (`?:`) and short-circuit operators (`&&`, `||`) never invoke destructors or `free()` on unconstructed temporary instances.
3. Extended reference binding lifetime across conditional expressions:
   - Transferred all candidate temporaries created in reference initializers (`const T& ref = cond ? T(1) : T(2)`) to the enclosing scope.
   - Preserved lifetime of whichever instance was dynamically constructed until enclosing scope exit without double-freeing unconstructed instances.
4. Validated thrown expression types and supported pointer payloads:
   - Added semantic validation (`is_supported_32bit_throw_type`) enforcing that thrown expressions evaluate to 32-bit scalar, enum, or pointer types matching WebAssembly tag 0 (`(i32) -> void`).
   - Rejected non-32-bit and floating-point throw expressions with informative diagnostics.
   - Supported pointer payload passing (`throw "msg"` / `catch (const char* e)`).
5. Validated 64-cleanup-object architecture:
   - Confirmed compile-time error enforcement when local cleanup variables reach 64 objects (`test_exceeding_max_cleanup_objects_errors`).
   - Confirmed graceful fallback for expressions with over 64 chained temporaries (`wide_string_concatenation_compiles`).

### Verification Results

- Exception test suite: all 41 tests in `tests/exceptions.rs` pass.
- Census audit: all 7 tests in `tests/census_fixture.rs` pass (all source files under 500 lines, functions under 80 lines, parameters <= 5).
- Oracle gate: all 909 corpus tests pass against native GCC/Clang compilers.

---

## 2026-11-21: C++17 SFINAE, Two-Phase Lookup, <type_traits>, and Constant Evaluation

Implemented C++17 template substitution failure handling, two-phase name lookup with argument-dependent lookup (ADL), expression SFINAE via `decltype`, LLVM libc++ 23.1.5 `<type_traits>` integration, and constant expression hardening.

### Summary of Work

1. Added typed substitution failures (`SubstitutionFailure` and `TemplateFailure`) in `src/diagnostics/mod.rs` and `src/sema/template_func.rs`. Invalid types formed in signature immediate contexts discard the candidate from overload resolution instead of aborting compilation.
2. Implemented expression SFINAE and `decltype` resolution in `src/sema/decltype_resolve.rs`. Analyzed operands on disposable `Sema` clones, applied C++17 value-category rules, added trailing return types, and bound parameters sequentially so later parameters can reference earlier ones.
3. Implemented two-phase name lookup and ADL in `src/sema/analyze_decls.rs`, `src/sema/template_call_dependence.rs`, and `src/sema/overload_adl.rs`. Captured visible declarations at template definition points, classified call and operator expressions by source position, tracked dependent local variables (`auto x = t;`), and restricted ADL associated namespaces to exclude enclosing classes of nested types.
4. Expanded partial specialization pattern matching in `src/sema/template_match*.rs`. Expanded alias templates before deduction, deduced multidimensional array bounds (`T[N]`, `T[]`), confirmed nested `decltype` patterns through source re-parsing, and rejected non-deducible partial specialization declarations.
5. Integrated an adapted C++17 subset of LLVM libc++ 23.1.5 `<type_traits>`. Added 36 headers under `stdlib/detail/type_traits/` and created `stdlib/detail/type_traits/builtins.h` to supply compiler builtins using partial specializations.
6. Hardened constant evaluation and static assertions in `src/sema/analyze.rs` and `src/sema/constexpr_globals.rs`. Resolved qualified static constexpr members, rejected non-constant expressions in `static_assert` conditions, and anchored diagnostic coordinates to offending expressions.
7. Maintained repository census limits and memory safety: decomposed oversized files and functions across `src/sema/` and `src/parser/` (all files under 500 lines, functions under 80 lines), and optimized iterative AST dropping in `src/ast/expr_drop.rs` to avoid redundant drop traversals.

### Verification Results

- Unit tests: all library unit tests pass.
- Census audit: `cargo run --bin cppbolt-census` reports 0 files over 500 lines, 0 functions over 80 lines, and 0 functions with more than 5 non-receiver parameters.
- Clippy: `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- Documentation: `mdbook build docs/book` succeeds with 0 errors, and `python3 tools/check_feature_matrix.py` validates all 281 feature matrix test links.

---

## 2026-11-16: Pointer-to-Member Operators, ABI Representation, and C++17 Fold Integration

Implemented C++ pointer-to-member language support (ISO C++17 [dcl.mptr], [expr.mptr.oper], [conv.mem]), flat 64-bit Itanium ABI representation, and fold expression integration across the parser, semantic analyzer, WebAssembly code generator, and test suite:

### Summary of Work

1. Extended the type system and AST in `src/ast/types.rs`, `src/ast/expr.rs`, and `src/ast/fold_op.rs`: added `Type::MemberPointer { class_name, member_type }`, binary operators `BinaryOp::DotStar` (`.*`) and `BinaryOp::ArrowStar` (`->*`), and fold operators `FoldOp::DotStar` and `FoldOp::ArrowStar`.
2. Implemented syntax parsing in `src/parser/decl_func.rs`, `src/parser/types.rs`, and `src/parser/expr.rs`: supported nested `Class::*` specifiers in simple and parenthesized member function declarators, and added precedence 14 Pratt parsing for `.*` and `->*` operators.
3. Implemented semantic analysis in `src/sema/member_pointer.rs`, `src/sema/assign.rs`, and `src/sema/template_deduce_type.rs`: resolved member pointer addresses (`&Class::member`), validated receiver and member pointer compatibility, enforced const qualification propagation, supported base-to-derived conversions (`Base::*` to `Derived::*`), and added structural template argument deduction for `Type::MemberPointer`.
4. Lowered pointer-to-member operations to WebAssembly in `src/codegen/member_pointer.rs` and `src/codegen/memory.rs`:
   - Data-member pointers lower to WebAssembly `i32` byte offsets within struct layouts, using `-1` to encode null data-member pointers.
   - Member-function pointers lower to WebAssembly `i64` packing `{ ptr: u32, adj: u32 }` with `0i64` as null. Bit 0 of `ptr` discriminates between non-virtual function table calls (bit 0 = 0) and virtual vtable calls (bit 0 = 1). The high 32 bits (`adj`) adjust the `this` pointer for multiple inheritance subobject navigation.
   - Dispatch lowers to `call_indirect`, staging operands through local frame slots.
   - Emitted guarded base-to-derived conversions adjusting member offsets and function `adj` fields for non-null pointers.
5. Enabled C++17 fold expressions over `.*` and `->*` in `src/sema/fold_expansion.rs`, expanding parameter packs into left- or right-associative binary expression trees.
6. Added positive test coverage in `tests/classes_oop/` and `tests/cxx17/` for data members, non-virtual methods, virtual methods, multiple inheritance adjustments, and fold expressions, along with negative diagnostic tests in `tests/negative/`.
7. Documented the architecture and ABI in ADR 0015 (`docs/adr/0015-pointer-to-member.md`), updated ADR 0014 (`docs/adr/0014-cxx17-fold-expressions.md`), and updated compiler documentation in `docs/book/internals/parser-and-ast.md` and `docs/book/user/supported-cpp.md`.

### Verification Results

- Corpus test runner: 583/583 corpus tests passed on disk, 0 failures.
- Unit tests: 134/134 unit tests passed in `src/lib.rs`, 0 failures.
- Native oracle gate: all mini_stl tests match native Clang and GCC byte-for-byte.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings under `-D warnings` across all targets.
- Book build: `mdbook build docs/book` succeeds with zero errors.

---

## 2026-11-16: C++17 Fold Expressions Implementation

Implemented C++17 fold expressions ([expr.prim.fold]) across the AST, parser, semantic analyzer, template substitution pipeline, and constant evaluator:

### Summary of Work

1. Added `FoldOp` in `src/ast/fold_op.rs` representing all 32 C++17 fold operators (arithmetic, bitwise, relational, logical, comma, assignment, compound assignment, and pointer-to-member), and added `Expr::FoldExpr { op, left, right, pos }` in `src/ast/expr.rs`.
2. Implemented fold expression parsing in `src/parser/expr_fold.rs` and `src/parser/prefix.rs`. The parser uses `has_depth1_ellipsis` lookahead to distinguish parenthesized folds without backtracking, parses all four shapes (unary left, unary right, binary left, binary right), and validates that binary folds use identical operator tokens.
3. Implemented semantic fold expansion in `src/sema/fold_expansion.rs`. During template substitution, `expand_fold_expr` classifies fold shapes, validates that multi-pack expansions share identical pack lengths, slices parameter packs, and constructs concrete binary expression trees with standard left or right associativity.
4. Implemented ISO C++17 [temp.variadic] empty pack rules: empty unary folds over `&&` evaluate to `true`, `||` to `false`, and `,` to a void expression (`Expr::IntLiteral(0, Type::Void)`), while other operators trigger compile errors when expanded over empty packs.
5. Integrated fold expressions into non-type template arguments, dependent array bounds (`src/parser/types_array.rs`), and compile-time evaluation (`src/sema/constexpr_eval.rs`, `src/sema/template_subst_nontype.rs`), allowing constructs like `template <int... Ns> struct S { int arr[(... + Ns)]; };` to evaluate during template instantiation.
6. Enforced diagnostics rejecting unexpanded fold expressions outside template bodies or fold expressions containing packs in both operands.
7. Added positive end-to-end test cases in `tests/cxx17/` and negative diagnostic tests in `tests/negative/`, along with parser unit tests in `src/parser/tests_fold.rs` and semantic unit tests in `src/sema/fold_tests.rs`.
8. Documented the architecture in ADR 0014 (`docs/adr/0014-cxx17-fold-expressions.md`) and updated compiler book internals (`docs/book/internals/parser-and-ast.md`, `docs/book/user/supported-cpp.md`).

### Verification Results

- Test runner: 576/576 corpus tests passed, 0 failures.
- Unit tests: 131/131 unit tests passed, 0 failures.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings under `-D warnings` across all targets.
- Book build: `mdbook build docs/book` succeeds with zero errors.

---

## 2026-11-16: C++11 Variadic Templates and Multiple-Inheritance Remediation

Resolved the remaining C++11 variadic template and multiple-inheritance defects identified during independent compiler audits:

### Summary of Work

1. Non-type template argument substitution now resolves `sizeof...(Ts)` and arithmetic expressions containing pack lengths (`src/sema/template_subst_nontype.rs`, `src/sema/template_subst.rs`, `src/sema/template_name.rs`). The substitution engine evaluates dependent NTTP expressions with active pack mappings and simplifies canonical argument strings for template instance lookup.
2. Implemented multiple-inheritance subobject offsets and pointer adjustments across the compiler (`src/sema/layout.rs`, `src/codegen/memory.rs`, `src/codegen/expr.rs`, `src/codegen/lvalue.rs`, `src/codegen/stmt_var_init.rs`, `src/codegen/function_global_init.rs`, `src/codegen/expr_call.rs`, `src/codegen/call_args.rs`). Derived-to-base and base-to-derived conversions adjust pointer and reference offsets, member access resolves secondary base field positions, and static casts validate inheritance relations.
3. Added template argument deduction for function pointers (`src/sema/template_deduce_type.rs`), deducing return types and parameter packs directly from function types or pointers (`R (*)(Args...)`) without requiring trailing call arguments.
4. Resolved nested dependent type declarations in local scopes (`src/parser/mod.rs`, `src/parser/types.rs`, `src/sema/template_subst.rs`). The parser recognizes member templates and qualified dependent types without misclassifying `<` tokens as comparison expressions, and template substitution replaces qualifier template instances.
5. Added local type alias declarations (`using Res = Target<Ts...>;`) and block-scoped resolution (`src/ast/stmt.rs`, `src/parser/stmt.rs`, `src/sema/mod.rs`, `src/sema/stmt.rs`, `src/sema/template_subst_stmt.rs`). Local aliases resolve inside their enclosing blocks without leaking to outer scopes.
6. Expanded `alignas` pack handling to support `alignof(Ts)...` expressions and non-type packs (`src/parser/struct_align.rs`, `src/sema/template_subst_align.rs`), computing maximum required alignments across all expanded elements.
7. Guarded multiple-inheritance pointer conversions to preserve null pointer values (`(Base2*)(Derived*)nullptr == nullptr`) without applying subobject offsets to zero addresses (`src/codegen/memory.rs`).
8. Propagated composite struct alignments into dependent `alignas` evaluation, and threaded caller coordinates into type substitution errors (`src/sema/template_subst_align.rs`, `src/sema/pack_expansion.rs`, `src/sema/template_subst.rs`).
9. Added `std::get<I>(tuple&&)` rvalue overloads and enabled nested template argument parsing in type lists (`stdlib/tuple`, `src/parser/types.rs`), permitting move extraction of move-only types from tuples.

### Verification Results

- Test runner: 563/563 corpus tests passed, 0 failures.
- Unit tests: 119/119 unit tests passed, 0 failures.
- Diagnostic positions harness: passed with zero fabricated coordinates.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Differential verification: byte-for-byte output match against host native Clang in C++11 mode across positive test suites.
- Clippy: 0 warnings under `-D warnings` across all targets.

---

## 2026-11-15: C++11 Variadic Templates Completion and Audit Remediation

Completed C++11 variadic template support across the compiler and embedded standard library based on an independent Clang comparison audit:

### Summary of Work

1. Resolved base pack expansion length from the base pack in `mapping` in `src/sema/template_subst_func.rs`, fixing `: Bases(v)...` when `v` is a non-pack scalar.
2. Extended call-graph reachability traversal in `src/codegen/analysis.rs` to visit `InitializerList`, `TypedInitList`, `PackExpansion`, and `SizeofExpr`. This preserves functions invoked through swallow idiom expressions (`int dummy[] = { (f(args), 0)... };`) during dead code elimination. Factored static local collection into `src/codegen/analysis_vars.rs`.
3. Preserved dependent array bounds (`int a[sizeof...(Ts)];`) during parsing as `Type::DependentArray` in `src/parser/types_array.rs`, evaluating the bound at template instantiation in `src/sema/template_subst_array.rs`.
4. Converted trailing non-type pack arguments to `TemplateArg::NonTypePack` in `src/sema/template_match.rs` for partial specializations like `Sum<First, Rest...>`.
5. Replaced heuristic parameter-count tie-breaking in `src/sema/template_partial_order.rs` with deduction from synthetic types per ISO C++11 [temp.func.order] (`deduces_from_synthetic`). Ambiguous overloads remain ambiguous.
6. Relaxed trailing pack validation for partial specializations in `src/parser/template.rs`, supporting multi-pack specializations such as `ZipCount<TypeList<As...>, TypeList<Bs...>>`.
7. Added semantic diagnostics in `src/sema/expr.rs` for `sizeof...(non_pack)` and pack expansions without unexpanded parameter packs.
8. Added parsing and substitution for template-template parameters and parameter packs (`template <template <typename...> class... Containers>`) in `src/parser/template_param.rs` and `src/sema/template_name.rs`.
9. Removed single-base restrictions in `src/sema/layout.rs`, laying out multiple base subobjects sequentially with offset adjustments, multi-base method dispatch, and duplicate direct base class rejection.
10. Supported `[args...]` and `[&args...]` in `src/parser/expr_lambda.rs` and expanded them into individual closure captures during substitution in `src/sema/template_subst_lambda.rs`. Factored free variable analysis into `src/sema/expr_lambda_free_vars.rs`.
11. Implemented structural deduction in `src/sema/template_deduce_type.rs` for function template parameters containing pack expansions inside template instances, such as `f(Tuple<As...>, Tuple<Bs...>)` and `g(Tuple<First, Rest...>)`.
12. Supported nested member class templates containing parameter packs (`Outer<Ts...>::Inner<Us...>`) in `src/parser/structs_nested.rs` and `src/sema/template_nested.rs`, substituting outer parameters while preserving inner parameters.
13. Added `alignas(Ts...)` and `alignof(T)` in `src/parser/struct_align.rs`, evaluating maximum alignment across pack types in `src/sema/template_subst_align.rs` and `src/sema/layout.rs`.
14. Removed the non-standard `f(args)...;` expression-statement extension from `src/parser/stmt.rs`.
15. Fixed explicit function-template argument pack deduction in `src/sema/template_deduce.rs`, starting trailing argument deduction at `non_pack_count + explicit_pack.len()` so calls like `foo<int, int>(1, 2)` do not duplicate deduced parameters or produce mismatched WebAssembly call arity.
16. Implemented alias template parameter pack expansion in `src/sema/template_alias.rs` by grouping trailing template arguments into `TemplateArg::TypePack` via `build_template_arg_map`.
17. Enforced compile-time diagnostics in `src/sema/template_spec.rs` rejecting unexpanded parameter packs in class template data member types and method parameter types.
18. Supported function pointer parameter packs (`void (*fn)(Args...)`) in `src/parser/decl_func.rs` and `src/sema/template_subst.rs`, expanding pack parameters in function pointer signatures.
19. Supported C++11 dynamic exception specifications (`throw(Ts...)` and `throw()`) in `src/parser/decl_func.rs`.
20. Preserved lvalue reference value categories in `src/codegen/lvalue.rs` and `src/codegen/overload.rs` for call expressions returning references (such as `std::forward<T>`), preventing spurious scalar loads during nested call argument conversion.

### Verification Results

- Test runner: 549/549 corpus tests passed, 0 failures.
- Unit and integration tests: 119 unit tests passed, 0 failed.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Native oracle gate: 100% byte-for-byte output match against host native Clang in C++11 mode across all positive tests.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-15: C++11 Variadic Templates, Parameter Packs, and std::tuple

Added C++11 variadic template support across the compiler and embedded standard library: parameter packs, pack expansion in expressions and types, `sizeof...`, argument deduction, forwarding references, function template partial ordering, and `<tuple>`.

### Summary of Work

1. Added `TokenKind::Ellipsis` (`...`) in `src/lexer/token.rs` and `src/lexer/mod.rs`, tokenizing triple dots as a single atomic token.
2. Added AST representations for variadics in `src/ast/`: `TemplateParam::is_pack` for type and non-type template parameter packs, `Param::is_pack` for function parameter packs, `Type::PackExpansion` for type packs, `Expr::SizeofPack` for compile-time pack length queries, `Expr::PackExpansion` for expression patterns, and an AST placeholder `Expr::FoldExpr` for future C++17 fold expressions.
3. Implemented parsing in `src/parser/`: template parameter packs (`typename... Ts`, `class... Ts`, `int... Ns`), function parameter packs (`Ts... args`, `Ts...`), `sizeof...(pack)` queries, and pack expansions across call argument lists, braced initializer lists, base class lists (`struct S : Bases...`), constructor member initializer lists (`: Bases(args)...`), and template argument lists (`Tuple<Tail...>`). Disallowed misplaced parameter packs in class templates and template aliases.
4. Implemented parameter pack expansion in `src/sema/pack_expansion.rs` (`TemplateArg`, `expand_type_list`, `expand_expr_list`). The engine checks that co-expanded packs share identical lengths, rejects unexpanded parameter packs, isolates pack elements with reserved `__pack_{name}_{i}` synthetic identifiers, and handles empty expansions.
5. Integrated compile-time `sizeof...(pack)` evaluation into `src/sema/constexpr_eval.rs` and `src/sema/expr.rs`, evaluating pack lengths directly from substitution mappings and context tables as `size_t` constants.
6. Implemented variable-arity class template instantiation and trailing pack matching in `src/sema/template_deduce.rs`, `src/sema/template_registry.rs`, and `src/sema/template_match.rs`, enabling recursive template definitions such as `Tuple<Head, Tail...> : Tuple<Tail...>`. Made deduction fallback deterministic.
7. Extended function template argument deduction in `src/sema/template_deduce.rs` to deduce parameter packs from trailing arguments, including empty packs and forwarding reference packs (`Args&&...`) with reference collapsing rules.
8. Implemented function template partial ordering in `src/sema/template_partial_order.rs` per C++11 [temp.func.order]. Ranked non-pack parameters as strictly more specialized than parameter packs, and integrated partial ordering into function template resolution (`src/sema/template_func.rs`) and overload selection (`src/sema/expr_call.rs`).
9. Enforced single inheritance on base class pack expansion in `src/sema/template_subst.rs`, diagnosing multiple base class expansions as unsupported to protect the WebAssembly record layout.
10. Added explicit error handling in `src/codegen/expr.rs` to diagnose and reject unsupported or unlowered AST expressions that reach code generation.
11. Updated the embedded standard library:
   - Added variadic `std::make_unique<T>(Args&&... args)` to `stdlib/memory`.
   - Added `std::integer_sequence`, `std::index_sequence`, and `std::make_index_sequence` to `stdlib/utility`.
   - Added `stdlib/tuple` implementing `std::tuple`, recursive base-indexed element storage, `std::make_tuple`, and `std::get<I>(tuple)` supporting up to 17 elements (indices 0..16).
12. Added test coverage:
    - Positive tests: `tests/templates_operators/variadic_func_basic.cpp`, `variadic_class_tuple.cpp`, `variadic_forwarding.cpp`, `variadic_partial_ordering.cpp`, and mini-STL tests `tests/stdlib/memory/variadic_make_unique.cpp`, `std_tuple_basic.cpp`, `std_tuple_large.cpp`.
    - Variant tests in `tests/templates_operators_variant/` and `tests/stdlib_variant/`.
    - Negative diagnostic tests in `tests/negative/variadic_ambiguous_call.cpp`, `variadic_unexpanded_pack.cpp`, `variadic_pack_mismatch.cpp`, `variadic_multiple_bases.cpp`.

### Verification Results

- Unit and integration tests: 119 unit tests passed, 0 failed.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Oracle gate: 100% byte-for-byte stdout match against host native GCC 11 and Clang 11.
- Corpus test runner: 528/528 tests passed, 0 failures.
- Golden baseline: all 382 entries match `tests/golden.txt`.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-12: Standard Library Correctness and Iterator Fixes

Addressed review findings across the embedded standard library and the semantic analyzer, and completed the C++17 `<iterator>` and `<functional>` surface.

### Summary of Work

1. Reworked `std::reverse_iterator` in `stdlib/detail/reverse_iterator.h` to derive `iterator_category`, `difference_type`, `value_type`, `reference`, and `pointer` from `iterator_traits<_Iter>` instead of a second template parameter, matching the standard one-parameter form. `<array>` now uses the standard typedefs, and `operator->`/`operator*` are covered for pointer and class iterators.
2. Removed modulo bias from `std::shuffle` and `std::sample` by adding URBG-aware rejection sampling in `stdlib/detail/algorithms/random.h`. The helper honors `min()`/`max()` and rejects values outside the largest evenly divisible range.
3. Fixed member-call analysis in `src/sema/expr_call.rs` so static member calls no longer assume an implicit `this` parameter, which previously panicked when a static function had no parameters.
4. Removed a duplicate `std::detail::DefaultLess` definition from `stdlib/algorithm`; the canonical definition remains in `stdlib/detail/algorithms/comp.h`.
5. Moved C++17 aggregate brace-elision resolution into `src/sema/init_aggregate.rs`, keeping `src/sema/init.rs` within the repository's file and function size limits.
6. Expanded `tests/stdlib/algorithm/algorithm_infra.cpp` and `tests/stdlib/algorithm/algorithm_merge_set_select.cpp` with reverse-iterator typing, out-of-range URBG rejection, and permutation-property coverage that does not depend on a specific shuffle implementation.

### Verification Results

- Unit and integration tests: 513 corpus tests on disk, 513 executed, 513 passed, 0 failures. All negative tests passed.
- Native oracle gate: every recorded expectation matches host GCC and Clang.
- Independent header gate: 350/350 tests pass with both `g++` and `clang++` using `-nostdinc++ -I stdlib`.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

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
11. Added regression test coverage in `tests/classes_oop/structs/` and differential test suites against native Clang in `tests/stdlib/array/array_comprehensive.cpp` and `tests/stdlib_variant/array/array_comprehensive_variant.cpp`.

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
8. Added unit, integration, variant, and negative diagnostic tests across `tests/templates_operators/`, `tests/stdlib/`, and `tests/negative/`.

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
9. Added corpus tests across `tests/c_basics/types/`, `tests/classes_oop/`, `tests/templates_operators/`, `tests/stdlib/`, and `tests/negative/`.

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
9. Added 5 variant tests: `tests/templates_operators_variant/template_class_methods_variant.cpp`, `tests/stdlib_variant/utility/pair_basic.cpp`, `tests/stdlib_variant/vector/vector_basic.cpp`, `tests/stdlib_variant/iterator/iterators_range_for_variant.cpp`, `tests/stdlib_variant/algorithm/algorithm_sort_variant.cpp`, and `tests/stdlib_variant/integration/combined_integration_variant.cpp`.

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
- Added test coverage in `tests/c_basics/pointers_arrays/range_for_array.cpp`, `tests/stdlib/string/range_for_string.cpp`, `tests/classes_oop/memory_raii/range_for_raii.cpp`, and `tests/negative/range_for_non_iterable.cpp`.
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
7. Enabled `tests/stdlib/map/map_basic.cpp`, `tests/stdlib/set/set_basic.cpp`, and the 20-seed differential fuzzing suite `tests/stdlib/map/differential_fuzz.cpp` in `tests/PASSING.txt`.

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

---

## 2026-11-10: C++17 <algorithm> Standard Library Integration

Integrated C++17 `<algorithm>` into cppbolt based on LLVM libc++ 23.1.5, avoiding ad-hoc compiler extensions.

### Summary of Work

1. Added modular headers in `stdlib/detail/algorithms/` and `stdlib/detail/iterator.h`, separating iterator operations from algorithm families. Registered headers in `src/preprocessor/include.rs` and included them from `stdlib/algorithm`.
2. Implemented iterator operations in `stdlib/detail/iterator.h` and comparators in `stdlib/detail/algorithms/comp.h`: `iterator_traits`, `distance`, `advance`, `next`, `prev`, `iter_swap`, `__less<>`, `__equal_to`, and `__greater<>`.
3. Integrated min/max and heap functions in `stdlib/detail/algorithms/min_max.h` and `stdlib/detail/algorithms/heap.h`: `min`, `max`, `minmax`, `clamp`, `min_element`, `max_element`, `minmax_element`, `make_heap`, `push_heap`, `pop_heap`, `sort_heap`, `is_heap`, and `is_heap_until`.
4. Integrated non-modifying sequence algorithms in `stdlib/detail/algorithms/non_modifying.h`: `all_of`, `any_of`, `none_of`, `for_each`, `for_each_n`, `count`, `count_if`, `mismatch`, `find`, `find_if`, `find_if_not`, `adjacent_find`, `search`, `find_end`, `find_first_of`, `search_n`, `equal`, and `lexicographical_compare`.
5. Integrated modifying sequence algorithms in `stdlib/detail/algorithms/modifying.h`: `copy`, `copy_n`, `copy_if`, `copy_backward`, `move`, `move_backward`, `fill`, `fill_n`, `transform`, `generate`, `generate_n`, `remove`, `remove_if`, `remove_copy`, `remove_copy_if`, `replace`, `replace_if`, `replace_copy`, `replace_copy_if`, `swap_ranges`, `reverse`, `reverse_copy`, `rotate`, `rotate_copy`, `unique`, and `unique_copy`.
6. Integrated binary search, partitioning, and permutation algorithms in `stdlib/detail/algorithms/binary_search.h`, `stdlib/detail/algorithms/partition.h`, and `stdlib/detail/algorithms/permutation.h`: `lower_bound`, `upper_bound`, `equal_range`, `binary_search`, `is_sorted`, `is_sorted_until`, `is_partitioned`, `partition`, `partition_copy`, `partition_point`, `stable_partition`, `is_permutation`, `next_permutation`, and `prev_permutation`.
7. Integrated merge, set operations, selection, and random sampling in `stdlib/detail/algorithms/merge.h`, `stdlib/detail/algorithms/set_ops.h`, `stdlib/detail/algorithms/selection.h`, and `stdlib/detail/algorithms/random.h`: `merge`, `inplace_merge`, `includes`, `set_union`, `set_intersection`, `set_difference`, `set_symmetric_difference`, `nth_element`, `partial_sort`, `partial_sort_copy`, `stable_sort`, `shuffle`, and `sample`.
8. Kept cppbolt's sorting implementation (`introsort_loop`, `heap_sort`, `insertion_sort`) in `stdlib/algorithm`.
9. Added tests across all algorithm families (`algorithm_infra.cpp`, `algorithm_min_max_heap.cpp`, `algorithm_non_modifying.cpp`, `algorithm_modifying.cpp`, `algorithm_binary_partition_perm.cpp`, `algorithm_merge_set_select.cpp`, `algorithm_comprehensive.cpp`), checking custom functors, lambda predicates, stability, URBGs, and boundary cases.

### Verification Results

- Corpus runner: 513 tests on disk, 513 executed, 513 passed, 0 failures.
- Native oracle gate: all 50 mini_stl tests match native Clang and GCC byte-for-byte.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-11: C++17 Algorithm Stabilization and Iterator Refinement

Addressed review findings in algorithms, iterator primitives, and header lookup.

### Summary of Work

1. Fixed `std::search_n` in `stdlib/detail/algorithms/non_modifying.h` to return immediately when `count == 1`.
2. Aligned `std::minmax_element` in `stdlib/detail/algorithms/min_max.h` with C++17 [alg.min.max] to return the first minimum and last maximum on equal elements.
3. Rewrote `std::partition` in `stdlib/detail/algorithms/partition.h` to use single-pass forward partitioning with at most `last - first` predicate applications.
4. Changed `std::reverse` in `stdlib/detail/algorithms/modifying.h` to use `!=` instead of `<`, supporting non-random-access bidirectional iterators.
5. Added standard iterator category tags (`input_iterator_tag`, `forward_iterator_tag`, `bidirectional_iterator_tag`, `random_access_iterator_tag`) in `stdlib/detail/iterator.h`. Implemented pointer overloads for `_Tp*` and `const _Tp*`, fallback loops for class iterators, and negative step handling in `std::prev`.
6. Added `std::less`, `std::greater`, and `std::equal_to` comparators in `stdlib/detail/algorithms/comp.h` and exposed them through `<functional>`.
7. Updated `std::unique_copy` in `stdlib/detail/algorithms/modifying.h` to store a value copy for single-pass input iterators and advance the output iterator with post-increment.
8. Refactored `embedded_header` in `src/preprocessor/include.rs` to a static table lookup, reducing function length from 80 lines to 12 lines.
9. Added tests in `tests/stdlib/algorithm/algorithm_comprehensive.cpp` for `search_n(..., 1, ...)`, duplicate elements in `minmax_element`, bidirectional iterators, negative `prev`, `const` pointers, and comparator objects.

### Verification Results

- Native oracle gate: all 50 mini_stl tests match native Clang and GCC byte-for-byte.
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with more than 5 parameters excluding self.
- Clippy: 0 warnings with `-D warnings` across all targets.

---

## 2026-11-22: WebAssembly Native Exception Handling and Scope-Aware Unwinding Redesign

Implemented C++17 exception handling (`try`, `catch`, `throw`) in WebAssembly using the native `try_table` proposal and a dynamic 64-bit liveness tracking mask.

### Summary of Work

1. WebAssembly Primitives: Registered exception tag 0 `(i32) -> void` in WebAssembly module section 13 and added `OP_TRY_TABLE` (0x1f), `OP_THROW` (0x08), and `OP_THROW_REF` (0x0a).
2. Lexer, Parser, and AST: Added `TokenKind::KwTry`, `KwCatch`, `KwThrow`, and `Ellipsis` (`...`). Lowered to `StmtKind::Throw(Option<Expr>)` and `StmtKind::Try { try_body, catch_param, catch_body }` in `src/parser/stmt_try.rs`.
3. Scope-Aware Unwinding Engine (`src/codegen/unwind.rs`):
   - Replaced per-call cleanup wrappers with function-level `try_table (catch_all_ref $unwind)` wrapping throwing and calling functions.
   - Added a 64-bit dynamic liveness bitmask (`cleanup_mask_local: i64`) updated at construction and scope exit so destructors run only for active objects in reverse construction order.
   - Added a compile-time check rejecting functions with 64 or more cleanup objects.
   - Wrapped unwinding cleanups in `try_table (catch_all -> unreachable)` to invoke `std::terminate` when a destructor throws during unwinding.
   - Added AST traversal (`stmt_can_throw_or_call`) to omit unwinding code from leaf functions.
4. Try/Catch Statement Lowering (`src/codegen/stmt_try.rs`):
   - Lowered typed catches with payload binding into frame slots.
   - Supported catch-all (`catch (...)`) with `exnref` capture.
   - Restricted catch-entry cleanups to objects constructed inside the `try` block, preserving outer local objects.
   - Lowered `throw;` to rethrow active exceptions (`throw_ref` or `throw 0`), and emitted `unreachable` (`std::terminate`) when evaluated outside catch blocks.
5. Constructor Failure Handling (`src/codegen/function_lifecycle.rs`):
   - Registered base and member initializers in the cleanup mask, destroying completed subobjects if a subsequent initializer or constructor body throws.
6. Shadow Stack Integrity: Restored caller frame pointer on the shadow stack (`global 0`) in the function unwind handler before propagating unhandled exceptions.
7. Tests: Added 15 integration tests in `tests/exceptions.rs` covering cross-function unwinding, nested try blocks, loop allocations, throwing destructors, shadow-stack integrity, rethrow termination, and cleanup limits.

### Verification Results

- All 143 unit tests passed (`cargo test --lib`).
- All 15 exception integration tests passed (`cargo test --test exceptions`).
- All 16 C++17 regressions passed (`cargo test --test cxx17_regressions`).
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with > 5 parameters without receiver.

---

## 2026-11-22: Arbitrary Nested Catch Depth and Temporary Unwinding Hardening

Hardened the function-level exception unwinding engine by removing the fixed catch-depth clamp and tracking temporary object cleanups across expressions.

### Summary of Work

1. Arbitrary Catch Nesting Depth (`src/codegen/function_context.rs`, `src/codegen/stmt_try.rs`):
   - Replaced the fixed four-slot `exnref` clamp with an AST pre-pass (`max_catch_depth`) that computes the peak catch nesting depth for each function.
   - Allocated `(max_depth + 1)` `TYPE_EXNREF` locals to assign dedicated slots to each nested handler, separating the function unwind handler (`unwind_exnref_local`) from active catch scopes.
   - Saved previous exception payloads on the shadow stack before entering nested handlers and restored them upon handler exit. This ensures that `throw;` correctly rethrows the enclosing handler's exception.
2. Temporary Object Unwinding (`src/codegen/call_args.rs`, `src/codegen/expr_call.rs`, `src/codegen/raii.rs`, `src/codegen/unwind.rs`):
   - Added temporary cleanup registration (`register_temp_cleanup`) so fully constructed temporaries run destructors and free heap buffers if a subsequent argument expression or called function throws.
   - Preserved nested sub-expression temporaries during by-value calls (`compile_call_arguments_and_deactivate_byval`), deactivating only the outer argument passed to the callee.
   - Supported reference lifetime extension (`stmt_var.rs`) by transferring temporaries bound to references into lexical scope tracking.
   - Permitted statements with wide chained expressions to compile cleanly by returning `None` from `register_temp_cleanup` when the 64-bit cleanup mask is saturated, relying on normal full-expression destruction.
3. Test Renaming and Consolidation:
   - Renamed `tests/exceptions_prototype.rs` to `tests/exceptions.rs` and expanded the suite to 32 tests covering nested catch depth, argument failures, chained temporaries, and lifetime extension.

### Verification Results

- All 143 unit tests passed (`cargo test --lib`).
- All 32 exception integration tests passed (`cargo test --test exceptions`).
- All 11 nesting stress tests passed (`cargo test --test nesting`).
- All 16 C++17 regressions passed (`cargo test --test cxx17_regressions`).
- Census audit: 0 files over 500 lines, 0 functions over 80 lines, 0 functions with > 5 parameters without receiver.
