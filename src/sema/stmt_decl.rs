//! Analysis of variable declarations.

use super::*;

/// The parts of a variable declaration that the analysis needs.
pub(crate) struct VarDecl<'e, 't> {
    /// The declared name.
    pub(crate) name: &'e mut String,
    /// The declared type, updated when the type is deduced.
    pub(crate) ty: &'t mut Type,
    /// The initializer, if any.
    pub(crate) init: &'e mut Option<Expr>,
    /// Whether the variable has static storage duration.
    pub(crate) is_static: bool,
    /// The line of the declaration.
    pub(crate) line: usize,
    /// The column of the declaration.
    pub(crate) col: usize,
}

impl Sema {
    /// Analyzes a variable declaration and registers the variable.
    pub(super) fn analyze_var_decl(&mut self, decl: VarDecl<'_, '_>) -> Result<(), CompileError> {
        let VarDecl {
            name,
            ty,
            init,
            is_static,
            line,
            col,
        } = decl;
        self.current_line = line;
        self.current_col = col;
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
        if let Some(scope) = self.scopes.last()
            && scope.vars.contains_key(name)
        {
            return Err(CompileError::new(
                format!("Redefinition of variable '{}'", name),
                line,
                col,
            ));
        }
        if let Type::Array(elem, 0) = ty.clone()
            && let Some(Expr::InitializerList(list)) = init
        {
            *ty = Type::Array(elem, list.len());
        }
        if let Some(init_expr) = init {
            self.analyze_var_init(name, ty, init_expr, line, col)?;
        }
        if is_static {
            self.declare_static_var(name, ty, line, col)?;
        } else {
            self.declare_local_var(name, ty, line, col)?;
        }
        Ok(())
    }

    /// Checks the initializer of a variable and completes its type.
    fn analyze_var_init(
        &mut self,
        name: &str,
        ty: &mut Type,
        init_expr: &mut Expr,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let init_ty = {
            // A constructor call in an initializer reports an
            // initialization diagnostic.
            let previous = self.in_initializer;
            self.in_initializer = true;
            let result = self.analyze_expr(init_expr);
            self.in_initializer = previous;
            result?
        };
        if *ty == Type::Auto {
            *ty = init_ty.strip_reference().clone();
        } else if let Type::Const(inner) = ty
            && **inner == Type::Auto
        {
            *ty = Type::Const(Box::new(init_ty.strip_reference().clone()));
        } else if let Type::Pointer(inner) = ty
            && **inner == Type::Auto
        {
            *ty = Type::Pointer(Box::new(
                init_ty
                    .deref_type()
                    .cloned()
                    .unwrap_or_else(|| init_ty.strip_reference().clone()),
            ));
        } else if let Type::Reference(inner) = ty {
            if **inner == Type::Auto {
                *ty = Type::Reference(Box::new(init_ty.strip_reference().clone()));
            } else if let Type::Const(c_inner) = &**inner
                && **c_inner == Type::Auto
            {
                *ty = Type::Reference(Box::new(Type::Const(Box::new(
                    init_ty.strip_reference().clone(),
                ))));
            }
        }
        if !self.can_assign(ty, &init_ty) {
            return Err(CompileError::new(
                format!(
                    "cannot initialize variable '{}' of type {} with {}",
                    name,
                    ty.to_c_type(),
                    init_ty.to_c_type()
                ),
                line,
                col,
            ));
        }
        Ok(())
    }

    /// Registers a variable with static storage duration.
    fn declare_static_var(
        &mut self,
        name: &str,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let function = self.current_func_name.as_deref().unwrap_or("");
        let global_name = crate::name::static_local_name(function, line, col, name);
        let sym = VarSymbol {
            name: global_name.clone(),
            ty: ty.clone(),
            is_global: true,
            stack_offset: 0,
        };
        self.global_vars.insert(global_name.clone(), sym.clone());
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new(
                "internal compiler error: missing variable scope",
                line,
                col,
            )
            .with_kind(crate::diagnostics::ErrorKind::Internal));
        };
        scope.vars.insert(name.to_string(), sym);
        Ok(())
    }

    /// Registers a variable with automatic storage duration.
    fn declare_local_var(
        &mut self,
        name: &str,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let var_size = ty.size_of(&self.struct_sizes) as i32;
        self.current_frame_size += (var_size + 3) & !3; // Align to 4
        let sym = VarSymbol {
            name: name.to_string(),
            ty: ty.clone(),
            is_global: false,
            stack_offset: -self.current_frame_size,
        };
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new(
                "internal compiler error: missing variable scope",
                line,
                col,
            )
            .with_kind(crate::diagnostics::ErrorKind::Internal));
        };
        scope.vars.insert(name.to_string(), sym);
        Ok(())
    }
}
