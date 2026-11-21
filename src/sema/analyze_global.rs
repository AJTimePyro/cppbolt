//! Global variable analysis: type completion, initializer checks, registration.

use super::*;

impl Sema {
    /// Rejects abstract variable types and completes a zero-length array type
    /// from an initializer list.
    pub(super) fn check_and_complete_var_type(
        &self,
        ty: &mut Type,
        init: &Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const()
            && let Some(layout) = self.struct_layouts.get(sname)
            && layout.is_abstract
        {
            return Err(CompileError::new(
                format!("Variable type '{}' is an abstract class", sname),
                line,
                col,
            ));
        }
        if let Type::Array(elem, crate::ast::types::UNBOUNDED_ARRAY) = ty.clone()
            && let Some(
                Expr::InitializerList { elements: list, .. }
                | Expr::TypedInitList { elements: list, .. },
            ) = init
        {
            *ty = Type::Array(elem, list.len());
        }
        Ok(())
    }

    pub(super) fn analyze_global_var(
        &mut self,
        name: &str,
        ty: &mut Type,
        init: &mut Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        self.current_line = line;
        self.current_col = col;
        self.resolve_and_instantiate_type(ty, line, col)?;
        self.check_and_complete_var_type(ty, init, line, col)?;
        if let Some(init_expr) = init {
            let init_ty = match init_expr {
                Expr::InitializerList { .. } | Expr::TypedInitList { .. } => {
                    self.resolve_var_init_type(ty, init_expr)?.0
                }
                _ => self.analyze_expr_keeping_reference(init_expr)?,
            };
            if !self.can_assign_expr(ty, &init_ty, Some(init_expr)) {
                return Err(CompileError::new(
                    format!(
                        "Type mismatch in global '{}': cannot assign {} to {}",
                        name,
                        init_ty.to_c_type(),
                        ty.to_c_type()
                    ),
                    line,
                    col,
                ));
            }
        }
        self.global_vars.insert(
            name.to_string(),
            VarSymbol {
                name: name.to_string(),
                ty: ty.clone(),
                is_global: true,
                stack_offset: 0,
            },
        );
        if let Some(alias) = QualifiedName::new(name).implicit_alias() {
            self.global_vars.insert(
                alias.to_string(),
                VarSymbol {
                    name: alias.to_string(),
                    ty: ty.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
        }
        Ok(())
    }
}
