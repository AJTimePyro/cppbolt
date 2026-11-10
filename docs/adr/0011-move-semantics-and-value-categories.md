# ADR 0011: Move Semantics, Value Categories, and Smart Pointers

## Status
Accepted

## Context
Standard C++11 introduced rvalue references (`T&&`), move constructors, move assignment operators, and move-enabled standard library primitives (`std::move`, `std::forward`, `std::unique_ptr`). In `cppbolt`, previous milestones handled object lifetime via copy constructors and C++17 copy elision for prvalues. Supporting efficient transfer of resources like heap allocations in `std::vector` and implementing move-only resource handles like `std::unique_ptr` required introducing rvalue references into the AST, formalizing value categories across expression analysis, establishing reference collapsing rules for template argument deduction, and preventing illegal copies of move-only types.

## Options Considered
- Option 1: Treat `T&&` as a syntactic synonym for `const T&` or pointer borrowing.
  - Minimal compiler changes, but violates C++ semantics.
  - Cannot distinguish move constructors from copy constructors during overload resolution.
  - Allows illegal copies of move-only types like `std::unique_ptr`, causing double-free bugs in WebAssembly linear memory.
- Option 2: Full value category tracking and reference distinction in the type system.
  - Explicitly adds `Type::RvalueReference(Box<Type>)` alongside `Type::Reference(Box<Type>)`.
  - Classifies expressions into lvalues and rvalues (prvalues and xvalues) during semantic analysis.
  - Implements standard overload ranking, reference collapsing, and copy suppression rules.

## Decision
1. Dedicated rvalue reference representation (`src/ast/types.rs`, `src/lexer/mod.rs`, `src/parser/decl.rs`).
   We represent rvalue references explicitly as `Type::RvalueReference(Box<Type>)`. The lexer recognizes `&&` as `TokenKind::AmpAmp`, which declarator and template argument parsing treat as an rvalue reference operator outside expression contexts.

2. Value category classification and reference binding (`src/sema/assign.rs`, `src/sema/overload.rs`, `src/ast/expr.rs`, `src/codegen/lvalue.rs`).
   We classify expressions into lvalues, prvalues, and xvalues. Named variables and reference parameters evaluate as lvalues regardless of declared type (including `T&&`). Expressions become xvalues through explicit rvalue reference casts or function calls returning `T&&`. Function calls returning `T&` evaluate as lvalues. An lvalue reference `T&` binds only to lvalues. Const references `const T&` bind to rvalues with a conversion penalty. Rvalue references `T&&` bind exclusively to rvalues.

3. Overload candidate ranking (`src/sema/overload.rs`, `src/sema/constructor.rs`, `src/sema/template_func.rs`).
   During function and constructor overload selection, candidate parameters matching the exact value category of the argument receive precedence over conversions. For an rvalue argument, `T&&` beats `const T&`. For an lvalue argument, `const T&` is viable while `T&&` is rejected. Function template calls validate parameter compatibility of non-deduced and concrete parameters before selecting an instantiated candidate.

4. Reference collapsing in template deduction (`src/sema/template_deduce.rs`).
   Forwarding references of the form `template <typename T> void f(T&& arg)` deduce `T = U&` when called with an lvalue of type `U` (including calls returning `U&`), and `T = U` when called with an rvalue. During template type substitution, reference combinations collapse per standard C++ rules:
   - `&` with `&` yields `&`
   - `&` with `&&` yields `&`
   - `&&` with `&` yields `&`
   - `&&` with `&&` yields `&&`

5. Implicit move synthesis and copy suppression (`src/sema/synthesize_ops.rs`, `src/sema/stmt_decl.rs`, `src/sema/assign.rs`, `src/sema/stmt.rs`, `src/codegen/stmt_return.rs`).
   When a class defines a destructor, fields with non-trivial move constructors, or base classes with move constructors, we synthesize defaulted move operations unless blocked by user-declared copy operations or custom destructors. Conversely, when a class declares a move constructor or move assignment operator, the compiler suppresses implicit copy operations across variable initialization, copy assignment, pass-by-value calls, and returns. Returning local automatic variables by value applies C++11 implicit move semantics to invoke move constructors before falling back to copies.

6. Standard library move support (`stdlib/utility`, `stdlib/vector`, `stdlib/memory`).
   - `<utility>` provides `std::move` (casting an expression to an xvalue rvalue reference), `std::forward` (preserving original value categories across forwarding references), move-based `std::swap`, and copy and move operations for `std::pair`.
   - `<vector>` implements move construction, move assignment, `push_back(T&&)`, and move-based reallocation during buffer growth.
   - `<memory>` implements `std::unique_ptr<T>` (providing exclusive resource ownership, move construction, move assignment, pointer dereferencing, `reset` with self-reset guard, and `release`) and `std::make_unique`.

## Consequences
### Positive
- Move-only types like `std::unique_ptr<T>` are fully supported and protected against illicit copy operations at compile time.
- Standard container reallocation in `std::vector` transfers objects via move constructors instead of expensive deep copies.
- Overload resolution accurately disambiguates between lvalue and rvalue arguments.
- Template algorithms and wrapper types can forward arguments without losing value category information.

### Negative / Trade-offs
- Overload scoring and reference binding logic in semantic analysis require maintaining distinct conversion metrics for reference categories.
- Classes declaring user move operations must explicitly declare copy operations if they wish to remain copyable.
