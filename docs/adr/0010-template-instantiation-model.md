# ADR 0010: Template Instantiation Model and Single-Phase Lookup

## Status
Accepted

## Context
The compiler must support C++ class and function templates with type parameters to compile templated standard library headers like `<vector>` and `<utility>`. Standard C++ specifies two-phase lookup: non-dependent names bind at definition time, and dependent names bind at instantiation time. Implementing two-phase lookup in `cppbolt` would require duplicating semantic analysis tables and AST structures for dependent versus non-dependent symbols before concrete types exist.

## Options Considered
- Option 1: Conforming two-phase lookup. This approach strictly conforms to the ISO C++ standard by rejecting non-dependent symbol errors in uninstantiated templates early. It adds significant complexity because the compiler must run separate symbol resolution and typing passes over uninstantiated AST structures.
- Option 2: Single-phase lookup at instantiation time. This approach fits the existing compiler pipeline. The parser records templates as blueprints, and semantic analysis checks and lowers AST nodes only when concrete types are supplied during instantiation. The drawback is that invalid non-dependent symbols in uninstantiated templates remain undetected unless instantiated.

## Decision
1. Single-phase lookup. We adopt single-phase lookup at instantiation time. The parser stores template declarations as recipes (`TemplateDecl`). Semantic analysis validates and resolves names only when a concrete specialization is requested.
2. Canonical specialization identity. Each unique combination of template and type arguments receives a canonical mangled name, such as `Holder<int>`. The instantiation cache ensures that multiple references to the same specialization share a single `StructDecl` or `FunctionDecl`.
3. Lazy member instantiation. Class template instantiation lays out fields and registers member function signatures immediately. Member function bodies are instantiated only when called or referenced, allowing use of types that satisfy only the required member operations.
4. Recursion depth budget. The compiler tracks instantiation depth dynamically and limits recursion to 500 levels, preventing stack exhaustion in WebAssembly execution environments.

## Consequences
### Positive
- Straightforward implementation that reuses existing AST rewriting and struct layout logic.
- Satisfies requirements for embedded standard library containers and algorithms.
- Avoids code bloat by ensuring unused member functions are never instantiated or emitted into WebAssembly output.

### Negative and Trade-offs
- Ill-formed uninstantiated templates containing non-existent non-dependent symbols compile without diagnostic unless instantiated.
