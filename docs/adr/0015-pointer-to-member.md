# ADR 0015: Pointer-to-Member Operators and ABI

## Status
Accepted

## Context
ISO C++17 defines pointer-to-member types in [dcl.mptr], member access operators in [expr.mptr.oper], and member pointer conversions in [conv.mem]. A pointer to member holds no concrete memory address. It represents either an offset to a data member within an instance of a class, or a method descriptor for invoking a member function on an instance.

The standard distinguishes two categories of pointer-to-member types:
1. Pointers to data members: `T C::*`
2. Pointers to member functions: `R (C::*)(Args...)`

Supporting pointers to members in a WebAssembly compiler raises four design problems:
1. Representing data-member offsets when offset 0 is a valid struct member, which precludes using 0 as the null pointer value.
2. Encoding member-function pointers so that a single type can invoke either a non-virtual member function (via function table index) or a virtual member function (via vtable slot lookup), while applying necessary `this` pointer adjustments for multiple inheritance.
3. Lowering base-to-derived conversions ([conv.mem]), where a pointer to a member of `Base` implicitly converts to a pointer to a member of `Derived`, inverting the usual object pointer conversion direction.
4. Integrating the pointer-to-member operators `.*` and `->*` with precedence parsing, lvalue evaluation, and C++17 fold expressions ([expr.prim.fold]).

## Options Considered

### Option 1: Synthetic helper thunks vs direct inlined dispatch
We considered generating a WebAssembly thunk for every pointer-to-member invocation or conversion. Taking the address of a member would return an index into the Wasm function table pointing to a compiler-generated wrapper function.

We rejected it. Generating helper thunks adds WebAssembly table entries, inflates code size, introduces an extra indirect call frame, and complicates constant evaluation. Inlined lowering resolves member access arithmetic and vtable indexing at the call site, so no wrapper overhead remains.

### Option 2: Fat struct representation in linear memory vs flat 64-bit scalar representation
We considered representing member-function pointers as two-word structs allocated in linear memory, passing pointers to these structs on the stack.

We rejected it. Storing member pointers in linear memory incurs memory traffic, requires shadow stack allocations for temporaries, and prevents passing member pointers in WebAssembly value registers.

We chose a flat 64-bit scalar representation (`i64`) based on the Itanium C++ ABI specification adapted for 32-bit ILP32 WebAssembly:
- Low 32 bits (`ptr`): encodes the function table index or vtable slot, using bit 0 as a discriminator.
- High 32 bits (`adj`): encodes the signed `this` adjustment for multiple inheritance.

This packs the dispatch descriptor into one WebAssembly `i64` register.

## Decision

### 1. AST Representation (`src/ast/types.rs`, `src/ast/expr.rs`, `src/ast/fold_op.rs`)
We represent pointer-to-member types as an explicit variant in `Type`:
```rust
Type::MemberPointer {
    class_name: String,
    member_type: Box<Type>,
}
```
If `member_type` is `Type::Function { .. }`, the type is a pointer to member function. Otherwise, it is a pointer to data member.

The AST distinguishes pointer-to-member operators:
- `BinaryOp::DotStar` for `object .* pmd`
- `BinaryOp::ArrowStar` for `pointer ->* pmd`
- `FoldOp::DotStar` and `FoldOp::ArrowStar` for C++17 fold expressions

### 2. Data-Member Pointer Representation and Null Encoding
Data-member pointers represent a byte offset into the struct layout and lower to WebAssembly `i32`:
- Valid offsets range from `0` to `2^31 - 1`.
- Null pointer to data member is represented by `-1` (`0xFFFFFFFF`), because offset `0` is the offset of the first member.
- Conversions from `nullptr` or integer constant `0` emit `i32.const -1`.
- Equality and inequality comparisons against null or other data-member pointers compare the `i32` values directly.
- Access via `.*` and `->*` evaluates the receiver address, adds the `i32` offset, and produces an lvalue address. If the object expression is an lvalue, `object .* pmd` produces an lvalue.

### 3. Member-Function Pointer Representation (`src/codegen/member_pointer.rs`)
Member-function pointers pack `{ ptr: u32, adj: u32 }` into a WebAssembly `i64` value `(adj << 32) | (ptr & 0xFFFFFFFF)`:
- `ptr` (low 32 bits):
  - Bit 0 = 0: Non-virtual function. The WebAssembly function table index is `ptr >> 1`.
  - Bit 0 = 1: Virtual function. The vtable slot index is `ptr >> 1`, looked up in the vptr of the subobject that declares the method (`adj` carries that subobject's offset within the pointer's class).
