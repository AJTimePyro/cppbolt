# Testing

Positive corpus tests contain `.cpp` and `.expected` files; optional `.input` supplies stdin. Runtime-trap
groups assert a nonzero Wasm trap. Negative tests contain `.expected_err` diagnostics. The runner discovers
tests by directory and must report on-disk, executed, passed, and failed counts for every group.

The current dirty-tree baseline is 202 tests, 202 executed, 183 passed, and 19 expected failures: six template
tests and thirteen container/algorithm tests. Reproduce it with:

```sh
cargo build --release
cargo run -p cppwasm-testrunner -- target/release/cppwasm-cli
```

Before refactoring, archive each compiling test's Wasm SHA-256 and stdout SHA-256 outside the tree. During
behavior-preserving phases compare both hashes. Positive output should also be compared against g++ and clang++,
and emitted modules should be validated with a WebAssembly validator when the tool is available.

Held-out tests must differ in logic, names, or values; padding-only copies are not evidence of independent
coverage.
