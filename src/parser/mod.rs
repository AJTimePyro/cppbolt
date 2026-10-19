//! Recursive-descent parser for the supported C++ subset.

use crate::ast::types::Type;
use crate::ast::*;
use crate::diagnostics::{CompileError, ErrorKind};
use crate::lexer::{Token, TokenKind};
use crate::name::QualifiedName;
use std::collections::{HashMap, HashSet};

mod atom;
mod decl;
mod enums_operators;
mod expr;
mod prefix;
mod stmt;
mod structs;
mod types;
mod unsupported;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    known_types: HashSet<String>,
    known_enums: HashSet<String>,
    pub type_aliases: HashMap<String, Type>,
    pub current_namespace: Vec<String>,
    pub open_namespaces: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut known_types = HashSet::new();
        known_types.insert("void".to_string());
        known_types.insert("bool".to_string());
        known_types.insert("char".to_string());
        known_types.insert("int".to_string());
        known_types.insert("long".to_string());
        known_types.insert("short".to_string());
        known_types.insert("float".to_string());
        known_types.insert("double".to_string());
        known_types.insert("unsigned".to_string());
        known_types.insert("signed".to_string());
        known_types.insert("auto".to_string());
        known_types.insert("size_t".to_string());
        let known_enums = HashSet::new();
        let mut type_aliases = HashMap::new();
        type_aliases.insert("nullptr_t".to_string(), Type::Pointer(Box::new(Type::Void)));
        type_aliases.insert(
            "std::nullptr_t".to_string(),
            Type::Pointer(Box::new(Type::Void)),
        );
        type_aliases.insert("ptrdiff_t".to_string(), Type::Int);
        type_aliases.insert("std::ptrdiff_t".to_string(), Type::Int);
        type_aliases.insert("size_t".to_string(), Type::UnsignedInt);
        type_aliases.insert("std::size_t".to_string(), Type::UnsignedInt);

        known_types.insert("nullptr_t".to_string());
        known_types.insert("std::nullptr_t".to_string());
        known_types.insert("ptrdiff_t".to_string());
        known_types.insert("std::ptrdiff_t".to_string());
        known_types.insert("string".to_string());
        known_types.insert("std::string".to_string());
        known_types.insert("ostream".to_string());
        known_types.insert("std::ostream".to_string());
        known_types.insert("istream".to_string());
        known_types.insert("std::istream".to_string());
        known_types.insert("ios_base".to_string());
        known_types.insert("std::ios_base".to_string());

        Self {
            tokens,
            pos: 0,
            known_types,
            known_enums,
            type_aliases,
            current_namespace: Vec::new(),
            open_namespaces: Vec::new(),
        }
    }

    pub(super) fn current(&self) -> &Token {
        static EOF: Token = Token {
            kind: TokenKind::Eof,
            line: 0,
            col: 0,
            span: crate::source::Span::point(crate::source::FileId(0), 0, 0),
        };
        self.tokens.get(self.pos).unwrap_or(&EOF)
    }

    pub(super) fn peek(&self) -> &TokenKind {
        &self.current().kind
    }

    pub(super) fn peek_ahead(&self, n: usize) -> &TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    pub(super) fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[idx]
    }

    pub(super) fn expect(&mut self, kind: TokenKind) -> Result<(), CompileError> {
        if self.peek() == &kind {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::new(
                format!(
                    "Expected {}, found {}",
                    kind.diagnostic_name(),
                    self.peek().diagnostic_name()
                ),
                self.current().line,
                self.current().col,
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, CompileError> {
        let mut declarations = Vec::new();
        while self.peek() != &TokenKind::Eof {
            self.parse_declaration(&mut declarations)?;
        }
        Ok(Program { declarations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    pub(super) fn validates(source: &str) -> bool {
        let Ok(tokens) = Lexer::new(source).tokenize() else {
            return false;
        };
        Parser::new(tokens).validate_supported_features().is_ok()
    }

    #[test]
    pub(super) fn ignores_unsupported_text_in_comments_and_strings() {
        assert!(validates("// operator()\\nint main() { return 0; }"));
        assert!(validates(
            "int main() { const char* s = \"operator*()\"; return 0; }"
        ));
    }

    #[test]
    pub(super) fn accepts_ternary_colons_inside_for_loops() {
        assert!(validates(
            "int main() { int x = 0; for (int i = 0; i < 2; i = i ? 1 : 0) { x = i; } return x; }"
        ));
    }

    #[test]
    pub(super) fn multi_dimensional_array_dimensions_wrap_inside_out()
    -> Result<(), Box<dyn std::error::Error>> {
        let tokens = Lexer::new("int matrix[2][3];").tokenize()?;
        let program = Parser::new(tokens).parse_program()?;
        assert!(matches!(program.declarations[0], Decl::GlobalVar { .. }));
        let Decl::GlobalVar { ty, .. } = &program.declarations[0] else {
            return Ok(());
        };
        let expected = Type::Array(Box::new(Type::Array(Box::new(Type::Int), 3)), 2);
        assert_eq!(ty, &expected);
        Ok(())
    }
}
