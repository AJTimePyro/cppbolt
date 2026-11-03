//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.

use crate::ast::decl::{Decl, FunctionDecl, Param};
use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::template::TemplateClass;
use crate::ast::types::Type;
use std::collections::HashMap;

pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) -> Type {
    match ty {
        Type::TypeParam(name) => mapping.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::Struct(name) => {
            if let Some((c, target_ty)) = injected_class {
                let short_c = c.rsplit_once("::").map(|(_, s)| s).unwrap_or(c);
                if (name == c
                    || name == short_c
                    || crate::QualifiedName::new(c).matches(crate::QualifiedName::new(name)))
                    && (!name.contains('<') || c.contains('<'))
                {
                    return target_ty.clone();
                }
            }
            if let Some(target) = mapping.get(name) {
                return target.clone();
            }
            if let Some((target, member)) = name
                .split_once("::")
                .and_then(|(base, member)| mapping.get(base).map(|target| (target, member)))
            {
                let base_str = target.to_c_type();
                return Type::Struct(format!("{}::{}", base_str, member));
            }
            if name.contains('<') {
                return Type::Struct(substitute_template_name(name, mapping));
            }
            ty.clone()
        }
        Type::TemplateInstance { name, args } => {
            let subst_args = args
                .iter()
                .map(|a| substitute_type(a, mapping, injected_class))
                .collect();
            Type::TemplateInstance {
                name: name.clone(),
                args: subst_args,
            }
        }
        Type::Const(inner) => {
            Type::Const(Box::new(substitute_type(inner, mapping, injected_class)))
        }
        Type::Pointer(inner) => {
            Type::Pointer(Box::new(substitute_type(inner, mapping, injected_class)))
        }
        Type::Reference(inner) => {
            Type::Reference(Box::new(substitute_type(inner, mapping, injected_class)))
        }
        Type::Array(inner, size) => Type::Array(
            Box::new(substitute_type(inner, mapping, injected_class)),
            *size,
        ),
        Type::Function {
            return_type,
            param_types,
        } => Type::Function {
            return_type: Box::new(substitute_type(return_type, mapping, injected_class)),
            param_types: param_types
                .iter()
                .map(|p| substitute_type(p, mapping, injected_class))
                .collect(),
        },
        _ => ty.clone(),
    }
}

pub fn substitute_expr(
    expr: &mut Expr,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    match expr {
        Expr::Call { callee, args, .. } => {
            if let Expr::Identifier(name) = &mut **callee {
                substitute_callee_identifier(name, mapping, injected_class);
            }
            substitute_expr(callee, mapping, injected_class);
            for arg in args {
                substitute_expr(arg, mapping, injected_class);
            }
        }
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. } => {
            substitute_expr(expr, mapping, injected_class);
        }
        Expr::Binary {
            left: target,
            right: value,
            ..
        }
        | Expr::Assign { target, value, .. }
        | Expr::CompoundAssign { target, value, .. } => {
            substitute_expr(target, mapping, injected_class);
            substitute_expr(value, mapping, injected_class);
        }
        Expr::ArraySubscript { array, index, .. } => {
            substitute_expr(array, mapping, injected_class);
            substitute_expr(index, mapping, injected_class);
        }
        Expr::MemberAccess { object, member, .. } => {
            substitute_expr(object, mapping, injected_class);
            substitute_member_name(member, mapping);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            for e in [cond, then_expr, else_expr] {
                substitute_expr(e, mapping, injected_class);
            }
        }
        Expr::Cast {
            target_type, expr, ..
        } => {
            *target_type = substitute_type(target_type, mapping, injected_class);
            substitute_expr(expr, mapping, injected_class);
        }
        Expr::SizeofType(ty) | Expr::TypedInitList { ty, .. } => {
            *ty = substitute_type(ty, mapping, injected_class);
        }
        Expr::New { .. } => substitute_new(expr, mapping, injected_class),
        Expr::InitializerList { elements, .. } => {
            for el in elements {
                substitute_expr(el, mapping, injected_class);
            }
        }
        _ => {}
    }
}

