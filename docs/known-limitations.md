# Known limitations

The compiler is a correctness-focused subset implementation and does not optimize generated code. Templates,
standard containers and algorithms, exceptions, RTTI, lambdas, threads, SFINAE, concepts, rvalue references,
friends, and multiple/virtual inheritance remain intentionally out of scope.

Source-map fidelity through nested macro expansion is incomplete. The code generator is still being decomposed
and its frame/scratch allocation is conservative. The current shell harness remains a migration point toward a
typed Rust test harness.

The `printf` runtime implements `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%f`, and `%g` with the
`0`, `-`, `+`, space, and `#` flags. Field width and precision are honoured for the numeric conversions; `%c`
and `%s` print without field width or precision, floating conversions ignore the sign flags, and `%p` and `%n`
are not implemented.
