# ADR 0009: WebAssembly C ABI and JavaScript Host Interoperability

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
When running the compiler inside Node.js or a browser Web Worker, JavaScript must pass source strings into the WebAssembly instance and retrieve compiled WebAssembly binaries or error messages without memory leaks. Furthermore, compiled C++ programs need an execution environment for basic I/O and process exit.

## Options Considered
- **Option 1: Use `wasm-bindgen` with complex JavaScript glue generation**
  - *Pros*: Automatic conversion between JS and Rust types.
  - *Cons*: Introduces large external dependencies, complex build tools, and generated wrapper scripts that complicate standalone embedding.
- **Option 2: Low-level C ABI with manual buffer management and a standardized JavaScript shim (`web/host.js`)**
  - *Pros*: Dependency-free; zero code bloat; explicit ownership semantics; portable across Node.js, Web Workers, and native embedders.
  - *Cons*: Requires disciplined pair-calling of allocation and deallocation APIs.

## Decision
1. **Compiler C ABI Entry Points ([`src/ffi.rs`](../../src/ffi.rs))**:
   - `alloc_buffer(size)`: Allocates an uninitialized input buffer for source code.
   - `dealloc_buffer(ptr, size)`: Frees an allocated input buffer if compilation is aborted.
   - `compile_wasm(ptr, len)`: Compiles UTF-8 source. Returns pointer to compiled `.wasm` bytes, or `0` on error.
   - `get_result_len()`: Returns length of the successful result buffer.
   - `free_result(ptr, len)`: Explicitly frees the boxed slice containing the compiled WebAssembly binary.
   - `get_error_ptr()` / `get_error_len()`: Returns the static diagnostic buffer (owned by the compiler; never freed by host).
2. **JavaScript Host Runtime ([`web/host.js`](../../web/host.js))**:
   - Implements standard `env` imports for generated programs: `print_char`, `print_int`, `print_double`, `read_char`, `exit`.
   - Formats floating point values and integer conversions in JavaScript matching the compiler's runtime contract.

## Consequences
### Positive
- Zero external build or runtime dependencies; the compiler is a pure `cdylib`.
- Repeated compilations keep linear memory bounded without leaks, as proven by [`tests/ffi_lifecycle.js`](../../tests/ffi_lifecycle.js).
- Identical `host.js` shim runs in both Node.js test harnesses and browser Web Workers.

### Negative / Trade-offs
- Hosts must adhere strictly to the allocation lifecycle: buffers must be released with their matching deallocation calls.
