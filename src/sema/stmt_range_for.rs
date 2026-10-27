use super::*;

struct RangeForParts {
    decl: Stmt,
    range: Expr,
    range_ty: Type,
    body: Stmt,
    pos: ExprPos,
}

fn is_lvalue_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Identifier(_) | Expr::ArraySubscript { .. } => true,
        Expr::MemberAccess {
            object, is_arrow, ..
        } => *is_arrow || is_lvalue_expr(object),
        Expr::Unary {
            op: UnaryOp::Deref, ..
        } => true,
        _ => false,
    }
}

fn make_var_decl(name: String, ty: Type, init: Option<Expr>, pos: ExprPos) -> Stmt {
    Stmt::new(
        StmtKind::VarDecl {
            name,
            ty,
            init,
            is_static: false,
        },
        pos.line,
        pos.col,
    )
}

fn make_member_call(object_name: &str, method: &str, pos: ExprPos) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::MemberAccess {
            object: Box::new(Expr::Identifier(object_name.to_string())),
            member: method.to_string(),
            is_arrow: false,
            pos,
        }),
        args: vec![],
        pos,
    }
}

fn build_for_loop_with_iterators(
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
            range_ty,
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
                    parts.range_ty.to_c_type()
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

        let begin_decl = make_var_decl(begin_name.clone(), ptr_ty.clone(), Some(parts.range), parts.pos);
        let end_init = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier(begin_name.clone())),
            right: Box::new(Expr::IntLiteral(size as i64, Type::Int)),
            pos: parts.pos,
        };
        let end_decl = make_var_decl(end_name.clone(), ptr_ty, Some(end_init), parts.pos);

        let for_loop = build_for_loop_with_iterators(
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

    fn desugar_range_for_class(
        &mut self,
        parts: RangeForParts,
        sname: &str,
    ) -> Result<Stmt, CompileError> {
        let is_const = parts.range_ty.is_const()
            || matches!(&parts.range_ty, Type::Reference(inner) if inner.is_const());
        let begin_sig = self.find_method_in_class_for_receiver(sname, "begin", is_const);
        let end_sig = self.find_method_in_class_for_receiver(sname, "end", is_const);
        if begin_sig.is_none() || end_sig.is_none() {
            return Err(CompileError::new(
                format!("class '{}' does not have matching begin() and end() methods", sname),
                parts.pos.line,
                parts.pos.col,
            ));
        }
        let iter_ty = begin_sig.unwrap().return_type.strip_reference().clone();

        let idx = self.range_for_counter;
        self.range_for_counter += 1;
        let range_name = format!("__range_{}", idx);
        let begin_name = format!("__begin_{}", idx);
        let end_name = format!("__end_{}", idx);

        let range_var_ty = if is_lvalue_expr(&parts.range) {
            Type::Reference(Box::new(parts.range_ty.clone()))
        } else {
            parts.range_ty.clone()
        };

        let range_decl = make_var_decl(range_name.clone(), range_var_ty, Some(parts.range), parts.pos);
        let begin_call = make_member_call(&range_name, "begin", parts.pos);
        let begin_decl = make_var_decl(begin_name.clone(), iter_ty.clone(), Some(begin_call), parts.pos);
        let end_call = make_member_call(&range_name, "end", parts.pos);
        let end_decl = make_var_decl(end_name.clone(), iter_ty, Some(end_call), parts.pos);

        let for_loop = build_for_loop_with_iterators(
            begin_name,
            end_name,
            parts.decl,
            parts.body,
            parts.pos,
        );

        Ok(Stmt::new(
            StmtKind::Compound(vec![range_decl, begin_decl, end_decl, for_loop]),
            parts.pos.line,
            parts.pos.col,
        ))
    }
}
