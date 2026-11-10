# Semantic Analysis Internals

The semantic analysis stage validates the Abstract Syntax Tree, computes record memory layouts, resolves symbols, checks type conversions, and scores function overloads, implemented in `src/sema/`.

---

## 1. Role in the Pipeline

1. **Symbol Registration & Scoping**: Registers struct definitions, function signatures, global variables, and type aliases across lexical and namespace scopes (`src/sema/analyze.rs`).
2. **Record Layout & Alignment**: Computes struct field offsets, overall sizes, and alignment padding according to the 32-bit ILP32 target model (`src/sema/layout.rs`).
3. **Vtable Layout & Hierarchy Analysis**: Analyzes single and linear inheritance hierarchies, computes base subobject offsets, identifies pure virtual methods (`= 0`), and assigns vtable function pointer slots in base-prefix order.
4. **Implicit Method Synthesis & Copy Suppression**: Synthesizes default constructors, copy constructors, copy assignment operators, move constructors, move assignment operators, and destructors (`src/sema/synthesize.rs`, `src/sema/synthesize_ops.rs`). Copy operations recursively copy base subobjects before member fields. Declaring user move operations suppresses implicit copy operations. Copying move-only types (`std::unique_ptr`) is rejected across variable initialization, assignment, call arguments, and return statements.
5. **Overload Resolution Ranking & Value Categories**: Implements standard overload ranking rules to pick the best function or constructor candidate and detect ambiguous calls (`src/sema/overload.rs`). Categorizes expressions as lvalues, prvalues, or xvalues. Rvalues bind to `const T&` or `T&&`, with exact value-category bonuses favoring rvalue references over const lvalue references.
6. **Forwarding References & Reference Collapsing**: Detects forwarding references in function templates (`template <typename T> void f(T&&)`) and applies standard reference collapsing rules (`&` + `&&` -> `&`, `&&` + `&&` -> `&&`) during template argument deduction (`src/sema/template_deduce.rs`).
7. **Type Inference, Conversions & Qualification**: Annotates expression nodes with resolved concrete types, validates assignments, arithmetic conversions, multi-level pointer `const` qualification rules, null pointer constant conversions (`0`, `0u`, `nullptr`), and C++17 copy elision rules.
8. **Compile-Time Constant Expression Evaluation**: Evaluates integer and boolean constant expressions at compile time (`src/sema/constexpr_eval.rs`) for non-type template parameters, array bounds, switch cases, and static assertions. Detects division by zero and shift-width overflows with source diagnostic locations.
9. **Template Specialization & Alias Expansion**: Resolves primary class templates, explicit (full) specializations (`template <> struct S<int>`), and partial specializations (`template <typename T> struct S<T*>`) using pattern matching and specificity scoring (`src/sema/template_spec.rs`, `src/sema/template_match.rs`). Resolves alias templates (`template <typename T> using Alias = Target;`) during type lookup (`src/sema/template_alias.rs`).
10. **AST Desugaring**: Lowers high-level syntax nodes (such as range-based `for` loops in `src/sema/stmt_range_for.rs` and structured bindings in `src/sema/stmt_structured_binding.rs`) into standard scoped primitives. Synthesizes pointer and iterator bounds while preserving RAII and loop control flow in both free functions and member methods.
11. **Closure Synthesis & Lambda Desugaring**: Synthesizes unique anonymous closure structs (`__lambda_N`) for C++17 lambda expressions (`src/sema/expr_lambda.rs`), generating fields for captures, constructor initialization, `operator()` call signatures, return type deduction, and static `__invoke` trampoline methods for implicit conversion to C function pointers.
12. **Recursive `operator->` Resolution**: Resolves chained member-arrow accesses recursively until a raw pointer is reached, enforcing cycle detection and receiver `const` propagation (`src/sema/expr_member.rs`).
13. **List Initialization**: Resolves braced initializers (`src/sema/init.rs`) across direct-list, copy-list, and value-initialization contexts. Enforces C++17 narrowing rules ([dcl.init.list] p7) and `explicit` constructor rejection in copy contexts, and handles aggregate base subobjects and brace elision.

---

## 2. Key Data Structures

- `Sema`: Central semantic state containing:
  - `structs: HashMap<String, StructLayout>`: Layouts, field offsets, vtable slots, and pure virtual tracking for all defined records.
  - `functions: HashMap<String, Vec<FunctionSig>>`: All registered functions and method overloads.
  - `globals: HashMap<String, Type>`: Global and namespace variable types.
  - `type_aliases: HashMap<String, Type>`: `typedef` and `using` type maps.
  - `class_templates: HashMap<String, (TemplateClass, Vec<TemplateParam>)>`: Primary class templates.
  - `class_specializations: HashMap<String, TemplateClass>`: Explicit full specializations (`template <> struct S<int>`).
  - `partial_specializations: HashMap<String, Vec<PartialSpec>>`: Candidate partial specializations ranked by specificity.
  - `alias_templates: HashMap<String, (Vec<TemplateParam>, Type)>`: Alias template definitions.
  - `structs_with_move: HashSet<String>`: Classes defining or synthesizing move constructors/assignment operators.
- `StructLayout`:
  ```rust
  pub struct StructLayout {
      pub is_class: bool,
      pub has_user_declared_constructors: bool,
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

1. **Exact Match**: Argument type equals parameter type (or identical enum/primitive). Exact matches reward matching value categories, such as an rvalue argument binding directly to an rvalue reference `T&&`.
2. **Reference Binding**: Binding an lvalue to `T&` or `const T&`, or binding an rvalue to `const T&` (with an rvalue-to-const-ref conversion rank penalty).
3. **Promotion**: Numeric promotion (`char`/`short` -> `int`, `float` -> `double`).
4. **Standard Conversion**: Derived-to-base pointer conversion, array decay, pointer qualification conversion (`T*` -> `const T*`), null pointer constant conversion (`0` / `nullptr` -> pointer / `bool`), pointer-to-void, or numeric widening/narrowing.
5. **User-Defined Conversion**: Implicit single-argument converting constructor.

Between viable candidates with identical base ranks, overload scoring awards a bonus to exact value category alignments: an rvalue argument strictly prefers `T&&` over `const T&`.

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
