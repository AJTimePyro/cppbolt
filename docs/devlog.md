# Development Log

Ongoing chronological record of major milestones, engineering refactors, and architectural evolutions in `cppwasm`.

---

## 2026-10-27: Comprehensive Documentation Architecture Restructure

### Context & Motivation
As the compiler matured through multiple refactoring phases—flattening from a multi-crate workspace into a single Cargo package, implementing the C++17 subset, adding the browser playground, and enforcing strict census invariants—the project documentation remained split across legacy top-level notes, 4-line decision stubs, and partial design drafts. Furthermore, discrepancies had accumulated (e.g. outdated descriptions of supported operator overloads versus the reality of `src/parser/unsupported.rs`).

### Work Executed
1. **Repository Layout Migration**:
   - Reorganized documentation into an mdBook hierarchy (`docs/book/`) separated into User Guides (`user/`) and Internals Deep Dives (`internals/`).
   - Moved architecture decision records into `docs/adr/`, formatted to a standardized template, and expanded reconstructed ADRs covering memory layout, free-list allocation, string representation, and C ABI interop.
   - Migrated testing documentation to `docs/book/internals/testing.md` and consolidated development workflows into `docs/contributing.md`.
   - Seeded `CHANGELOG.md` from the full 98-commit git history under `[Unreleased]` development milestones.
   - Promoted `ROADMAP.md` to the repository root.
2. **Technical Alignment & Discrepancy Resolution**:
   - Clarified that the compiler pipeline lowers the typed AST directly to WebAssembly bytecode without an intermediate representation (IR) or bytecode optimizer pass.
   - Documented the preprocessor as a full first-class compilation stage handling macro hide sets, recursive expansion, origin mapping, and constant condition evaluation.
   - Audited every claim in language support against the test suite and source code, correcting documentation around operator overloading (clarifying that while primitive pointer `->`, numeric postfix `++`/`--`, and primitive dereference `*` are supported, operator overloads for `->`, `()`, `*()`, and postfix `++`/`--` on class types are explicitly rejected).
3. **In-Code Documentation & Tooling**:
   - Added `#![warn(missing_docs)]` to `src/lib.rs` and documented public pipeline entry points and core data structures.
   - Updated GitHub Actions workflow (`.github/workflows/pages.yml`) to build mdBook, `cargo doc`, and the web playground simultaneously.
