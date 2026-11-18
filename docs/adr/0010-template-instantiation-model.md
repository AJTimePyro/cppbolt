# ADR 0010: Template Instantiation Model and Two-Phase Lookup

## Status
Accepted

## Context
Compiling templated standard library headers like `<vector>`, `<utility>`, and `<algorithm>` requires class and function templates with type parameters (`stdlib/vector`, `stdlib/utility`, `stdlib/algorithm`, `src/ast/template.rs`, `src/sema/template_registry.rs`). Standard C++ uses two-phase lookup: unqualified names bind in the definition context, and dependent calls add ADL candidates at instantiation. Real headers also need out-of-class member method definitions, member function templates inside class templates and plain structs, dependent member type aliases, and multi-arity overload sets.

## Options Considered
- Option 1: Two-phase lookup conforming to ISO C++.
  - Keeps definition-time bindings stable when later declarations would otherwise win overload resolution.
  - Lets dependent calls find functions in the associated namespaces of their instantiated arguments.
  - Adds a definition-time snapshot of visible overloads and an ADL candidate pass to the existing instantiation path.
- Option 2: Single-phase lookup at instantiation time.
  - Fits the existing pipeline with no extra state, but a declaration later in the file can rebind a name in an earlier template.

## Decision
1. Two-phase name lookup (`src/ast/template.rs`, `src/sema/analyze.rs`, `src/sema/overload.rs`).
   `Program` stores template declarations as `TemplateDecl` nodes. When we register declarations, we record the overloads visible at each function template's definition, keyed by name, line and column. While we re-analyze a substituted template body or signature, ordinary unqualified lookup only sees that recorded set. A declaration that appears later in the file cannot rebind the name. Dependent calls also collect ADL candidates from the instantiated argument types and merge them with the ordinary set. We resolve types when a concrete specialization is requested (`src/sema/template_registry.rs`).

2. Canonical specialization identity (`src/sema/template_registry.rs`).
   We name each specialization using its template name and canonical argument types (such as `Holder<int>` or `std::vector<int>`). We cache instantiated struct and function declarations in `Sema` so identical specializations reuse the same generated definitions. Only completed specializations enter the cache: a specialization still being built lives on the instantiation stack, and a failed one restores the state it wrote so a later request can retry it (`src/sema/template_state.rs`).

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
   We cap recursion depth at 500 levels, rejecting recursive template expansions with an explicit diagnostic before exhausting the WebAssembly call stack. The stack and depth counter are unwound on every exit path, including errors, so a rejected expansion cannot leave later instantiations artificially deep.

9. Nested class instantiation in class templates (`src/sema/template_nested.rs`).
   When instantiating a class template containing member struct declarations (such as `AVLTree<T>::Node`), we capture nested AST declarations in `TemplateClass` and clone them with outer type arguments substituted. Each nested struct receives its qualified canonical name (such as `std::detail::AVLTree<int, int>::Node`), computes its record layout, and registers synthesized lifecycle hooks and member methods.

10. Dependent type specifiers and typename disambiguation (`src/parser/types.rs`, `src/sema/template_nested.rs`).
    We parse the `typename` prefix before qualified type specifiers in template and function declarations (`typename T::iterator`). Semantic analysis resolves dependent names through qualified alias tables and registered nested types, rejecting unqualified `typename` specifiers and non-type dependent names with distinct diagnostics.

11. Eager constructor dependency analysis (`src/sema/constructor.rs`, `src/sema/stmt.rs`).
    When analyzing an instantiated constructor, semantic analysis inspects member initializer expressions and base class subobjects after parameter bindings enter scope. Dependent default constructors for member subobjects and array elements instantiate immediately, populating member layouts and destructor cleanups before WebAssembly code emission.

## Consequences
### Positive
- Reuses existing AST substitution and struct layout routines without separate uninstantiated typing passes (`src/sema/template_subst.rs`, `tests/templates_operators/template_class_basic.cpp`).
- Unused member methods are neither analyzed nor emitted to WebAssembly, keeping output binaries compact (`tests/templates_operators/template_lazy_member.cpp`).
- Standard containers like `std::vector` and `std::pair` compile and execute with full RAII semantics, dynamic growth, and range-based for loops (`tests/mini_stl/vector_basic.cpp`, `tests/mini_stl/iterators_range_for.cpp`).
- Associative containers `std::map`, `std::set`, and their backing `detail::AVLTree` compile cleanly with full nested node structures, balance rotations, and iterator traversals (`tests/mini_stl/map_basic.cpp`, `tests/mini_stl/set_basic.cpp`, `tests/mini_stl/differential_fuzz.cpp`).
- Sorting algorithms with default or custom comparators resolve correctly across multiple arities (`tests/mini_stl/algorithm_sort.cpp`, `tests/mini_stl/sort_comparator.cpp`).

### Negative / Trade-offs
- Ill-formed non-dependent expressions in uninstantiated template bodies do not trigger compile errors unless an instantiation requests them.
- The definition-time snapshot tracks overloads but not friend declarations introduced after a template definition.
- Hidden friends declared in a class template are not associated with a concrete specialization, so ADL does not find them.
