use super::*;

struct RangeForParts {
    decl: Stmt,
    range: Expr,
    body: Stmt,
    pos: ExprPos,
}

impl Sema {
    pub(super) fn desugar_range_for(&mut self, stmt: Stmt) -> Result<Stmt, CompileError> {
        let (decl, mut range, body) = match stmt.kind {
            StmtKind::RangeFor { decl, range, body } => (*decl, range, *body),
            _ => return Ok(stmt),
        };
        let line = stmt.line;
        let col = stmt.col;
        let pos = ExprPos::new(line, col);

        let range_ty = self.analyze_expr(&mut range)?;
        let clean_range_ty = range_ty.strip_reference().strip_const().clone();
        let parts = RangeForParts {
            decl,
            range,
            body,
            pos,
        };

        match clean_range_ty {
            Type::Array(elem_ty, size) => {
                self.desugar_range_for_array(parts, *elem_ty, size)
            }
            Type::Struct(sname) => {
                self.desugar_range_for_class(parts, &sname)
            }
            _ => Err(CompileError::new(
                format!(
                    "range expression must be an array or class with begin() and end(), found '{}'",
                    range_ty.to_c_type()
                ),
                line,
                col,
            )),
        }
    }

    fn desugar_range_for_array(
        &mut self,
        parts: RangeForParts,
        elem_ty: Type,
        size: usize,
    ) -> Result<Stmt, CompileError> {
        let idx = self.range_for_counter;
        self.range_for_counter += 1;

        let begin_name = format!("__begin_{}", idx);
        let end_name = format!("__end_{}", idx);
        let ptr_ty = Type::Pointer(Box::new(elem_ty));

        let begin_decl = Stmt::new(
            StmtKind::VarDecl {
                name: begin_name.clone(),
                ty: ptr_ty.clone(),
                init: Some(parts.range),
                is_static: false,
            },
            parts.pos.line,
            parts.pos.col,
        );

        let end_init = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier(begin_name.clone())),
            right: Box::new(Expr::IntLiteral(size as i64, Type::Int)),
            pos: parts.pos,
        };
        let end_decl = Stmt::new(
            StmtKind::VarDecl {
                name: end_name.clone(),
                ty: ptr_ty,
                init: Some(end_init),
                is_static: false,
            },
            parts.pos.line,
            parts.pos.col,
        );

        let for_loop = self.build_for_loop_with_iterators(
            begin_name,
            end_name,
            parts.decl,
            parts.body,
            parts.pos,
        );

        Ok(Stmt::new(
            StmtKind::Compound(vec![begin_decl, end_decl, for_loop]),
            parts.pos.line,
            parts.pos.col,
        ))
    }

    fn build_for_loop_with_iterators(
        &self,
        begin_name: String,
        end_name: String,
        mut decl: Stmt,
        body: Stmt,
        pos: ExprPos,
    ) -> Stmt {
        let cond = Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Identifier(begin_name.clone())),
            right: Box::new(Expr::Identifier(end_name)),
            pos,
        };

        let step = Expr::Unary {
            op: UnaryOp::PreInc,
            expr: Box::new(Expr::Identifier(begin_name.clone())),
            pos,
        };

        let deref_begin = Expr::Unary {
            op: UnaryOp::Deref,
            expr: Box::new(Expr::Identifier(begin_name)),
            pos,
        };
        if let StmtKind::VarDecl { init, .. } = &mut decl.kind {
            *init = Some(deref_begin);
        }

        let loop_body = Stmt::new(StmtKind::Compound(vec![decl, body]), pos.line, pos.col);
        Stmt::new(
            StmtKind::For {
                init: None,
                cond: Some(cond),
                step: Some(step),
                body: Box::new(loop_body),
            },
            pos.line,
            pos.col,
        )
    }

    fn desugar_range_for_class(
        &mut self,
        parts: RangeForParts,
        sname: &str,
    ) -> Result<Stmt, CompileError> {
        Err(CompileError::new(
            format!("range-based for on class '{}' not yet implemented", sname),
            parts.pos.line,
            parts.pos.col,
        ))
    }
}
