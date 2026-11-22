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

### 4. Scope-Isolated Try/Catch and Rethrow (`src/codegen/stmt_try.rs`)
A `try`/`catch` statement compiles to an inner `try_table` targeting its catch block:
- Catching tag 0 binds the integer payload into a frame slot if named (`catch (int e)`).
- Catch-all (`catch (...)`) captures `exnref` for rethrow via `throw_ref`.
- Only cleanups registered within the `try` block range (`start_clean_idx..end_clean_idx`) are executed when entering the catch block; outer function objects remain untouched and active.
- `throw;` inside a catch block rethrows the active exception (`throw_ref` or `throw 0`). Outside any catch block (`catch_depth == 0`), `throw;` emits `OP_UNREACHABLE` to trigger `std::terminate()`.

### 5. Terminating Destructors (`src/codegen/unwind.rs`)
All destructor invocations in `emit_cleanup_range` are enclosed within a `try_table (catch_all -> unreachable)` block. If any destructor throws while unwinding an active exception, WebAssembly immediately traps with `unreachable` (`std::terminate`).

### 6. Constructor Failure Subobject Cleanup (`src/codegen/function_lifecycle.rs`)
Constructors register fully initialized base classes and member fields in the function's cleanup mask immediately after construction. If a subsequent member initializer or constructor body throws, completed subobjects are destroyed in reverse order before propagating the exception, and the cleanup mask is zeroed upon successful constructor exit.

## Consequences

### Positive
- Matches C++17 exception control flow and RAII lifetime semantics across function boundaries, nested blocks, and loops.
- Uses native WebAssembly instructions without JavaScript runtime trampolines.
- Emits no unwinding instructions in leaf functions.
- Destroys active automatic objects in reverse construction order.

### Negative / Trade-offs
- Limits single functions to 64 tracked cleanup objects, checked at compile time.
- Uses an integer payload (tag 0); class hierarchies (`std::exception`) with RTTI type matching are deferred to a subsequent milestone.
