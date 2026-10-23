# Architecture

The compiler pipeline is `preprocessor -> lexer -> parser -> semantic analysis -> direct Wasm emission`.
The native CLI owns files and processes; the library API accepts source text and returns either Wasm bytes or
a `CompileError`. The release compiler module built for `wasm32-unknown-unknown` is currently
845,976 bytes raw and 244,156 bytes with `gzip -9 -n`.  The module has grown +9.5 %
from 772,391 bytes at commit `45153ff`.
The core does not spawn processes, access the filesystem, or depend on a host runtime.

The project is a single Cargo package with one library and three binaries:

| Target | Responsibility |
|---|---|
| `src/lib.rs` | Preprocessor, lexer, parser, semantic analysis, code generation, and the Wasm encoder |
| `src/main.rs` (`cppwasm-cli`) | Native command-line front end, filesystem includes, and file output |
| `src/ffi.rs` | C ABI for embedding the compiler in another process or runtime |
| `src/bin/cppwasm-testrunner.rs` | Corpus execution and expected-output checks |
| `src/bin/cppwasm-census.rs` | Reports the size and parameter count of every Rust function and the size of every file |

Inside the library, the source, preprocessor, lexer, parser/AST, semantic analysis, code generation, runtime
emission, and Wasm encoding stages are separate modules. The library never touches the filesystem or spawns a
process; callers provide includes through `IncludeProvider`. New functionality should enter at the AST, semantic
model, code generator, runtime, and corpus-test layers together.
