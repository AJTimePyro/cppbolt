# C++ to WebAssembly subset compiler

This repository contains a dependency-free Rust compiler for a defined subset of C++17. It preprocesses,
lexes, parses, semantically checks, and emits WebAssembly directly, with a native CLI and a browser/Node C ABI.
The project is correctness-focused and intentionally does not implement templates, standard containers,
exceptions, RTTI, or other Phase 3/4 features.

Build and run the current corpus:

```sh
cargo build --release
cargo run -p cppwasm-testrunner -- target/release/cppwasm-cli
```

The architecture and ABI contracts are documented in [`docs/`](docs/README.md). The embedded headers are in
[`stdlib/`](stdlib/), and the JavaScript host shim is [`web/host.js`](web/host.js).
