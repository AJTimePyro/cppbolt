# ADR 0004: Compile-Time Embedded Standard Library Headers

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
When running inside a browser Web Worker or sandboxed environment, the compiler cannot perform asynchronous or synchronous host filesystem reads to load `#include <iostream>` or other standard headers (`docs/decisions/004-embedded-stdlib.md`, commit `d23afed`, `web/playground-worker.js`). Packaging a virtual filesystem (VFS) or separate header bundle complicates asset loading and deployment (`docs/decisions/004-embedded-stdlib.md`, commit `d23afed`).

## Options Considered
- **Option 1: Virtual Filesystem (VFS) or external bundle loaded over HTTP**
  - *Pros*: Headers can be updated without recompiling the compiler binary.
  - *Cons*: Requires network round-trips, asynchronous preloading in web workers, and complex filesystem shims.
- **Option 2: Embed headers as static string slices via `include_str!`**
  - *Pros*: Zero filesystem or network dependencies at runtime; headers are always available instantly; compiler binary is completely self-contained.
  - *Cons*: Headers must be authored and maintained directly within the repository; slightly increases compiler binary size.

## Decision
Embed the mini standard library headers (`stdlib/`) directly into the compiler binary using Rust's `include_str!` macro (`docs/decisions/004-embedded-stdlib.md`, commit `d23afed`, `src/preprocessor/mod.rs#L34-L50`). The compiler's `IncludeProvider` resolves standard include directives (e.g. `<iostream>`, `<string>`, `<cstdio>`) from these embedded string slices without touching the filesystem.

## Consequences
### Positive
- The mini standard library is available without host filesystem access in browser Web Workers (`docs/decisions/004-embedded-stdlib.md`, commit `d23afed`, `src/preprocessor/mod.rs#L34-L50`).
- Headers are independently checked against real C++ compilers (`g++` and `clang++` with `-nostdinc++`) via `tools/verify_mini_stl_gxx.py` and `tests/oracle_gate.rs` (`docs/decisions/004-embedded-stdlib.md`, commit `d23afed`, `tests/oracle_gate.rs`).
- Deterministic header compilation behavior across both native CLI and WebAssembly-hosted builds (`src/lib.rs#L40-L48`).

### Negative / Trade-offs
- Standard library modifications require recompiling the compiler crate (`src/preprocessor/mod.rs`).
- Only headers written to comply with the supported C++17 subset can be included (`src/parser/unsupported.rs`).
