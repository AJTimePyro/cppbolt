# Semantic Analysis Internals

The semantic analysis stage validates the Abstract Syntax Tree, computes record memory layouts, resolves symbols, checks type conversions, and scores function overloads, implemented in `src/sema/`.

---

## 1. Role in the Pipeline

1. **Symbol Registration & Scoping**: Registers struct definitions, function signatures, global variables, and type aliases across lexical and namespace scopes (`src/sema/analyze.rs`).
2. **Record Layout & Alignment**: Computes struct field offsets, overall sizes, and alignment padding according to the 32-bit ILP32 target model (`src/sema/layout.rs`).
3. **Vtable Layout & Hierarchy Analysis**: Analyzes single and linear inheritance hierarchies, computes base subobject offsets, identifies pure virtual methods (`= 0`), and assigns vtable function pointer slots in base-prefix order.
4. **Implicit Method Synthesis**: Synthesizes default constructors, copy constructors, and destructors for structs that do not define them explicitly (`src/sema/constructor.rs`).
5. **Overload Resolution Ranking**: Implements standard overload ranking rules to pick the best function or constructor candidate and detect ambiguous calls (`src/sema/overload.rs`).
6. **Type Inference & Checking**: Annotates expression nodes with resolved concrete types, validates assignments, usual arithmetic conversions, and C++17 copy elision rules.
7. **AST Desugaring**: Lowers high-level syntax nodes (such as range-based `for` loops in `src/sema/stmt_range_for.rs`) into standard scoped primitives, synthesizing pointer and iterator bounds while preserving RAII and loop control flow.
8. **Closure Synthesis & Lambda Desugaring**: Synthesizes unique anonymous closure structs (`__lambda_N`) for C++17 lambda expressions (`src/sema/expr_lambda.rs`), generating fields for captures, constructor initialization, `operator()` call signatures, return type deduction, and static `__invoke` trampoline methods for implicit conversion to C function pointers.
9. **Recursive `operator->` Resolution**: Resolves chained member-arrow accesses recursively until a raw pointer is reached, enforcing cycle detection and receiver `const` propagation (`src/sema/expr_member.rs`).

---

## 2. Key Data Structures

- `Sema`: Central semantic state containing:
  - `structs: HashMap<String, StructLayout>`: Layouts, field offsets, vtable slots, and pure virtual tracking for all defined records.
  - `functions: HashMap<String, Vec<FunctionSig>>`: All registered functions and method overloads.
  - `globals: HashMap<String, Type>`: Global and namespace variable types.
  - `type_aliases: HashMap<String, Type>`: `typedef` and `using` type maps.
- `StructLayout`:
  ```rust
  pub struct StructLayout {
      pub is_class: bool,
      pub base_classes: Vec<String>,
      pub base_offsets: Vec<(String, usize)>,
      pub fields: Vec<StructField>,
      pub size: usize,
      pub align: usize,
      pub has_vtable: bool,
      pub vtable_slots: Vec<String>,
      pub private_members: HashSet<String>,
      pub is_abstract: bool,
      pub pure_virtuals: HashSet<String>,
  }
  ```
- `FunctionSig`: Signature metadata (parameters, return type, `is_const`, `is_virtual`, default arguments).
- `OverloadCandidate`: Candidate function paired with argument conversion ranks.

---

## 3. Overload Resolution Ranking

When a call expression matches multiple candidates, `src/sema/overload.rs` scores each parameter conversion against the provided arguments using five rank tiers:

1. **Exact Match**: Argument type equals parameter type (or identical enum/primitive).
2. **Reference Binding**: Binding an lvalue to `T&` or `const T&`.
3. **Promotion**: Numeric promotion (`char`/`short` -> `int`, `float` -> `double`).
4. **Standard Conversion**: Derived-to-base pointer conversion, array decay, pointer-to-void, or numeric widening/narrowing.
5. **User-Defined Conversion**: Implicit single-argument converting constructor.

### Ambiguity Detection
If two candidates have identical rank scores across all arguments, overload resolution halts with:
```text
file.cpp:line:col: error: call of overloaded 'func(...)' is ambiguous
```
Verified by `tests/c_basics/functions/overload_resolution.cpp` and `tests/negative/ambiguous_overload_call.cpp`.

---

## 4. Record Layout & Vtable Example

Given a polymorphic class hierarchy:

```cpp
struct Base {
    int id;
    virtual void run();
};

struct Derived : public Base {
    double value;
    void run() override;
    virtual void extra();
};
```

The semantic analyzer produces the following `StructLayout` for `Derived`:
- `size`: 16 bytes (ILP32: 4 bytes `vptr` at offset 0, 4 bytes `id` at offset 4, 8 bytes `value` at offset 8).
- `align`: 8 bytes (due to `double` alignment requirement).
- `has_vtable`: `true`
- `vtable_slots`: `["Derived::run", "Derived::extra"]` (derived `run` replaces `Base::run` at slot 0; `extra` is appended at slot 1).
- `base_offsets`: `[("Base", 0)]`
