# Contributing

Use `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets` before
submitting changes. Run the corpus runner for compiler behavior changes. Keep the core independent of files,
environment, threads, and browser APIs.

Prefer typed enums and IDs over strings, `Result` over panics, and small modules with one responsibility.
Comments should explain invariants or ABI decisions, not narrate changes. Do not add unsupported language
features as part of cleanup work. New tests must prove behavior and variant tests must not be padding copies.
