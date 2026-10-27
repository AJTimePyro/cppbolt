# ADR 0002: Direct WebAssembly Emission Without Intermediate Representation or Optimization

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Compilers traditionally lower an AST to one or more Intermediate Representations (e.g. SSA form, 3-address code, control flow graphs) before executing optimization passes and target code generation. This adds substantial conceptual and runtime complexity, increases memory consumption during compilation, and requires additional data structures that enlarge the compiled wasm binary of the compiler itself.

## Options Considered
- **Option 1: Multi-stage pipeline with an explicit IR and bytecode optimizer**
  - *Pros*: Potential for constant folding, dead code elimination, and register/local allocation optimizations.
  - *Cons*: Higher compiler binary size, more complex codebase, increased memory usage during compilation, slower compilation throughput in browser workers.
- **Option 2: Direct syntax-directed emission from typed AST to WebAssembly binary sections**
  - *Pros*: Highly transparent lowering; minimal memory overhead; rapid compilation speed; simple direct mapping between C++ language semantics and WebAssembly instructions.
  - *Cons*: Generated code is unoptimized; scratch locals and stack allocations are conservative.

## Decision
The backend writes WebAssembly binary sections directly from the typed AST and semantic symbol table without an intermediate representation or optimization pipeline. Generated-code optimization is explicitly out of scope for v1.

## Consequences
### Positive
- The compiler remains small, fast, and dependency-free.
- Target ABI and generated instructions directly reflect source semantics, making debugging and verification predictable.
- Clean separation between semantic analysis (`src/sema/`) and opcode emission (`src/codegen/`, `src/wasm/`).

### Negative / Trade-offs
- Generated WebAssembly bytecode contains unoptimized instruction sequences (e.g. redundant local loads/stores).
- Frame allocation must be conservatively managed via fixed offsets and LEB128 patch points.
