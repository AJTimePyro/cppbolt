# ADR 0010: Template Instantiation Model and Single-Phase Lookup

## Status
Accepted

## Context
Compiling templated standard library headers like `<vector>`, `<utility>`, and `<algorithm>` requires supporting class and function templates with type parameters (`stdlib/vector`, `stdlib/utility`, `stdlib/algorithm`, `src/ast/template.rs`, `src/sema/template_registry.rs`). Standard C++ specifies two-phase lookup: non-dependent names bind at definition time, and dependent names bind at instantiation time. Implementing two-phase lookup in `cppbolt` would require duplicate symbol tables and separate typing passes for uninstantiated ASTs before concrete types exist. In addition, real-world headers require out-of-class member method definitions, member function templates within both class templates and plain structs, dependent member type aliases, and multi-arity overload sets.

## Options Considered
- Option 1: Two-phase lookup conforming to ISO C++.
  - Rejects invalid non-dependent symbols in uninstantiated templates early.
  - Requires separate symbol resolution and typing passes over uninstantiated AST structures, adding substantial parser and semantic analysis complexity.
- Option 2: Single-phase lookup at instantiation time.
  - Fits the existing compiler pipeline; the parser stores uninstantiated template declarations (`TemplateDecl`), and semantic analysis checks and lowers AST nodes when concrete types are supplied.
  - Malformed non-dependent code in uninstantiated templates remains undetected unless instantiated.

## Decision
1. Single-phase name lookup (`src/ast/template.rs`, `src/sema/template_registry.rs`).
   We store template declarations directly in `Program` as `TemplateDecl` nodes without early name binding (`src/ast/template.rs`). We validate and resolve types only when a concrete specialization is requested (`src/sema/template_registry.rs`).

2. Canonical specialization identity (`src/sema/template_registry.rs`).
   We name each specialization using its template name and canonical argument types (such as `Holder<int>` or `std::vector<int>`). We cache instantiated struct and function declarations in `Sema` so identical specializations reuse the same generated definitions.

3. Lazy member function instantiation (`src/sema/template_registry.rs`, `src/sema/expr_ops.rs`).
   We lay out fields and register member function signatures when instantiating a class template. We defer member function body analysis and emission until called or referenced, permitting template arguments that do not support unused member methods. Non-virtual destructors instantiate eagerly alongside virtual methods so synthesized cleanup logic can drop values on scope exit.

4. Out-of-class template method resolution (`src/sema/template_registry.rs`).
   When the parser encounters `template <typename T> void Class<T>::method()`, we look up the primary class template and attach the method AST directly to its declaration before any instantiation occurs.

5. Member function templates (`src/sema/constructor.rs`, `src/sema/template_registry.rs`).
   Member methods declaring their own template parameters (such as `pair(U1&&, U2&&)` or `DefaultLess::operator()<T>`) bypass eager class method registration and register into `member_templates`. We deduce their arguments and synthesize concrete methods during call resolution. For non-template structs, instantiated methods synthesize into dedicated struct declarations so codegen compiles them with concrete types.

6. Instantiated dependent type aliases (`src/sema/template_registry.rs`).
   When instantiating a class template that declares member type aliases (such as `using iterator = T*`), we substitute the template arguments into the target type and register the qualified alias (such as `std::vector<int>::iterator` resolving to `int*`).

7. Multi-arity function template candidate sets (`src/sema/template_registry.rs`, `src/sema/template_func.rs`).
   Function template registration maintains a candidate vector for each identifier. Overload resolution filters candidates by parameter count and default arguments before argument deduction, allowing overloads like `sort(first, last)` and `sort(first, last, comp)` to coexist.

8. Recursion depth budget (`src/sema/template_registry.rs`).
   We cap recursion depth at 500 levels, rejecting recursive template expansions with an explicit diagnostic before exhausting the WebAssembly call stack.

## Consequences
### Positive
- Reuses existing AST substitution and struct layout routines without separate uninstantiated typing passes (`src/sema/template_subst.rs`, `tests/templates_operators/template_class_basic.cpp`).
- Unused member methods are neither analyzed nor emitted to WebAssembly, keeping output binaries compact (`tests/templates_operators/template_lazy_member.cpp`).
- Standard containers like `std::vector` and `std::pair` compile and execute with full RAII semantics, dynamic growth, and range-based for loops (`tests/mini_stl/vector_basic.cpp`, `tests/mini_stl/iterators_range_for.cpp`).
- Sorting algorithms with default or custom comparators resolve correctly across multiple arities (`tests/mini_stl/algorithm_sort.cpp`, `tests/mini_stl/sort_comparator.cpp`).

### Negative / Trade-offs
- Ill-formed non-dependent expressions in uninstantiated template bodies do not trigger compile errors unless an instantiation requests them.
