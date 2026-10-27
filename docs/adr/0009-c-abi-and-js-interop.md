# ADR 0009: WebAssembly C ABI and JavaScript Host Interoperability

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
When running the compiler inside Node.js or a browser Web Worker, JavaScript must pass source strings into the WebAssembly instance and retrieve compiled WebAssembly binaries or error messages without memory leaks (`src/ffi.rs`, `web/playground-worker.js`, `tests/ffi_lifecycle.js`). Furthermore, compiled C++ programs need an execution environment for basic I/O and process exit (`web/host.js`, commit `d23afed`, `tests/browser_flow.rs`).

## Options Considered
- **Option 1: Use `wasm-bindgen` with complex JavaScript glue generation**
  - *Pros*: Automatic conversion between JS and Rust types.
  - *Cons*: Introduces large external dependencies, complex build tools, and generated wrapper scripts that complicate standalone embedding.
- **Option 2: Low-level C ABI with manual buffer management and a standardized JavaScript shim (`web/host.js`)**
  - *Pros*: Dependency-free; zero code bloat; explicit ownership semantics; portable across Node.js, Web Workers, and native embedders.
  - *Cons*: Requires disciplined pair-calling of allocation and deallocation APIs.

## Decision
1. **Compiler C ABI Entry Points ([`src/ffi.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/ffi.rs))**:
   - `alloc_buffer(size)`: Allocates an uninitialized input buffer for source code (`src/ffi.rs#L30-L40`).
   - `dealloc_buffer(ptr, size)`: Frees an allocated input buffer if compilation is aborted (`src/ffi.rs#L42-L50`).
   - `compile_wasm(ptr, len)`: Compiles UTF-8 source. Returns pointer to compiled `.wasm` bytes, or `0` on error (`src/ffi.rs#L52-L80`).
   - `get_result_len()`: Returns length of the successful result buffer (`src/ffi.rs#L82-L88`).
   - `free_result(ptr, len)`: Explicitly frees the boxed slice containing the compiled WebAssembly binary (`src/ffi.rs#L90-L98`).
   - `get_error_ptr()` / `get_error_len()`: Returns the static diagnostic buffer (owned by the compiler; never freed by host) (`src/ffi.rs#L100-L115`).
2. **JavaScript Host Runtime ([`web/host.js`](https://github.com/cppwasm/cpp_web/blob/main/web/host.js))**:
   - Implements standard `env` imports for generated programs: `print_char`, `print_int`, `print_double`, `read_char`, `exit` (`web/host.js#L15-L60`).
   - Formats floating point values and integer conversions in JavaScript matching the compiler's runtime contract (`web/host.js#L62-L80`).

## Consequences
### Positive
- Zero external build or runtime dependencies; the compiler is a pure `cdylib` (`Cargo.toml`, `src/ffi.rs`).
- Repeated compilations keep linear memory bounded without leaks, as proven by [`tests/ffi_lifecycle.js`](https://github.com/cppwasm/cpp_web/blob/main/tests/ffi_lifecycle.js) (`tests/ffi_lifecycle.rs`).
- Identical `host.js` shim runs in both Node.js test harnesses and browser Web Workers (`web/host.js`, `tests/browser_flow.rs`).

### Negative / Trade-offs
- Hosts must adhere strictly to the allocation lifecycle: buffers must be released with their matching deallocation calls (`src/ffi.rs#L30-L70`, `tests/ffi_lifecycle.js`).
