# Contributing Guide

Thank you for contributing to `cppbolt`! This document outlines development workflows, code standards, how to add a language feature end-to-end, and the required pull request checklist.

---

## 1. Development Environment & Build

Building the project requires a standard Rust toolchain (2024 edition).

```sh
# Build the native CLI compiler (target/release/cppwasm-cli)
cargo build --release

# Run Rust unit and integration tests
cargo test --all-targets

# Run formatting and linting gates
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

---

## 2. Running Tests & the Compiler

### Running the Corpus Runner
All C++ test suites (positive tests, negative diagnostics, and runtime traps) are managed by `cppwasm-testrunner`:

```sh
# Run all discovered test groups
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli

# Run a specific feature group (e.g. c_basics)
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli c_basics
```

For full details on the test categories (variants, independent tests, golden hashes, native oracle gates, and census metrics), see the dedicated [Testing System Guide](internals/testing.md).

### Compiling and Running a Sample
Compile a C++ source file using the native CLI:

```sh
# Compile C++ source to a WebAssembly binary
cargo run --release --bin cppwasm-cli -- path/to/sample.cpp -o output.wasm
```

You can run the resulting `.wasm` binary in Node.js using the test runner shim:

```sh
node tests/test_wasm_compiler.js output.wasm
```

Or test it in the browser playground:

```sh
cargo build --release --target wasm32-unknown-unknown --lib
tools/build_site.sh
python3 -m http.server --directory site 8000
```

---

## 3. Adding a New Language Feature End-to-End

When adding a new C++ language feature or expression form to `cppbolt`, modifications typically flow through each stage of the compiler pipeline:

1. **Tokens (`src/lexer/`)**:
   - If the feature introduces new keywords or operators, add variants to `src/lexer/token.rs` and update lexer scanning in `src/lexer/mod.rs`.
2. **Abstract Syntax Tree (`src/ast/`)**:
   - Define corresponding AST nodes in `src/ast/expr.rs`, `src/ast/stmt.rs`, `src/ast/decl.rs`, or `src/ast/types.rs`. Ensure nodes record line and column positions for diagnostic accuracy.
3. **Parser (`src/parser/`)**:
   - Add parsing logic in `src/parser/expr.rs`, `src/parser/stmt.rs`, or `src/parser/decl.rs`.
   - If lifting a restriction on a previously unsupported feature, update `src/parser/unsupported.rs`.
   - Track nesting limits using `self.depth` (`src/depth.rs`) to prevent stack overflow on pathological inputs.
4. **Semantic Analysis (`src/sema/`)**:
   - Implement type checking, name lookup, and conversions in `src/sema/analyze.rs`, `src/sema/expr.rs`, or `src/sema/lookup.rs`.
   - Synthesize necessary implicit copy/default constructors or record layouts.
5. **Code Generation (`src/codegen/`)**:
   - Lower the typed AST node to WebAssembly bytecodes in `src/codegen/expr.rs`, `src/codegen/stmt.rs`, or dedicated helpers.
   - Respect stack frame conventions in `src/codegen/function_context.rs` and memory layout rules in `src/runtime/memory_layout.rs`.
6. **Corpus Tests (`tests/`)**:
   - Add positive test cases (`.cpp` and `.expected`) in `tests/<feature_group>/`.
   - Add corresponding anti-overfitting structural tests in `tests/<feature_group>_variant/`.
   - If rejecting invalid syntax, add a negative test (`.cpp` and `.expected_err`) in `tests/negative/`.
   - Update `tests/PASSING.txt` and regenerate golden hashes if expected changes are approved:
     ```sh
     cargo run --release --bin cppwasm-testrunner -- --golden tests/golden.txt target/release/cppwasm-cli
     ```

---

## 4. Code Standards & Repository Rules

- **No Compiler Logic Modifications Without Tests**: Every supported behavior must have a positive corpus test. Every unsupported construct must produce a clean diagnostic, not a crash or panic.
- **Panic-Free Policy**: Never use `unwrap()`, `expect()`, or `panic!()` on code paths that can process user input. Use `Result<T, CompileError>`.
- **Function and File Size Limits**: Enforced by `src/bin/cppwasm-census.rs` and verified by `tests/census_fixture.rs`:
  - No function may exceed 80 lines.
  - No function may take more than 5 parameters (excluding `self`).
  - No file in `src/` may exceed 500 lines.
- **Module Tree Reachability**: Every Rust source file under `src/` must be wired into the crate module tree; verified by `tests/module_tree.rs`.
- **Preserve Documentation Integrity**: Comments should document invariants, ABI decisions, and memory models, rather than narrating line-by-line changes.

---

## 5. Pull Request Checklist

Before submitting a pull request, ensure all of the following items are complete:

- [ ] **Tests Pass**: `cargo test --all-targets` exits 0.
- [ ] **Corpus Runner Passes**: `cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli` reports 0 unexpected failures and 0 manifest violations.
- [ ] **Code Quality Gates Pass**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo run --release --bin cppwasm-census` reports 0 violations.
- [ ] **Feature Matrix Updated**: New features or changed statuses are reflected in [Supported C++](user/supported-cpp.md) with links to test files.
- [ ] **Documentation Updated**: Relevant mdBook chapters or doc comments (`///`, `//!`) updated.
- [ ] **ADR Created (if applicable)**: An Architecture Decision Record was added to [`docs/adr/`](adr/template.md) if an architectural, memory model, or ABI decision was made.
