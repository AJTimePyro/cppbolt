# Summary

[Introduction](index.md)

# User Guide

- [Getting Started](user/getting-started.md)
- [Supported C++ Subset](user/supported-cpp.md)
- [Browser Integration](user/browser-integration.md)
- [Diagnostics & Errors](user/errors.md)

# Compiler Internals

- [Architecture Overview](internals/architecture.md)
- [Preprocessor](internals/preprocessor.md)
- [Lexer](internals/lexer.md)
- [Parser & AST](internals/parser-and-ast.md)
- [Semantic Analysis](internals/semantic-analysis.md)
- [WASM Codegen](internals/wasm-codegen.md)
- [Runtime & Standard Library](internals/runtime.md)
- [Testing System](internals/testing.md)

# Standard Library

- [`<type_traits>` (LLVM libc++ 23.1.5)](stdlib/type_traits.md)

# Architecture Decision Records

- [ADR Template](adr/template.md)
- [ADR 0001: Own Compiler](adr/0001-own-compiler.md)
- [ADR 0002: Direct Wasm Emission](adr/0002-direct-wasm.md)
- [ADR 0003: ILP32 ABI](adr/0003-ilp32.md)
- [ADR 0004: Embedded Standard Library](adr/0004-embedded-stdlib.md)
- [ADR 0005: No Exceptions or RTTI](adr/0005-no-exceptions-rtti.md)
- [ADR 0006: Linear Memory Layout](adr/0006-linear-memory-layout.md)
- [ADR 0007: Free-List Allocator](adr/0007-free-list-allocator.md)
- [ADR 0008: String Representation](adr/0008-string-representation.md)
- [ADR 0009: C ABI and JS Interop](adr/0009-c-abi-and-js-interop.md)
- [ADR 0010: Template Instantiation Model and Two-Phase Lookup](adr/0010-template-instantiation-model.md)
- [ADR 0011: Move Semantics, Value Categories, and Smart Pointers](adr/0011-move-semantics-and-value-categories.md)
- [ADR 0012: Template Specialization, Non-Type Parameters, and Alias Templates](adr/0012-template-specialization-nttp-aliases.md)
- [ADR 0013: C++11 Variadic Templates, Parameter Packs, and Partial Ordering](adr/0013-variadic-templates.md)
- [ADR 0014: C++17 Fold Expressions](adr/0014-cxx17-fold-expressions.md)
- [ADR 0015: Pointer-to-Member Operators and ABI](adr/0015-pointer-to-member.md)

# Development

- [Contributing Guide](contributing.md)
- [Development Log](devlog.md)
