# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Two-Phase Lookup and ADL (C++17)
- Ordinary lookup inside a function template now uses the overloads visible at the template's definition. A function declared later in the file can no longer rebind a name in that template.
- Dependent unqualified calls collect ADL candidates from the associated namespaces and classes of the instantiated arguments, and merge them with ordinary candidates. Associated entities include base classes, template type arguments, pointers, references and arrays. Qualified calls do not use ADL.
- Friend functions defined inside a class are parsed and registered for ADL through that class. Ordinary unqualified lookup cannot find them.
- Dependent calls in trailing return types, `decltype` and `void_t` detection continue to use substitution failures for invalid immediate-context calls while nested template instantiation errors stay hard.
- Function and member templates declared after an earlier template no longer enter that template's ordinary lookup. Self-recursive templates still see themselves.
- ADL also runs for names shared with builtins such as `free`, and friend operators parse as function names.
- Non-dependent calls inside templates now freeze both ordinary lookup and ADL at the template definition. Dependent calls keep instantiation-time ADL. Definition snapshots also cover class-template methods, member-template signatures and using declarations.
- Hidden friends declared in class templates are registered for each concrete specialization and found through ADL only. Repeated instantiation does not duplicate them. Their bodies are analyzed like other instantiated functions, so default arguments apply.
- Calls to a class-template's own hidden friend from its methods keep instantiation-time ADL. A later `using namespace` does not enter an earlier template's lookup.
- Operator calls inside templates now use the same dependence rules as named calls. The compiler classifies binary and unary operator expressions on the definition AST, so a non-dependent operator uses definition-time lookup and a dependent operator keeps instantiation-time ADL. A non-dependent class-type operator with no definition-visible overload is a compile error, and codegen finds unary operators in the operand's associated namespace.
- The parser accepts friend function templates inside a class template, and semantic analysis registers them for ADL per specialization. Each specialization keeps its own signature; repeated instantiation or lookup does not duplicate candidates, and ordinary unqualified lookup still cannot see them.
- Added tests for friend visibility across definition points and for dependent-call classification in nested calls, nested expressions, class-template methods, member templates, and calls sharing a source line.

### Standard Library (C++17)
- Added C++17 SFINAE library traits: `std::enable_if`/`enable_if_t`, `std::void_t`, `std::declval`, and `std::is_integral`/`is_integral_v` in `<type_traits>` and `<utility>`. They reuse the compiler's substitution-failure machinery instead of special-casing names. Supporting changes fold static const data members and variable template specializations into non-type arguments, expand alias templates when checking partial-specialization patterns, parse unnamed template parameters and variable templates, treat `volatile` as a type-start qualifier, and keep both constant maps in instantiation snapshots so rejected candidates leave no state behind. Fixed base-class constant aliasing so a derived trait's own value shadows the inherited one.

### Partial Specialization (C++17)
- Partial-specialization matching is now SFINAE-safe. An invalid type in a specialization's pattern, including a `decltype` operand or a dependent nested type, discards that specialization and lets the compiler try the next candidate instead of aborting; errors from instantiating another template stay hard. Added tests for ambiguous `decltype` and hard instantiation, and for empty, nested and mismatched packs, pack fallback, repeat after failure, pointer patterns, ordering, and SFINAE alternatives.

### Substitution Failure (C++17)
- Added a typed `SubstitutionFailure` distinct from ordinary semantic errors. The compiler now discards only the candidate whose function or member template signature forms an invalid type, then continues with the remaining candidates.
- Signature substitution no longer rewrites bodies or constructor initializers. The compiler substitutes member template bodies after selecting a candidate, so rejected candidates register no method or pending instantiation.
- Hard errors from instantiating another template while resolving a signature type, and errors in a selected specialization's body, still abort compilation. When every candidate is rejected, the call reports `no matching function for call to '...'` with every rejection reason. A rejected member template reports the conversion failure that ruled it out.
- The compiler reuses the signature formed during candidate selection when instantiating the winner, substituting only its body. Deduction of two trailing function parameter packs now assigns trailing call arguments to the last pack, matching clang and GCC.
- Dependent nested types in template signatures (`typename T::type`, `typename Traits<T>::type`) resolve through aliases registered when the base class is instantiated. A missing nested type discards the candidate; an error from instantiating the base template stays hard. Nested aliases resolve in parameter and return types, including aliases of aliases, default template arguments, and function-pointer parameters. An invalid nested type inside a class template's own alias now fails that instantiation instead of being stored as an opaque type.

