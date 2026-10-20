# Known limitations

The compiler is a correctness-focused subset implementation and does not optimize generated code. Templates,
standard containers and algorithms, exceptions, RTTI, lambdas, threads, SFINAE, concepts, rvalue references,
friends, and virtual inheritance remain intentionally out of scope. Virtual base specifiers are rejected with
`virtual inheritance is not supported`. Multiple inheritance is only partially supported: data layout and
constructors work, but non-static method lookup follows the first base class.

Source-map fidelity through nested macro expansion is incomplete. The code generator is still being decomposed
and its frame/scratch allocation is conservative. The current shell harness remains a migration point toward a
typed Rust test harness.

The parser, semantic analyzer, code generator, and preprocessor share a single depth limit of 2000 AST or
expression levels (`src/depth.rs`). Wide left-associative operator chains, `else-if` chains, and string
concatenations are walked iteratively and do not consume depth per term: 20,000-term integer sums, 4,000 string
concatenations, and 5,000-branch `else-if` chains compile. Recursive nesting beyond the limit is rejected with a
normal diagnostic instead of exhausting the native or WebAssembly stack.

The limit was measured on the WebAssembly-hosted compiler under Node/V8. The wasm module is linked with an
8 MiB stack (`.cargo/config.toml`); the native CLI runs the compiler on a 64 MiB thread (`src/main.rs`). With
those stacks, 1000-deep nested calls, ternaries, namespaces, and blocks compile, while 5000-deep versions are
rejected at the 2000 limit before the V8 call stack is exhausted (observed around 4000 recursive frames in the
wasm build). Include depth is separately limited to 128 (129 is rejected).

The `printf` runtime implements `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%f`, and `%g` with the
`0`, `-`, `+`, space, and `#` flags. Field width and precision are honoured for the numeric conversions; `%c`
and `%s` print without field width or precision, floating conversions ignore the sign flags, and `%p` and `%n`
are not implemented.
