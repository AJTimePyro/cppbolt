//! Literal scanning for strings, characters, and numbers.

use super::{Lexer, Token, TokenKind, decode_escape};
use crate::diagnostics::CompileError;

impl Lexer {
    pub(super) fn read_string(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<String, CompileError> {
        self.advance();
        let mut text = String::new();

        while let Some(c) = self.advance() {
            if c == '"' {
                return Ok(text);
            }
            if c != '\\' {
                text.push(c);
                continue;
            }
            if let Some(escape) = self.advance() {
                text.push(decode_escape(escape));
            }
        }

        Err(CompileError::new(
            "Unterminated string literal",
            start_line,
            start_col,
        ))
    }

    pub(super) fn read_char_literal(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<char, CompileError> {
        self.advance();
        let value = match self.advance() {
            Some('\\') => match self.advance() {
                Some(escaped) => decode_escape(escaped),
                None => {
                    return Err(CompileError::new(
                        "Unterminated char literal",
                        start_line,
                        start_col,
                    ));
                }
            },
            Some(c) => c,
            None => {
                return Err(CompileError::new(
                    "Empty char literal",
                    start_line,
                    start_col,
                ));
            }
        };

        if self.advance() != Some('\'') {
            return Err(CompileError::new(
                "Multi-character char literal or missing closing quote",
                start_line,
                start_col,
            ));
        }

        Ok(value)
    }

    pub(super) fn read_number(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, CompileError> {
        if self.peek() == Some('0') && matches!(self.peek_next(), Some('x') | Some('X')) {
            return self.read_hex_number(start_line, start_col);
        }

        let mut text = String::new();
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                text.push(c);
                self.advance();
            } else if c == 'e' || c == 'E' {
                is_float = true;
                text.push(c);
                self.advance();
                if let Some(sign) = self.peek()
                    && (sign == '+' || sign == '-')
                {
                    text.push(sign);
                    self.advance();
                }
            } else {
                break;
            }
        }

        if is_float {
            if matches!(self.peek(), Some('f' | 'F')) {
                text.extend(self.advance());
            }
            let digits = text.trim_end_matches(['f', 'F']);
            let value: f64 = digits.parse().map_err(|_| {
                CompileError::new(
                    format!("Invalid float literal '{}'", text),
                    start_line,
                    start_col,
                )
            })?;
            return Ok(Token::new(
                TokenKind::FloatLiteral(value, text),
                start_line,
                start_col,
            ));
        }

        while let Some(c) = self.peek() {
            if matches!(c, 'u' | 'U' | 'l' | 'L') {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let digits = text.trim_end_matches(['u', 'U', 'l', 'L']);
        let value: i64 = digits.parse().map_err(|_| {
            CompileError::new(
                format!("Invalid integer literal '{}'", text),
                start_line,
                start_col,
            )
        })?;
        Ok(Token::new(
            TokenKind::IntLiteral(value, text),
            start_line,
            start_col,
        ))
    }

    fn read_hex_number(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, CompileError> {
        let mut text = String::new();
        // The caller has already verified the `0x`/`0X` prefix.
        text.extend(self.advance());
        text.extend(self.advance());
        while let Some(c) = self.peek() {
            if c.is_ascii_hexdigit() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        while let Some(c) = self.peek() {
            if matches!(c, 'u' | 'U' | 'l' | 'L') {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let digits = text[2..].trim_end_matches(['u', 'U', 'l', 'L']);
        let value = i64::from_str_radix(digits, 16).map_err(|_| {
            CompileError::new(
                format!("Invalid hex literal '{}'", text),
                start_line,
                start_col,
            )
        })?;
        Ok(Token::new(
            TokenKind::IntLiteral(value, text),
            start_line,
            start_col,
        ))
    }
}
