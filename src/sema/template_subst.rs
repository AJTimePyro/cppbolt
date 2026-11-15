//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.

use crate::ast::decl::Decl;
use crate::ast::expr::Expr;
use crate::ast::template::TemplateClass;
use crate::ast::types::Type;
use crate::sema::pack_expansion::{expand_expr_list, expand_type_list, TemplateArg};
pub use crate::sema::template_subst_func::substitute_function;
pub use crate::sema::template_subst_stmt::substitute_stmt;
use std::collections::HashMap;

pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Type {
    match ty {
        Type::TypeParam(name) => match mapping.get(name) {
            Some(TemplateArg::Type(t)) => t.clone(),
            Some(TemplateArg::NonType(n)) => Type::NonType(*n),
            _ => ty.clone(),
        },
        Type::Struct(name) => substitute_struct_type(name, mapping, injected_class),
        Type::TemplateInstance { name, args } => {
            let subst_args = expand_type_list(args, mapping).unwrap_or_else(|_| {
                args.iter().map(|a| substitute_type(a, mapping, injected_class)).collect()
            });
            Type::TemplateInstance {
                name: name.clone(),
                args: subst_args,
            }
        }
        Type::Const(inner) => {
            let subst_inner = substitute_type(inner, mapping, injected_class);
            if subst_inner.is_reference() {
                subst_inner
            } else {
                Type::Const(Box::new(subst_inner))
            }
        }
        Type::Pointer(inner) => {
            Type::Pointer(Box::new(substitute_type(inner, mapping, injected_class)))
        }
        Type::Reference(inner) => {
            collapse_reference(false, substitute_type(inner, mapping, injected_class))
        }
        Type::RvalueReference(inner) => {
            collapse_reference(true, substitute_type(inner, mapping, injected_class))
        }
        Type::Array(inner, size) => Type::Array(
            Box::new(substitute_type(inner, mapping, injected_class)),
            *size,
        ),
        Type::DependentArray(inner, param_name) => {
            let subst_inner = substitute_type(inner, mapping, injected_class);
            let size = match mapping.get(param_name) {
                Some(TemplateArg::NonType(n)) if *n >= 0 => *n as usize,
                Some(TemplateArg::NonType(_)) => usize::MAX,
                _ => 0,
            };
            Type::Array(Box::new(subst_inner), size)
        }
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
        Type::PackExpansion(inner) => {
            Type::PackExpansion(Box::new(substitute_type(inner, mapping, injected_class)))
        }
        _ => ty.clone(),
    }
}

