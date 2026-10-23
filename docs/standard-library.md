# Embedded standard library

Headers under `stdlib/` are embedded into the core with `include_str!`. A header is *present* when the file
exists and *supported* when its surface is covered by the corpus: streams, strings, C stdio, math, formatting,
allocation, and the small supporting headers are supported. The `vector`, `map`, and `set` headers are present
but partial: they compile the subset exercised by the mini-STL fixtures and are not a supported container
surface, because templates are outside the subset. Every header is compiled by this compiler and checked with
real g++/clang++ using `-nostdinc++`.

The I/O contract uses shared lookahead between C and C++ input. Mixing C and C++ input after
`sync_with_stdio(false)` is outside the contract. Stream `unget()` is not supported; input is consumed through
the shared lookahead. Output formatting for floating values is implemented by the host interface.
