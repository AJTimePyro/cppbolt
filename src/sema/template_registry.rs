//! Template registration, lookup, instantiation caching, and lazy member instantiation.

use super::analyze::this_param;
use super::template_deduce::{parse_types_list, split_template_call_name};
pub(crate) use super::template_registry_helpers::insert_template_aliases;
use super::template_subst::{substitute_struct, substitute_type};
use super::*;
use crate::ast::decl::{Decl, FunctionDecl};
use crate::ast::expr::Expr;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::{ErrorKind, SubstitutionFailure, TemplateFailure};
use crate::name::QualifiedName;
use std::collections::HashMap;

/// Stable identity for a function template definition, used to find the
/// ordinary lookup set captured when it was declared.
pub(crate) fn function_template_identity(name: &str, line: usize, col: usize) -> String {
    format!("{name}@{line}:{col}")
}

impl Sema {
    pub fn register_function_template(&mut self, decl: FunctionDecl, params: Vec<TemplateParam>) {
        let short = decl
            .name
            .rsplit("::")
            .next()
            .unwrap_or(&decl.name)
            .to_string();
        let q = QualifiedName::new(&decl.name);
        let keys = [
            decl.name.clone(),
            short,
            q.plain().to_string(),
            q.std_name().into_owned(),
        ];
        let item = (decl, params);
        for key in keys {
            let list = self.function_templates.entry(key).or_default();
            if let Some((existing_decl, _)) = list.iter_mut().find(|(d, _)| {
                d.params.len() == item.0.params.len()
                    && d.return_type == item.0.return_type
                    && d.params
                        .iter()
                        .zip(&item.0.params)
                        .all(|(p1, p2)| p1.ty == p2.ty)
            }) {
                if existing_decl.body.is_none() && item.0.body.is_some() {
                    existing_decl.body = item.0.body.clone();
                }
            } else {
                list.push(item.clone());
            }
        }
    }