### Expression SFINAE (C++17)
- Added `Type::Decltype { expr, parenthesized }`: the operand stays in the AST and is resolved after substitution. C++17 value-category rules are applied, including the declared-type rule for an unparenthesized id-expression or member access and `T&` for a parenthesized lvalue.
- Added trailing return types on function declarations (`auto f(T x) -> decltype(x.member())`). The trailing type participates in template signature substitution and candidate viability.
- A `decltype` operand is analyzed on a disposable clone of `Sema`, so a rejected candidate cannot change the expression, symbol tables, scopes, or instantiation state. Expression errors in the immediate context become substitution failures; errors from instantiating another template stay hard. Parameter types are completed before parameters are bound, so a `decltype` parameter type cannot recurse; an unparenthesized id-expression or member access uses the entity's declared type, keeping reference members and dropping receiver-added const. Global symbols are visible while field and parameter types are resolved, and `sizeof`/`alignof` both keep nested instantiation errors hard.

### Template Instantiation State
- Class template instantiation caches only completed specializations. A specialization still being built stays on the instantiation stack and satisfies recursive references, but is not treated as finished. A failed instantiation restores the layouts, aliases, globals, methods, synthesized declarations and counters it wrote, so a later request can retry it from a clean state.
- The instantiation stack and depth counter are now unwound on every exit path, including errors from specialization selection and nested-type resolution. Function templates drop their in-progress marker when body analysis fails, so a failed specialization is never reused as a successful one.
- Added regression tests for failed class and function instantiations, recursive and nested instantiation, repeated and distinct specializations, substitution failures followed by valid overloads, and member-template rejection without leaked registrations.

### Fold Expressions (C++17)
- Added unary and binary folds over all 32 fold operators, with standard empty-pack fallbacks (`&&` to `true`, `||` to `false`, `,` to a void expression) and evaluation in constexpr contexts, array bounds, and non-type template arguments.
- Fixed folds over overloaded operators, assignment folds, folds whose pattern contains `new` or `sizeof...`, and chained results that need a converting constructor.
- Fixed wrong results from overloaded binary-operator chains longer than 16 operators. The chain recompiles a prefix only when the left operand needs a user-defined conversion, which keeps long chains correct up to the expression-depth limit.

### Compiler Audit Remediation
- Typed `new` expressions as pointers to the allocated type so derived-to-base conversion applies after multiple inheritance.
- Deduplicated vtables by physical vptr offset and kept null covariant returns null.
- Rejected explicit template packs with too few call arguments and propagated invalid dependent array bounds instead of sizing zero.
- Ranked variadic overloads by fixed parameter count and kept unbound member template packs during class instantiation.
- Limited override checks to genuine virtual declarations and ran member destructors on explicit `return` in a destructor.
- Restored the repository census check by splitting long functions and large files, and removed unused integer sequence helpers.

### Alias and Specialization Identity
- Resolved alias-named base mem-initializers to the canonical direct base, so `using Alias = Base<int>; struct D : Alias { D() : Alias(42) {} };` forwards its constructor arguments correctly. Covers namespace-qualified, enclosing-class, nested specialization, plain-struct, non-polymorphic, multiple-alias, multiple-inheritance, three-level, and covariant cases.
- Canonicalized constructor argument collection and reachability so alias-named bases keep their constructor and destructor functions reachable; a genuinely unresolved mem-initializer is now a compile error instead of silently default-constructing the base.
- Fixed missing destructors for direct template-specialization bases: early body analysis no longer drops a destructor before registration, and lifecycle synthesis seeds resource types before template instantiation so synthesized base destructors are emitted.
- Resolved `new Alias(...)` and alias-typed variables/pointers to the canonical specialization type, fixing the pointer-type mismatch and destructor omission for `new`/`delete` through aliases.
- Canonicalized `final` method and destructor propagation for alias-named bases, and removed a dead alias-layout cloning pass that never ran.
- Added regression tests for every case plus negative tests for invalid mem-initializers and overriding a `final` method through an alias; refreshed golden hashes.

