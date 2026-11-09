# ADR 0012: Template Specialization, Non-Type Parameters, and Alias Templates

## Status
Accepted

## Context
Standard containers like `std::array<T, N>` and type inspection traits like `std::is_same<T, U>` require template capabilities beyond basic type-parameterized classes (`stdlib/array`, `stdlib/type_traits`, `src/ast/template.rs`, `src/sema/template_registry.rs`). Specifically, the compiler needs:
1. Non-type template parameters (NTTP) holding compile-time integral and boolean constants.
2. Compile-time constant expression evaluation to fold arithmetic, bitwise, logical, ternary, and `sizeof` expressions inside template arguments and array dimensions.
3. Explicit (full) and partial class template specialization to select optimized or specialized implementations based on argument types.
4. Alias templates (`template <typename T> using Alias = Target;`) for type transformations such as `remove_reference_t<T>`.
5. Default template arguments for type parameters and non-type parameters.

## Options Considered
- Option 1: Full C++17 constexpr interpreter with arbitrary function execution.
  Supporting general constexpr function evaluation requires executing loops, local variables, and function calls at compile time. This would introduce a heavy AST interpreter into the semantic analysis pass.
- Option 2: Dedicated integral constant fold engine and single-phase specialization matcher.
  We implement a dedicated compile-time expression evaluator for integer and boolean constant expressions, alongside pattern matching and specificity scoring during class template instantiation. This keeps the compiler compact and matches the existing single-phase template instantiation pipeline.

## Decision
1. Dedicated compile-time constant expression evaluator (`src/sema/constexpr_eval.rs`).
   We evaluate compile-time constant expressions for integral and boolean types, covering integer literals, boolean literals, unary operations (`+`, `-`, `~`, `!`), binary arithmetic (`+`, `-`, `*`, `/`, `%`), bitwise operators (`&`, `|`, `^`, `<<`, `>>`), relational and equality comparisons (`<`, `<=`, `>`, `>=`, `==`, `!=`), logical operators (`&&`, `||`), conditional ternaries (`? :`), and `sizeof` queries over types and expressions.

2. Non-type template parameters in AST and parser (`src/ast/template.rs`, `src/ast/types.rs`, `src/parser/template.rs`, `src/parser/expr.rs`).
   We represent non-type template parameters in `TemplateParam` with an optional declared type. In expressions, template argument angle brackets `<...>` disambiguate closing delimiters `>` and `>>` from relational operators by tracking template argument nesting and parenthesized sub-expressions. Evaluated constant arguments lower to `Type::NonType(i128)`. Dependent array types retain their size expression string until template substitution.

3. Explicit and partial class template specialization (`src/sema/template_spec.rs`, `src/sema/template_match.rs`).
   We record explicit specializations (`template <> struct S<Args> { ... };`) and partial specializations (`template <Params> struct S<Pattern> { ... };`) in `Sema`. During instantiation, `resolve_instantiation_target` checks for an exact explicit specialization match first. If none matches, it scores candidate partial specializations using structural pattern matching. It ranks matches by specificity score, selecting the most specialized pattern and rejecting ambiguous ties with an explicit diagnostic.

4. Alias templates (`src/sema/template_alias.rs`, `src/parser/template.rs`).
   We parse `template <Params> using Name = Target;` into `TemplateKind::Alias`. When semantic analysis encounters an alias template instance, it binds the template arguments and substitutes them into the target type, unwrapping dependent aliases before instantiating concrete types.

5. Default template arguments (`src/sema/template_deduce.rs`, `src/parser/template.rs`).
   We parse trailing default type and non-type expressions in template parameter lists (`= Type`, `= Expr`). When instantiating templates with fewer arguments than parameters, `fill_default_template_args` populates omitted trailing arguments from default definitions.

6. Static member propagation across template inheritance and aliases (`src/sema/template_spec.rs`, `src/sema/template_registry.rs`).
   When instantiating templates that inherit static constants from base classes or typedef aliases (such as `is_same<T, T>` deriving from `true_type`), `alias_static_fields` propagates static variable symbols into the derived template instance while preserving the underlying storage variable name.

## Consequences
### Positive
- Compiles `std::array<T, N>` with full iterator support, index access, bounds queries, and member methods (`tests/mini_stl/array_basic.cpp`).
- Compiles foundational type traits in `<type_traits>` including `integral_constant`, `true_type`, `false_type`, `is_same`, `remove_reference`, and `remove_reference_t` (`tests/mini_stl/type_traits_basic.cpp`).
- Evaluates compile-time array bounds expressions cleanly across tests without runtime overhead (`tests/c_basics/pointers_arrays/constexpr_fold_basic.cpp`).
- Full and partial specializations resolve predictably according to standard C++ partial ordering rules (`tests/templates_operators/template_full_spec.cpp`, `tests/templates_operators/template_partial_spec.cpp`).

### Negative / Trade-offs
- Non-type template parameters are currently limited to integral and boolean types; floating-point and pointer NTTPs remain unsupported.
- General user-defined `constexpr` functions cannot yet be evaluated inside template arguments.
