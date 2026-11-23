//! Variable template instantiation.

use super::*;
use std::collections::HashMap;

impl Sema {
    /// Instantiates a variable template reference such as `is_integral_v<int>`
    /// and registers the resulting global constant.
    pub(super) fn instantiate_variable_template(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<Type>, CompileError> {
        let Some((base, args_str)) = super::template_deduce::split_template_call_name(name) else {
            return Ok(None);
        };
        let Some((params, ty, init)) = self.variable_templates.get(base).cloned() else {
            return Ok(None);
        };
        // Each specialization is registered once. Re-emit its constant under
        // the spelling used at this site so differently qualified references
        // (e.g. `std::v<int>` vs `v<int>`) see the same value.
        if let Some((cached_ty, cached_const)) =
            self.instantiated_variable_templates.get(name).cloned()
        {
            if let Some(value) = cached_const {
                self.static_const_values.insert(name.to_string(), value);
            }
            return Ok(Some(cached_ty));
        }
        let known: Vec<String> = self
            .class_templates
            .keys()
            .chain(self.alias_templates.keys())
            .cloned()
            .collect();
        let mut args = super::template_deduce::parse_types_list(args_str, &known)?;
        // An argument can be an alias template or a dependent nested type;
        // expand it before deducing so the specialization sees the real type.
        for arg in &mut args {
            self.resolve_and_instantiate_type(arg, line, col)?;
        }
        let effective =
            super::template_deduce::fill_default_template_args(base, &params, &args, (line, col))?;
        let mapping = super::template_deduce::build_template_arg_map(&params, &effective);
        let mut inst_ty = super::template_subst::substitute_type(&ty, &mapping, None)?;
        self.resolve_and_instantiate_type(&mut inst_ty, line, col)?;

        let mut inst_init = None;
        if let Some(initializer) = &init {
            inst_init = Some(self.fold_variable_initializer(
                name,
                initializer,
                &mapping,
                &inst_ty,
                (line, col),
            )?);
        }

        self.global_vars.insert(
            name.to_string(),
            VarSymbol {
                name: name.to_string(),
                ty: inst_ty.clone(),
                is_global: true,
                is_param: false,
                stack_offset: 0,
            },
        );
        self.synthesized_decls.push(crate::ast::Decl::GlobalVar {
            name: name.to_string(),
            ty: inst_ty.clone(),
            init: inst_init,
            line,
            col,
        });
        let recorded_const = self.static_const_values.get(name).copied();
        self.instantiated_variable_templates
            .insert(name.to_string(), (inst_ty.clone(), recorded_const));
        Ok(Some(inst_ty))
    }

    /// Substitutes and folds a variable template's initializer, registering a
    /// constant when the folded expression is evaluable.
    fn fold_variable_initializer(
        &mut self,
        name: &str,
        initializer: &crate::ast::expr::Expr,
        mapping: &HashMap<String, crate::sema::pack_expansion::TemplateArg>,
        inst_ty: &Type,
        pos: (usize, usize),
    ) -> Result<crate::ast::expr::Expr, CompileError> {
        let (line, col) = pos;
        let mut expr = initializer.clone();
        super::template_subst::substitute_expr(&mut expr, mapping, None)?;
        // A static const member operand (the common `Trait<T>::value` case)
        // is folded to a literal so the global initializer is self-contained.
        let mut folded = expr.clone();
        if let crate::ast::expr::Expr::Identifier(member) = &expr
            && let Some(value) = self.resolve_static_const_nontype(member, line, col)?
        {
            folded = crate::ast::expr::Expr::IntLiteral(value as i64, inst_ty.clone());
        }
        if let Ok(value) = super::constexpr_eval::eval_constexpr_with_context(
            &folded,
            Some(&self.struct_sizes),
            Some(&self.static_const_values),
            None,
        ) {
            self.static_const_values.insert(name.to_string(), value);
        }
        Ok(folded)
    }
}
