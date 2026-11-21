//! Non-type template argument resolution and nested-type lookup.

use super::template_deduce::{parse_types_list, split_template_call_name};
use super::template_nested_lookup::{
    count_base_paths, declares_member_alias, nested_type_declarers, selected_class_definition,
};
use super::*;
use crate::diagnostics::ErrorKind;
use crate::name::QualifiedName;
use std::collections::HashSet;

impl Sema {
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
    pub(super) fn eval_dependent_nontype(&mut self, expr_str: &str) -> Option<i128> {
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
                } else if let Some(value) = self.static_const_values.get(name).copied() {
                    // A namespace-scope constexpr variable used as a non-type
                    // argument parses as an opaque name; its folded value was
                    // collected with the other compile-time constants.
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
                // A fully qualified spelling like `::C::n` is parsed as a
                // dependent expression, but it names a folded constant. Try
                // the plain spelling before evaluating it as an expression.
                let plain = expr.strip_prefix("::").unwrap_or(expr);
                if let Some(value) = self.static_const_values.get(plain).copied() {
                    *arg = Type::NonType(value);
                } else if let Some(value) = self.eval_dependent_nontype(expr) {
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
    pub(super) fn resolve_static_const_nontype(
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
        let Ok(mut args) = super::template_deduce::parse_types_list(args_str, &known) else {
            return Ok(None);
        };
        // Expand alias and nested type arguments before instantiating the
        // owner, so `Trait<Alias<T>>::value` reads the right specialization.
        for arg in &mut args {
            if self.resolve_and_instantiate_type(arg, line, col).is_err() {
                return Ok(None);
            }
        }
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

    /// Resolve a nested type through the owner and, when absent there, its
    /// base classes. The search stops at a class that declares the name, so a
    /// derived member hides an inherited one.
    fn finish_nested_type_lookup(
        &self,
        owner: &str,
        member: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<(String, Option<Type>)>, CompileError> {
        let declarers = nested_type_declarers(self, owner, member);
        if declarers.len() > 1 {
            let paths: usize = declarers
                .iter()
                .map(|decl| count_base_paths(self, owner, decl, &mut HashSet::new()))
                .sum();
            if paths > 1 {
                let error = CompileError::new(
                    format!("ambiguous nested type '{}' in '{}'", member, owner),
                    line,
                    col,
                );
                return Err(if self.in_substitution_context {
                    error.with_kind(ErrorKind::Substitution)
                } else {
                    error
                });
            }
        }
        let Some(declarer) = declarers.first() else {
            return Ok(Some((owner.to_string(), None)));
        };
        let qualified = format!("{}::{}", declarer, member);
        let ty = self
            .lookup_type_alias(&qualified)
            .unwrap_or_else(|| Type::Struct(qualified.clone()));
        self.check_nested_type_access(&qualified, line, col)?;
        Ok(Some((declarer.clone(), Some(ty))))
    }

    /// Resolves a qualified name to its type. Missing nested types yield
    /// `None`; instantiation errors stay hard.
    pub(super) fn resolve_nested_type_name(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<(String, Option<Type>)>, CompileError> {
        // A nested member of a concrete specialization can have a stale alias
        // registered from a different definition (for example the primary).
        // Validate it against the selected definition below instead.
        if !name.contains(">::")
            && let Some(alias) = self.lookup_type_alias(name)
        {
            self.check_nested_type_access(name, line, col)?;
            return Ok(Some((name.to_string(), Some(alias))));
        }
        if !name.contains(">::")
            && let Some((owner, member)) = name.rsplit_once("::")
        {
            return self.finish_nested_type_lookup(owner, member, line, col);
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
        let member = &name[gt_pos + 3..];
        if let Some((owner, Some(ty))) = self.finish_nested_type_lookup(&inst, member, line, col)? {
            return Ok(Some((owner, Some(ty))));
        }
        if selected_class_definition(self, &inst)
            .is_some_and(|class| !declares_member_alias(&class, member))
        {
            return Ok(Some((inst, None)));
        }
        Ok(Some((inst, None)))
    }

    /// Resolves a delayed `sizeof` operand type, instantiating a qualified
    /// owner on demand when its nested type comes from a base class.
    pub(super) fn resolve_type_name_for_sizeof(
        &mut self,
        name: &str,
    ) -> Result<Option<Type>, CompileError> {
        Ok(self
            .resolve_nested_type_name(name, 0, 0)?
            .and_then(|(_, ty)| ty))
    }
}
