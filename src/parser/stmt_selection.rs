use super::*;

impl Parser {
    /// Parses the optional init-statement and condition of an `if` or `switch` selection statement.
    pub(super) fn parse_selection_init_and_cond(
        &mut self,
    ) -> Result<(Option<Box<Stmt>>, Expr), CompileError> {
        let _guard = self.enter_expression()?;
        if self.peek() == &TokenKind::Semi {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            return Ok((None, cond));
        }

        if self.is_type_start() {
            let saved_pos = self.pos;
            match self.parse_var_decl() {
                Ok(decl) => {
                    let cond = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    return Ok((Some(Box::new(decl)), cond));
                }
                Err(error) => {
                    if self.nesting_exceeded.get() {
                        return Err(error);
                    }
                    self.pos = saved_pos;
                }
            }
        }

        let first = self.parse_expr()?;
        if self.peek() == &TokenKind::Semi {
            let line = self.current().line;
            let col = self.current().col;
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            let init_stmt = Stmt::new(StmtKind::Expr(first), line, col);
            return Ok((Some(Box::new(init_stmt)), cond));
        }

        self.expect(TokenKind::RParen)?;
        Ok((None, first))
    }

    /// Parses an `if` statement, desugaring any C++17 init-statement into an enclosing compound block.
    pub(super) fn parse_if_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;
        let (first_init, cond) = self.parse_selection_init_and_cond()?;
        let then_stmt = Box::new(self.parse_stmt()?);
        let if_stmt = self.parse_if_branches(tok.line, tok.col, (cond, then_stmt))?;
        if let Some(init) = first_init {
            Ok(Stmt::new(
                StmtKind::Compound(vec![*init, if_stmt]),
                tok.line,
                tok.col,
            ))
        } else {
            Ok(if_stmt)
        }
    }

    /// Iteratively gathers `else if` and `else` branches to avoid stack recursion on long chains.
    fn parse_if_branches(
        &mut self,
        line: usize,
        col: usize,
        first_branch: (Expr, Box<Stmt>),
    ) -> Result<Stmt, CompileError> {
        let mut branches = vec![first_branch];
        let mut else_stmt = None;
        while self.peek() == &TokenKind::KwElse {
            self.advance();
            if self.peek() == &TokenKind::KwIf {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let (init, cond) = self.parse_selection_init_and_cond()?;
                let then_stmt = Box::new(self.parse_stmt()?);
                if let Some(init_s) = init {
                    let inner = self.parse_if_branches(line, col, (cond, then_stmt))?;
                    else_stmt = Some(Box::new(Stmt::new(
                        StmtKind::Compound(vec![*init_s, inner]),
                        line,
                        col,
                    )));
                    break;
                }
                branches.push((cond, then_stmt));
            } else {
                else_stmt = Some(Box::new(self.parse_stmt()?));
                break;
            }
        }
        Ok(Stmt::new(
            StmtKind::If {
                branches,
                else_stmt,
            },
            line,
            col,
        ))
    }

    /// Parses a `switch` statement, desugaring any C++17 init-statement into an enclosing compound block.
    pub(super) fn parse_switch_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;
        let (init, expr) = self.parse_selection_init_and_cond()?;
        let body = Box::new(self.parse_stmt()?);
        let switch_stmt = Stmt::new(StmtKind::Switch { expr, body }, tok.line, tok.col);
        if let Some(init_s) = init {
            Ok(Stmt::new(
                StmtKind::Compound(vec![*init_s, switch_stmt]),
                tok.line,
                tok.col,
            ))
        } else {
            Ok(switch_stmt)
        }
    }
}
