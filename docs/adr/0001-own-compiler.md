# ADR 0001: Implement Custom Subset Compiler Instead of Embedding LLVM

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Running C++ code in the browser typically relies on Emscripten and LLVM/Clang (`docs/decisions/001-own-compiler.md`, commit `d23afed`). However, shipping LLVM or Clang inside a browser environment (or compiling Clang itself to WebAssembly) results in tens of megabytes of binary payload, high memory usage, long startup latency, and unpredictable cold starts (`docs/decisions/001-own-compiler.md`, commit `d23afed`, `README.md`, commit `57df2cf`). Furthermore, for educational playgrounds or targeted web workloads, the full complexity of C++ is not required <!-- TODO: confirm with author -->.

## Options Considered
- **Option 1: Embed Clang/LLVM compiled to WebAssembly**
  - *Pros*: Full C++17/C++20 standards compliance out of the box; mature optimization pipeline.
  - *Cons*: Enormous shipped wasm size (>30 MB), multi-second browser initialization latency, heavy memory overhead.
- **Option 2: Implement a custom, dependency-free C++ compiler in Rust**
  - *Pros*: Predictable shipped binary size (~840 KB raw, ~244 KB gzipped), instant browser initialization, zero runtime dependencies, explicit and tightly controlled language subset boundaries.
  - *Cons*: Must author the preprocessor, lexer, parser, semantic analyzer, and code generator from scratch; standard library and complex features must be built incrementally.

## Decision
Implement a custom C++ subset compiler directly in Rust targeting WebAssembly (`docs/decisions/001-own-compiler.md`, commit `d23afed`, `README.md`, commit `57df2cf`). The compiler accepts an explicit, well-defined subset of C++17 and emits WebAssembly binary format directly.

## Consequences
### Positive
- Shipped WebAssembly size is kept under 1 MB uncompressed and ~245 KB gzipped (`README.md`, commit `57df2cf`).
- Zero external runtime or compiler dependencies; fast startup in browsers and Web Workers (`docs/decisions/001-own-compiler.md`, commit `d23afed`, `Cargo.toml`).
- The accepted language subset is explicit, deterministic, and tailored for browser execution (`docs/decisions/001-own-compiler.md`, commit `d23afed`).

### Negative / Trade-offs
- Features outside the subset (e.g. templates, full standard library algorithms, exceptions, RTTI) must be rejected or implemented by hand (`docs/decisions/005-no-exceptions-rtti.md`, commit `d23afed`, `src/parser/unsupported.rs`).
