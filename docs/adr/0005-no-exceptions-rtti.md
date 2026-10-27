# ADR 0005: Exclusion of Exceptions and RTTI from v1 Subset

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Full C++ runtime features like exception handling (`try`/`catch`/`throw`) and Run-Time Type Information (`typeid`, `dynamic_cast`) require substantial runtime machinery: stack unwinding tables, personality functions, type descriptor tables, and integration with the host environment or WebAssembly exception handling proposals.

## Options Considered
- **Option 1: Implement WebAssembly exception handling proposal or Emscripten-style setjmp/longjmp emulation**
  - *Pros*: Supports idiomatic C++ error handling.
  - *Cons*: Immensely complicates the direct WebAssembly code generator; requires complex unwinding frames; emulated unwinding severely degrades performance.
- **Option 2: Exclude exceptions and RTTI from the v1 compiler subset**
  - *Pros*: Code generator remains clean, simple, and deterministic; RAII cleanups are strictly local and stack-based; zero unwinding runtime overhead.
  - *Cons*: C++ programs relying on `try`/`catch` or polymorphic dynamic casting are not supported in v1.

## Decision
Exclude exceptions and RTTI from the v1 compiler subset. Unsupported constructs produce explicit compile-time diagnostics (`ErrorKind::Unsupported` or `ErrorKind::Syntax`) rather than silently lowering to incomplete or hazardous runtime behaviors.

## Consequences
### Positive
- Destructor invocations and RAII cleanups follow straightforward lexical scope exit rules.
- Vtables contain only function pointers for virtual method dispatch, without type descriptor pointers.
- Eliminates the need for external unwinding libraries or complex WebAssembly EH proposals.

### Negative / Trade-offs
- Code using `throw`, `try`, `catch`, `typeid`, or `dynamic_cast` fails compilation.
- Allocator failures in `new` trap deterministically instead of throwing `std::bad_alloc`.