### Virtual Lifecycle and Integration
- Constructors and destructors now initialize/destroy only direct non-virtual bases, in declaration and reverse declaration order, so indirect bases are no longer processed repeatedly.
- Fixed phantom primary vtables: classes without their own virtuals reuse the actual polymorphic base subobject's vptr, including when non-polymorphic bases are declared first.
- Resolved covariant return types through template specializations and aliases, validated overrides against every base declaration, and rejected ambiguous or inaccessible covariant conversions.
- Fixed member-function pointers to template-specialization methods by forcing deferred instantiation and resolving a real function-table index instead of silently using index zero.
- Rejected derivation from classes whose virtual destructor is final, including implicit derived destructors.

### Virtual Function Infrastructure
- Canonicalized virtual function parameters before building VirtualKey: top-level cv is dropped, arrays and functions decay to pointers, and pointee/reference qualification is preserved.
- Validated covariant pointer and reference returns with accessibility, ambiguity and cv rules, and added a per-vtable-entry return adjustment so overrides yield the correct base subobject through ordinary and member-pointer calls.
- Instantiated template-specialization bases before derived layout, so vptrs, slots and override checks are correct for non-template derived classes.
- Unified destructor resolution so explicit destructor calls and vtable entries use the same function index.
- Applied base-subobject adjustments when binding references to secondary bases, and fixed ref-qualified dispatch on rvalue receivers.

### Virtual Function Identity
- Keyed virtual dispatch on the semantic function signature (name, parameters, cv/ref qualifiers) instead of the name alone, giving overloaded virtuals distinct vtable slots and correct final overriders across multiple inheritance.
- Validated `override`/`final` against matching signatures, including return-type covariance and C++17 exception-specification compatibility; rejected static, non-virtual, and mismatched overrides, and enforced `final` on destructors.
- Parsed class-level `final` and rejected deriving from final classes; rejected virtual member-function templates.
- Added regression tests for overload identity, qualifier mismatches, covariant returns, noexcept compatibility, and class-level final.

### Pointers to Members (C++17)
- Added data-member (`T C::*`) and member-function (`R (C::*)()`) pointer types across AST, parser, semantic analysis, and WebAssembly codegen, with `.*`/`->*` application, overloaded member-function resolution by target type, base-to-derived conversions, virtual dispatch, multiple-inheritance subobject adjustments, and `noexcept` conversion rules.
- Added fold expressions over the `.*` and `->*` operators, and documented the value representation and dispatch ABI in ADR 0015.

### Pointer-to-Member Audit Remediation
- Preserved and validated `override` and `final`: they now imply virtual, same-signature methods override without a keyword, and overriding a `final` method or using `override` with no base virtual is rejected.
- Fixed invalid Wasm when synthesized constructors copied a base subobject containing 8-byte fields (`long long`/`double`); base subobjects are now copied bytewise.
- Applied unambiguous, accessible-base checks and const-correctness at `.*`/`->*`, reusing the same inheritance path and access logic as base-to-derived member-pointer conversions.
- Derived call result types from the callee type in codegen, so member-function pointers returning floating-point, pointers, references, `void`, and structures lower correctly.
- Allowed converting `noexcept` member-function pointers to the potentially-throwing type, while rejecting the reverse.
- Added regression tests for each finding and updated the pointer-to-member ABI documentation.

