# Architecture

The compiler pipeline is `preprocessor -> lexer -> parser -> semantic analysis -> direct Wasm emission`.
The native CLI owns files and processes; the library API accepts source text and returns either Wasm bytes or
a `CompileError`. The core does not spawn processes, access the filesystem, or depend on a host runtime.

The workspace is split into four crates:

| Crate | Responsibility |
|---|---|
| `cppwasm-core` | Preprocessor, lexer, parser, semantic analysis, code generation, and the Wasm encoder |
| `cppwasm-cli` | Native command-line front end, filesystem includes, and file output |
| `cppwasm-ffi` | C ABI for embedding the compiler in another process or runtime |
| `cppwasm-testrunner` | Corpus execution and expected-output checks |

Inside `cppwasm-core`, the source, preprocessor, lexer, parser/AST, semantic analysis, code generation, runtime
emission, and Wasm encoding stages are separate modules. The core never touches the filesystem or spawns a
process; callers provide includes through `IncludeProvider`. New functionality should enter at the AST, semantic
model, code generator, runtime, and corpus-test layers together.
