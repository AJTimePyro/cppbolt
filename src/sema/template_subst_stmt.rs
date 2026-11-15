//! Statement-level template AST substitution.

use super::pack_expansion::{
    build_slice_mapping, collect_packs_in_expr, rename_pack_identifiers, validate_pack_lengths,
    TemplateArg,
};
use super::template_subst::{substitute_expr, substitute_type};
use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use std::collections::{HashMap, HashSet};

fn substitute_expr_stmt(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Option<StmtKind>, CompileError> {
    if let Expr::PackExpansion { pattern, pos } = expr {
        let known: HashSet<String> = mapping
            .iter()
            .filter(|(_, v)| v.is_pack())
            .map(|(k, _)| k.clone())
            .collect();
        let mut packs = HashSet::new();
        collect_packs_in_expr(pattern, &known, &mut packs);
        if packs.is_empty() {
            return Err(CompileError::new(
                "pack expansion does not contain any unexpanded parameter packs",
                pos.line,
                pos.col,
            ));
        }
        let len = validate_pack_lengths(&packs, mapping, *pos)?;
        let mut stmts = Vec::new();
        for i in 0..len {
            let subst_map = build_slice_mapping(mapping, &packs, i);
            let mut copy = (**pattern).clone();
            rename_pack_identifiers(&mut copy, &packs, mapping, i);
            substitute_expr(&mut copy, &subst_map, injected_class)?;
            stmts.push(Stmt::new(StmtKind::Expr(copy), pos.line, pos.col));
        }
        Ok(Some(StmtKind::Compound(stmts)))
    } else {
        substitute_expr(expr, mapping, injected_class)?;
        Ok(None)
    }
}

pub fn substitute_stmt(
    stmt: &mut Stmt,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    match &mut stmt.kind {
        StmtKind::Expr(expr) => {
            if let Some(new_kind) = substitute_expr_stmt(expr, mapping, injected_class)? {
                stmt.kind = new_kind;
            }
        }
        StmtKind::Case(expr, _) => {
            substitute_expr(expr, mapping, injected_class)?;
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                substitute_stmt(s, mapping, injected_class)?;
            }
        }
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (cond, then_stmt) in branches {
                substitute_expr(cond, mapping, injected_class)?;
                substitute_stmt(then_stmt, mapping, injected_class)?;
            }
            if let Some(else_s) = else_stmt {
                substitute_stmt(else_s, mapping, injected_class)?;
            }
        }
        StmtKind::While { .. }
        | StmtKind::DoWhile { .. }
        | StmtKind::RangeFor { .. }
        | StmtKind::For { .. } => {
            substitute_loop_stmt(&mut stmt.kind, mapping, injected_class)?;
        }
        StmtKind::StructuredBinding { ty, init, .. } => {
            *ty = substitute_type(ty, mapping, injected_class)?;
            substitute_expr(init, mapping, injected_class)?;
        }
        StmtKind::Switch { expr, body } => {
            substitute_expr(expr, mapping, injected_class)?;
            substitute_stmt(body, mapping, injected_class)?;
        }
        StmtKind::Return(Some(expr)) => substitute_expr(expr, mapping, injected_class)?,
        StmtKind::VarDecl { ty, init, .. } => {
            *ty = substitute_type(ty, mapping, injected_class)?;
            if let Some(init_e) = init {
                substitute_expr(init_e, mapping, injected_class)?;
            }
        }
        StmtKind::Return(None)
        | StmtKind::Break
        | StmtKind::Continue
        | StmtKind::Default
        | StmtKind::Empty
        | StmtKind::UsingNamespace(_)
        | StmtKind::UsingDecl(_)
        | StmtKind::NamespaceAlias { .. } => {}
    }
    Ok(())
}

fn substitute_loop_stmt(
    kind: &mut StmtKind,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    match kind {
        StmtKind::While { cond, body } => {
            substitute_expr(cond, mapping, injected_class)?;
            substitute_stmt(body, mapping, injected_class)?;
        }
        StmtKind::DoWhile { body, cond } => {
            substitute_stmt(body, mapping, injected_class)?;
            substitute_expr(cond, mapping, injected_class)?;
        }
        StmtKind::RangeFor { decl, range, body } => {
            substitute_stmt(decl, mapping, injected_class)?;
            substitute_expr(range, mapping, injected_class)?;
            substitute_stmt(body, mapping, injected_class)?;
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            if let Some(init_s) = init {
                substitute_stmt(init_s, mapping, injected_class)?;
            }
            if let Some(cond_e) = cond {
                substitute_expr(cond_e, mapping, injected_class)?;
            }
            if let Some(step_e) = step {
                substitute_expr(step_e, mapping, injected_class)?;
            }
            substitute_stmt(body, mapping, injected_class)?;
        }
        _ => {}
    }
    Ok(())
}
