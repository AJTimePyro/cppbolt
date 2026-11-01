# ADR 0010: Template Instantiation Model and Single-Phase Lookup

## Status
Accepted

## Context
Compiling templated standard library headers like `<vector>` and `<utility>` requires supporting class and function templates with type parameters (`stdlib/utility`, `src/ast/template.rs`, `src/sema/template_registry.rs`). Standard C++ specifies two-phase lookup: non-dependent names bind at definition time, and dependent names bind at instantiation time. Implementing two-phase lookup in `cppbolt` would require duplicate symbol tables and separate typing passes for uninstantiated ASTs before concrete types exist.

## Options Considered
- **Option 1: Two-phase lookup conforming to ISO C++**
  - *Pros*: Rejects invalid non-dependent symbols in uninstantiated templates early.
  - *Cons*: Requires separate symbol resolution and typing passes over uninstantiated AST structures, adding substantial parser and semantic analysis complexity.
- **Option 2: Single-phase lookup at instantiation time**
  - *Pros*: Fits the existing compiler pipeline; the parser stores uninstantiated template declarations (`TemplateDecl`), and semantic analysis checks and lowers AST nodes when concrete types are supplied.
  - *Cons*: Malformed non-dependent code in uninstantiated templates remains undetected unless instantiated.

## Decision
1. **Single-Phase Name Lookup (`src/ast/template.rs`, `src/sema/template_registry.rs`)**:
   - Store template declarations directly in `Program` as `TemplateDecl` nodes without early name binding (`src/ast/template.rs`).
   - Validate and resolve types only when a concrete specialization is requested (`src/sema/template_registry.rs`).
2. **Canonical Specialization Identity (`src/sema/template_registry.rs`)**:
   - Name each specialization using its template name and canonical argument types (e.g., `Holder<int>`).
   - Cache instantiated struct and function declarations in `Sema` so identical specializations reuse the same generated definitions.
3. **Lazy Member Function Instantiation (`src/sema/template_registry.rs`, `src/sema/expr_ops.rs`)**:
   - Lay out fields and register member function signatures when instantiating a class template.
   - Defer member function body analysis and emission until called or referenced, permitting template arguments that do not support unused member methods.
   - Instantiate non-virtual destructors eagerly alongside virtual methods so synthesized cleanup logic can drop values on scope exit.
4. **Recursion Depth Budget (`src/sema/template_registry.rs`)**:
   - Cap recursion depth at 500 levels (`src/sema/template_registry.rs#L114-L120`), rejecting recursive template expansions with an explicit diagnostic before exhausting the WebAssembly call stack.

## Consequences
### Positive
- Reuses existing AST substitution and struct layout routines without separate uninstantiated typing passes (`src/sema/template_subst.rs`, `tests/templates_operators/template_class_basic.cpp`).
- Unused member methods are neither analyzed nor emitted to WebAssembly, keeping output binaries compact (`tests/templates_operators/template_lazy_member.cpp`).
- Distinct instantiations produce isolated, concrete struct layouts and mangled function signatures (`tests/templates_operators/template_multi_instance.cpp`).

### Negative / Trade-offs
- Ill-formed non-dependent expressions in uninstantiated template bodies do not trigger compile errors unless an instantiation requests them.
