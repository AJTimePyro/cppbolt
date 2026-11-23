# ADR 0016: WebAssembly Exception Handling and Stack Unwinding

## Status
Accepted

## Context
ISO C++17 exception handling ([except]) specifies structured exception propagation via `throw`, `try`, and `catch`. When an exception is thrown, the runtime must:
1. Transfer control to the nearest dynamically enclosing matching `catch` clause.
2. Unwind the stack, destroying all fully constructed automatic objects with automatic storage duration in reverse order of completion of their construction ([except.ctor]).
3. Destroy fully constructed base classes and non-static data members if a constructor exits via an exception.
4. Call `std::terminate()` immediately if a destructor invoked during stack unwinding exits with an exception ([except.terminate]), or if `throw;` is evaluated with no active exception.
5. Preserve outer scope objects when an exception is caught locally within an enclosing function.
6. Restore caller stack frames on the shadow stack (`global 0`) before propagating an unhandled exception across function boundaries.

Initially, [ADR 0005](0005-no-exceptions-rtti.md) excluded exception handling from cppbolt to keep the direct WebAssembly code generator simple. Early prototyping experimented with per-call `try_table` cleanup wrappers, but an architectural audit revealed severe correctness flaws:
- Objects belonging to active parent scopes were prematurely destroyed.
- Inner cleanup wrappers destroyed the same objects multiple times.
- Shadow stack allocations were corrupted by nested call unwinding.
- Member, virtual, and indirect call sites were skipped.

WebAssembly Phase 4 Exception Handling adds standard primitives (`try_table`, `catch`, `catch_ref`, `catch_all`, `catch_all_ref`, `throw`, and `throw_ref`) that permit structured exception handling directly in WebAssembly without Emscripten `setjmp`/`longjmp` emulation or external DWARF unwinding tables.

## Options Considered

### Option 1: Emscripten-style JS/setjmp-longjmp emulation
- *Pros*: Runs on older WebAssembly runtimes lacking native exception support.
- *Cons*: Degrades runtime throughput; requires JavaScript trampolines and state machines across function calls; increases WebAssembly binary size.

### Option 2: Per-call `try_table` cleanup wrappers
- *Pros*: Simple to generate at individual call expressions.
- *Cons*: Incompatible with C++ scoping rules. Destroys objects from outer scopes prematurely, duplicates destructions on nested calls, corrupts the shadow stack, and misses member or indirect calls.

### Option 3: Function-level `try_table` with dynamic 64-bit liveness tracking
- *Pros*:
  - Single enclosing `try_table (catch_all_ref $unwind)` per throwing or calling function.
  - A 64-bit bitmask (`cleanup_mask_local: i64`) tracks object construction and destruction at runtime, running destructors in reverse order only for active instances.
  - Leaf functions without calls or destructors emit no unwinding instructions.
  - Cleanups wrapped in `try_table (catch_all -> unreachable)` enforce `std::terminate` if a destructor throws during unwinding.
  - Integrates directly with the ILP32 shadow stack (`global 0`).
- *Cons*:
  - Caps simultaneously tracked cleanup objects in a single function at 64.

## Decision

We chose **Option 3**.

### 1. WebAssembly Tag and Opcodes (`src/runtime/opcodes.rs`, `src/wasm/module.rs`)
We register WebAssembly exception tag 0 with signature `(i32) -> void` in section 13 of the WebAssembly module, representing primitive exception payloads. We support opcodes `OP_TRY_TABLE` (`0x1F`), `OP_THROW` (`0x08`), and `OP_THROW_REF` (`0x0A`).

### 2. Function-Level Unwinding (`src/codegen/function.rs`, `src/codegen/unwind.rs`)
For any function that possesses class members/bases, is `main`, takes struct parameters, or contains calls, throws, or try blocks (determined via `stmt_can_throw_or_call`), the function body is wrapped in a function-level `try_table`:
```wat
(block $normal_exit
  (block (ref null exn) $unwind
    (try_table (catch_all_ref $unwind)
      ;; function body
      br $normal_exit
    )
    unreachable
  )
  ;; function unwind handler:
  ;; 1. local.set $exnref
  ;; 2. execute active cleanups in reverse construction order
  ;; 3. restore caller shadow-stack frame: global.set 0 (fp + frame_size)
  ;; 4. local.get $exnref; throw_ref
)
```
Pure leaf functions (such as getters or arithmetic helpers) omit this wrapper entirely.

