# ADR 0005: Exclusion of Exceptions and RTTI from v1 Subset

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Exceptions and RTTI are excluded from v1 (`docs/decisions/005-no-exceptions-rtti.md`, commit `d23afed`). Unsupported constructs produce diagnostics rather than silently lowering to a different runtime model (`docs/decisions/005-no-exceptions-rtti.md`, commit `d23afed`, `src/parser/unsupported.rs`). Full C++ runtime features like exception handling (`try`/`catch`/`throw`) and Run-Time Type Information (`typeid`, `dynamic_cast`) require substantial runtime machinery (stack unwinding tables, personality functions, type descriptor tables) that are omitted to keep the direct lowering model simple and deterministic (`docs/decisions/005-no-exceptions-rtti.md`, `src/codegen/mod.rs`).

## Options Considered
- **Option 1: Implement WebAssembly exception handling proposal or Emscripten-style setjmp/longjmp emulation**
  - *Pros*: Supports idiomatic C++ error handling.
  - *Cons*: Immensely complicates the direct WebAssembly code generator; requires complex unwinding frames; emulated unwinding severely degrades performance.
- **Option 2: Exclude exceptions and RTTI from the v1 compiler subset**
  - *Pros*: Code generator remains clean, simple, and deterministic; RAII cleanups are strictly local and stack-based; zero unwinding runtime overhead.
  - *Cons*: C++ programs relying on `try`/`catch` or polymorphic dynamic casting are not supported in v1.

## Decision
Exclude exceptions and RTTI from the v1 compiler subset (`docs/decisions/005-no-exceptions-rtti.md`, commit `d23afed`). Unsupported constructs produce explicit compile-time diagnostics (`ErrorKind::Unsupported` or `ErrorKind::Syntax`) rather than silently lowering to incomplete or hazardous runtime behaviors (`src/parser/unsupported.rs`, `src/diagnostics/mod.rs`).

## Consequences
### Positive
- Unsupported constructs produce clear compile-time diagnostics rather than silently lowering to a different runtime model (`docs/decisions/005-no-exceptions-rtti.md`, commit `d23afed`, `tests/negative/`).
- Destructor invocations and RAII cleanups follow straightforward lexical scope exit rules (`src/codegen/stmt.rs#L180-L240`, `tests/classes_oop/memory_raii/class_raii.cpp`).
- Vtables contain only function pointers for virtual method dispatch, without type descriptor pointers (`src/codegen/vtables.rs#L20-L45`).

### Negative / Trade-offs
- Code using `throw`, `try`, `catch`, `typeid`, or `dynamic_cast` fails compilation with explicit diagnostics (`src/diagnostics/mod.rs`, `src/parser/expr.rs`).
- Allocator failures in `new` trap deterministically instead of throwing `std::bad_alloc` (`src/runtime/allocator.rs#L85-L102`).
