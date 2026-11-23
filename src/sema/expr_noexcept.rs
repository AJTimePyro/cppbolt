//! C++17 `noexcept(expr)` evaluation and potentially-throwing expression analysis.

use super::Sema;
use crate::ast::expr::Expr;
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashMap;

impl Sema {
    /// Determines whether an expression is potentially-throwing under C++17 [except.spec].
    pub fn is_potentially_throwing(&self, expr: &Expr) -> bool {
        self.is_potentially_throwing_with_types(expr, None)
    }

    /// Determines whether an expression is potentially-throwing, with optional local variable types.
    pub fn is_potentially_throwing_with_types(
        &self,
        expr: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        match expr {
            Expr::Throw(..) | Expr::New { .. } => true,
            Expr::Noexcept(..)
            | Expr::IntLiteral(..)
            | Expr::FloatLiteral(..)
            | Expr::BoolLiteral(..)
            | Expr::CharLiteral(..)
            | Expr::StringLiteral(..)
            | Expr::This
            | Expr::Identifier(..)
            | Expr::SizeofExpr(..)
            | Expr::SizeofType(..)
            | Expr::AlignofType(..)
            | Expr::SizeofPack { .. }
            | Expr::Lambda { .. } => false,
            Expr::Cast { expr: inner, .. }
            | Expr::PackExpansion { pattern: inner, .. }
            | Expr::Delete { expr: inner, .. } => {
                self.is_potentially_throwing_with_types(inner, var_types)
            }
            Expr::Unary {
                op, expr: inner, ..
            } => self.unary_is_potentially_throwing(*op, inner, var_types),
            Expr::Binary {
                op, left, right, ..
            } => self.binary_is_potentially_throwing(op.op_name(), left, right, var_types),
            Expr::ArraySubscript { array, index, .. } => {
                self.subscript_is_potentially_throwing(array, index, var_types)
            }
            Expr::Assign {
                target: left,
                value: right,
                ..
            }
            | Expr::CompoundAssign {
                target: left,
                value: right,
                ..
            } => {
                self.is_potentially_throwing_with_types(left, var_types)
                    || self.is_potentially_throwing_with_types(right, var_types)
            }
            Expr::MemberAccess { object, .. } => {
                self.is_potentially_throwing_with_types(object, var_types)
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.is_potentially_throwing_with_types(cond, var_types)
                    || self.is_potentially_throwing_with_types(then_expr, var_types)
                    || self.is_potentially_throwing_with_types(else_expr, var_types)
            }
            Expr::InitializerList { .. }
            | Expr::TypedInitList { .. }
            | Expr::FoldExpr { .. }
            | Expr::Call { .. } => self.is_complex_expr_potentially_throwing(expr, var_types),
        }
    }

    fn subscript_is_potentially_throwing(
        &self,
        array: &Expr,
        index: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        if self.is_potentially_throwing_with_types(array, var_types)
            || self.is_potentially_throwing_with_types(index, var_types)
        {
            return true;
        }
        if self.resolve_object_struct_name(array, var_types).is_none() {
            return false;
        }
        self.operator_is_potentially_throwing("operator[]", &[array, index], var_types)
    }

    fn unary_is_potentially_throwing(
        &self,
        op: crate::ast::expr_ops::UnaryOp,
        inner: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        if self.is_potentially_throwing_with_types(inner, var_types) {
            return true;
        }
        let Some(name) = unary_operator_name(op) else {
            return false;
        };
        if self.resolve_object_struct_name(inner, var_types).is_none() {
            return false;
        }
        self.operator_is_potentially_throwing(name, &[inner], var_types)
    }

    fn binary_is_potentially_throwing(
        &self,
        name: Option<&str>,
        left: &Expr,
        right: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        if self.is_potentially_throwing_with_types(left, var_types)
            || self.is_potentially_throwing_with_types(right, var_types)
        {
            return true;
        }
        let Some(name) = name else {
            return false;
        };
        if self.resolve_object_struct_name(left, var_types).is_none()
            && self.resolve_object_struct_name(right, var_types).is_none()
        {
            return false;
        }
        self.operator_is_potentially_throwing(name, &[left, right], var_types)
    }

