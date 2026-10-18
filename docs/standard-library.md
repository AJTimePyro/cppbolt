# Embedded standard library

Headers under `stdlib/` are embedded into the core with `include_str!`. They are compiled by this compiler
and are also checked with real g++/clang++ using `-nostdinc++`. The supported headers are the files present in
that directory, with the primary v1 surface being streams, strings, C stdio, math, formatting, allocation, and
small supporting headers.

The I/O contract uses shared lookahead between C and C++ input. Mixing C and C++ input after
`sync_with_stdio(false)` is outside the contract. Output formatting for floating values is implemented by the
host interface.