### Variadic Templates, Parameter Packs & std::tuple (C++11)
- Added parameter packs across AST and parser: template type packs (`typename... Ts`, `class... Ts`), non-type parameter packs (`int... Ns`), template-template parameter packs (`template <template <typename...> class... Containers>`), and function parameter packs (`Ts... args`, `Ts...`).
- Added pack expansion expressions (`expr...`) and type pack expansions across function call arguments, initializer lists, base class specifiers (`struct S : Bases...`), constructor member initializers (`: Bases(args)...`), template argument lists (`Tuple<Tail...>`, `f<Ts...>(args...)`), lambda captures (`[args...]`, `[&args...]`), and alignment specifiers (`alignas(Ts...)`).
- Implemented multiple inheritance and multiple base-class pack expansions (`struct S : Bases...`), laying out multiple base subobjects sequentially with offset adjustments, multi-base method and constructor/destructor dispatch, and rejection of duplicate direct base classes.
- Added compile-time `sizeof...(pack)` query operator and `alignof(T)` in AST, parser, and semantic constant expression evaluation.
- Implemented parameter pack substitution engine in `src/sema/pack_expansion.rs` supporting homogeneous and heterogeneous type and non-type packs, length validation across multiple packs expanded within the same pattern, unexpanded parameter pack detection, reserved `__pack_{name}_{i}` synthetic identifiers, and empty pack omission.
- Implemented template argument deduction for variadic function templates and parameter packs, including empty trailing packs, deterministic deduction fallback, forwarding reference packs (`Args&&...`) with reference collapsing rules, and structural pack deduction in template instances (`f(Tuple<As...>, Tuple<Bs...>)`).
- Implemented standard deduction-based function template partial ordering per ISO C++11 [temp.func.order] in `src/sema/template_partial_order.rs` using synthetic type deduction, correctly ranking more specialized overloads and maintaining ambiguity for ties.
- Added nested member class templates containing parameter packs (`Outer<Ts...>::Inner<Us...>`), substituting outer arguments during enclosing template instantiation while preserving inner parameters.
- Preserved dependent array bounds (`int a[sizeof...(Ts)];`) until template instantiation.
- Extended dead-code elimination traversal in `src/codegen/analysis.rs` over `InitializerList`, `TypedInitList`, and `PackExpansion`, ensuring functions called in standard swallow idioms (`(f(args), 0)...`) remain reachable.
- Removed non-standard `func(args)...;` statement extension, strictly enforcing standard C++11 pack expansion contexts.
- Updated embedded `<memory>` with variadic `std::make_unique<T>(Args&&... args)`.
- Added `<tuple>` header providing `std::tuple`, recursive base-indexed element storage, `std::make_tuple`, and `std::get<I>(tuple)` supporting up to 17 elements (indices 0..16).
- Fixed explicit function-template argument pack deduction when explicit arguments are provided (`foo<int, int>(1, 2)`), preventing duplicate argument deduction and signature-arity mismatch in WebAssembly call emission.
- Implemented alias template parameter pack expansion (`template <typename... Ts> using Alias = Target<Ts...>;`), grouping trailing arguments into `TemplateArg::TypePack`.
- Enforced compile-time diagnostics rejecting unexpanded parameter packs in class template data members and method parameter types.
- Supported parameter packs in function pointer declarations (`void (*fn)(Args...)`) across parsing, substitution, and indirect invocation.
- Supported C++11 dynamic exception specifications `throw(Ts...)` and `throw()` on function templates.
- Preserved lvalue reference value categories across nested call expressions returning references (`std::forward<T>`), preventing spurious WebAssembly scalar loads in forwarding call chains.
- Supported `sizeof...(Ts)` expressions in non-type template arguments and dependent NTTP substitution (`IntConst<sizeof...(Ts)>`).
- Implemented multiple-inheritance subobject offsets and pointer adjustments across layout, conversions, member access, method calls, and static casts, with null-pointer preservation.
- Added function pointer template argument deduction for `R (*fn)(Args...)`, deducing return types and parameter packs directly without trailing call arguments.
- Supported nested dependent type declarations (`Outer<Ts...>::Inner<Us...> var;`) in local scopes.
- Supported local type alias declarations (`using Res = Target<Ts...>;`) and block-scoped alias resolution.
- Supported `alignas` pack expansions with `alignof(Ts)...` expressions and non-type parameter packs (`alignas(Ns...)`).
- Added `std::get<I>(tuple&&)` rvalue overload for extracting move-only types from rvalue tuples.
- Added explicit error diagnostics in WebAssembly code generation rejecting unsupported or unlowered AST expressions.
- Added positive, variant, and negative diagnostic test suites across `tests/templates_operators/`, `tests/mini_stl/`, and `tests/negative/`.