    fn is_complex_expr_potentially_throwing(
        &self,
        expr: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        match expr {
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                elements
                    .iter()
                    .any(|e| self.is_potentially_throwing_with_types(e, var_types))
            }
            Expr::FoldExpr { left, right, .. } => {
                left.as_ref()
                    .is_some_and(|e| self.is_potentially_throwing_with_types(e, var_types))
                    || right
                        .as_ref()
                        .is_some_and(|e| self.is_potentially_throwing_with_types(e, var_types))
            }
            Expr::Call { callee, args, .. } => {
                if args
                    .iter()
                    .any(|arg| self.is_potentially_throwing_with_types(arg, var_types))
                {
                    return true;
                }
                if self.is_potentially_throwing_with_types(callee, var_types) {
                    return true;
                }
                self.is_callee_potentially_throwing(callee, args.len(), var_types)
            }
            _ => true,
        }
    }

    /// Checks if a call target (direct function, function pointer, or method) can throw.
    fn is_callee_potentially_throwing(
        &self,
        callee: &Expr,
        arg_count: usize,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        match callee {
            Expr::Identifier(name) => {
                self.is_identifier_callee_potentially_throwing(name, arg_count, var_types)
            }
            Expr::MemberAccess { object, member, .. } => {
                self.is_member_callee_potentially_throwing(object, member, arg_count, var_types)
            }
            _ => true,
        }
    }

    fn is_identifier_callee_potentially_throwing(
        &self,
        name: &str,
        arg_count: usize,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        let q2 = QualifiedName::new(name).alternate().into_owned();
        let candidates = self
            .functions
            .get(name)
            .or_else(|| self.functions.get(&q2))
            .cloned()
            .unwrap_or_default();
        if !candidates.is_empty() {
            let matching: Vec<_> = candidates
                .into_iter()
                .filter(|sig| sig.param_types.len() >= arg_count)
                .collect();
            if matching.is_empty() {
                return true;
            }
            return matching.iter().any(|sig| !sig.qualifiers.is_noexcept);
        }
        if let Some(ty) = var_types.and_then(|vt| vt.get(name)) {
            let inner_fn = match ty {
                Type::Pointer(inner) | Type::Reference(inner) => inner.as_ref(),
                other => other,
            };
            if let Type::Function { qualifiers, .. } = inner_fn {
                return !qualifiers.is_noexcept;
            }
        }
        if let Some(var) = self.lookup_var(name).or_else(|| self.global_vars.get(name)) {
            let inner_fn = match &var.ty {
                Type::Pointer(inner) | Type::Reference(inner) => inner.as_ref(),
                other => other,
            };
            if let Type::Function { qualifiers, .. } = inner_fn {
                return !qualifiers.is_noexcept;
            }
        }
        true
    }

    fn is_member_callee_potentially_throwing(
        &self,
        object: &Expr,
        member: &str,
        arg_count: usize,
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        let struct_name = self.resolve_object_struct_name(object, var_types);
        if let Some(sname) = struct_name {
            let overloads = self.find_method_overloads_in_class(&sname, member, false);
            if !overloads.is_empty() {
                let matching: Vec<_> = overloads
                    .into_iter()
                    .filter(|sig| {
                        let expected = if sig.is_static {
                            arg_count
                        } else {
                            arg_count + 1
                        };
                        sig.param_types.len() == expected
                    })
                    .collect();
                if !matching.is_empty() {
                    return matching.iter().any(|sig| !sig.qualifiers.is_noexcept);
                }
            }
        }
        true
    }

    fn operator_is_potentially_throwing(
        &self,
        name: &str,
        operands: &[&Expr],
        var_types: Option<&HashMap<String, Type>>,
    ) -> bool {
        if let Some(sname) = operands
            .first()
            .and_then(|e| self.resolve_object_struct_name(e, var_types))
            && let Some(sig) = self.find_method_in_class(&sname, name)
        {
            return !sig.qualifiers.is_noexcept;
        }
        for candidate in [name.to_string(), format!("std::{name}")] {
            let Some(sigs) = self.functions.get(&candidate) else {
                continue;
            };
            let matching: Vec<_> = sigs
                .iter()
                .filter(|sig| sig.param_types.len() == operands.len())
                .collect();
            if !matching.is_empty() {
                return matching.iter().any(|sig| !sig.qualifiers.is_noexcept);
            }
        }
        false
    }

    fn resolve_object_struct_name(
        &self,
        object: &Expr,
        var_types: Option<&HashMap<String, Type>>,
    ) -> Option<String> {
        match object {
            Expr::Identifier(obj_name) => var_types
                .and_then(|vt| vt.get(obj_name))
                .and_then(extract_struct_name)
                .or_else(|| {
                    self.lookup_var(obj_name)
                        .or_else(|| self.global_vars.get(obj_name))
                        .and_then(|v| extract_struct_name(&v.ty))
                }),
            Expr::This => self.current_class.clone(),
            Expr::Cast { target_type, .. } => extract_struct_name(target_type),
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(fn_name) = callee.as_ref() {
                    self.functions
                        .get(fn_name)
                        .and_then(|sigs| sigs.first())
                        .and_then(|sig| extract_struct_name(&sig.return_type))
                } else {
                    None
                }
            }
            Expr::Unary {
                op: crate::ast::expr_ops::UnaryOp::Deref,
                expr,
                ..
            } => self.resolve_object_struct_name(expr, var_types),
            _ => None,
        }
    }
}

fn unary_operator_name(op: crate::ast::expr_ops::UnaryOp) -> Option<&'static str> {
    use crate::ast::expr_ops::UnaryOp;
    Some(match op {
        UnaryOp::Not => "operator!",
        UnaryOp::Neg => "operator-",
        UnaryOp::BitNot => "operator~",
        UnaryOp::Plus => "operator+",
        UnaryOp::AddrOf => "operator&",
        UnaryOp::Deref => "operator*",
        UnaryOp::PreInc | UnaryOp::PostInc => "operator++",
        UnaryOp::PreDec | UnaryOp::PostDec => "operator--",
    })
}

fn extract_struct_name(ty: &Type) -> Option<String> {
    match ty.strip_reference() {
        Type::Struct(name) => Some(name.clone()),
        Type::Pointer(inner) => match inner.strip_reference() {
            Type::Struct(name) => Some(name.clone()),
            _ => None,
        },
        _ => None,
    }
}
