# ADR 0008: String Representation and Interning

## Status
Accepted (reconstructed) <!-- TODO: confirm with author -->

## Context
Strings in C++ exist both as static string literals (`const char*`) and dynamically resizable objects (`std::string`). The compiler requires a deterministic layout for string literals in the WebAssembly data section and a memory-safe structure for `std::string` in the embedded standard library.

## Options Considered
- **Option 1: Duplicate string literals per occurrence**
  - *Pros*: Simple code generation.
  - *Cons*: Bloats WebAssembly data section and linear memory footprint.
- **Option 2: Intern string literals into the static data section and represent `std::string` via dynamic heap buffers**
  - *Pros*: Minimizes data section footprint; deduplicates identical literals; provides standard null-terminated `const char*` interoperability.
  - *Cons*: Requires interning bookkeeping during compilation.

## Decision
1. **String Literals (`const char*`)**:
   - String literals are deduplicated and interned into `Codegen::string_table` ([`src/codegen/strings.rs`](../../src/codegen/strings.rs)).
   - Interned strings are placed consecutively with trailing null terminators (`\0`) into the WebAssembly data section starting at `memory_layout::DATA_BASE`.
2. **Standard String (`std::string`)**:
   - Defined in [`stdlib/string`](../../stdlib/string) using three fields: `char* _data`, `size_t _len`, and `size_t _cap`.
   - Dynamic expansions use `_ensure_cap` with geometric buffer doubling backed by `malloc` and `free`.
   - Small String Optimization (SSO) is omitted in v1 in favor of predictable buffer layouts.

## Consequences
### Positive
- Identical string literals share single data section addresses.
- `std::string::c_str()` provides a zero-overhead pointer compatible with C standard library functions like `printf` and `puts`.
- Destructors cleanly release heap memory on scope exit via RAII.

### Negative / Trade-offs
- Without SSO, small strings always perform a heap allocation.
