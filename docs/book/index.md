# Introduction

Welcome to the **cppwasm** documentation.

`cppwasm` is a dependency-free, correctness-oriented C++ subset compiler written from scratch in Rust. It compiles standard C++17 source code directly into WebAssembly (`.wasm`) binary format for instant execution in web browsers, Web Workers, Node.js, and headless test runners.

Try the compiler directly online in the [Browser Playground](https://ajtimepyro.is-a.dev/cpp-web-compiler/).

## Book Navigation

- **[User Guide](user/getting-started.md)**:
  - [Getting Started](user/getting-started.md): Installation, command-line usage, and compiling samples.
  - [Supported C++ Feature Matrix](user/supported-cpp.md): Complete matrix of supported C++ constructs, test cases, and known limitations.
  - [Browser Integration](user/browser-integration.md): C ABI, Web Worker integration, and JavaScript host environment.
  - [Diagnostics & Errors](user/errors.md): Diagnostic format, source spans, and catalog of emitted errors.
- **[Compiler Internals](internals/architecture.md)**:
  - [Architecture Overview](internals/architecture.md): The five-stage pipeline and direct AST-to-Wasm lowering model.
  - [Preprocessor](internals/preprocessor.md), [Lexer](internals/lexer.md), [Parser & AST](internals/parser-and-ast.md), [Semantic Analysis](internals/semantic-analysis.md), and [WASM Codegen](internals/wasm-codegen.md).
  - [Runtime & Standard Library](internals/runtime.md): Memory layout, free-list allocator, and embedded STL headers.
  - [Testing System](internals/testing.md): Corpus runner, variant tests, and native compiler oracle gates.
- **[Architecture Decision Records](adr/0001-own-compiler.md)**:
  - Architectural, ABI, and language decisions with documented evidence and trade-offs.
- **[Contributing Guide](contributing.md)**:
  - Building, running tests, code standards, and PR requirements.
