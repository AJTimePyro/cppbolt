# Parser & AST Internals

The parser transforms the flat token stream into a structured Abstract Syntax Tree (`ast::Program`), implemented in `src/parser/` and defined in `src/ast/`.

---

## 1. Role in the Pipeline

1. **Unsupported Feature Fast-Reject**: Before executing the full grammar parser, `Parser::validate_supported_features` (`src/parser/unsupported.rs`) scans the token stream to detect explicitly unsupported constructs (such as `const &` method ref-qualifiers) and returns clean diagnostics.
2. **Grammar Parsing (Recursive Descent)**: Parses top-level declarations (functions, structs, classes, globals, enums, type aliases, templates, specializations) and statements (`if`, `while`, `for`, `switch`, `return`, compound blocks). Recognizes out-of-class constructor and destructor definitions across nested namespace and class qualifiers without requiring return type specifiers.
3. **Expression Parsing (Pratt Parser)**: Parses expressions using precedence climbing and Pratt parsing (`src/parser/expr.rs`), handling operator hierarchies, pointer-to-member operators (`.*`, `->*`), ternary conditionals, array subscripts, member access, casts, lambdas, object constructions, and template argument angle-bracket disambiguation with `>>` token splitting and parenthesized operator depth tracking.
4. **Per-Construct Recursion Budgeting**: To prevent thread stack exhaustion when compiling deep programs in WebAssembly engines, the parser tracks recursion limits via `DepthCounters` across expressions, parentheses, call arguments, statements, and blocks.

---

## 2. Key Data Structures

- `Parser`: Holds the token slice, current token index, and active `DepthCounters`.
- `ast::Program`: Root AST node containing declarations and file-scope type aliases:
  ```rust
  pub struct Program {
      pub declarations: Vec<Decl>,
      pub type_aliases: HashMap<String, Type>,
  }
  ```
- `ast::Decl`: Top-level declaration node:
  - `Function(FunctionDecl)`: Free functions, member methods, constructors, destructors (with `Param::is_pack` for parameter packs).
  - `GlobalVar { name, ty, init, line, col }`: Global and namespace-level variables.
  - `StructDecl { name, is_class, base_classes, fields, methods, ... }`: Class and struct definitions (with `BaseClassSpecifier` supporting pack expansion `Bases...`).
  - `EnumDecl { name, is_scoped, variants }`: Enumerations.
  - `NamespaceAlias { alias, target, line, col }`: C++17 namespace aliases.
  - `UsingNamespace(String)`, `UsingDecl(String)`: Using directives and declarations.
  - `Template(TemplateDecl)`: Function templates, class templates, and alias templates (`template <...> using Alias = Target;`), with `TemplateParam::is_pack` indicating type and non-type parameter packs.
- `ast::Stmt`: Statement nodes:
  - `Expr(Expr)`, `Block(Vec<Stmt>)`, `If { cond, then_branch, else_branch }`, `While`, `DoWhile`, `For`, `RangeFor { decl, range, body }`, `Switch`, `Return(Option<Expr>)`, `Break`, `Continue`, `VarDecl`.
- `ast::Expr`: Expression tree nodes:
  - `IntLiteral`, `FloatLiteral`, `StringLiteral`, `BoolLiteral`, `Identifier`, `BinaryOp` (including pointer-to-member operators `DotStar` and `ArrowStar`), `UnaryOp`, `Call`, `MemberCall`, `MemberAccess`, `ArraySubscript`, `New`, `Delete`, `Ternary`, `Assign`, `Cast`, `Lambda`.
  - `SizeofPack { name, pos }`: Compile-time `sizeof...(pack)` query returning the number of elements in the parameter pack.
  - `PackExpansion { pattern, pos }`: Pack expansion pattern `pattern...` in function calls, initializers, base classes, and constructor initializers.
  - `FoldExpr { op, left, right, pos }`: C++17 fold expression over parameter packs covering all 32 fold operators across unary left, unary right, binary left, and binary right shapes (including `FoldOp::DotStar` and `FoldOp::ArrowStar`).
  - `InitializerList { elements, pos }`: Bare braced expression lists `{ a, b, c }`.
  - `TypedInitList { ty, elements, pos }`: Explicitly-typed temporary and direct-list expressions `Type{ a, b, c }`.
  - Every `Expr` variant carries an `ExprPos` (`line`, `col`) recording the position of its leading token.
- `ast::types::Type`: Representation of types:
  - Primitives: `Int`, `LongLong`, `Short`, `Char`, `Float`, `Double`, `Bool`, `Void`, `Nullptr` (`decltype(nullptr)`), and unsigned variants.
  - Compounds: `Pointer(Box<Type>)`, `MemberPointer { class_name, member_type }` (pointers to data members `T C::*` and member functions `R (C::*)(Args...)`), `Reference(Box<Type>)` (lvalue ref `T&`), `RvalueReference(Box<Type>)` (rvalue ref `T&&`), `Array(Box<Type>, usize)`, `DependentArray(Box<Type>, String)`.
  - Variadics: `PackExpansion(Box<Type>)` representing an expanded type pattern `Ts...`.
  - Aggregates and Generics: `Struct(String)`, `Enum(String)`, `TypeParam(String)`, `NonType(i128)`, `TemplateInstance { name, args }`, `Function { return_type, params }`.
  - Dependent expressions: `Decltype { expr, parenthesized }` keeps the operand expression and whether it was parenthesized, so C++17 value-category rules can be applied after template substitution. `DependentNested(String)` records a qualified type such as `T::type` whose base was substituted.

---

## 3. Error Handling & Invariants

- **Syntax Errors**: Emitted via `CompileError::new(..., line, col).with_kind(ErrorKind::Syntax)` when an expected token (e.g. semicolon, identifier, closing delimiter) is missing.
- **Nesting Depth Guards**: Bounded by `src/depth.rs`:
  - Expressions: max 4,000
  - Parentheses: max 1,600
  - Compound blocks: max 1,600
  - Statements: max 1,600
  If any depth counter exceeds its budget, parsing terminates immediately with `ErrorKind::Internal` ("Nesting depth exceeds the supported limit").
- **Unsupported Feature Rejection**: Rejects constructs before partial AST creation to prevent invalid states in subsequent compiler passes.

---

## 4. Real Input & AST Example

Taken from `tests/c_basics/types/arithmetic.cpp`:

### Input C++ Source
```cpp
int main() {
    int a = 10 + 20;
    return a;
}
```

### Resulting AST Representation
```rust
Program {
    decls: vec![
        Decl::Function(FunctionDecl {
            name: "main".into(),
            return_type: Type::Int,
            params: vec![],
            body: Some(Stmt {
                kind: StmtKind::Compound(vec![
                    Stmt {
                        kind: StmtKind::VarDecl {
                            name: "a".into(),
                            ty: Type::Int,
                            init: Some(Expr::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::IntLiteral(10, Type::Int)),
                                right: Box::new(Expr::IntLiteral(20, Type::Int)),
                                pos: ExprPos { line: 2, col: 13 },
                            }),
                        },
                        line: 2,
                        col: 5,
                    },
                    Stmt {
                        kind: StmtKind::Return(Some(Expr::Identifier(
                            "a".into(),
                            ExprPos { line: 3, col: 12 },
                        ))),
                        line: 3,
                        col: 5,
                    },
                ]),
                line: 1,
                col: 12,
            }),
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_static: false,
            ctor_initializers: vec![],
            line: 1,
            col: 1,
        }),
    ],
}
```