### 3. Dynamic Liveness Bitmask (`src/codegen/unwind.rs`)
Each function context allocates a 64-bit local variable (`cleanup_mask_local: i64`):
- Upon successful construction of an automatic object, field, or base subobject, `emit_mark_cleanup_active` sets `cleanup_mask |= (1 << index)`.
- Upon normal scope exit or destruction, `emit_mark_cleanup_inactive` clears `cleanup_mask &= ~(1 << index)`.
- Functions registering 64 or more cleanup objects fail compilation with an explicit error.
- During unwinding, `emit_cleanup_range` iterates over registered entries in reverse order, executing only those whose bit is set in the liveness mask.

### 4. Scope-Isolated Try/Catch, Typed Matching, and Object Lifecycle (`src/codegen/stmt_try.rs`, `src/codegen/stmt_throw.rs`, `src/sema/exception_types.rs`)
A `try` statement compiles to an inner `try_table` targeting its catch handlers:
- A `try` statement supports multiple sequential `catch` clauses (such as `catch (Derived&)`, `catch (Base&)`, and `catch (...)`).
- ISO C++17 [except.handle] handler ordering is strictly enforced: `catch (...)` must appear last in the handler sequence. If any catch clause follows `catch (...)`, semantic analysis rejects the program with a compile-time error.
- Exception objects are heap-allocated with a 16-byte header: `[offset 0: type_id (u32), offset 4: dtor_table_idx (i32), offset 8: flags (u32), offset 12: size (u32)]`, followed by the exception payload at offset 16. WebAssembly exception tag 0 carries the 32-bit pointer to this header.
- Thrown expressions initialize the payload: class types invoke move constructors for eligible local variables or xvalues, copy constructors, or byte copies directly into the heap buffer; scalar types and pointers store their raw values.
- If payload construction throws an exception, a `try_table` guard frees the allocated exception buffer before rethrowing the constructor exception.
- In `try_table`, tag 0 branches to a typed dispatch block that sequentially tests each catch clause using `__cppbolt_can_catch(thrown_id, handler_id)`.
- `__cppbolt_can_catch` evaluates exact type matches, ISO C++17 [conv.qual] pointer qualification conversions (including multi-level conversions such as `T**` to `const T* const*` and `T* const*`), void pointer matches (`void*` to mutable pointers, `const void*` to any pointer), `nullptr` exceptions to pointer handlers, and derived-to-base class inheritance. Pointer type IDs use a bitfield encoding (bits 0..15 base ID, bits 16..19 indirection depth, bits 20..27 const mask per level) that prevents collisions across different indirection levels and class counts.
- For derived exceptions caught by base handlers, `__cppbolt_can_catch` returns the base subobject byte offset. For pointer catches, the code applies this offset directly to the dereferenced pointer value while preserving null pointers.
- Catch parameters bind according to declared type:
  - Catch by reference (`catch (T&)` or `catch (const T&)`): stores the payload address (adjusted for subobject offset) into a local slot, enabling zero-copy access and in-place mutation.
  - Catch by value (`catch (T)`): allocates a local slot and invokes the copy constructor (or performs a byte copy) from the payload address, destroying the local copy at catch scope exit even when the parameter is unnamed. Slicing occurs when catching a derived object as a base value.
