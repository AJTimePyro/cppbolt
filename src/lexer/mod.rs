//! Hand-written lexer for the supported C++ subset.
//!
//! See the mdBook guide at `docs/book/internals/lexer.md`.

mod literals;
pub mod token;

pub use token::{Token, TokenKind};

/// Decodes one backslash escape from a string or character literal.
pub(crate) fn decode_escape(c: char) -> char {
    match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '0' => '\0',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        other => other,
    }
}

use crate::diagnostics::CompileError;
use crate::source::{FileId, Span};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    file_id: FileId,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self::new_with_file(input, FileId(0))
    }

    pub fn new_with_file(input: &str, file_id: FileId) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            file_id,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            let start_line = self.line;
            let start_col = self.col;
            let start_offset = self.pos as u32;

            if self.skip_trivia(c, start_line, start_col)? {
                continue;
            }

            if c == '"' {
                let value = self.read_string(start_line, start_col)?;
                self.push_token(
                    &mut tokens,
                    TokenKind::StringLiteral(value),
                    start_line,
                    start_col,
                    start_offset,
                );
                continue;
            }

            if c == '\'' {
                let value = self.read_char_literal(start_line, start_col)?;
                self.push_token(
                    &mut tokens,
                    TokenKind::CharLiteral(value),
                    start_line,
                    start_col,
                    start_offset,
                );
                continue;
            }

            if c.is_ascii_digit()
                || (c == '.' && self.peek_next().is_some_and(|next| next.is_ascii_digit()))
            {
                let kind = self.read_number(start_line, start_col)?.kind;
                self.push_token(&mut tokens, kind, start_line, start_col, start_offset);
                continue;
            }

            if c.is_alphabetic() || c == '_' {
                let ident = self.read_ident();
                let kind = classify_word(&ident);
                self.push_token(&mut tokens, kind, start_line, start_col, start_offset);
                continue;
            }

            self.advance();
            let kind = self.scan_operator(c, start_line, start_col)?;
            self.push_token(&mut tokens, kind, start_line, start_col, start_offset);
        }

        let mut eof = Token::new(TokenKind::Eof, self.line, self.col);
        eof.span = Span::new(self.file_id, self.line as u32, self.col as u32, 0);
        tokens.push(eof);
        Ok(tokens)
    }

    fn skip_trivia(
        &mut self,
        c: char,
        start_line: usize,
        start_col: usize,
    ) -> Result<bool, CompileError> {
        if c.is_whitespace() {
            self.advance();
            return Ok(true);
        }
        if c == '/' && self.peek_next() == Some('/') {
            self.advance();
            self.advance();
            while let Some(ch) = self.peek() {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
            return Ok(true);
        }
        if c == '/' && self.peek_next() == Some('*') {
            self.advance();
            self.advance();
            let mut closed = false;
            while let Some(ch) = self.advance() {
                if ch == '*' && self.peek() == Some('/') {
                    self.advance();
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(CompileError::new(
                    "Unterminated block comment",
                    start_line,
                    start_col,
                ));
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn push_token(
        &self,
        tokens: &mut Vec<Token>,
        kind: TokenKind,
        line: usize,
        col: usize,
        start_offset: u32,
    ) {
        let mut token = Token::new(kind, line, col);
        token.span = Span::new(
            self.file_id,
            line as u32,
            col as u32,
            self.pos as u32 - start_offset,
        );
        tokens.push(token);
    }

    fn read_ident(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        text
    }

    fn scan_operator(
        &mut self,
        first: char,
        line: usize,
        col: usize,
    ) -> Result<TokenKind, CompileError> {
        let kind = match first {
            '+' => self.one_of(
                &[('+', TokenKind::PlusPlus), ('=', TokenKind::PlusAssign)],
                TokenKind::Plus,
            ),
            '-' => self.one_of(
                &[
                    ('-', TokenKind::MinusMinus),
                    ('>', TokenKind::Arrow),
                    ('=', TokenKind::MinusAssign),
                ],
                TokenKind::Minus,
            ),
            '*' => self.one_of(&[('=', TokenKind::StarAssign)], TokenKind::Star),
            '/' => self.one_of(&[('=', TokenKind::SlashAssign)], TokenKind::Slash),
            '%' => self.one_of(&[('=', TokenKind::PercentAssign)], TokenKind::Percent),
            '&' => self.one_of(
                &[('&', TokenKind::AndAnd), ('=', TokenKind::AmpAssign)],
                TokenKind::Amp,
            ),
            '|' => self.one_of(
                &[('|', TokenKind::OrOr), ('=', TokenKind::PipeAssign)],
                TokenKind::Pipe,
            ),
            '^' => self.one_of(&[('=', TokenKind::CaretAssign)], TokenKind::Caret),
            '~' => TokenKind::Tilde,
            '!' => self.one_of(&[('=', TokenKind::BangEq)], TokenKind::Bang),
            '=' => self.one_of(&[('=', TokenKind::EqEq)], TokenKind::Assign),
            '<' => self.scan_angle(false),
            '>' => self.scan_angle(true),
            ':' => self.one_of(&[(':', TokenKind::ColonColon)], TokenKind::Colon),
            '?' => TokenKind::Question,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            unknown => {
                return Err(CompileError::new(
                    format!("Unexpected character '{}'", unknown),
                    line,
                    col,
                ));
            }
        };
        Ok(kind)
    }

    fn one_of(&mut self, options: &[(char, TokenKind)], fallback: TokenKind) -> TokenKind {
        for (expected, kind) in options {
            if self.peek() == Some(*expected) {
                self.advance();
                return kind.clone();
            }
        }
        fallback
    }

    fn scan_angle(&mut self, right: bool) -> TokenKind {
        let (single, double, assign) = if right {
            (TokenKind::Gt, TokenKind::Shr, TokenKind::ShrAssign)
        } else {
            (TokenKind::Lt, TokenKind::Shl, TokenKind::ShlAssign)
        };
        let doubled = if right { '>' } else { '<' };
        if self.peek() == Some(doubled) {
            self.advance();
            if self.peek() == Some('=') {
                self.advance();
                return assign;
            }
            return double;
        }
        if self.peek() == Some('=') {
            self.advance();
            return if right {
                TokenKind::GtEq
            } else {
                TokenKind::LtEq
            };
        }
        single
    }
}

fn classify_word(word: &str) -> TokenKind {
    match word {
        "int" => TokenKind::KwInt,
        "long" => TokenKind::KwLong,
        "char" => TokenKind::KwChar,
        "bool" => TokenKind::KwBool,
        "float" => TokenKind::KwFloat,
        "double" => TokenKind::KwDouble,
        "unsigned" => TokenKind::KwUnsigned,
        "signed" => TokenKind::KwSigned,
        "short" => TokenKind::KwShort,
        "void" => TokenKind::KwVoid,
        "auto" => TokenKind::KwAuto,
        "const" => TokenKind::KwConst,
        "static" => TokenKind::KwStatic,
        "if" => TokenKind::KwIf,
        "else" => TokenKind::KwElse,
        "for" => TokenKind::KwFor,
        "while" => TokenKind::KwWhile,
        "do" => TokenKind::KwDo,
        "switch" => TokenKind::KwSwitch,
        "case" => TokenKind::KwCase,
        "default" => TokenKind::KwDefault,
        "break" => TokenKind::KwBreak,
        "continue" => TokenKind::KwContinue,
        "return" => TokenKind::KwReturn,
        "sizeof" => TokenKind::KwSizeof,
        "enum" => TokenKind::KwEnum,
        "struct" => TokenKind::KwStruct,
        "class" => TokenKind::KwClass,
        "public" => TokenKind::KwPublic,
        "private" => TokenKind::KwPrivate,
        "protected" => TokenKind::KwProtected,
        "virtual" => TokenKind::KwVirtual,
        "inline" => TokenKind::KwInline,
        "extern" => TokenKind::KwExtern,
        "template" => TokenKind::KwTemplate,
        "typename" => TokenKind::KwTypename,
        "new" => TokenKind::KwNew,
        "delete" => TokenKind::KwDelete,
        "this" => TokenKind::KwThis,
        "operator" => TokenKind::KwOperator,
        "override" => TokenKind::KwOverride,
        "namespace" => TokenKind::KwNamespace,
        "using" => TokenKind::KwUsing,
        "typedef" => TokenKind::KwTypedef,
        "nullptr" => TokenKind::KwNullptr,
        "static_cast" => TokenKind::KwStaticCast,
        "constexpr" => TokenKind::KwConstexpr,
        "decltype" => TokenKind::KwDecltype,
        "explicit" => TokenKind::KwExplicit,
        "friend" => TokenKind::KwFriend,
        "throw" => TokenKind::KwThrow,
        "true" => TokenKind::BoolLiteral(true),
        "false" => TokenKind::BoolLiteral(false),
        _ => TokenKind::Ident(word.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::lexer::TokenKind;
    use crate::source::FileId;

    #[test]
    fn tokens_carry_line_column_and_length() -> Result<(), Box<dyn std::error::Error>> {
        let mut lexer = Lexer::new_with_file("int x = 42;\nx++;", FileId(7));
        let tokens = lexer.tokenize()?;
        let spans: Vec<_> = tokens.iter().map(|token| token.span).collect();

        assert_eq!(
            (spans[0].file, spans[0].line, spans[0].col, spans[0].len),
            (FileId(7), 1, 1, 3)
        );
        assert_eq!((spans[3].line, spans[3].col, spans[3].len), (1, 9, 2));
        assert_eq!((spans[5].line, spans[5].col, spans[5].len), (2, 1, 1));
        assert_eq!((spans[6].line, spans[6].col, spans[6].len), (2, 2, 2));
        assert_eq!(tokens[6].kind, TokenKind::PlusPlus);
        Ok(())
    }
}
