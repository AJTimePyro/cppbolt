# Testing

The runner discovers tests by directory and reports on-disk, executed, passed, and failed counts for every
group. Positive tests contain `.cpp` and `.expected` files; optional `.input` supplies stdin. Runtime-trap
groups assert a nonzero Wasm trap. Negative tests contain `.expected_err` diagnostics.

The current baseline is 218 tests, 218 executed, 199 passed, and 19 expected failures: six template tests and
thirteen container/algorithm tests. Reproduce it with:

```sh
cargo build --release
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

## Test categories

### Main tests

`tests/<feature>` contains tests for supported behavior. Most are hand-maintained; large parameter matrices are
generated. Every supported language or library feature must have a positive test, and every new compiler
behavior should extend this corpus.

### Variants

`tests/<feature>_variant` contains a counterpart for the tests in the matching main group. Variants guard
against overfitting to the main corpus and must differ in logic, structure, names, or values; padding-only
copies are not evidence of independent coverage. The runner enforces three outcomes for each same-named pair:

- Identical sources, or sources that differ only in whitespace or comments, **fail** the gate.
- Sources that differ only in identifiers or literals **warn**.
- Any other difference passes silently.

Some variants are generated rather than hand-written. For example, `tools/generate_lvalue_matrix.py` emits the
large `lvalue_matrix` pair; see [Generated differential tests](#generated-differential-tests).

### Independent tests

`tests/independent` is authored without access to the compiler source. Authors work from the C++17
specification and public compiler behavior, not from `src/`, the main corpus, or the variant corpus. Expected
output is produced by `g++ -std=c++17 -O0` and verified with `clang++ -std=c++17 -O0`. The directory uses the
same `.cpp`, `.expected`, and optional `.input` layout as other positive groups.

After the directory is established, the implementing agent must never edit or delete files in it. Only the
repository owner may approve additions or corrections.

`tests/PASSING.txt` is the pass manifest. Independent tests that pass must be listed there. An independent test
that is absent from the manifest is an expected failure; record each one with a one-line reason under
"Expected failures" in `tests/independent/README.md`. Do not list expected failures in `PASSING.txt`.

`tests/independent/MANIFEST.sha256` pins every file in the directory except itself. The runner checks it before
any mode and fails when a listed file changes or disappears, or when an unlisted file appears. This makes every
edit an explicit manifest change that the owner must approve.

To add an independent batch:

1. Author the sources outside the repository without reading the compiler implementation, main tests, or
   variant tests. Keep the tests within the documented C++ subset if they should pass.
2. Compile each source with both `g++ -std=c++17 -O0` and `clang++ -std=c++17 -O0`, confirm that their stdout
   matches, and store that output as `<name>.expected`. Add `<name>.input` when stdin is required.
3. Copy the batch into `tests/independent` and run the corpus. Passing tests should be added to
   `tests/PASSING.txt`; expected failures should be documented in `tests/independent/README.md` instead.
4. After owner review, regenerate the manifest and commit it with the batch:

```sh
cd tests/independent
find . -type f ! -name MANIFEST.sha256 -printf '%P\0' | sort -z | xargs -0 sha256sum > MANIFEST.sha256
```

Independent tests are ordinary positive tests for the runner and golden check. They are included in the native
oracle because valid C++17 should compile with the host compiler even when this compiler has an expected
failure.

### Generated differential tests

Generated differential tests are produced mechanically from a parameter matrix rather than hand-written. They
are useful when a feature has many type, lvalue, or operator combinations that are impractical to enumerate by
hand. The checked-in `.expected` output is generated with `g++ -std=c++17 -O0` and must also match
`clang++ -std=c++17 -O0` before it is committed.

These tests are not independent: the generator, the matrix, and the expected outputs are visible while the
compiler is implemented. They complement variants but do not replace independent coverage. Regenerate them with
the owning tool, review the full diff, and refresh the golden baseline only when the behavior change is
intentional.

The runner can also replay a group through the native reference path:

```sh
cargo run --bin cppwasm-testrunner -- c_basics ref
```

Reference mode compiles each positive test with `g++ -std=c++17 -O0` and compares its output with the checked-in
`.expected` file.

## Golden and native gates

Before a behavior-preserving refactor, run the checked-in golden baseline:

```sh
cargo build --release
cargo run --release --bin cppwasm-testrunner -- --check-golden tests/golden.txt target/release/cppwasm-cli
```

`tests/golden.txt` records the SHA-256 of the emitted Wasm module and of stdout for every positive test that
compiles. A refactor must leave every entry unchanged. Regenerate the manifest deliberately with `--golden`
when behavior changes; tests that do not compile are absent, so a test that starts compiling also appears as a
mismatch. Variant and independent tests are included under their directory names.

When the tools are available, run the native oracle and the embedded-header gate:

```sh
cargo run --release --bin cppwasm-testrunner -- --oracle
python3 tools/verify_mini_stl_gxx.py
```

The oracle replays every positive test through g++ and clang++ and compares native output with the recorded
`.expected` files. The header gate compiles the compiler corpus against `stdlib/` with `-nostdinc++`, Sanitizer
instrumentation, and both compilers; the allocator-specific `free_list_reuse` outputs are reported as skips.
Emitted modules should also be validated with a WebAssembly validator when one is installed.