fn substitute_member_name(member: &mut String, mapping: &HashMap<String, Type>) {
    if let Some(rest) = member.strip_prefix('~')
        && let Some(ty) = mapping.get(rest)
    {
        let c_ty = ty.to_c_type();
        let short = c_ty.rsplit("::").next().unwrap_or(&c_ty);
        *member = format!("~{short}");
    }
}

fn substitute_new(
    expr: &mut Expr,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    let Expr::New {
        ty,
        args,
        array_size,
        placement,
        ..
    } = expr
    else {
        return;
    };
    *ty = substitute_type(ty, mapping, injected_class);
    for arg in args {
        substitute_expr(arg, mapping, injected_class);
    }
    if let Some(size) = array_size {
        substitute_expr(size, mapping, injected_class);
    }
    if let Some(place) = placement {
        substitute_expr(place, mapping, injected_class);
    }
}

fn substitute_callee_identifier(
    name: &mut String,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    if let Some((_, target_ty)) = injected_class.filter(|(c, _)| *c == name.as_str()) {
        *name = target_ty.to_c_type();
    } else if let Some(target) = mapping.get(name.as_str()) {
        *name = target.to_c_type();
    } else if let Some((target, member)) = name
        .split_once("::")
        .and_then(|(b, m)| mapping.get(b).map(|t| (t, m)))
    {
        *name = format!("{}::{}", target.to_c_type(), member);
    } else if name.contains('<') {
        *name = substitute_template_name(name, mapping);
    }
}

pub fn substitute_template_name(name: &str, mapping: &HashMap<String, Type>) -> String {
    let Some(open) = name.find('<') else {
        return name.to_string();
    };
    let Some(close) = name.rfind('>') else {
        return name.to_string();
    };
    if open >= close {
        return name.to_string();
    }
    let base = &name[..open];
    let inside = &name[open + 1..close];
    let suffix = &name[close + 1..];
    let mut result = inside.to_string();
    for (param, ty) in mapping {
        result = replace_whole_word(&result, param, &ty.to_c_type());
    }
    format!("{}<{}>{}", base, result, suffix)
}

fn replace_whole_word(text: &str, word: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut remainder = text;
    while let Some(idx) = remainder.find(word) {
        let bytes = remainder.as_bytes();
        let not_qualified = !(idx >= 2 && &bytes[idx - 2..idx] == b"::");
        let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let before_ok = not_qualified && (idx == 0 || !is_ident(bytes[idx - 1]));
        let after_idx = idx + word.len();
        let after_ok = after_idx == bytes.len() || !is_ident(bytes[after_idx]);
        if before_ok && after_ok {
            result.push_str(&remainder[..idx]);
            result.push_str(replacement);
        } else {
            result.push_str(&remainder[..after_idx]);
        }
        remainder = &remainder[after_idx..];
    }
    result.push_str(remainder);
    result
}

pub fn substitute_stmt(
    stmt: &mut Stmt,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    match &mut stmt.kind {
        StmtKind::Expr(expr) => substitute_expr(expr, mapping, injected_class),
        StmtKind::Compound(stmts) => {
            for s in stmts {
                substitute_stmt(s, mapping, injected_class);
            }
        }
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (cond, then_stmt) in branches {
                substitute_expr(cond, mapping, injected_class);
                substitute_stmt(then_stmt, mapping, injected_class);
            }
            if let Some(else_s) = else_stmt {
                substitute_stmt(else_s, mapping, injected_class);
            }
        }
        StmtKind::While { cond, body } => {
            substitute_expr(cond, mapping, injected_class);
            substitute_stmt(body, mapping, injected_class);
        }
        StmtKind::DoWhile { body, cond } => {
            substitute_stmt(body, mapping, injected_class);
            substitute_expr(cond, mapping, injected_class);
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            if let Some(init_s) = init {
                substitute_stmt(init_s, mapping, injected_class);
            }
            if let Some(cond_e) = cond {
                substitute_expr(cond_e, mapping, injected_class);
            }
            if let Some(step_e) = step {
                substitute_expr(step_e, mapping, injected_class);
            }
            substitute_stmt(body, mapping, injected_class);
        }
        StmtKind::Return(Some(expr)) => substitute_expr(expr, mapping, injected_class),
        StmtKind::VarDecl { ty, init, .. } => {
            *ty = substitute_type(ty, mapping, injected_class);
            if let Some(init_e) = init {
                substitute_expr(init_e, mapping, injected_class);
            }
        }
        StmtKind::VarDeclGroup(group) => {
            for s in group {
                substitute_stmt(s, mapping, injected_class);
            }
        }
        _ => {}
    }
}

