//! Recursive-descent parser for the supported C++ subset.
//!
//! See the mdBook guide at `docs/book/internals/parser-and-ast.md`.

use crate::ast::types::Type;
use crate::ast::*;
use crate::diagnostics::{CompileError, ErrorKind};
use crate::lexer::{Token, TokenKind};
use crate::name::QualifiedName;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

mod atom;
mod decl;
mod decl_func;
mod enums_operators;
mod expr;
mod prefix;
mod stmt;
mod structs;
mod types;
mod unsupported;

/// Decrements the parser's nesting counter when a recursive parse completes,
/// including on an early error return.
use crate::depth::{DepthCounters, DepthGuard};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    known_types: HashSet<String>,
    known_enums: HashSet<String>,
    pub type_aliases: HashMap<String, Type>,
    pub current_namespace: Vec<String>,
    pub open_namespaces: Vec<String>,
    nesting: DepthCounters,
    nesting_exceeded: Cell<bool>,
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
            nesting: DepthCounters::new(),
            nesting_exceeded: Cell::new(false),
        }
    }

    /// Reserves one level of the counter of one nesting construct.
    fn enter_nesting_of(
        &self,
        counter: &crate::depth::DepthCounter,
    ) -> Result<DepthGuard, CompileError> {
        match counter.enter(self.current().line, self.current().col) {
            Ok(guard) => Ok(guard),
            Err(error) => {
                self.nesting_exceeded.set(true);
                Err(error)
            }
        }
    }

    /// Reserves one level of the expression counter.
    fn enter_expression(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.expressions())
    }

    /// Reserves one level of the parenthesis counter.
    fn enter_parenthesis(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.parentheses())
    }

    /// Reserves one level of the call counter.
    fn enter_call(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.calls())
    }

    /// Reserves one level of the statement counter.
    fn enter_statement(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.statements())
    }

    /// Reserves one level of the block counter.
    fn enter_block(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.blocks())
    }

    /// Reserves one level of the declaration counter.
    fn enter_declaration(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.declarations())
    }

    /// Reserves one level of the type counter.
    fn enter_type(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.types())
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

    /// The position of the token the parser is looking at.
    pub(super) fn current_pos(&self) -> ExprPos {
        let token = self.current();
        ExprPos::new(token.line, token.col)
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

    /// Parses a comma-separated expression list followed by `)`, the argument list of a call, a `new` expression, a parenthesized initializer or a constructor initializer.
    pub(super) fn parse_paren_expr_list(&mut self) -> Result<Vec<Expr>, CompileError> {
        let mut args = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            args.push(self.parse_expr()?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    /// Prefixes `name` with the enclosing namespace when it is not already qualified.
    pub(super) fn qualify_name(&self, name: String) -> String {
        if self.current_namespace.is_empty() || name.contains("::") {
            name
        } else {
            format!("{}::{}", self.current_namespace.join("::"), name)
        }
    }

    /// Records a type alias under `name` and marks the name as known.
    pub(super) fn register_type_alias(&mut self, name: String, ty: Type) {
        self.type_aliases.insert(name.clone(), ty);
        self.known_types.insert(name);
    }

    /// Records a type alias declared inside a class under both the qualified `Class::alias` and the plain `alias` spelling.
    pub(super) fn register_member_alias(&mut self, owner: &str, alias: String, ty: Type) {
        let qualified = format!("{}::{}", owner, alias);
        self.type_aliases.insert(qualified.clone(), ty.clone());
        self.type_aliases.insert(alias.clone(), ty);
        self.known_types.insert(qualified);
        self.known_types.insert(alias);
    }

    /// Parses the `::`-separated tail of a qualified name, appending every
    /// segment to `id`.
    ///
    /// `error_pos` is the position reported when a segment is missing;
    /// `None` reports the position the parser stopped at.
    pub(super) fn parse_qualified_tail(
        &mut self,
        mut id: String,
        error_pos: Option<(usize, usize)>,
    ) -> Result<String, CompileError> {
        while self.peek() == &TokenKind::ColonColon {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(sub) => {
                    id = format!("{}::{}", id, sub);
                }
                _ => {
                    let (line, col) =
                        error_pos.unwrap_or_else(|| (self.current().line, self.current().col));
                    return Err(CompileError::new(
                        "Expected identifier after '::'",
                        line,
                        col,
                    ));
                }
            }
        }
        Ok(id)
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

    #[test]
    pub(super) fn parses_range_based_for_loop() -> Result<(), Box<dyn std::error::Error>> {
        let tokens = Lexer::new("void f() { for (int x : arr) { x; } }").tokenize()?;
        let program = Parser::new(tokens).parse_program()?;
        let Some(Decl::Function(func)) = program.declarations.first() else {
            return Err("expected function declaration".into());
        };
        let Some(body) = &func.body else {
            return Err("expected function with body".into());
        };
        let StmtKind::Compound(stmts) = &body.kind else {
            return Err("expected compound body".into());
        };
        assert!(matches!(
            stmts.first(),
            Some(Stmt {
                kind: StmtKind::RangeFor { .. },
                ..
            })
        ));
        Ok(())
    }
}
