# Roadmap

1. Complete source-map spans and structured diagnostics through preprocessing and AST nodes.
2. Keep the compiler as one Cargo package with an OS-agnostic library, a native CLI, an isolated C ABI module, and a corpus runner.
3. Replace string-keyed semantic state with arena IDs and normalize expression value categories.
4. Decompose code generation and replace conservative frame allocation with measured per-function layouts.
5. Replace the shell runner with a Rust harness and make baseline/hash comparison reproducible.
6. Only after the v1 cleanup, consider Phase 3 templates and containers.
