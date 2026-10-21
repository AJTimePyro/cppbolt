# Diagnostics

`CompileError` contains an `ErrorKind`, message, file, line, and column and implements `Display` and
`std::error::Error`. The display form is:

```text
file.cpp:line:column: error: message
```

The lexer attaches a `Span` (`FileId`, line, column, length) to every emitted token. The preprocessor records a
structured origin for every output line, so diagnostics from included headers report the header path and line
without embedding marker comments in the text. Diagnostics must report source locations from tokens or AST nodes
and must not expose Rust debug formatting.

Every expression node of the AST records the line and column of its first token, so a diagnostic about a
dereference, a member access, a call, a subscript, or an assignment points at the offending expression. Lowering
errors that cannot name an expression report the position of the statement being lowered. No diagnostic reports a
fabricated position: the corpus test `tests/diagnostic_positions.rs` compiles every corpus file and fails when a
diagnostic names a line that does not exist in the file it names, or when it reports the first line and column of a
file whose error is not really there.
