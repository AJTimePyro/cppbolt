# ADR 0002: Direct WebAssembly Emission Without Intermediate Representation or Optimization

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Compilers traditionally lower an AST to one or more Intermediate Representations (e.g. SSA form, 3-address code, control flow graphs) before executing optimization passes and target code generation <!-- TODO: confirm with author -->. This adds substantial conceptual and runtime complexity, increases memory consumption during compilation, and requires additional data structures that enlarge the compiled wasm binary of the compiler itself (`docs/decisions/002-direct-wasm.md`, commit `d23afed`).

## Options Considered
- **Option 1: Multi-stage pipeline with an explicit IR and bytecode optimizer**
  - *Pros*: Potential for constant folding, dead code elimination, and register/local allocation optimizations.
  - *Cons*: Higher compiler binary size, more complex codebase, increased memory usage during compilation, slower compilation throughput in browser workers.
- **Option 2: Direct syntax-directed emission from typed AST to WebAssembly binary sections**
  - *Pros*: Highly transparent lowering; minimal memory overhead; rapid compilation speed; simple direct mapping between C++ language semantics and WebAssembly instructions.
  - *Cons*: Generated code is unoptimized; scratch locals and stack allocations are conservative.

## Decision
The backend writes WebAssembly binary sections directly from the typed AST and semantic symbol table without an intermediate representation or optimization pipeline (`docs/decisions/002-direct-wasm.md`, commit `d23afed`, `src/codegen/mod.rs`, `src/wasm/module.rs`). Generated-code optimization is explicitly out of scope for v1 (`docs/decisions/002-direct-wasm.md`, commit `d23afed`).

## Consequences
### Positive
- The compiler remains small, fast, and dependency-free (`docs/decisions/002-direct-wasm.md`, commit `d23afed`, `Cargo.toml`).
- Target ABI and generated instructions directly reflect source semantics, making debugging and verification predictable (`docs/decisions/002-direct-wasm.md`, commit `d23afed`, `tests/golden/`).
- Clean separation between semantic analysis and opcode emission (`src/sema/mod.rs`, `src/codegen/mod.rs`, `src/lib.rs#L34-L52`).

### Negative / Trade-offs
- Generated WebAssembly bytecode contains unoptimized instruction sequences (e.g. redundant local loads/stores) because optimization is out of scope (`docs/decisions/002-direct-wasm.md`, commit `d23afed`).
- Frame allocation must be conservatively managed via fixed offsets and LEB128 patch points (`src/codegen/function_context.rs#L24-L50`).
