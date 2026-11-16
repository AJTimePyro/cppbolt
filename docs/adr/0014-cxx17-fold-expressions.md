# ADR 0014: C++17 Fold Expressions

## Status
Accepted

## Context
Parameter pack reduction in C++11 required recursive template instantiations or index sequence tricks. For example, computing the sum of a variadic pack or chaining function calls required a base case template and an inductive specialization step. This introduced compile-time instantiation overhead and boilerplate.

ISO C++17 [expr.prim.fold] introduced fold expressions over parameter packs. The standard defines four fold shapes across 32 operators:
1. Unary left fold: `(... + args)`
2. Unary right fold: `(args + ...)`
3. Binary left fold: `(init + ... + args)`
4. Binary right fold: `(args + ... + init)`

The language specifies exact associativity for each shape, special fallback values for empty parameter packs (`&&` yields `true`, `||` yields `false`, `,` yields `void()`), and requires fold evaluation inside constant expressions, non-type template parameters (NTTP), and dependent array bounds.

## Options Considered

### Option 1: Lower fold expressions to runtime loops in WebAssembly codegen
Lowering fold expressions in `src/codegen/` would generate loops or synthetic helper functions in WebAssembly linear memory and execution stacks.

We rejected this option. Runtime loops introduce call overhead, complicate lvalue reference tracking, and cannot evaluate during semantic analysis. Constant expressions, dependent array dimensions, and non-type template arguments require compile-time evaluation. Adding runtime constructs conflicts with the direct AST-to-Wasm lowering design (ADR 0002).

### Option 2: Expand fold expressions into concrete binary expression trees during semantic analysis
Expand fold expressions during template substitution in `src/sema/fold_expansion.rs`. Parameter packs expand into concrete trees of `Expr::Binary`, `Expr::Assign`, or `Expr::CompoundAssign` before WebAssembly code generation.

We chose this option. It extends the single-phase AST expansion architecture established in ADR 0010 and ADR 0013. The existing constant expression evaluator (`src/sema/constexpr_eval.rs`) and code generator (`src/codegen/`) process the resulting binary expression trees with zero runtime changes.

## Decision

### 1. AST representation (`src/ast/fold_op.rs`, `src/ast/expr.rs`)
We define `FoldOp` covering all 32 C++17 fold operators:
- Binary operators: `+`, `-`, `*`, `/`, `%`, `^`, `&`, `|`, `<<`, `>>`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `,` (`FoldOp::Binary`)
- Simple assignment: `=` (`FoldOp::Assign`)
- Compound assignments: `+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `&=`, `|=`, `<<=`, `>>=` (`FoldOp::Compound`)
- Pointer-to-member operators: `.*` (`FoldOp::DotStar`), `->*` (`FoldOp::ArrowStar`)

The expression AST represents fold expressions via `Expr::FoldExpr`:
```rust
FoldExpr {
    op: FoldOp,
    left: Option<Box<Expr>>,
    right: Option<Box<Expr>>,
    pos: ExprPos,
}
```
Left and right child expressions hold the pack pattern or optional init expression, with `None` indicating the ellipsis side.

### 2. Syntactic parsing and operator verification (`src/parser/expr_fold.rs`, `src/parser/prefix.rs`)
Parenthesized expression parsing detects fold expressions via `has_depth1_ellipsis`. This lookahead checks whether an ellipsis appears at top-level depth within parentheses next to a fold operator, distinguishing fold expressions from parenthesized pack expansions or comma lists without backtracking.

The parser handles all four fold shapes:
- Unary left fold `(... op pack)`: stores `left: None`, `right: Some(pack)`.
- Unary right fold `(pack op ...)`: stores `left: Some(pack)`, `right: None`.
- Binary left fold `(init op ... op pack)`: stores `left: Some(init)`, `right: Some(pack)`.
- Binary right fold `(pack op ... op init)`: stores `left: Some(pack)`, `right: Some(init)`.

For binary folds, the parser verifies that the operator before and after the ellipsis match. If `op1 != op2`, the compiler emits a syntax error identifying the mismatched operator tokens.

### 3. Classification and semantic expansion (`src/sema/fold_expansion.rs`, `src/sema/template_subst.rs`)
During template substitution, `expand_fold_expr` classifies the expression by inspecting which operand contains unexpanded parameter packs:
- If both operands contain packs, semantic analysis rejects the expression.
- If neither operand contains packs, semantic analysis rejects the expression.
- If an unexpanded fold expression reaches non-template expression analysis (`src/sema/expr.rs`), it emits a diagnostic stating that the fold contains no unexpanded parameter packs.

The engine verifies that all packs co-expanded within the fold have matching lengths (`validate_pack_lengths`). It then slices the pack expression for each index from `0..len` using `build_slice_mapping` and `rename_pack_identifiers`.

### 4. Associativity and tree construction
The expansion engine combines sliced expressions according to standard C++17 associativity:
- Unary left fold `(... op pack)`: expands as `(((E0 op E1) op E2) ... op En-1)`.
- Unary right fold `(pack op ...)`: expands as `(E0 op (E1 op ... (En-2 op En-1)))`.
- Binary left fold `(init op ... op pack)`: expands as `((((init op E0) op E1) ... ) op En-1)`.
- Binary right fold `(pack op ... op init)`: expands as `(E0 op (E1 op ... (En-1 op init)))`.
- When the parameter pack contains exactly one element, unary folds evaluate directly to that single element without applying the operator.

### 5. Empty pack rules
When a unary fold encounters an empty parameter pack (`len == 0`), ISO C++17 permits only three operators:
- `&&` yields `true` (`Expr::BoolLiteral(true)`).
- `||` yields `false` (`Expr::BoolLiteral(false)`).
- `,` yields an empty void expression (`Expr::IntLiteral(0, Type::Void)`), emitted as a no-op in WebAssembly codegen.
- Any other operator on an empty unary fold triggers a compile error stating that no fallback value exists for that operator. Binary folds over empty packs evaluate directly to the provided `init` expression.

### 6. Non-type template arguments and constexpr integration
Fold expressions in non-type template arguments (`src/sema/template_subst_nontype.rs`), dependent array bounds (`src/parser/types_array.rs`), and constexpr evaluation (`src/sema/constexpr_eval.rs`) expand during template substitution. The resulting binary expression trees evaluate through standard constant folding, supporting constructs such as `template <int... Ns> struct S { int arr[(... + Ns)]; };` and `IntConst<(Ns + ...)>`.

## Consequences

### Positive
- All four C++17 fold expression shapes parse and expand across all 32 operators.
- WebAssembly code emission requires no runtime pack or loop support; all folds lower to concrete scalar or compound expressions during template substitution.
- Standard empty pack fallbacks (`&&`, `||`, `,`) work without manual user workarounds.
- Constant expression evaluation and non-type template arguments support fold expressions directly.

### Negative / Trade-offs
- Pointer-to-member fold operators (`.*`, `->*`) parse into AST but trigger a diagnostic during semantic expansion because `cppbolt` does not yet support pointer-to-member types.
- Deep folds generate linear trees of binary AST nodes proportional to the pack size, which consumes memory during compilation of very large packs.
