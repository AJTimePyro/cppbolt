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
mod expr_lambda;
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
    pub namespace_aliases: HashMap<String, String>,
    pub known_namespaces: HashSet<String>,
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
        type_aliases.insert("nullptr_t".to_string(), Type::Nullptr);
        type_aliases.insert("std::nullptr_t".to_string(), Type::Nullptr);
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
            namespace_aliases: HashMap::new(),
            known_namespaces: HashSet::new(),
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
            if self.peek() == &TokenKind::LBrace {
                args.push(self.parse_initializer()?);
            } else {
                args.push(self.parse_expr()?);
            }
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    /// Parses a braced expression list `{ ... }` and returns the elements and position.
    pub(super) fn parse_brace_expr_list(&mut self) -> Result<(Vec<Expr>, ExprPos), CompileError> {
        let pos = self.current_pos();
        self.expect(TokenKind::LBrace)?;
        let mut list = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            list.push(self.parse_initializer()?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok((list, pos))
    }

    /// Resolves namespace aliases in `name` to the canonical target.
    pub fn resolve_namespace_alias(&self, name: &str) -> String {
        let mut current = name.to_string();
        for _ in 0..16 {
            let mut matched = false;
            let parts: Vec<&str> = current.split("::").collect();
            for i in (1..parts.len()).rev() {
                let prefix = parts[..i].join("::");
                if let Some(target) = self.lookup_parser_namespace_alias(&prefix) {
                    let suffix = parts[i..].join("::");
                    current = format!("{}::{}", target, suffix);
                    matched = true;
                    break;
                }
            }
            if !matched && let Some(target) = self.lookup_parser_namespace_alias(&current) {
                current = target.clone();
                matched = true;
            }
            if !matched {
                break;
            }
        }
        current
    }

    fn lookup_parser_namespace_alias(&self, name: &str) -> Option<&String> {
        if let Some(target) = self.namespace_aliases.get(name) {
            return Some(target);
        }
        if !self.current_namespace.is_empty() {
            let qualified = format!("{}::{}", self.current_namespace.join("::"), name);
            if let Some(target) = self.namespace_aliases.get(&qualified) {
                return Some(target);
            }
        }
        None
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
        Ok(Program {
            declarations,
            type_aliases: self.type_aliases.clone(),
        })
    }
}

#[cfg(test)]
mod tests;
