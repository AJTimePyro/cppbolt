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