fn substitute_func_name(func_name: &str, injected_class: Option<(&str, &Type)>) -> String {
    if let Some((cname, target_ty)) = injected_class {
        let short_cname = cname.rsplit_once("::").map(|(_, s)| s).unwrap_or(cname);
        if func_name == cname || func_name == short_cname {
            return target_ty.to_c_type();
        }
        if func_name == format!("~{}", cname) || func_name == format!("~{}", short_cname) {
            return format!("~{}", target_ty.to_c_type());
        }
    }
    func_name.to_string()
}

pub fn substitute_function(
    func: &FunctionDecl,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
    new_name: Option<String>,
) -> FunctionDecl {
    let mut remaining_template_params = Vec::new();
    let mut active_mapping = mapping.clone();
    for p in &func.template_params {
        if injected_class.is_some() {
            remaining_template_params.push(p.clone());
            active_mapping.remove(&p.name);
        } else if !mapping.contains_key(&p.name) {
            remaining_template_params.push(p.clone());
        }
    }
    let mapping = &active_mapping;
    let name = new_name.unwrap_or_else(|| substitute_func_name(&func.name, injected_class));

    let return_type = substitute_type(&func.return_type, mapping, injected_class);
    let params = func
        .params
        .iter()
        .map(|p| {
            let mut default_val = p.default_val.clone();
            if let Some(def) = &mut default_val {
                substitute_expr(def, mapping, injected_class);
            }
            Param {
                name: p.name.clone(),
                ty: substitute_type(&p.ty, mapping, injected_class),
                default_val,
            }
        })
        .collect();

    let ctor_initializers = func
        .ctor_initializers
        .iter()
        .map(|(m, inits)| {
            let mut subst_inits = inits.clone();
            for init in &mut subst_inits {
                substitute_expr(init, mapping, injected_class);
            }
            (m.clone(), subst_inits)
        })
        .collect();

    let mut body = func.body.clone();
    if let Some(body_stmt) = &mut body {
        substitute_stmt(body_stmt, mapping, injected_class);
    }

    FunctionDecl {
        name,
        return_type,
        params,
        body,
        is_virtual: func.is_virtual,
        is_pure_virtual: func.is_pure_virtual,
        is_const: func.is_const,
        is_explicit: func.is_explicit,
        ref_qualifier: func.ref_qualifier,
        is_static: func.is_static,
        ctor_initializers,
        template_params: remaining_template_params,
        line: func.line,
        col: func.col,
    }
}

pub fn substitute_struct(
    template_class: &TemplateClass,
    mapping: &HashMap<String, Type>,
    instantiated_name: &str,
    line: usize,
    col: usize,
) -> Decl {
    let target_ty = Type::Struct(instantiated_name.to_string());
    let injected = Some((template_class.name.as_str(), &target_ty));

    let fields = template_class
        .fields
        .iter()
        .map(|(fname, fty)| (fname.clone(), substitute_type(fty, mapping, injected)))
        .collect();

    let methods = template_class
        .methods
        .iter()
        .map(|m| substitute_function(m, mapping, injected, None))
        .collect();

    let static_fields = template_class
        .static_fields
        .iter()
        .map(|(sname, sty, sinit)| {
            let mut init = sinit.clone();
            if let Some(init_expr) = &mut init {
                substitute_expr(init_expr, mapping, injected);
            }
            (sname.clone(), substitute_type(sty, mapping, injected), init)
        })
        .collect();

    Decl::StructDecl {
        name: instantiated_name.to_string(),
        is_class: template_class.is_class,
        base_classes: template_class.base_classes.clone(),
        fields,
        methods,
        static_fields,
        private_members: template_class.private_members.clone(),
        nested_structs: Vec::new(),
        line,
        col,
    }
}