### Template Specialization, Non-Type Parameters & Alias Templates
- Added class template explicit (full) specialization (`template <> struct S<int> { ... };`) replacing primary template definitions on exact argument matches.
- Added class template partial specialization (`template <typename T> struct S<T*> { ... };`) with pattern matching and specificity scoring.
- Added non-type template parameters (NTTP) for integral and boolean types (`template <typename T, int N>`), supporting default NTTP expressions and rejecting non-type arguments passed to type parameters.
- Implemented compile-time constant expression evaluation in semantic analysis for arithmetic, bitwise, comparison, logical, ternary, and `sizeof` expressions, with overflow, division-by-zero, and shift-range validation.
- Added alias templates (`template <typename T> using Name = Target;`) with dependent alias substitution.
- Added `<array>` header providing `std::array<T, N>` with bounds checking, iterators, and initializer-list construction.
- Added `<type_traits>` header providing `integral_constant`, `true_type`, `false_type`, `is_same`, `is_same_v`, `remove_reference`, and `remove_reference_t`.
- Hardened template angle-bracket parsing with nested depth tracking, `>>` token splitting, and backtracking rollback.

### Move Semantics, Value Categories & Smart Pointers
- Added rvalue references (`T&&`) across AST, parser, and semantic analysis with value category classification (lvalue, prvalue, xvalue).
- Implemented implicit synthesis of move constructors and move assignment operators, and enforced copy suppression for move-declared and move-only types across initialization, assignment, call, and return contexts.
- Implemented move overload resolution ranking preferring rvalue references for rvalues and temporaries.
- Implemented forwarding references (`template <typename T> void f(T&&)`) with reference collapsing rules (`&` + `&&` -> `&`, `&&` + `&&` -> `&&`).
- Added `<memory>` header providing `std::unique_ptr<T>` with single-owner move semantics, `reset`, `release`, and `std::make_unique`.
- Enhanced `<utility>` with `std::move`, `std::forward`, and move-enabled `std::pair`.
- Enhanced `<vector>` with move construction, move assignment, `push_back(T&&)`, and move-based reallocation.

## [1.1.0] - 2026-11-09

### Member Structured Bindings, Range-For Loops & Template Array Decay
- Preserved analyzed method bodies and deduced return types in semantic analysis so structured bindings (`auto [a, b] = p;`, `auto&`, `const auto&`), range-based `for` loops, and `auto` return types reach codegen in non-template member functions.
- Applied array-to-pointer decay (`int[N]` to `int*`) during template argument deduction for non-reference parameters per C++17 [temp.deduct.call] p2. This resolves deduction conflicts in `std::sort(arr, arr + n)` and user iterator templates.
- Added recursive array type matching in template deduction for explicitly array-typed parameters.
- Fixed member initialization for array fields initialized with braced initializer lists, writing scalar and aggregate struct elements directly to member offsets.
- Added regression tests in `tests/cxx17_regressions.rs`.

### C++17 Core Language Stabilization
- Added comma sequence operator `(a, b)` evaluating left-to-right with right-operand result, parsed at precedence 1 to separate expressions from call arguments, multi-declarator definitions, and parameter defaults.
- Added C++17 selection statement initializers (`if (init; condition)` and `switch (init; condition)`) with scoping confined to the selection statement.
- Added non-static data member initializers (`= expr` and `{ ... }`) in struct, class, and template declarations.
- Updated constructor code generation to initialize fields from their default member initializers when omitted from constructor member initializer lists.
- Updated aggregate initialization to initialize omitted trailing fields from their default member initializers, conforming to C++17 [dcl.init.aggr].

### npm Async Execution & Timeouts
- Added `AsyncCppBoltRunner` and `compileAndRunAsync` with `timeoutMs`, `AbortSignal`, `stdin`, and streaming `onStdout` support (`node:worker_threads` on Node.js, bundled Web Worker in browsers).
- Exported `TimeoutError`, returning any stdout emitted before termination.
- Fixed WebAssembly instantiation failure propagation so malformed bytecode rejects with `CompileError`/`LinkError` instead of resolving with exit code 134.
- Handled runtime traps (`RuntimeError` and call stack `RangeError`) without masking arbitrary JavaScript exceptions as exit status 134.
- Made `host` optional in `AsyncCppBoltRunner` constructor and added `settled` guards for worker lifecycle cleanup.
- Added worker `exit` event listener in Node runner to reject cleanly when a worker thread terminates unexpectedly.

