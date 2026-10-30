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
        if self.peek() == Some('0') && matches!(self.peek_next(), Some('b') | Some('B')) {
            return self.read_binary_number(start_line, start_col);
        }
        if self.peek() == Some('0') && !self.is_float_number_ahead() {
            return self.read_octal_number(start_line, start_col);
        }
        self.read_decimal_or_float(start_line, start_col)
    }

    fn is_float_number_ahead(&self) -> bool {
        let mut idx = self.pos;
        while idx < self.chars.len() {
            let c = self.chars[idx];
            if c.is_ascii_digit() || c == '\'' {
                idx += 1;
            } else {
                return c == '.' || c == 'e' || c == 'E';
            }
        }
        false
    }

    fn read_integer_suffixes(&mut self, text: &mut String) {
        while let Some(c) = self.peek() {
            if matches!(c, 'u' | 'U' | 'l' | 'L') {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_radix_digits(
        &mut self,
        text: &mut String,
        digits: &mut String,
        name: &str,
        loc: (usize, usize),
        is_valid_digit: impl Fn(char) -> bool,
        check_invalid_digit: impl Fn(char) -> Option<CompileError>,
    ) -> Result<(), CompileError> {
        let (start_line, start_col) = loc;
        let mut last_was_sep = false;
        while let Some(c) = self.peek() {
            if is_valid_digit(c) {
                text.push(c);
                digits.push(c);
                self.advance();
                last_was_sep = false;
            } else if c == '\'' {
                if digits.is_empty() || last_was_sep {
                    return Err(CompileError::new(
                        format!("Invalid digit separator in {name} literal"),
                        start_line,
                        start_col,
                    ));
                }
                text.push(c);
                self.advance();
                last_was_sep = true;
            } else if let Some(err) = check_invalid_digit(c) {
                return Err(err);
            } else {
                break;
            }
        }
        if last_was_sep {
            return Err(CompileError::new(
                format!("Digit separator cannot appear at end of {name} literal"),
                start_line,
                start_col,
            ));
        }
        if digits.is_empty() {
            return Err(CompileError::new(
                format!("Invalid {name} literal '{}'", text),
                start_line,
                start_col,
            ));
        }
        Ok(())
    }

    fn read_octal_number(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, CompileError> {
        let mut text = String::new();
        let mut digits = String::new();
        text.extend(self.advance());
        digits.push('0');
        self.read_radix_digits(
            &mut text,
            &mut digits,
            "octal",
            (start_line, start_col),
            |c| ('0'..='7').contains(&c),
            |c| {
                if c == '8' || c == '9' {
                    Some(CompileError::new(
                        format!("Invalid digit '{}' in octal literal", c),
                        start_line,
                        start_col,
                    ))
                } else {
                    None
                }
            },
        )?;
        self.read_integer_suffixes(&mut text);
        let value = i64::from_str_radix(&digits, 8).map_err(|_| {
            CompileError::new(
                format!("Invalid octal literal '{}'", text),
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

    fn read_binary_number(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, CompileError> {
        let mut text = String::new();
        text.extend(self.advance());
        text.extend(self.advance());
        let mut digits = String::new();
        self.read_radix_digits(
            &mut text,
            &mut digits,
            "binary",
            (start_line, start_col),
            |c| c == '0' || c == '1',
            |c| {
                if c.is_ascii_digit() {
                    Some(CompileError::new(
                        format!("Invalid digit '{}' in binary literal", c),
                        start_line,
                        start_col,
                    ))
                } else {
                    None
                }
            },
        )?;
        self.read_integer_suffixes(&mut text);
        let value = i64::from_str_radix(&digits, 2).map_err(|_| {
            CompileError::new(
                format!("Invalid binary literal '{}'", text),
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
        text.extend(self.advance());
        text.extend(self.advance());
        let mut digits = String::new();
        self.read_radix_digits(
            &mut text,
            &mut digits,
            "hex",
            (start_line, start_col),
            |c| c.is_ascii_hexdigit(),
            |_| None,
        )?;
        self.read_integer_suffixes(&mut text);
        let value = i64::from_str_radix(&digits, 16).map_err(|_| {
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

    fn read_decimal_or_float(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, CompileError> {
        let mut text = String::new();
        let mut digits_clean = String::new();
        let mut is_float = false;
        let mut last_was_sep = false;
        let mut has_digits = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                digits_clean.push(c);
                self.advance();
                last_was_sep = false;
                has_digits = true;
            } else if c == '\'' {
                if !has_digits || last_was_sep {
                    return Err(CompileError::new(
                        "Invalid digit separator",
                        start_line,
                        start_col,
                    ));
                }
                text.push(c);
                self.advance();
                last_was_sep = true;
            } else if c == '.' && !is_float {
                if last_was_sep {
                    return Err(CompileError::new(
                        "Digit separator cannot appear adjacent to decimal point",
                        start_line,
                        start_col,
                    ));
                }
                is_float = true;
                text.push(c);
                digits_clean.push(c);
                self.advance();
                has_digits = false;
                last_was_sep = false;
            } else if (c == 'e' || c == 'E') && has_digits {
                if last_was_sep {
                    return Err(CompileError::new(
                        "Digit separator cannot appear adjacent to exponent",
                        start_line,
                        start_col,
                    ));
                }
                is_float = true;
                text.push(c);
                digits_clean.push(c);
                self.advance();
                has_digits = false;
                last_was_sep = false;
                if let Some(sign) = self.peek()
                    && (sign == '+' || sign == '-')
                {
                    text.push(sign);
                    digits_clean.push(sign);
                    self.advance();
                }
            } else {
                break;
            }
        }

        if last_was_sep {
            return Err(CompileError::new(
                "Digit separator cannot appear at end of literal",
                start_line,
                start_col,
            ));
        }

        if is_float {
            if matches!(self.peek(), Some('f' | 'F' | 'l' | 'L')) {
                text.extend(self.advance());
            }
            let value: f64 = digits_clean.parse().map_err(|_| {
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

        self.read_integer_suffixes(&mut text);

        let value: i64 = digits_clean.parse().map_err(|_| {
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
}

#[cfg(test)]
mod tests {
    use super::super::Lexer;
    use crate::lexer::TokenKind;

    #[test]
    fn parses_octal_and_binary_and_digit_separators() -> Result<(), Box<dyn std::error::Error>> {
        let mut lexer = Lexer::new("0755 0b1010 1'000'000 0xDE'AD 0b11'00 42.125'5 1.0e1'0");
        let tokens = lexer.tokenize()?;

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(493, "0755".into()));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(10, "0b1010".into()));
        assert_eq!(
            tokens[2].kind,
            TokenKind::IntLiteral(1000000, "1'000'000".into())
        );
        assert_eq!(
            tokens[3].kind,
            TokenKind::IntLiteral(0xDEAD, "0xDE'AD".into())
        );
        assert_eq!(tokens[4].kind, TokenKind::IntLiteral(12, "0b11'00".into()));

        let TokenKind::FloatLiteral(v1, _) = tokens[5].kind else {
            return Err("expected float literal".into());
        };
        assert!((v1 - 42.1255).abs() < 1e-6);

        let TokenKind::FloatLiteral(v2, _) = tokens[6].kind else {
            return Err("expected float literal".into());
        };
        assert!((v2 - 1.0e10).abs() < 1e-3);

        Ok(())
    }

    #[test]
    fn rejects_invalid_octal_digits() {
        let mut lexer = Lexer::new("0789");
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn rejects_invalid_digit_separators() {
        let cases = [
            "1''0", "1'e0", "1.0e'1", "1.0e+'1", "1.0e-'1", "1.'0", "1'.0", "1.0'f", "0x'1",
            "0x1'", "0x1'u", "0x1''2", "0b'1", "0b1'", "0b1'u", "0b1''0", "0'", "0'u", "0''7",
            "100'", "100'u", "100'l",
        ];
        for case in cases {
            let mut lexer = Lexer::new(case);
            assert!(lexer.tokenize().is_err(), "expected error for '{}'", case);
        }
    }
}
