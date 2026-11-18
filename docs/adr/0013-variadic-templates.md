# ADR 0013: C++11 Variadic Templates, Parameter Packs, and Partial Ordering

## Status
Accepted

## Context
Standard containers and utilities such as `std::tuple`, variadic `std::make_unique<T>(args...)`, and index sequences require C++11 variadic templates (`stdlib/tuple`, `stdlib/memory`, `stdlib/utility`, `src/ast/template.rs`, `src/sema/pack_expansion.rs`). Specifically, the language requires:
1. Template type and non-type parameter packs (`template <typename... Ts>`, `template <int... Ns>`) and function parameter packs (`Ts... args`).
2. Pack expansion in all standard expansion contexts: function call argument lists, braced initializer lists, base class specifiers (`struct S : Bases...`), constructor member initializers (`: Bases(args)...`), and template argument lists (`Tuple<Tail...>`).
3. Compile-time `sizeof...(pack)` queries returning the pack element count as a constant expression.
4. Parameter pack deduction from call arguments, including trailing empty packs and forwarding reference packs (`Args&&...`) with reference collapsing.
5. Function template partial ordering ([temp.func.order]) and overload resolution ranking ([over.match.best]), preferring more specialized signatures and prioritizing non-pack parameters over parameter packs.
6. Design compatibility for future C++17 fold expressions without requiring AST redesign.

## Options Considered
- Option 1: Lower parameter packs to runtime tuples or array descriptors in WebAssembly code generation.
  - Would burden the WebAssembly code generator (`src/codegen/`) with runtime dynamic dispatch, variable-argument calling conventions, and pack unpacking loops.
  - Conflicts with the direct AST-to-Wasm lowering architecture (ADR 0002).
- Option 2: Compile-time pack lowering to concrete AST nodes during template substitution (Single-Phase AST Expansion).
  - Parameter packs expand into concrete, individual AST expressions, types, and declarations during template instantiation in semantic analysis (`src/sema/template_subst.rs`, `src/sema/pack_expansion.rs`).
  - WebAssembly code generation sees only concrete types, structs, and calls, requiring zero runtime pack support.
  - Extends the established single-phase template model (ADR 0010, ADR 0012).

## Decision
1. Single-phase AST pack lowering (`src/sema/pack_expansion.rs`, `src/sema/template_subst.rs`).
   We expand parameter packs into concrete AST nodes during semantic substitution before code generation. WebAssembly code emission receives standard non-variadic AST nodes.

2. Variadic AST and token representation (`src/lexer/token.rs`, `src/ast/`).
   We tokenize `...` as `TokenKind::Ellipsis`. AST structures represent variadics explicitly: `TemplateParam::is_pack`, `Param::is_pack`, `Type::PackExpansion`, `Expr::SizeofPack`, `Expr::PackExpansion`, and an AST node placeholder `Expr::FoldExpr` reserving structure for C++17 fold expressions.

3. Parameter pack expansion engine (`src/sema/pack_expansion.rs`).
   We introduce `TemplateArg` representing concrete template arguments (`Type`, `NonType`, `TypePack`, `NonTypePack`). `expand_type_list` and `expand_expr_list` expand patterns across argument lists. When multiple parameter packs are co-expanded within the same pattern, the engine enforces identical pack lengths and diagnoses mismatches. Unexpanded parameter packs used outside pack expansions trigger compile errors. Base class pack expansion supports multiple inheritance (`struct S : Bases...`), laying out multiple base subobjects sequentially with offset adjustments, pointer conversions (`src/codegen/memory.rs`), and rejecting duplicate direct base classes.

4. Compile-time `sizeof...(pack)` and `alignof(T)` evaluation (`src/sema/constexpr_eval.rs`, `src/sema/expr.rs`, `src/sema/template_subst_nontype.rs`).
   The compile-time constant expression evaluator looks up parameter pack lengths directly from substitution mappings and context tables, returning the element count as a `size_t` constant expression. This applies across expression evaluation and non-type template argument (NTTP) substitution (`IntConst<sizeof...(Ts)>`). Alignment specifiers (`alignas(Ts...)`, `alignas(alignof(Ts)...)`, `alignas(Ns...)`) evaluate the strictest alignment across pack types and non-type expressions.

5. Variable-arity class templates, alias templates, template-template packs, and partial specialization (`src/sema/template_deduce.rs`, `src/sema/template_match.rs`, `src/sema/template_alias.rs`, `src/sema/stmt.rs`).
   Class templates match trailing argument lists into parameter packs. Pattern matching and partial specialization ranking support recursive base inheritance such as `Tuple<Head, Tail...> : Tuple<Tail...>`. Alias templates map parameter packs via `build_template_arg_map` into `TemplateArg::TypePack` both at global and local block scopes (`using Res = Target<Ts...>;`). Template-template parameters and packs (`template <template <typename...> class... Containers>`), nested dependent type declarations (`Outer<Ts...>::Inner<Us...>`), and function pointer parameter packs (`void (*fn)(Args...)`) are supported.

6. Variadic deduction and forwarding reference preservation (`src/sema/template_deduce.rs`, `src/sema/template_deduce_type.rs`, `src/codegen/lvalue.rs`).
   Function template argument deduction extracts parameter packs from trailing call arguments, nested template instances (`f(Tuple<As...>, Tuple<Bs...>)`), and function pointer types (`R (*)(Args...)`). When explicit template arguments are provided, deduction begins after the explicit pack prefix, preventing argument duplication. For forwarding reference packs (`Args&&...`), argument lvalues deduce as lvalue references (`T&`) and rvalues as object types (`T`), collapsing references (`&` + `&&` -> `&`, `&&` + `&&` -> `&&`). Reference-returning calls preserve their lvalue category through argument conversion and code generation.

7. Function template partial ordering ([temp.func.order]) (`src/sema/template_partial_order.rs`).
   We implement deduction from synthetic types per ISO C++11 [temp.func.order]. Candidate function templates synthesize unique types for their template parameters and deduce arguments against the other candidate. A non-pack parameter is strictly more specialized than a parameter pack. Overload resolution selects the uniquely most specialized function template per [over.match.best] and rejects ambiguous ties.

8. Embedded standard library headers (`stdlib/`).
   - `<memory>`: Implements variadic `std::make_unique<T>(Args&&... args)` forwarding arguments directly to the target constructor.
   - `<tuple>`: Implements `std::tuple` using recursive base class inheritance, `std::make_tuple`, and `std::get<I>(tuple)` supporting up to 17 elements (indices 0..16) without runtime overhead.

## Consequences
### Positive
- Full C++11 variadic template semantics supported across function templates, class templates, template-template parameters, multiple inheritance base expansions, nested member templates, lambda captures, deduction, and forwarding references.
- WebAssembly codegen requires no changes; pack expansions lower to concrete AST nodes prior to emission.
- Standard headers `<tuple>`, `<memory>`, and `<utility>` compile natively within the embedded standard library.
- Overload resolution disambiguates variadic and non-variadic overloads per ISO C++11 partial ordering rules.

### Negative / Trade-offs
- `std::tuple` element access via `std::get` and `std::tuple_element` supports up to 17 elements (indices 0..16).
