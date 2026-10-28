# Parser & AST Internals

The parser transforms the flat token stream into a structured Abstract Syntax Tree (`ast::Program`), implemented in `src/parser/` and defined in `src/ast/`.

---

## 1. Role in the Pipeline

1. **Unsupported Feature Fast-Reject**: Before executing the full grammar parser, `Parser::validate_supported_features` (`src/parser/unsupported.rs`) scans the token stream to detect explicitly unsupported C++ constructs (e.g. `operator()`, `operator->`, `operator*()`, `postfix ++/--`, namespace aliases, `const` ref method overloads) and returns clean diagnostics.
2. **Grammar Parsing (Recursive Descent)**: Parses top-level declarations (functions, structs, classes, globals, enums, type aliases) and statements (`if`, `while`, `for`, `switch`, `return`, compound blocks).
3. **Expression Parsing (Pratt Parser)**: Parses expressions using precedence climbing / Pratt parsing (`src/parser/expr.rs`), handling complex operator hierarchies, ternary conditionals, array subscripts, member access, casts, and object constructions.
4. **Per-Construct Recursion Budgeting**: To prevent thread stack exhaustion when compiling deep programs in WebAssembly engines, the parser tracks recursion limits via `DepthCounters` across expressions, parentheses, call arguments, statements, and blocks.

---

## 2. Key Data Structures

- `Parser`: Holds the token slice, current token index, and active `DepthCounters`.
- `ast::Program`: Root AST node containing a list of top-level declarations:
  ```rust
  pub struct Program {
      pub decls: Vec<Decl>,
  }
  ```
- `ast::Decl`: Top-level declaration node:
  - `Function(FunctionDecl)`: Free functions, member methods, constructors, destructors.
  - `GlobalVar { name, ty, init, line, col }`: Global and namespace-level variables.
  - `StructDecl { name, is_class, base_classes, fields, methods, constructors, destructor, ... }`: Class and struct definitions.
  - `EnumDecl { name, variants }`: Enumerations.
  - `TypeAlias { name, target_type }`: `typedef` and `using Alias = Type`.
  - `NamespaceDecl { name, decls }`: Namespace scopes.
- `ast::Stmt`: Statement nodes:
  - `Expr(Expr)`, `Block(Vec<Stmt>)`, `If { cond, then_branch, else_branch }`, `While`, `DoWhile`, `For`, `RangeFor { decl, range, body }`, `Switch`, `Return(Option<Expr>)`, `Break`, `Continue`, `VarDecl`.
- `ast::Expr`: Expression tree nodes:
  - `IntLiteral`, `FloatLiteral`, `StringLiteral`, `BoolLiteral`, `Identifier`, `BinaryOp`, `UnaryOp`, `Call`, `MemberCall`, `MemberAccess`, `ArraySubscript`, `New`, `Delete`, `Ternary`, `Assign`, `Cast`.
  - Every `Expr` variant carries an `ExprPos` (`line`, `col`) recording the position of its leading token.
- `ast::types::Type`: Representation of types:
  - `Int`, `LongLong`, `Short`, `Char`, `Float`, `Double`, `Bool`, `Void`.
  - `Pointer(Box<Type>, bool /* is_const */)`, `Reference(Box<Type>)`, `Array(Box<Type>, usize)`.
  - `Struct(String)`, `Enum(String)`, `FunctionPointer { ret, params }`.

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
