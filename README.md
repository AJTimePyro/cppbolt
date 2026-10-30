# CPPBolt

A dependency-free C++ to WebAssembly subset compiler written in Rust.

`cppbolt` compiles a defined subset of C++17 directly into WebAssembly (`.wasm`) binaries without relying on LLVM, Emscripten, or external native toolchains. It is built to run both natively as a CLI and in the browser (via a WebAssembly build of the compiler itself and a minimal JavaScript host runtime).

The compiler is correctness-focused and implements an explicit, predictable language subset. It avoids complex features like templates, standard containers and algorithms, exceptions, and RTTI, choosing deterministic compilation speed and small binary size.

---

## Quick Demo

### 1. Build and Run Natively

Build the release compiler and run the test suite:

```sh
# Build the native CLI compiler (target/release/cppwasm-cli)
cargo build --release

# Run the testrunner against the compiled CLI
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

Compile a simple C++ program to WebAssembly:

```sh
echo '
#include <iostream>

int main() {
    std::cout << "Hello from WebAssembly C++!" << std::endl;
    return 0;
}
' > hello.cpp

target/release/cppwasm-cli hello.cpp -o hello.wasm
```

### 2. Try in the Browser (Playground)

You can try the compiler directly in your browser:
- **Live Online Playground**: [https://ajtimepyro.is-a.dev/cppbolt/](https://ajtimepyro.is-a.dev/cppbolt/)
- **Online Documentation**: [https://ajtimepyro.is-a.dev/cppbolt/book/](https://ajtimepyro.is-a.dev/cppbolt/book/)

To build and run the playground locally:

```sh
# Build the compiler library for WebAssembly
cargo build --release --target wasm32-unknown-unknown --lib

# Assemble the browser playground assets
tools/build_site.sh

# Serve locally
python3 -m http.server --directory site 8000
# Open http://localhost:8000 in your browser
```

The browser playground runs the compiler in a dedicated Web Worker to ensure UI responsiveness, and executes the compiled WebAssembly binary using [`web/host.js`](web/host.js).

---

## Documentation

Comprehensive project documentation is published via mdBook at [https://ajtimepyro.is-a.dev/cppbolt/book/](https://ajtimepyro.is-a.dev/cppbolt/book/):

- **[Getting Started](docs/book/user/getting-started.md)** — Guide to installing, building, and using the compiler.
- **[Supported C++ Subset](docs/book/user/supported-cpp.md)** — Full feature matrix with test links and known limitations.
- **[Browser Integration](docs/book/user/browser-integration.md)** — WebAssembly C ABI, host JavaScript runtime, and worker architecture.
- **[Diagnostics & Errors](docs/book/user/errors.md)** — Compiler diagnostics catalog with real trigger examples.
- **[Compiler Architecture](docs/book/internals/architecture.md)** — Pipeline architecture, stages, and data flows.
- **[Stage Internals](docs/book/internals/lexer.md)** — Deep dives into the [Preprocessor](docs/book/internals/preprocessor.md), [Lexer](docs/book/internals/lexer.md), [Parser & AST](docs/book/internals/parser-and-ast.md), [Semantic Analysis](docs/book/internals/semantic-analysis.md), [WASM Codegen](docs/book/internals/wasm-codegen.md), and [Runtime](docs/book/internals/runtime.md).
- **[Testing System](docs/book/internals/testing.md)** — Test runner, golden baseline, variant tests, and native compiler oracle.
- **[Architecture Decision Records (ADRs)](docs/adr/template.md)** — Accepted architectural and ABI decisions.
- **[Contributing Guide](docs/contributing.md)** — How to contribute, verify changes, and add language features.
- **[Development Log](docs/devlog.md)** — Ongoing development notes.

---

## Project Structure

| Path | Responsibility |
|---|---|
| [`src/lib.rs`](src/lib.rs) | Core compiler pipeline (`preprocessor -> lexer -> parser -> sema -> codegen`) |
| [`src/main.rs`](src/main.rs) | Native CLI front end (`cppwasm-cli`) with filesystem include resolution |
| [`src/ffi.rs`](src/ffi.rs) | WebAssembly C ABI entry points for embedding in browser or Node.js |
| [`src/bin/cppwasm-testrunner.rs`](src/bin/cppwasm-testrunner.rs) | Corpus test runner, golden baseline verification, and native oracle gates |
| [`src/bin/cppwasm-census.rs`](src/bin/cppwasm-census.rs) | Code-quality scanner enforcing function length and parameter limits |
| [`stdlib/`](stdlib/) | Embedded standard library C++ headers (`<iostream>`, `<string>`, `<cstdio>`, etc.) |
| [`web/`](web/) | Browser playground HTML, worker script, and JavaScript host environment (`host.js`) |
| [`tests/`](tests/) | Compiler test suite (positive tests, negative error tests, runtime traps, variants) |
| [`docs/`](docs/) | mdBook documentation, ADRs, contributing guide, and devlog |
