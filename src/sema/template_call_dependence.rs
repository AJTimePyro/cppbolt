//! Classifies calls in an uninstantiated template as dependent or not.
//!
//! Substitution erases template parameter names before a body is analyzed, so
//! the decision is made here, on the original AST, and kept by source position.

use crate::ast::decl::FunctionDecl;
use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use std::collections::HashSet;

struct CallScan {
    names: HashSet<String>,
    call_positions: HashSet<(usize, usize)>,
}

impl CallScan {
    fn ty(&self, ty: &Type) -> bool {
        match ty {
            Type::TypeParam(name) => self.names.contains(name),
            Type::Const(inner)
            | Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::RvalueReference(inner)
            | Type::Array(inner, _)
            | Type::DependentArray(inner, _)
            | Type::PackExpansion(inner) => self.ty(inner),
            Type::TemplateInstance { name, args } => {
                self.names.contains(name) || args.iter().any(|arg| self.ty(arg))
            }
            Type::Struct(name) | Type::Enum(name) => self.names.contains(name),
            Type::DependentNested(name) | Type::DependentNonType(name) => {
                self.names.iter().any(|n| name.contains(n))
            }
            _ => false,
        }
    }

    /// Returns whether the expression mentions a dependent name. Any call in
    /// it is recorded when one of its arguments or its callee does.
    fn expr(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(name) => self.names.contains(name),
            Expr::Binary { left, right, .. } => self.expr(left) | self.expr(right),
            Expr::Unary { expr, .. }
            | Expr::SizeofExpr(expr)
            | Expr::Delete { expr, .. }
            | Expr::Cast { expr, .. }
            | Expr::MemberAccess { object: expr, .. } => self.expr(expr),
            Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
                self.expr(target) | self.expr(value)
            }
            Expr::Call {
                callee, args, pos, ..
            } => {
                let dependent =
                    self.expr(callee) | args.iter().map(|a| self.expr(a)).fold(false, |a, b| a | b);
                if dependent {
                    self.call_positions.insert((pos.line, pos.col));
                }
                dependent
            }
            Expr::ArraySubscript { array, index, .. } => self.expr(array) | self.expr(index),
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => self.expr(cond) | self.expr(then_expr) | self.expr(else_expr),
            Expr::InitializerList { elements, .. } => elements
                .iter()
                .map(|e| self.expr(e))
                .fold(false, |a, b| a | b),
            Expr::TypedInitList { ty, elements, .. } => {
                self.ty(ty)
                    | elements
                        .iter()
                        .map(|e| self.expr(e))
                        .fold(false, |a, b| a | b)
            }
            Expr::New {
                ty,
                args,
                array_size,
                placement,
                ..
            } => {
                self.ty(ty)
                    | args.iter().map(|a| self.expr(a)).fold(false, |a, b| a | b)
                    | array_size.as_ref().is_some_and(|e| self.expr(e))
                    | placement.as_ref().is_some_and(|e| self.expr(e))
            }
            Expr::PackExpansion { pattern, .. } => self.expr(pattern),
            Expr::FoldExpr { left, right, .. } => {
                left.as_ref().is_some_and(|e| self.expr(e))
                    | right.as_ref().is_some_and(|e| self.expr(e))
            }
            Expr::Lambda {
                params,
                return_type,
                body,
                ..
            } => {
                let dependent = params.iter().any(|p| self.ty(&p.ty))
                    || return_type.as_ref().is_some_and(|ty| self.ty(ty));
                self.stmt(body);
                dependent
            }
            Expr::SizeofType(ty) | Expr::AlignofType(ty) => self.ty(ty),
            _ => false,
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
                for s in stmts {
                    self.stmt(s);
                }
            }
            StmtKind::Expr(expr)
            | StmtKind::StaticAssert {
                condition: expr, ..
            } => {
                self.expr(expr);
            }
            StmtKind::VarDecl { name, ty, init, .. } => {
                if self.ty(ty) {
                    self.names.insert(name.clone());
                }
                if let Some(expr) = init {
                    self.expr(expr);
                }
            }
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                for (cond, body) in branches {
                    self.expr(cond);
                    self.stmt(body);
                }
                if let Some(body) = else_stmt {
                    self.stmt(body);
                }
            }
            StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
                self.expr(cond);
                self.stmt(body);
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(init) = init {
                    self.stmt(init);
                }
                if let Some(cond) = cond {
                    self.expr(cond);
                }
                if let Some(step) = step {
                    self.expr(step);
                }
                self.stmt(body);
            }
            StmtKind::RangeFor { decl, range, body } => {
                self.stmt(decl);
                self.expr(range);
                self.stmt(body);
            }
            StmtKind::StructuredBinding { ty, init, .. } => {
                self.ty(ty);
                self.expr(init);
            }
            StmtKind::Switch { expr, body } => {
                self.expr(expr);
                self.stmt(body);
            }
            StmtKind::Case(expr, _) => {
                self.expr(expr);
            }
            StmtKind::Return(Some(expr)) => {
                self.expr(expr);
            }
            StmtKind::TypeAlias { ty, .. } => {
                self.ty(ty);
            }
            _ => {}
        }
    }
}

/// Positions of calls whose arguments mention a template parameter, the
/// injected class name of the enclosing template, or a local whose type does.
/// Those calls get instantiation-time ADL.
pub(crate) fn dependent_call_positions(
    decl: &FunctionDecl,
    template_params: &[TemplateParam],
    class_name: Option<&str>,
) -> HashSet<(usize, usize)> {
    let mut names: HashSet<String> = template_params.iter().map(|p| p.name.clone()).collect();
    if let Some(name) = class_name {
        names.insert(name.to_string());
        if let Some(short) = name.rsplit("::").next() {
            names.insert(short.to_string());
        }
    }
    let mut scan = CallScan {
        names,
        call_positions: HashSet::new(),
    };
    let dependent_params: Vec<String> = decl
        .params
        .iter()
        .filter(|param| scan.ty(&param.ty))
        .map(|param| param.name.clone())
        .collect();
    for name in dependent_params {
        scan.names.insert(name);
    }
    if let Type::Decltype { expr, .. } = &decl.return_type {
        scan.expr(expr);
    }
    for param in &decl.params {
        if let Type::Decltype { expr, .. } = &param.ty {
            scan.expr(expr);
        }
    }
    if let Some(body) = &decl.body {
        scan.stmt(body);
    }
    for (_, init) in &decl.ctor_initializers {
        for expr in init {
            scan.expr(expr);
        }
    }
    scan.call_positions
}
