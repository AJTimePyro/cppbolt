# ADR 0010: Template Instantiation Model and Single-Phase Lookup

## Status
Accepted

## Context
The compiler must support C++ class and function templates with type parameters to compile templated standard library headers like `<vector>` and `<utility>`. Standard C++ specifies two-phase lookup: non-dependent names bind at definition time, and dependent names bind at instantiation time. Implementing two-phase lookup in `cppbolt` would require duplicating semantic analysis tables and AST structures for dependent versus non-dependent symbols before concrete types exist.

## Options Considered
- **Option 1: Conforming two-phase lookup**
  - *Pros*: Strictly conforms to the ISO C++ standard; rejects early any non-dependent symbol errors in uninstantiated templates.
  - *Cons*: High complexity. Requires duplicate symbol resolution and typing passes across uninstantiated AST structures.
- **Option 2: Single-phase lookup at instantiation time**
  - *Pros*: Fits the existing pipeline. Sema parses templates as recipes and executes full type checking and rewriting only upon concrete instantiation.
  - *Cons*: Permissive on uninstantiated templates. A template containing an unresolved non-dependent symbol compiles without error if never instantiated.

## Decision
1. **Single-phase lookup**: We adopt single-phase lookup at instantiation time as documented in section 5.14 of the template plan. The parser preserves template declarations as recipes (`TemplateDecl`). Semantic analysis checks and resolves names only when a concrete specialization is requested.
2. **Type identity and canonical naming**: Every distinct specialization receives a canonical name (for example, `Holder<int>`). The instantiation cache ensures that multiple references to the same specialization share a single `StructDecl` or `FunctionDecl`.
3. **Lazy member instantiation**: Class template instantiation lays out fields and registers member function signatures. Member function bodies are instantiated only when called or referenced, preserving C++ compatibility for types that satisfy only the members used.
4. **Recursion depth budget**: Instantiation depth is tracked dynamically and limited to 500 levels, preventing stack exhaustion in browser Web Workers.

## Consequences
### Positive
- Straightforward implementation that leverages existing AST rewriting and struct layout machinery.
- Satisfies all requirements for the embedded standard library containers and algorithms.
- Guarantees zero code bloat: unused member functions are never instantiated or emitted into WebAssembly output.

### Negative / Trade-offs
- Ill-formed uninstantiated templates that reference non-existent non-dependent symbols will pass compilation unless instantiated. This deviation is documented for users.
