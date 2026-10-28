//! Semantic desugaring of C++17 structured bindings.

use super::*;

impl Sema {
    pub(super) fn desugar_structured_binding(&mut self, stmt: Stmt) -> Result<Stmt, CompileError> {
        let StmtKind::StructuredBinding {
            bindings,
            ty,
            mut init,
        } = stmt.kind
        else {
            return Ok(stmt);
        };
        let init_ty = self.analyze_expr(&mut init)?;
        let fields = self.resolve_binding_fields(&init_ty, stmt.line, stmt.col, bindings.len())?;

        let hidden_name = format!("__sb_{}_{}", stmt.line, stmt.col);
        let hidden_decl = Stmt::new(
            StmtKind::VarDecl {
                name: hidden_name.clone(),
                ty: ty.clone(),
                init: Some(init),
                is_static: false,
            },
            stmt.line,
            stmt.col,
        );

        let is_const_ref = matches!(ty, Type::Reference(ref inner) if inner.is_const())
            || matches!(ty, Type::Const(_));
        let mut stmts = vec![hidden_decl];
        stmts.extend(self.build_binding_decls(
            &hidden_name,
            is_const_ref,
            &bindings,
            &fields,
            (stmt.line, stmt.col),
        ));

        Ok(Stmt::new(
            StmtKind::VarDeclGroup(stmts),
            stmt.line,
            stmt.col,
        ))
    }

    fn resolve_binding_fields(
        &self,
        init_ty: &Type,
        line: usize,
        col: usize,
        binding_count: usize,
    ) -> Result<Vec<StructField>, CompileError> {
        let struct_name = match init_ty.strip_reference().strip_const() {
            Type::Struct(name) => name.clone(),
            _ => {
                return Err(CompileError::new(
                    "cannot decompose non-class type in structured binding",
                    line,
                    col,
                ));
            }
        };
        let fields = match self.struct_layouts.get(&struct_name) {
            Some(layout) => layout.fields.clone(),
            None => {
                return Err(CompileError::new(
                    format!("undefined struct '{}'", struct_name),
                    line,
                    col,
                ));
            }
        };
        if binding_count != fields.len() {
            return Err(CompileError::new(
                format!(
                    "{} names provided in structured binding, but struct '{}' only has {} fields",
                    binding_count,
                    struct_name,
                    fields.len()
                ),
                line,
                col,
            ));
        }
        Ok(fields)
    }

    fn build_binding_decls(
        &self,
        hidden_name: &str,
        is_const_ref: bool,
        bindings: &[String],
        fields: &[StructField],
        pos: (usize, usize),
    ) -> Vec<Stmt> {
        let (line, col) = pos;
        bindings
            .iter()
            .zip(fields.iter())
            .map(|(name, field)| {
                let binding_ty = if is_const_ref {
                    Type::Reference(Box::new(Type::Const(Box::new(Type::Auto))))
                } else {
                    Type::Reference(Box::new(Type::Auto))
                };
                let member_expr = Expr::MemberAccess {
                    object: Box::new(Expr::Identifier(hidden_name.to_string())),
                    member: field.name.clone(),
                    is_arrow: false,
                    pos: ExprPos::new(line, col),
                };
                Stmt::new(
                    StmtKind::VarDecl {
                        name: name.clone(),
                        ty: binding_ty,
                        init: Some(member_expr),
                        is_static: false,
                    },
                    line,
                    col,
                )
            })
            .collect()
    }
}