### Environment-Specific npm Entry Points
- Split the npm package into a shared API module and three entry points: `dist/index.js` (Node.js CommonJS), `dist/index.browser.mjs` (browser ES modules and bundlers), and `dist/cppbolt.browser.js` (classic `<script>` global).
- Added conditional exports so `browser` resolves to the browser ESM build, `node` resolves to the Node.js build, and `default` falls back to the browser ESM build.
- Added explicit `"./browser"` export subpath in `package.json` for bundlers and direct imports.
- Updated `createCompiler` in the Node entry point to accept optional file paths, URLs, or WebAssembly buffers, aligning with documentation and the browser entry point.
- The browser ESM entry resolves `wasm/cppbolt.wasm` relative to its own module URL and accepts an explicit wasm URL.

### Single-Source JavaScript Host
- Made `npm/src/host.ts` (`createWasmHost`) the only implementation of the execution host used by the npm package, playground, corpus runner, and tests.
- Added `CppBoltRunner` and `compileAndRun` to the npm package, exported from the new `npm/src/index.ts` entry point.
- Removed the tracked `web/host.js`; `tools/build_web_host.sh` generates it from the TypeScript source, and the playground build, corpus runner, and `tests/browser_flow.rs` invoke it as needed.

### Extended Casts (`reinterpret_cast`)
- Added C++17 `reinterpret_cast<T>(expr)` support across scalar integers, pointers, and reference types under the ILP32 data model.
- Disallowed scalar floating-point conversions and user-defined conversion operators in `reinterpret_cast`.
- Implemented bit-preserving WebAssembly lowering for pointer-to-pointer, integer-to-pointer, and pointer-to-integer casts.
- Added end-to-end tests in `tests/c_basics/types/reinterpret_cast.cpp`.
- Completed all features in Roadmap Section 2 (Language Feature Expansion).

### Namespace Aliases
- Supported C++17 namespace alias declarations (`namespace NewName = OriginalName;`) at file, namespace, and block scopes.
- Implemented transitive resolution of namespace aliases across types, functions, variables, and `using namespace` directives.
- Diagnosed invalid alias redefinitions in semantic analysis.
- Added test coverage in `tests/classes_oop/statics_namespaces/namespace_aliases.cpp` and updated `tests/negative/namespace_alias.cpp`.


### Type Qualification, Pointer Conversions & String Literal Semantics
- Rejected arbitrary integer-to-pointer conversions, allowing only null pointer constants (`0`, `0u`, and `nullptr`).
- Rejected zero-valued variables and `false` as pointer initializers.
- Added `Type::Nullptr` for `nullptr`, `std::nullptr_t`, and `decltype(nullptr)` with Itanium mangling `Dn`, supporting implicit conversions to pointers and `bool`.
- Enforced C++17 multi-level qualification conversions (`int**` converts to `const int* const*`, while `int**` to `const int**` and dropping `const` qualifiers fail).
- Allowed implicit object-pointer-to-`void*` conversion; required explicit `static_cast` for `void*` to typed pointers.
- Typed string literals as `const char[N]` with null terminators (`sizeof("hello") == 6`), decaying to `const char*` and rejecting assignment to `char*`.
- Scored qualification conversions and null pointer constant conversions in function overload and constructor resolution.
- Added test coverage in `tests/c_basics/types/pointer_conversions.cpp` and 7 negative tests in `tests/negative/`.

### C++17 Numeric Literals, Undef Directive & Raw Token Pasting
- Added binary integer literals (`0b`, `0B`) and single-quote digit separators (`1'000'000`, `0b1010'0101`).
- Rejected non-octal digits (`8`, `9`) in octal literals.
- Added preprocessor `#undef` directive.
- Implemented raw macro token pasting (`##`) without expanding argument tokens first.
- Added test coverage in `tests/c_basics/types/numeric_literals.cpp`, `tests/c_basics/preprocessor/macro_token_paste_raw.cpp`, and 3 negative tests in `tests/negative/`.

### Project Rename
- Renamed the remaining legacy project-name references to `cppbolt`. The binaries are now `cppbolt-cli`, `cppbolt-testrunner`, and `cppbolt-census`, and the test runner's default compiler path is `target/release/cppbolt-cli`.