fn substitute_struct_type(
    name: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Type {
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
    if let Some(TemplateArg::Type(target)) = mapping.get(name) {
        return target.clone();
    }
    if let Some((target, member)) = name
        .split_once("::")
        .and_then(|(base, member)| mapping.get(base).and_then(|t| t.as_type()).map(|t| (t, member)))
    {
        let base_str = target.to_c_type();
        return Type::Struct(format!("{}::{}", base_str, member));
    }
    if name.contains('<') {
        return Type::Struct(substitute_template_name(name, mapping));
    }
    Type::Struct(name.to_string())
}

fn substitute_call(
    callee: &mut Expr,
    args: &mut Vec<Expr>,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) {
    if let Expr::Identifier(name) = callee {
        substitute_callee_identifier(name, mapping, injected_class);
    }
    substitute_expr(callee, mapping, injected_class);
    if let Ok(expanded) = expand_expr_list(args, mapping) {
        *args = expanded;
    } else {
        for arg in args {
            substitute_expr(arg, mapping, injected_class);
        }
    }
}

fn substitute_init_elements(
    elements: &mut Vec<Expr>,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) {
    if let Ok(expanded) = expand_expr_list(elements, mapping) {
        *elements = expanded;
    } else {
        for el in elements {
            substitute_expr(el, mapping, injected_class);
        }
    }
}

fn substitute_identifier(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
) {
    let Expr::Identifier(name) = expr else {
        return;
    };
    if let Some(TemplateArg::NonType(val)) = mapping.get(name) {
        let ty = if (*val as i32 as i128) == *val {
            Type::Int
        } else {
            Type::LongLong
        };
        *expr = Expr::IntLiteral(*val as i64, ty);
    } else if name.contains('<') {
        *name = substitute_template_name(name, mapping);
    }
}

pub fn substitute_expr(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) {
    match expr {
        Expr::Call { callee, args, .. } => substitute_call(callee, args, mapping, injected_class),
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. } => {
            substitute_expr(expr, mapping, injected_class);
        }
        Expr::Binary { left: t, right: v, .. }
        | Expr::Assign { target: t, value: v, .. }
        | Expr::CompoundAssign { target: t, value: v, .. } => {
            substitute_expr(t, mapping, injected_class);
            substitute_expr(v, mapping, injected_class);
        }
        Expr::ArraySubscript { array, index, .. } => {
            substitute_expr(array, mapping, injected_class);
            substitute_expr(index, mapping, injected_class);
        }
        Expr::MemberAccess { object, member, .. } => {
            substitute_expr(object, mapping, injected_class);
            substitute_member_name(member, mapping);
        }
        Expr::Ternary { cond, then_expr, else_expr } => {
            for e in [cond, then_expr, else_expr] {
                substitute_expr(e, mapping, injected_class);
            }
        }
        Expr::Cast { target_type, expr, .. } => {
            *target_type = substitute_type(target_type, mapping, injected_class);
            substitute_expr(expr, mapping, injected_class);
        }
        Expr::SizeofType(ty) => *ty = substitute_type(ty, mapping, injected_class),
        Expr::TypedInitList { ty, elements, .. } => {
            *ty = substitute_type(ty, mapping, injected_class);
            substitute_init_elements(elements, mapping, injected_class);
        }
        Expr::New { .. } => substitute_new(expr, mapping, injected_class),
        Expr::InitializerList { elements, .. } => {
            substitute_init_elements(elements, mapping, injected_class);
        }
        Expr::Identifier(_) => substitute_identifier(expr, mapping),
        Expr::SizeofPack { name, .. } => {
            if let Some(len) = mapping.get(name).and_then(|a| a.pack_len()) {
                *expr = Expr::IntLiteral(len as i64, Type::UnsignedInt);
            }
        }
        Expr::PackExpansion { pattern, .. } => {
            substitute_expr(pattern, mapping, injected_class);
        }
        _ => {}
    }
}

fn substitute_member_name(member: &mut String, mapping: &HashMap<String, TemplateArg>) {
    if let Some(rest) = member.strip_prefix('~')
        && let Some(TemplateArg::Type(ty)) = mapping.get(rest)
    {
        let c_ty = ty.to_c_type();
        let short = c_ty.rsplit("::").next().unwrap_or(&c_ty);
        *member = format!("~{short}");
    }
}