- `adj` (high 32 bits): Signed byte adjustment added to the `this` pointer before dispatch to navigate multiple inheritance subobjects.
- Null representation: `0i64` (`ptr == 0 && adj == 0`). Comparisons against null check `eqz` or `ne` against `0i64`.

Under multiple inheritance a polymorphic class has one vptr per polymorphic base subobject, not a single vptr at offset zero. Each vtable entry is twelve bytes: a function-table index, a signed `this` adjustment converting the caller's subobject pointer into the one the final overrider expects, and a signed return adjustment converting a covariant return value into the base interface's return type. This keeps ordinary virtual calls and member-pointer virtual calls on the same dispatch path.

### 4. Member-Function Call Lowering
The compiler lowers `(obj.*pmf)(args...)` and `(ptr->*pmf)(args...)` in `src/codegen/member_pointer.rs`:
1. Evaluates the receiver expression (`obj` lvalue address or `ptr` value) and saves it to a 4-byte local frame slot (`this_slot`).
2. Evaluates the member function pointer `pmf` into a 64-bit scratch register.
3. Unpacks `adj`: shifts the `i64` right by 32, wraps to `i32`, adds it to `this_slot`, and updates `this_slot`.
4. Tests bit 0 of `ptr`:
   - If bit 0 is 1 (virtual): loads the vtable pointer from offset 0 of `this_slot`, shifts `ptr` right by 1, multiplies by 12, adds to the vtable address, and loads the target function index from the 12-byte vtable entry.
   - If bit 0 is 0 (non-virtual): shifts `ptr` right by 1 to get the target function index.
   - Stores the resolved function index into a 4-byte local frame slot (`fn_slot`).
5. Loads `this_slot` as the first argument, compiles remaining call arguments, loads `fn_slot`, and emits `call_indirect`.

### 5. Base-to-Derived Conversions (`src/sema/assign.rs`, `src/codegen/memory.rs`)
ISO C++17 [conv.mem] permits implicit conversion from `Base::*` to `Derived::*` when `Derived` is an unambiguous, accessible derived class of `Base`. The direction is inverted compared to object pointers: a member pointer expecting `Base` can safely operate on a `Derived` instance, but not vice versa. Ambiguous bases (multiple paths to the same base) and inaccessible (private/protected) bases are rejected during semantic analysis.

The compiler handles this in `src/codegen/memory.rs`:
- Data-member pointers: when non-null (`val != -1`), the conversion adds `base_subobject_offset(Derived, Base)` to the data-member offset. Null values remain `-1`.
- Member-function pointers: when non-null (`val != 0i64`), the conversion adds `(base_subobject_offset(Derived, Base) as i64) << 32` to the `adj` field. Null values remain `0i64`.

### 6. C++17 Fold Expression Integration (`src/sema/fold_expansion.rs`)
Fold expressions over `.*` and `->*` are expanded during template substitution in `expand_fold_expr`. When a pack expands across `.*` or `->*`, the engine constructs concrete `Expr::Binary` trees with left or right associativity matching standard C++17 rules. The existing semantic analyzer and WebAssembly code generator process the resulting trees without special runtime cases.

## Consequences

### Positive
- Full support for ISO C++17 pointer-to-member operations, covering data members, non-virtual methods, virtual methods, multiple inheritance, base-to-derived conversions, and null comparisons.
- Application (`.*`/`->*`) enforces const-correctness and rejects ambiguous or inaccessible base conversions; a `noexcept` member-function pointer converts to the throwing type but not the reverse.
- Calls through member-function pointers yield the declared return type, including floating-point and struct results.
- Flat `i64` member-function pointer representation fits into WebAssembly registers without linear memory allocations.
- Offset -1 distinguishes null data-member pointers from member pointers at offset 0.
- Direct integration with C++17 fold expressions across all four fold shapes.

### Negative / Trade-offs
- Calling a member function pointer allocates two 4-byte frame slots (`this_slot` and `fn_slot`) in the caller frame to stage the indirect call operands.
- Virtual base classes remain unsupported in multiple inheritance hierarchies.