### List Initialization & Aggregates
- Added support for braced list initialization across declarations, member initializers, return values, function arguments, assignments, and heap allocations.
- Direct-list initialization allows explicit constructors; copy-list initialization rejects them.
- Empty braces (`{}`) zero-initialize scalars, clear aggregate fields, or invoke default constructors.
- Added C++17 aggregate classification, supporting public non-virtual base subobjects and flattened brace elision.
- Enforced narrowing conversion rejection per C++17 [dcl.init.list] p7, with compile-time constant range checks for integer literals.
- Added dynamic allocation for single objects (`new T{...}`) and arrays (`new T[n]{...}`), value-initializing omitted trailing elements in a WebAssembly loop.
- Lowered typed temporaries (`Type{args...}.member()`) to local stack storage.
- Added diagnostics for narrowing errors, explicit constructor mismatches, excess initializer counts, and `std::initializer_list` (blocked on template support).
- Added 6 positive tests in `tests/classes_oop/structs/` and 10 negative diagnostic tests in `tests/negative/`.

### C++17 Compatibility Fixes & Test Corpus Reorganization
- **Feature**: Implemented C++17 language constructs across parser, semantic analysis, and WebAssembly codegen:
  - **Array Parameter Decay**: Adjusted function parameters declared with array syntax (`int arr[]`, `int arr[N]`) to pointer types (`int* arr`).
  - **Scoped Enums**: Supported `enum class` with distinct type identities and qualified lookup (`Scope::Val`), prohibiting implicit conversion to integer in overload resolution.
  - **Delegating Constructors**: Lowered target constructor invocations (`C() : C(args) {}`) and enforced mutual exclusivity against member/base initializers.
  - **Constant Expression Switch Cases**: Extended switch case labels to accept arbitrary constant expressions and scoped enum variants evaluated at compile time.
  - **User-Defined Conversion Operators**: Supported `operator T() const` across variable initialization, assignment, explicit casts, and binary expressions, respecting `explicit` qualification.
  - **Ref-Qualified Member Functions**: Added `&` and `&&` ref-qualifiers on member methods with receiver value-category dispatch and distinct symbol mangling (`_R`, `_O`).
  - **Structured Bindings**: Added decomposition for aggregate structs (`auto [a, b] = p;`, `auto&`, `const auto&`) desugared into hidden bindings and member references.
  - **Global Initializer Reachability**: Updated dead-code elimination root collection to traverse function calls in global variable initializers.
  - **Exception Diagnostics**: Emitted explicit target diagnostic on encountering `throw` (`C++ exception handling ('throw') is not supported on this WebAssembly target`).
- **Tests & Tooling**: Reorganized test directories into functional subdirectories (`control_flow`, `enums`, `functions`, `io_printf`, `misc`, `pointers_arrays`, `preprocessor`, `types`, `constructors_destructors`, `inheritance`, `memory_raii`, `methods`, `operators`, `statics_namespaces`, `structs`) and upgraded `cppbolt-testrunner` with recursive discovery and nested output path mirroring.

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
- **Census Scanner**: Added `cppbolt-census` tool to automatically check repository code metrics (lines per function, parameter counts, file sizes).
- **Module Tree**: Added integration test ensuring every `.rs` file in `src/` is reachable from the crate module tree.

### Test Harness, Variant Gates & Golden Baselines
- **Harness**: Developed `cppbolt-testrunner` executing tests grouped by feature directories with expected output and error checks.
- **Golden Baseline**: Implemented `tests/golden.txt` cryptographic SHA-256 baseline pinning emitted WebAssembly modules and stdout across positive tests.
- **Anti-Overfitting Variants**: Created `_variant` suites with structural divergence gates preventing trivial identifier-only copy-paste tests.
- **Native Oracle**: Implemented native oracle gate compiling corpus tests through `g++` and `clang++` to guarantee standards conformance.
- **Cleanroom Independent Corpus**: Established `tests/independent` cleanroom test harness pinned by cryptographic manifest.

### Initial Compiler Pipeline & Single Cargo Package
- **Architecture**: Flattened the initial multi-crate workspace into a single, cohesive Cargo package with a core library and CLI/test runner binaries.
- **Pipeline**: Established direct WebAssembly emission pipeline: `preprocessor -> lexer -> parser -> sema -> wasm codegen`.
- **Target ABI**: Defined ILP32 target model (32-bit pointers, 64 KiB guard page, 8 MiB downward stack, upward heap).
