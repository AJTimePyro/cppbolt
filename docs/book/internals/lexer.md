# Lexer Internals

The lexer scans preprocessed C++ source text into a linear stream of strongly typed tokens, implemented in [`src/lexer/`](https://github.com/cppwasm/cpp_web/blob/main/src/lexer/).

---

## 1. Role in the Pipeline

1. **Tokenization**: Classifies source text into keywords, identifiers, numeric literals (decimal, hexadecimal, octal, floating-point), string literals, character literals, and punctuation/operators.
2. **Escape Decoding**: Decodes standard C escape sequences in string and character literals (`\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, and hexadecimal/octal escapes) via [`src/lexer/literals.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/lexer/literals.rs).
3. **Span Attachment**: Measures the exact line, column, and character length of every token, producing a [`Span`](https://github.com/cppwasm/cpp_web/blob/main/src/source.rs) used throughout the parser and semantic analyzer for accurate error reporting.

---

## 2. Key Data Structures

- [`Lexer`](https://github.com/cppwasm/cpp_web/blob/main/src/lexer/mod.rs): Scans the input text slice maintaining current byte offset, line number, and column.
- [`Token`](https://github.com/cppwasm/cpp_web/blob/main/src/lexer/token.rs): The fundamental token unit emitted by the lexer:
  ```rust
  pub struct Token {
      pub kind: TokenKind,
      pub line: usize,
      pub col: usize,
      pub length: usize,
  }
  ```
- [`TokenKind`](https://github.com/cppwasm/cpp_web/blob/main/src/lexer/token.rs): Enumeration of all accepted lexical symbols:
  - **Literals**: `IntLiteral(i64, String)`, `FloatLiteral(f64, String)`, `CharLiteral(char)`, `StringLiteral(String)`, `BoolLiteral(bool)`
  - **Identifiers**: `Ident(String)`
  - **Keywords**: `KwInt`, `KwReturn`, `KwClass`, `KwVirtual`, `KwIf`, `KwWhile`, `KwConst`, etc.
  - **Operators & Punctuators**: `Plus`, `Minus`, `Star`, `Slash`, `Percent`, `AmpAmp`, `PipePipe`, `Arrow`, `ColonColon`, `LBrace`, `RBrace`, etc.
  - **Control**: `Eof`
- [`Span`](https://github.com/cppwasm/cpp_web/blob/main/src/source.rs): Represents source location ranges (`file: FileId`, `line: usize`, `col: usize`, `len: usize`).

---

## 3. Error Handling

The lexer produces `CompileError` with `ErrorKind::Lexical`:
- **Unterminated String or Character Literals**: Emits error if newline or EOF is encountered before the closing quote.
- **Invalid Number Formats**: Rejects malformed numeric literals (e.g. invalid exponent syntax or invalid octal digits).
- **Unrecognized Characters**: Emits an error pointing to the unknown character.

---

## 4. Real Input & Token Output Example

Taken from [`tests/c_basics/arithmetic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/arithmetic.cpp):

### Input C++ Source
```cpp
int main() {
    int a = 10 + 20;
    return a;
}
```

### Resulting Token Stream
```rust
Token { kind: TokenKind::KwInt, line: 1, col: 1, length: 3 }
Token { kind: TokenKind::Ident("main".into()), line: 1, col: 5, length: 4 }
Token { kind: TokenKind::LParen, line: 1, col: 9, length: 1 }
Token { kind: TokenKind::RParen, line: 1, col: 10, length: 1 }
Token { kind: TokenKind::LBrace, line: 1, col: 12, length: 1 }

Token { kind: TokenKind::KwInt, line: 2, col: 5, length: 3 }
Token { kind: TokenKind::Ident("a".into()), line: 2, col: 9, length: 1 }
Token { kind: TokenKind::Assign, line: 2, col: 11, length: 1 }
Token { kind: TokenKind::IntLiteral(10, "10".into()), line: 2, col: 13, length: 2 }
Token { kind: TokenKind::Plus, line: 2, col: 16, length: 1 }
Token { kind: TokenKind::IntLiteral(20, "20".into()), line: 2, col: 18, length: 2 }
Token { kind: TokenKind::Semi, line: 2, col: 20, length: 1 }

Token { kind: TokenKind::KwReturn, line: 3, col: 5, length: 6 }
Token { kind: TokenKind::Ident("a".into()), line: 3, col: 12, length: 1 }
Token { kind: TokenKind::Semi, line: 3, col: 13, length: 1 }

Token { kind: TokenKind::RBrace, line: 4, col: 1, length: 1 }
Token { kind: TokenKind::Eof, line: 4, col: 2, length: 0 }
```
