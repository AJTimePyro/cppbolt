# ADR 0006: WebAssembly Linear Memory Layout and Stack Safety

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
WebAssembly provides a single contiguous linear memory array per instance (`src/runtime/memory_layout.rs`, commit `0b21628`). The compiler must layout static data, the execution stack for locals whose addresses are taken, and dynamic heap allocations while preventing silent memory corruption between stack and heap (`src/runtime/memory_layout.rs`, commit `0b21628`, `tests/wasm_stack_safety.rs`).

## Options Considered
- **Option 1: Upward-growing stack, downward heap**
  - *Pros*: Simple stack increment.
  - *Cons*: Stack overflow grows into data/heap areas, causing silent memory corruption unless bounds checks are added to every allocation.
- **Option 2: 64 KiB guard region, downward-growing stack, upward-growing heap**
  - *Pros*: Follows standard ABI conventions; stack overflows hit a reserved zero-page guard region; clear separation between stack and heap.
  - *Cons*: Stack pointer must be initialized to a high address and decremented per call frame.

## Decision
Establish the linear memory layout defined in [`src/runtime/memory_layout.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/runtime/memory_layout.rs) (commit `0b21628`):
1. **Guard Page**: Address `0` to `65,535` (`64 KiB`). Reserved region. Address `0` serves as the null pointer representation.
2. **Stack Region**: Address `65,536` (`64 KiB`) to `8,454,143` (`8 MiB + 64 KiB`). Grows downwards. The global stack pointer is initialized to `STACK_TOP` (`8,454,144`).
3. **Static Data Region**: Begins at `DATA_BASE` (`STACK_TOP`). Holds string literals, static variables, and vtables.
4. **Heap Region**: Begins immediately after static data and grows upwards toward the maximum configured memory limit (default 256 MiB).

In function prologues, stack frame allocations check that the new stack pointer does not decrement below `GUARD_PAGE_END` (`65,536`). If it does, the program immediately triggers a WebAssembly `unreachable` trap (`src/codegen/function_context.rs#L50-L75`, `tests/wasm_stack_safety.rs`).

## Consequences
### Positive
- Stack overflow cannot silently corrupt static data or the heap; it traps deterministically before entering the guard page (`src/runtime/memory_layout.rs#L40-L65`, `tests/wasm_stack_safety.rs`).
- Address zero serves as the null pointer representation and is never returned by valid allocations (`src/runtime/memory_layout.rs#L12-L20`).
- Fixed constants ensure reproducibility between native test harnesses and browser hosts (`src/runtime/memory_layout.rs#L8-L30`, `tests/browser_flow.rs`).

### Negative / Trade-offs
- Maximum stack depth is fixed at 8 MiB per WebAssembly instance (`src/runtime/memory_layout.rs#L22`).
- Prologue emissions require explicit stack bounds checks against `GUARD_PAGE_END` (`src/codegen/function_context.rs#L50-L75`).