- Handled exception slots are tracked in `active_exception_slots`. Exiting a catch handler normally, via `return`, or via loop `break` or `continue` invokes the exception destructor (via `call_indirect` on `dtor_table_idx`) and frees the heap allocation with `free`.
- Catch clause bodies execute inside a nested exception handler. If a new exception propagates out of a catch body, the runtime destroys the currently handled exception object before propagating the new exception. Rethrow (`throw;`) rethrows the active exception object without destroying it.
- Outside any catch block (`catch_depth == 0`), `throw;` emits `OP_UNREACHABLE` to trigger `std::terminate()`.
- Dead-code elimination scans reachable functions to collect classes referenced in `throw` expressions, `catch` parameters, and base subobjects, retaining only those destructors and their call trees rather than marking all program classes reachable indiscriminately.
- Semantic analysis validates throw and catch types: throwing `void`, incomplete types, or abstract classes fails compilation; catching abstract classes by value or catching rvalue references (`catch (T&&)`) fails compilation.

### 5. Terminating Destructors (`src/codegen/unwind.rs`)
All destructor invocations in `emit_cleanup_range` are enclosed within a `try_table (catch_all -> unreachable)` block. If any destructor throws while unwinding an active exception, WebAssembly immediately traps with `unreachable` (`std::terminate`).

### 6. Constructor Failure Subobject Cleanup (`src/codegen/function_lifecycle.rs`)
Constructors register fully initialized base classes and member fields in the function's cleanup mask immediately after construction. If a subsequent member initializer or constructor body throws, completed subobjects are destroyed in reverse order before propagating the exception, and the cleanup mask is zeroed upon successful constructor exit.

### 7. Temporary Object Lifetime and Unwinding (`src/codegen/call_args.rs`, `src/codegen/expr_call.rs`, `src/codegen/raii.rs`)
Fully constructed temporary objects created during expression evaluation register cleanup entries with the liveness bitmask:
- If a later argument expression or a called function exits with an exception, completed temporaries run their destructors and free their heap buffers in reverse construction order.
- In conditional expressions (`?:`, `&&`, `||`), temporaries in unexecuted branches are never constructed. Normal-path cleanup (`emit_clean_temporaries`) and unwinding cleanup guard destructor and `free` calls with liveness bitmask and non-null checks, preventing double-frees or null pointer dereferences on unconstructed instances.
- When an argument passes by value, the caller deactivates cleanup tracking only for the outer parameter object passed to the callee, leaving intermediate sub-expression temporaries active for full-expression destruction.
- Reference binding (`const T& ref = ...`) extends the temporary's lifetime by transferring its cleanup entry to the enclosing lexical scope. When bound to conditional expressions (`const T& ref = cond ? T(1) : T(2)`), all candidate temporaries transfer to the scope, and scope exit destroys only the dynamically constructed instance.
- If a statement contains more than 64 temporaries across chained binary expressions, `register_temp_cleanup` stops allocating mask slots, and the temporaries fall back to normal full-expression reclamation.

## Consequences

### Positive
- Matches C++17 exception control flow, typed catch matching, and RAII lifetime semantics across function boundaries, nested blocks, and loops.
- Supports class-type exceptions with dynamic allocation, copy construction, subobject slicing, and destructor cleanup.
- Supports single and multiple inheritance conversions with non-zero subobject offset adjustments in catch handlers.
- Supports catching by value, const reference, and non-const reference with in-place mutation.
- Lowers multiple catch clauses per try block to sequential dispatch blocks with WebAssembly `try_table`.
- Enforces ISO C++17 handler ordering rejecting clauses after `catch (...)`.
- Uses native WebAssembly instructions without JavaScript runtime trampolines.
- Emits no unwinding instructions in leaf functions.
- Destroys active automatic objects in reverse construction order.
- Supports arbitrary statically nested catch handlers and expression temporary cleanup.
- Handles conditional expression temporaries and reference binding lifetime extension.

### Negative / Trade-offs
- Limits single functions to 64 tracked automatic/local variable cleanup objects, checked at compile time.
- Dynamic exception objects allocate heap memory through `malloc` and `free`.
- Multi-level qualification conversions beyond single-level pointers (such as `const T* const*` catching `T**`) are currently unhandled and will fail to catch.
- Volatile pointer qualifications are not modeled in AST types and remain unsupported.