    pub fn resolve_and_instantiate_type(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        match ty {
            Type::TemplateInstance { name, args } => {
                let plain = QualifiedName::new(name).plain();
                if self.variable_templates.contains_key(name.as_str())
                    || self.variable_templates.contains_key(plain)
                {
                    let rendered = format!(
                        "{}<{}>",
                        name,
                        args.iter()
                            .map(|a| a.to_c_type())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if self
                        .instantiate_variable_template(&rendered, line, col)
                        .ok()
                        .flatten()
                        .is_some()
                        && let Some(value) = self.static_const_values.get(&rendered).copied()
                    {
                        *ty = Type::NonType(value);
                        return Ok(());
                    }
                }
                for arg in args.iter_mut() {
                    self.resolve_and_instantiate_type(arg, line, col)?;
                }
                if let Some(expanded) = self.expand_alias_template(name, args, line, col)? {
                    *ty = expanded;
                    return Ok(());
                }
                let inst_name = self.instantiate_class_template(name, args, line, col)?;
                *ty = Type::Struct(inst_name);
            }
            Type::Decltype {
                expr,
                parenthesized,
            } => {
                let resolved = self.resolve_decltype_type(expr, *parenthesized, line, col)?;
                *ty = resolved;
                self.resolve_and_instantiate_type(ty, line, col)?;
            }
            Type::DependentNested(name) => {
                // A dependent qualified name used as a non-type template
                // argument can name a static const member (`Trait<T>::value`),
                // which is a constant rather than a nested type.
                if let Some(value) = self.resolve_static_const_nontype(name, line, col)? {
                    *ty = Type::NonType(value);
                    return Ok(());
                }
                let Some(resolved) = self.resolve_nested_type_name(name, line, col)? else {
                    let (base, member) = name.rsplit_once("::").unwrap_or(("", name.as_str()));
                    return Err(CompileError::new(
                        format!("no type named '{}' in '{}'", member, base),
                        line,
                        col,
                    )
                    .with_kind(ErrorKind::Substitution));
                };
                *ty = resolved;
                self.resolve_and_instantiate_type(ty, line, col)?;
            }
            Type::Struct(sname) => {
                let sname_clone = sname.clone();
                self.resolve_struct_type_alias(ty, &sname_clone, line, col)?;
                if let Type::Struct(resolved_name) = ty {
                    self.check_nested_type_access(&sname_clone, line, col)?;
                    if resolved_name != &sname_clone {
                        self.check_nested_type_access(resolved_name, line, col)?;
                    }
                } else {
                    self.resolve_and_instantiate_type(ty, line, col)?;
                }
            }
            Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::RvalueReference(inner)
            | Type::Const(inner) => {
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            Type::Array(inner, size) => {
                if *size == usize::MAX {
                    return Err(CompileError::new(
                        "Array dimension must be non-negative integer constant",
                        line,
                        col,
                    ));
                }
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            Type::MemberPointer {
                class_name,
                member_type,
            } => {
                *class_name = self.resolve_alias_in_qualified_name(class_name);
                self.resolve_and_instantiate_type(member_type, line, col)?;
            }
            Type::Function {
                return_type,
                param_types,
                ..
            } => {
                self.resolve_and_instantiate_type(return_type, line, col)?;
                for p in param_types.iter_mut() {
                    self.resolve_and_instantiate_type(p, line, col)?;
                }
            }
            Type::BoundMemberFunction { member_type, .. } => {
                self.resolve_and_instantiate_type(member_type, line, col)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Resolves a signature type: a missing nested type discards the template
    /// candidate, while an error from instantiating another template stays hard.
    pub(super) fn resolve_signature_type(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), TemplateFailure> {
        self.resolve_and_instantiate_type(ty, line, col)
            .map_err(|error| {
                if error.kind == ErrorKind::Substitution {
                    TemplateFailure::Substitution(SubstitutionFailure::new(
                        error.message,
                        error.line,
                        error.col,
                    ))
                } else {
                    TemplateFailure::Hard(error)
                }
            })
    }

    fn resolve_struct_type_alias(
        &mut self,
        ty: &mut Type,
        sname: &str,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        if let Some(alias) = self.resolve_nested_type_name(sname, line, col)? {
            *ty = alias;
        }
        Ok(())
    }

    /// Resolves a qualified name to its type. Missing nested types yield
    /// `None`; instantiation errors stay hard.
    /// Splits `Trait<Args...>::member` at the last `::` that is not nested
    /// inside template argument brackets.
    fn split_static_member_name(name: &str) -> Option<(&str, &str)> {
        let bytes = name.as_bytes();
        let mut depth = 0i32;
        let mut i = bytes.len();
        while i > 0 {
            i -= 1;
            match bytes[i] {
                b'>' => depth += 1,
                b'<' => depth -= 1,
                b':' if i + 1 < bytes.len() && bytes[i + 1] == b':' && depth == 0 => {
                    return Some((&name[..i], &name[i + 2..]));
                }
                _ => {}
            }
        }
        None
    }

    /// Evaluates a dependent non-type argument expression (for example
    /// `!is_integral<int>::value`) using the constants of instantiated static
    /// const members that are in scope.
    fn eval_dependent_nontype(&mut self, expr_str: &str) -> Option<i128> {
        let wrapped = format!("({})", expr_str);
        let tokens = crate::lexer::Lexer::new(&wrapped).tokenize().ok()?;
        let mut parser = crate::parser::Parser::new(tokens);
        parser.allow_unknown_types = true;
        // A fresh parser has no template registry; seed it so qualified
        // template-id operands such as `Trait<T>::value` parse correctly.
        for name in self
            .class_templates
            .keys()
            .chain(self.alias_templates.keys())
            .chain(self.variable_templates.keys())
        {
            parser.register_template_name(name);
        }
        let expr = parser.parse_expr().ok()?;
        self.instantiate_nontype_refs(&expr);
        super::constexpr_eval::eval_constexpr_with_context(
            &expr,
            Some(&self.struct_sizes),
            Some(&self.static_const_values),
            None,
        )
        .ok()
    }

    /// Instantiates class templates named by qualified static members inside
    /// a non-type argument expression so their constants become available.
    fn instantiate_nontype_refs(&mut self, expr: &crate::ast::expr::Expr) {
        use crate::ast::expr::Expr;
        match expr {
            Expr::Identifier(name) if name.contains('<') => {
                let _ = self.instantiate_variable_template(name, 0, 0);
                if name.contains("::") {
                    let _ = self.resolve_static_const_nontype(name, 0, 0);
                }
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::SizeofExpr(expr) => {
                self.instantiate_nontype_refs(expr)
            }
            Expr::Binary { left, right, .. } => {
                self.instantiate_nontype_refs(left);
                self.instantiate_nontype_refs(right);
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.instantiate_nontype_refs(cond);
                self.instantiate_nontype_refs(then_expr);
                self.instantiate_nontype_refs(else_expr);
            }
            _ => {}
        }
    }

    /// Folds a non-type template argument spelled as a static const member or
    /// an expression over such members into a concrete constant.
    pub(super) fn try_resolve_nontype_arg(
        &mut self,
        arg: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        match arg {
            Type::Struct(name) => {
                if let Some(value) = self.resolve_static_const_nontype(name, line, col)? {
                    *arg = Type::NonType(value);
                } else if name.contains('<')
                    && self
                        .instantiate_variable_template(name, line, col)
                        .ok()
                        .flatten()
                        .is_some()
                    && let Some(value) = self.static_const_values.get(name).copied()
                {
                    *arg = Type::NonType(value);
                }
            }
            Type::DependentNonType(expr) => {
                if let Some(value) = self.eval_dependent_nontype(expr) {
                    *arg = Type::NonType(value);
                }
            }
            Type::TemplateInstance { .. } => {
                // A variable template reference such as `is_integral_v<T>`
                // parses as a TemplateInstance; fold it to its constant.
                let rendered = arg.to_c_type();
                if self
                    .instantiate_variable_template(&rendered, line, col)
                    .ok()
                    .flatten()
                    .is_some()
                    && let Some(value) = self.static_const_values.get(&rendered).copied()
                {
                    *arg = Type::NonType(value);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Evaluates a static const data member used as a non-type template
    /// argument, e.g. `std::is_integral<int>::value`. The owning class is
    /// instantiated on demand so its constant is registered first.
    fn resolve_static_const_nontype(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<i128>, CompileError> {
        let Some((class_part, member)) = Self::split_static_member_name(name) else {
            return Ok(None);
        };
        let Some((base, args_str)) = super::template_deduce::split_template_call_name(class_part)
        else {
            return Ok(None);
        };
        let known: Vec<String> = self.class_templates.keys().cloned().collect();
        let Ok(args) = super::template_deduce::parse_types_list(args_str, &known) else {
            return Ok(None);
        };
        // If instantiating the owner fails, let the caller's normal nested
        // type resolution report it with the right hard/soft classification.
        let Ok(inst) = self.instantiate_class_template(base, &args, line, col) else {
            return Ok(None);
        };
        let plain = QualifiedName::new(&inst).plain();
        let value = self
            .static_const_values
            .get(&format!("{}::{}", inst, member))
            .copied()
            .or_else(|| {
                self.static_const_values
                    .get(&format!("{}::{}", plain, member))
                    .copied()
            });
        // The canonical name can include filled default arguments. Keep the
        // value under the spelling used at the call site as well, so an
        // enclosing constant expression can read it.
        if let Some(value) = value {
            self.static_const_values.insert(name.to_string(), value);
        }
        Ok(value)
    }

    fn resolve_nested_type_name(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<Type>, CompileError> {
        if let Some(alias) = self.lookup_type_alias(name) {
            return Ok(Some(alias));
        }
        let Some(gt_pos) = name.rfind('>') else {
            return Ok(None);
        };
        if !name[gt_pos..].starts_with(">::") {
            return Ok(None);
        }
        let inst_part = &name[..gt_pos + 1];
        let Some((tmpl_name, args_str)) = split_template_call_name(inst_part) else {
            return Ok(None);
        };
        let known: Vec<String> = self
            .class_templates
            .keys()
            .chain(self.alias_templates.keys())
            .cloned()
            .collect();
        let Ok(mut args) = parse_types_list(args_str, &known) else {
            return Ok(None);
        };
        for arg in &mut args {
            // A static const member used as a non-type argument (for example
            // `is_integral<T>::value`) is a constant, not a nested type.
            self.try_resolve_nontype_arg(arg, line, col)?;
            self.resolve_and_instantiate_type(arg, line, col)?;
        }
        // An error while instantiating the base template is outside the
        // signature's immediate context, even if it names a dependent type.
        let inst = self
            .instantiate_class_template(tmpl_name, &args, line, col)
            .map_err(|error| {
                if error.kind == ErrorKind::Substitution {
                    error.with_kind(ErrorKind::Semantic)
                } else {
                    error
                }
            })?;
        let canon = format!("{}::{}", inst, &name[gt_pos + 3..]);
        if let Some(alias) = self.lookup_type_alias(&canon) {
            return Ok(Some(alias));
        }
        if self.struct_layouts.contains_key(&canon) {
            return Ok(Some(Type::Struct(canon)));
        }
        Ok(None)
    }

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
        let args = super::template_deduce::parse_types_list(args_str, &known)?;
        let effective =
            super::template_deduce::fill_default_template_args(base, &params, &args, (line, col))?;
        let mapping = super::template_deduce::build_template_arg_map(&params, &effective);
        let mut inst_ty = super::template_subst::substitute_type(&ty, &mapping, None)?;
        self.resolve_and_instantiate_type(&mut inst_ty, line, col)?;

        let mut inst_init = None;
        if let Some(initializer) = &init {
            let mut expr = initializer.clone();
            super::template_subst::substitute_expr(&mut expr, &mapping, None)?;
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
            inst_init = Some(folded);
        }

        self.global_vars.insert(
            name.to_string(),
            VarSymbol {
                name: name.to_string(),
                ty: inst_ty.clone(),
                is_global: true,
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

    pub fn instantiate_class_template(
        &mut self,
        name: &str,
        args: &[Type],
        line: usize,
        col: usize,
    ) -> Result<String, CompileError> {
        if let Some(expanded) = self.expand_alias_template(name, args, line, col)? {
            if let Type::Struct(sname) = expanded {
                return Ok(sname);
            }
            return Ok(expanded.to_c_type());
        }

        let (template_class, template_params) = self.resolve_class_template(name, line, col)?;

        // A non-type argument spelled as a static const member of another
        // class template (`Trait<T>::value`) must be folded to its constant
        // value before argument-kind validation.
        let mut resolved_args = args.to_vec();
        for (param, arg) in template_params.iter().zip(resolved_args.iter_mut()) {
            if param.param_type.is_some() {
                self.try_resolve_nontype_arg(arg, line, col)?;
            }
        }

        let effective_args = super::template_deduce::fill_default_template_args(
            name,
            &template_params,
            &resolved_args,
            (line, col),
        )?;

        if self.instantiation_depth >= 500 {
            return Err(CompileError::new(
                "template instantiation depth exceeded (limit: 500)",
                line,
                col,
            ));
        }

        let canonical_args = effective_args
            .iter()
            .map(|a| a.to_c_type())
            .collect::<Vec<_>>()
            .join(", ");
        let plain_name = QualifiedName::new(name).plain();
        let inst_name = format!("{}<{}>", plain_name, canonical_args);

        // A completed specialization is cached. An in-progress one is still
        // on the stack and must satisfy recursive references without being
        // treated as finished.
        if self.instantiated_classes.contains(&inst_name)
            || self.instantiation_stack.iter().any(|s| s == &inst_name)
        {
            return Ok(inst_name);
        }

        self.instantiation_stack.push(inst_name.clone());
        self.instantiation_depth += 1;
        let snapshot = super::template_state::InstantiationSnapshot::capture(self);

        let result = self
            .resolve_instantiation_target(
                super::template_spec::InstantiationTarget {
                    name,
                    inst_name: &inst_name,
                    template_params: &template_params,
                    effective_args: &effective_args,
                    pos: (line, col),
                },
                template_class,
            )
            .and_then(|(target_class, mapping)| {
                self.execute_class_instantiation(&target_class, mapping, &inst_name, (line, col))
            });

        self.instantiation_stack.pop();
        self.instantiation_depth -= 1;

        match result {
            Ok(()) => {
                self.instantiated_classes.insert(inst_name.clone());
                Ok(inst_name)
            }
            Err(error) => {
                if self.speculating {
                    self.speculative_hard = true;
                }
                snapshot.restore(self);
                Err(error)
            }
        }
    }

    fn execute_class_instantiation(
        &mut self,
        template_class: &TemplateClass,
        mapping: HashMap<String, crate::sema::pack_expansion::TemplateArg>,
        inst_name: &str,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        self.instantiate_nested_structs(template_class, inst_name, &mapping, pos)?;

        let mut struct_decl = substitute_struct(
            template_class,
            &mapping,
            inst_name,
            pos,
            Some(&self.struct_aligns),
        )?;
        let Decl::StructDecl {
            fields,
            methods,
            static_fields,
            base_classes,
            base_access,
            is_class,
            private_members,
            alignment,
            ..
        } = &mut struct_decl
        else {
            return Ok(());
        };

        for (_, fty, _) in fields.iter_mut() {
            self.resolve_and_instantiate_type(fty, line, col)?;
        }

        for base in base_classes.iter() {
            let _ = self.ensure_class_instantiated(base, line, col);
        }

        for m in methods.iter_mut() {
            if m.template_params.is_empty() {
                let saved_identity = self.current_template_identity.take();
                self.current_template_identity =
                    Some(function_template_identity(&m.name, m.line, m.col));
                let result = (|| {
                    self.resolve_and_instantiate_type(&mut m.return_type, line, col)?;
                    for p in &mut m.params {
                        self.resolve_and_instantiate_type(&mut p.ty, line, col)?;
                    }
                    Ok(())
                })();
                self.current_template_identity = saved_identity;
                result?;
            }
        }

        self.synthesize_instantiation_lifecycle(inst_name, base_classes, fields, methods);

        self.compute_struct_layout(RecordLayout {
            name: inst_name,
            is_class: *is_class,
            is_final: false,
            base_classes,
            base_access,
            fields,
            methods,
            private_members,
            line,
            col,
            alignment: alignment
                .as_ref()
                .map(|ty| ty.align_of(&self.struct_aligns)),
        })?;

        self.register_instantiated_static_fields(inst_name, static_fields, base_classes, line, col);
        self.register_instantiated_member_aliases(&template_class.name, inst_name, &mapping);
        let inst_ty = Type::Struct(inst_name.to_string());
        for friend in &template_class.friends {
            let mut instantiated = super::template_subst_func::substitute_function(
                friend,
                &mapping,
                Some((template_class.name.as_str(), &inst_ty)),
                None,
            )?;
            if !friend.template_params.is_empty() {
                // The enclosing class parameters are already substituted; the
                // friend keeps its own parameters until a call deduces them.
                self.register_hidden_friend_template(
                    inst_name,
                    instantiated.clone(),
                    friend.template_params.clone(),
                );
                continue;
            }
            self.register_friend_function(inst_name, instantiated.clone())?;
            let prev_class = self.current_class.take();
            self.current_class = Some(inst_name.to_string());
            let analyzed = self.analyze_function(&mut instantiated);
            self.current_class = prev_class;
            analyzed?;
            self.synthesized_decls
                .push(crate::ast::Decl::Function(instantiated));
        }
        let all_methods = std::mem::take(methods);
        self.synthesized_decls.push(struct_decl);
        self.setup_instantiated_methods(inst_name, all_methods)?;
        Ok(())
    }

    fn register_instantiated_static_fields(
        &mut self,
        inst_name: &str,
        static_fields: &[(String, Type, Option<Expr>)],
        base_classes: &[String],
        line: usize,
        col: usize,
    ) {
        for sf in static_fields.iter() {
            let qual_name = format!("{}::{}", inst_name, sf.0);
            self.global_vars.insert(
                qual_name.clone(),
                VarSymbol {
                    name: qual_name.clone(),
                    ty: sf.1.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
            if let Some(init) = &sf.2 {
                let mut init_clone = init.clone();
                let _ = self.analyze_expr(&mut init_clone);
                if sf.1.is_const() {
                    let constants = self.static_const_values.clone();
                    if let Ok(value) = super::constexpr_eval::eval_constexpr_with_context(
                        &init_clone,
                        Some(&self.struct_sizes),
                        Some(&constants),
                        None,
                    ) {
                        self.static_const_values.insert(qual_name, value);
                    }
                }
            }
        }
        for base in base_classes {
            let _ = self.ensure_class_instantiated(base, line, col);
            self.alias_static_fields(base, inst_name);
        }
    }

    fn register_instantiated_member_aliases(
        &mut self,
        tmpl_name: &str,
        inst_name: &str,
        mapping: &HashMap<String, crate::sema::pack_expansion::TemplateArg>,
    ) {
        let plain_tmpl = QualifiedName::new(tmpl_name).plain();
        let plain_inst = QualifiedName::new(inst_name).plain();
        let is_std =
            QualifiedName::new(inst_name).in_std() || QualifiedName::new(tmpl_name).in_std();
        let mut new_aliases = Vec::new();
        for (alias_name, ty) in &self.type_aliases {
            let Some((owner, member)) = alias_name.rsplit_once("::") else {
                continue;
            };
            if QualifiedName::new(owner).plain() == plain_tmpl
                && let Ok(subst_ty) = substitute_type(ty, mapping, None)
            {
                new_aliases.push((format!("{inst_name}::{member}"), subst_ty.clone()));
                if is_std {
                    new_aliases.push((format!("std::{plain_inst}::{member}"), subst_ty.clone()));
                }
                if plain_inst != inst_name {
                    new_aliases.push((format!("{plain_inst}::{member}"), subst_ty));
                }
            }
        }
        for (name, ty) in new_aliases {
            self.type_aliases.insert(name, ty);
        }
    }

    pub(super) fn retry_deferred_methods(&mut self) -> Result<(), CompileError> {
        const MAX_PASSES: usize = 8;
        for _ in 0..MAX_PASSES {
            if self.deferred_methods.is_empty() {
                return Ok(());
            }
            // Take the batch so retry-triggered instantiations can queue more.
            let batch = std::mem::take(&mut self.deferred_methods);
            for deferred in batch {
                let DeferredMethod { class_name, method } = deferred;
                let method_identity =
                    function_template_identity(&method.name, method.line, method.col);
                let mut analyzed = method;
                if !analyzed.is_static {
                    analyzed
                        .params
                        .insert(0, this_param(&class_name, analyzed.is_const));
                }
                let prev_class = self.current_class.take();
                self.current_class = Some(class_name.clone());
                let saved_identity = self.current_template_identity.take();
                self.current_template_identity = Some(method_identity);
                let res = self.analyze_function(&mut analyzed);
                self.current_template_identity = saved_identity;
                self.current_class = prev_class;
                if let Err(e) = res {
                    if e.kind == ErrorKind::Internal
                        || self.object_use_classes.contains(&class_name)
                    {
                        return Err(e);
                    }
                    continue;
                }
                if !analyzed.is_static && !analyzed.params.is_empty() {
                    analyzed.params.remove(0);
                }
                self.update_registered_method_return_type(&class_name, &analyzed);
                self.add_synthesized_method(&class_name, analyzed);
            }
        }
        if self.deferred_methods.is_empty() {
            Ok(())
        } else {
            Err(crate::internal_error!(
                1,
                1,
                "deferred method analysis did not converge"
            ))
        }
    }

    pub fn instantiate_pending_method(&mut self, mangled_name: &str) -> Result<(), CompileError> {
        if let Some((class_name, mut method_decl)) =
            self.pending_template_methods.remove(mangled_name)
        {
            let prev_class = self.current_class.take();
            self.current_class = Some(class_name.clone());
            let mut func_decl = method_decl.clone();
            if !method_decl.is_static {
                func_decl
                    .params
                    .insert(0, this_param(&class_name, method_decl.is_const));
            }
            let saved_identity = self.current_template_identity.take();
            self.current_template_identity = Some(function_template_identity(
                &method_decl.name,
                method_decl.line,
                method_decl.col,
            ));
            let res = self.analyze_function(&mut func_decl);
            self.current_template_identity = saved_identity;
            self.current_class = prev_class;
            res?;
            method_decl.body = func_decl.body;
            method_decl.ctor_initializers = func_decl.ctor_initializers;
            let mut found = false;
            for decl in &mut self.synthesized_decls {
                if let Decl::StructDecl { name, methods, .. } = decl
                    && name == &class_name
                {
                    if let Some(existing) = methods
                        .iter_mut()
                        .find(|m| m.name == method_decl.name && m.params == method_decl.params)
                    {
                        *existing = method_decl.clone();
                    } else {
                        methods.push(method_decl.clone());
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                self.synthesized_decls.push(Decl::StructDecl {
                    name: class_name,
                    is_class: false,
                    is_final: false,
                    base_classes: vec![],
                    base_access: vec![],
                    fields: vec![],
                    methods: vec![method_decl],
                    static_fields: vec![],
                    private_members: vec![],
                    nested_structs: vec![],
                    alignment: None,
                    line: 0,
                    col: 0,
                });
            }
        }
        Ok(())
    }
}
