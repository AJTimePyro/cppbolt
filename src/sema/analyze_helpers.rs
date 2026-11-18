//! Helpers for program analysis.

use super::*;


pub(super) fn method_matches_out_of_class(
    m: &FunctionDecl,
    short_method: &str,
    params: &[Param],
    is_const: bool,
) -> bool {
    (m.name == short_method || m.name == format!("~{}", short_method))
        && m.params.len() == params.len()
        && m.is_const == is_const
        && m.body.is_none()
        && m.params.iter().zip(params).all(|(p1, p2)| p1.ty == p2.ty)
}

/// Collects class names used as complete objects (never through a pointer or
/// reference). Only those odr-use a destructor; Clang leaves a specialization
/// reached solely through a pointer uninstantiated.
pub(super) fn collect_object_type(ty: &Type, out: &mut HashSet<String>) {
    match ty {
        Type::Struct(s) => {
            out.insert(s.clone());
        }
        Type::TemplateInstance { name, args } => {
            out.insert(format!(
                "{}<{}>",
                name,
                args.iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Type::Const(inner) | Type::Array(inner, _) => collect_object_type(inner, out),
        _ => {}
    }
}

pub(super) fn collect_object_expr(expr: &Expr, out: &mut HashSet<String>) {
    if let Expr::New { ty, .. } = expr {
        collect_object_type(ty, out);
    }
}

pub(super) fn collect_object_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::VarDecl { ty, init, .. } => {
            collect_object_type(ty, out);
            if let Some(e) = init {
                collect_object_expr(e, out);
            }
        }
        StmtKind::StructuredBinding { ty, init, .. } => {
            collect_object_type(ty, out);
            collect_object_expr(init, out);
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                collect_object_stmt(s, out);
            }
        }
        StmtKind::Expr(e) => collect_object_expr(e, out),
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (_, s) in branches {
                collect_object_stmt(s, out);
            }
            if let Some(s) = else_stmt {
                collect_object_stmt(s, out);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Switch { body, .. } => collect_object_stmt(body, out),
        StmtKind::RangeFor { decl, body, .. } => {
            collect_object_stmt(decl, out);
            collect_object_stmt(body, out);
        }
        _ => {}
    }
}

pub(super) fn collect_object_use_classes(program: &Program) -> HashSet<String> {
    let mut out = HashSet::new();
    for decl in &program.declarations {
        match decl {
            Decl::GlobalVar { ty, .. } => collect_object_type(ty, &mut out),
            Decl::Function(f) => {
                if let Some(body) = &f.body {
                    collect_object_stmt(body, &mut out);
                }
            }
            Decl::StructDecl {
                base_classes,
                fields,
                static_fields,
                ..
            } => {
                out.extend(base_classes.iter().cloned());
                for (_, ty, _) in fields {
                    collect_object_type(ty, &mut out);
                }
                for (_, ty, _) in static_fields {
                    collect_object_type(ty, &mut out);
                }
            }
            _ => {}
        }
    }
    out
}

