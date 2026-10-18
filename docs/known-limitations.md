# Known limitations

The compiler is a correctness-focused subset implementation and does not optimize generated code. Templates,
standard containers and algorithms, exceptions, RTTI, lambdas, threads, SFINAE, concepts, rvalue references,
friends, and multiple/virtual inheritance remain intentionally out of scope.

Source-map fidelity through nested macro expansion is incomplete. The code generator is still being decomposed
and its frame/scratch allocation is conservative. The current shell harness remains a migration point toward a
typed Rust test harness.