fn substitute_new(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
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
    if let Ok(expanded) = expand_expr_list(args, mapping) {
        *args = expanded;
    } else {
        for arg in args {
            substitute_expr(arg, mapping, injected_class);
        }
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
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) {
    if let Some((_, target_ty)) = injected_class.filter(|(c, _)| *c == name.as_str()) {
        *name = target_ty.to_c_type();
    } else if let Some(TemplateArg::Type(target)) = mapping.get(name.as_str()) {
        *name = target.to_c_type();
    } else if let Some((target, member)) = name
        .split_once("::")
        .and_then(|(b, m)| mapping.get(b).and_then(|t| t.as_type()).map(|t| (t, m)))
    {
        *name = format!("{}::{}", target.to_c_type(), member);
    } else if name.contains('<') {
        *name = substitute_template_name(name, mapping);
    }
}

pub fn substitute_template_name(name: &str, mapping: &HashMap<String, TemplateArg>) -> String {
    let (Some(open), Some(close)) = (name.find('<'), name.rfind('>')) else {
        return name.to_string();
    };
    if open >= close {
        return name.to_string();
    }
    let (base, inside, suffix) = (&name[..open], &name[open + 1..close], &name[close + 1..]);
    let mut result = inside.to_string();
    for (param, arg) in mapping {
        match arg {
            TemplateArg::Type(ty) => {
                result = replace_whole_word(&result, param, &ty.to_c_type());
            }
            TemplateArg::NonType(val) => {
                result = replace_whole_word(&result, param, &val.to_string());
            }
            TemplateArg::TypePack(types) => {
                let joined = types.iter().map(|t| t.to_c_type()).collect::<Vec<_>>().join(", ");
                let with_ellipsis = format!("{param}...");
                let with_space = format!("{param} ...");
                if result.contains(&with_ellipsis) {
                    result = result.replace(&with_ellipsis, &joined);
                } else if result.contains(&with_space) {
                    result = result.replace(&with_space, &joined);
                } else {
                    result = replace_whole_word(&result, param, &joined);
                }
            }
            TemplateArg::NonTypePack(vals) => {
                let joined = vals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                let with_ellipsis = format!("{param}...");
                let with_space = format!("{param} ...");
                if result.contains(&with_ellipsis) {
                    result = result.replace(&with_ellipsis, &joined);
                } else if result.contains(&with_space) {
                    result = result.replace(&with_space, &joined);
                } else {
                    result = replace_whole_word(&result, param, &joined);
                }
            }
        }
    }
    let clean = clean_template_arg_string(&result);
    format!("{}<{}>{}", base, clean, suffix)
}

fn clean_template_arg_string(s: &str) -> String {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '<' => {
                depth += 1;
                current.push(c);
            }
            '>' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    parts.join(", ")
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

pub fn substitute_struct(
    template_class: &TemplateClass,
    mapping: &HashMap<String, TemplateArg>,
    instantiated_name: &str,
    line: usize,
    col: usize,
) -> Decl {
    let target_ty = Type::Struct(instantiated_name.to_string());
    let injected = Some((template_class.name.as_str(), &target_ty));

    let fields = template_class
        .fields
        .iter()
        .map(|(fname, fty, finit)| {
            let mut init = finit.clone();
            if let Some((init_expr, _, _)) = &mut init {
                substitute_expr(init_expr, mapping, injected);
            }
            (fname.clone(), substitute_type(fty, mapping, injected), init)
        })
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

    let mut base_classes = Vec::new();
    for base in &template_class.base_classes {
        if let Some(stripped) = base.strip_suffix("...") {
            if let Some(TemplateArg::TypePack(types)) = mapping.get(stripped) {
                for ty in types {
                    base_classes.push(ty.to_c_type());
                }
            } else {
                base_classes.push(substitute_template_name(base, mapping));
            }
        } else if let Some(TemplateArg::Type(target)) = mapping.get(base) {
            base_classes.push(target.to_c_type());
        } else if base.contains('<') {
            base_classes.push(substitute_template_name(base, mapping));
        } else {
            base_classes.push(base.clone());
        }
    }

    Decl::StructDecl {
        name: instantiated_name.to_string(),
        is_class: template_class.is_class,
        base_classes,
        fields,
        methods,
        static_fields,
        private_members: template_class.private_members.clone(),
        nested_structs: Vec::new(),
        line,
        col,
    }
}

pub fn collapse_reference(outer_is_rvalue: bool, inner: Type) -> Type {
    if inner.is_reference() {
        let is_lvalue = inner.is_lvalue_reference();
        let target = inner.deref_type().cloned().unwrap_or(inner);
        if !outer_is_rvalue || is_lvalue {
            Type::Reference(Box::new(target))
        } else {
            Type::RvalueReference(Box::new(target))
        }
    } else if outer_is_rvalue {
        Type::RvalueReference(Box::new(inner))
    } else {
        Type::Reference(Box::new(inner))
    }
}
