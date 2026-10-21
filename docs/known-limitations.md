# Known limitations

The compiler is a correctness-focused subset implementation and does not optimize generated code. Templates,
standard containers and algorithms, exceptions, RTTI, lambdas, threads, SFINAE, concepts, rvalue references,
friends, and virtual inheritance remain intentionally out of scope. Virtual base specifiers are rejected with
`virtual inheritance is not supported`. Multiple inheritance is rejected at the second base specifier with
`multiple inheritance is not supported`; single inheritance and one base subobject remain supported.

Source-map fidelity through nested macro expansion is incomplete. The code generator is still being decomposed
and its frame/scratch allocation is conservative. The corpus and ABI gates are Rust binaries plus Node-hosted
JavaScript; there is no remaining shell test harness.

The parser, the semantic analyzer, the code generator and the preprocessor each keep one nesting counter *per
construct* (`src/depth.rs`), so one construct can no longer exhaust the budget of another: a parenthesis costs one
level of the parenthesis budget, a compound block costs one level of the block budget, and a nested call costs one
level of the call budget. The limits are:

| construct | limit | deepest accepted input (native and wasm) |
| --- | --- | --- |
| expressions | 4000 | a parenthesized conditional costs two expression levels |
| parentheses | 1600 | 1600 |
| call argument lists | 1500 | 1500 |
| statements | 1600 | 1599 nested `if` statements (the innermost statement is one level deeper) |
| compound blocks | 1600 | 1599 nested blocks (the enclosing block is one level deeper) |
| declarations | 1600 | - |
| type expressions | 1600 | - |
| preprocessor conditions | 1600 | - |

The limits were measured on the WebAssembly-hosted compiler under Node/V8, which is the binding host: the
WebAssembly build reaches 1600 nested parentheses, 1600 nested parenthesized conditionals, 1500 nested calls and
1599 nested `if` statements, and the native CLI accepts exactly the same depths. A deeper input is rejected with
the normal `Nesting depth exceeds the supported limit` diagnostic before the stack is exhausted, on both hosts.
The native CLI runs the compiler on a 64 MiB thread (`src/main.rs`) and the WebAssembly build is linked with an
8 MiB linear-memory stack (`.cargo/config.toml`); the recursion itself lives in the engine's shadow stack, so a
host that compiles many deep programs in one process has less room for the last one - the depth check therefore
runs before the stack is exhausted. Include depth is separately limited to 128 (129 is rejected).

The tests pin the same numbers as literals (`tests/nesting.rs` and `tests/wasm_stack_safety.rs`) and check that
the compiler agrees with them, so changing a limit fails the tests instead of silently moving the boundary.

Wide left-associative operator chains, `else-if` chains, and string concatenations are walked iteratively and do
not consume depth per term: 20,000-term integer sums, 4,000 string concatenations, and 5,000-branch `else-if`
chains compile.

The `printf` runtime implements `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, and
`%G`, including the `h`/`hh` length modifiers, with the `0`, `-`, `+`, space, and `#` flags. The `+` and space
flags are ignored for unsigned integer conversions. Zero padding falls back to spaces for infinities and NaN, and
`#` retains the decimal point and trailing zeros for floating conversions. Field width and precision are honoured
for the integer conversions, the floating conversions, and `%s`; `%c` honours field width and ignores precision.
A `*` precision argument is supported, but a `*` field width is not. `%p` and `%n` are not implemented.
