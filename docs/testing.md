# Testing

Positive corpus tests contain `.cpp` and `.expected` files; optional `.input` supplies stdin. Runtime-trap
groups assert a nonzero Wasm trap. Negative tests contain `.expected_err` diagnostics. The runner discovers
tests by directory and must report on-disk, executed, passed, and failed counts for every group.

The current dirty-tree baseline is 214 tests, 214 executed, 195 passed, and 19 expected failures: six template
tests and thirteen container/algorithm tests. Reproduce it with:

```sh
cargo build --release
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

Before a behavior-preserving refactor, run the checked-in golden baseline:

```sh
cargo build --release
cargo run --release --bin cppwasm-testrunner -- --check-golden tests/golden.txt target/release/cppwasm-cli
```

`tests/golden.txt` records the SHA-256 of the emitted Wasm module and of stdout for every positive test that
compiles. A refactor must leave every entry unchanged. Regenerate the manifest deliberately with `--golden`
when behavior changes; tests that do not compile are absent, so a test that starts compiling also appears as a
mismatch.

When the tools are available, run the native oracle and the embedded-header gate:

```sh
cargo run --release --bin cppwasm-testrunner -- --oracle
python3 tools/verify_mini_stl_gxx.py
```

The oracle replays every positive test through g++ and clang++ and compares native output with the recorded
`.expected` files. The header gate compiles the positive corpus against `stdlib/` with `-nostdinc++`, Sanitizer
instrumentation, and both compilers; the allocator-specific `free_list_reuse` outputs are reported as skips.
Emitted modules should also be validated with a WebAssembly validator when one is installed.

Held-out tests must differ in logic, names, or values; padding-only copies are not evidence of independent
coverage.
