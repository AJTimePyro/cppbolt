//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.
pub use super::template_subst_expr_helpers::substitute_expr;
pub use super::template_subst_func::SubstContext;

use crate::ast::decl::Decl;
use crate::ast::expr::Expr;
use crate::ast::template::TemplateClass;
use crate::ast::types::{FunctionQualifiers, Type};
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::{TemplateArg, expand_type_list};
pub use crate::sema::template_name::substitute_template_name;
pub use crate::sema::template_subst_func::{substitute_function, substitute_function_with_sema};
use std::collections::HashMap;

pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    let mut ctx = SubstContext {
        mapping,
        injected_class,
        pos: (0, 0),
        struct_aligns: None,
        sema: None,
    };
    substitute_type_full(ty, &mut ctx)
}

/// `substitute_type` with Sema-backed resolution for dependent bounds.
pub fn substitute_type_with_sema(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    sema: Option<&mut crate::sema::Sema>,
) -> Result<Type, CompileError> {
    let mut ctx = SubstContext {
        mapping,
        injected_class,
        pos: (0, 0),
        struct_aligns: None,
        sema,
    };
    substitute_type_full(ty, &mut ctx)
}
fn substitute_template_instance(
    name: &str,
    args: &[Type],
    mapping: &HashMap<String, TemplateArg>,
    pos: (usize, usize),
) -> Result<Type, CompileError> {
    let subst_name = if name.contains('<') {
        substitute_template_name(name, mapping)
    } else {
        name.to_string()
    };
    Ok(Type::TemplateInstance {
        name: subst_name,
        args: expand_type_list(args, mapping, pos)?,
    })
}

pub fn substitute_type_full(ty: &Type, ctx: &mut SubstContext<'_>) -> Result<Type, CompileError> {
    if let Some(res) = substitute_modified_type(ty, ctx)? {
        return Ok(res);
    }
    match ty {
        Type::TypeParam(name) => {
            super::template_subst_align::substitute_type_param(name, ty, ctx.mapping, ctx.pos)
        }
        Type::Struct(name) => Ok(substitute_struct_type(
            name,
            ctx.mapping,
            ctx.injected_class,
        )),
        Type::TemplateInstance { name, args } => {
            substitute_template_instance(name, args, ctx.mapping, ctx.pos)
        }
        Type::Decltype {
            expr,
            parenthesized,
        } => {
            let mut substituted = (**expr).clone();
            substitute_expr(&mut substituted, ctx.mapping, ctx.injected_class)?;
            Ok(Type::Decltype {
                expr: Box::new(substituted),
                parenthesized: *parenthesized,
            })
        }
        Type::DependentArray(inner, param_name) => {
            let subst_inner = substitute_type_full(inner, ctx)?;
            let size = super::template_subst_array::evaluate_dependent_array_size(
                param_name,
                ctx.mapping,
                ctx.injected_class,
                ctx.sema.as_deref_mut(),
            )?;
            Ok(Type::Array(Box::new(subst_inner), size))
        }
        Type::Function {
            return_type,
            param_types,
            ..
        } => substitute_function_type(return_type, param_types, ctx),
        Type::PackExpansion(inner) => Ok(Type::PackExpansion(Box::new(substitute_type_full(
            inner, ctx,
        )?))),
        Type::DependentAlignas(inner, pack_name) => {
            super::template_subst_align::substitute_dependent_alignas(
                inner,
                pack_name,
                ctx.mapping,
                ctx.injected_class,
                ctx.struct_aligns,
            )
        }
        Type::Aligned(inner, align) => Ok(Type::Aligned(
            Box::new(substitute_type_full(inner, ctx)?),
            *align,
        )),
        Type::DependentNonType(expr_str)
            if expr_str.starts_with(crate::sema::pack_expansion::ADDRESS_ARG_PREFIX) =>
        {
            Ok(ty.clone())
        }
        Type::DependentNonType(expr_str) => {
            super::template_subst_nontype::evaluate_dependent_nontype(
                expr_str,
                ctx.mapping,
                ctx.injected_class,
            )
        }
        _ => Ok(ty.clone()),
    }
}

fn substitute_modified_type(
    ty: &Type,
    ctx: &mut SubstContext<'_>,
) -> Result<Option<Type>, CompileError> {
    let mut sub = ctx.without_aligns();
    match ty {
        Type::Const(inner) => {
            let subst = substitute_type_full(inner, &mut sub)?;
            if subst.is_reference() {
                Ok(Some(subst))
            } else {
                Ok(Some(Type::Const(Box::new(subst))))
            }
        }
        Type::Pointer(inner) => Ok(Some(Type::Pointer(Box::new(substitute_type_full(
            inner, &mut sub,
        )?)))),
        Type::Reference(inner) => Ok(Some(collapse_reference(
            false,
            substitute_type_full(inner, &mut sub)?,
        ))),
        Type::RvalueReference(inner) => Ok(Some(collapse_reference(
            true,
            substitute_type_full(inner, &mut sub)?,
        ))),
        Type::Array(inner, size) => Ok(Some(Type::Array(
            Box::new(substitute_type_full(inner, &mut sub)?),
            *size,
        ))),
        Type::MemberPointer {
            class_name,
            member_type,
        } => {
            let cls = match sub.mapping.get(class_name) {
                Some(TemplateArg::Type(Type::Struct(s) | Type::TypeParam(s))) => s.clone(),
                _ => class_name.clone(),
            };
            let mty = substitute_type_full(member_type, &mut sub)?;
            Ok(Some(Type::MemberPointer {
                class_name: cls,
                member_type: Box::new(mty),
            }))
        }
        _ => Ok(None),
    }
}

fn substitute_function_type(
    return_type: &Type,
    param_types: &[Type],
    ctx: &mut SubstContext<'_>,
) -> Result<Type, CompileError> {
    let subst_params = super::pack_expansion::expand_type_list(param_types, ctx.mapping, (0, 0))?;
    let mut sub = SubstContext {
        mapping: ctx.mapping,
        injected_class: ctx.injected_class,
        pos: (0, 0),
        struct_aligns: None,
        sema: ctx.sema.as_deref_mut(),
    };
    Ok(Type::Function {
        return_type: Box::new(substitute_type_full(return_type, &mut sub)?),
        param_types: subst_params,
        qualifiers: FunctionQualifiers::default(),
    })
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
    if let Some((target, member)) = name.split_once("::").and_then(|(base, member)| {
        mapping
            .get(base)
            .and_then(|t| t.as_type())
            .map(|t| (t, member))
    }) {
        let base_str = target.to_c_type();
        return Type::Struct(format!("{}::{}", base_str, member));
    }
    if name.contains('<') {
        return Type::Struct(substitute_template_name(name, mapping));
    }
    Type::Struct(name.to_string())
}

/// Resolves dependent `sizeof`/`alignof` operands left in a substituted member
/// initializer, reporting a missing nested type at the instantiation site.
fn resolve_initializer_operands(
    expr: &mut Expr,
    sema: Option<&mut crate::sema::Sema>,
    (line, col): (usize, usize),
) -> Result<(), CompileError> {
    let Some(sema) = sema else {
        return Ok(());
    };
    super::template_subst_array::resolve_sizeof_operands(expr, sema, "in initializer").map_err(
        |mut error| {
            if error.line == 0 && error.col == 0 {
                error.line = line;
                error.col = col;
            }
            error
        },
    )
}

fn substitute_fields(
    fields: &[crate::ast::decl::StructFieldDecl],
    injected: Option<(&str, &Type)>,
    (line, col): (usize, usize),
    sub: &mut SubstContext<'_>,
) -> Result<Vec<crate::ast::decl::StructFieldDecl>, CompileError> {
    let mapping = sub.mapping;
    let mut result = Vec::new();
    for (fname, fty, finit) in fields {
        let mut init = finit.clone();
        if let Some((init_expr, _, _)) = &mut init {
            substitute_expr(init_expr, mapping, injected)?;
            resolve_initializer_operands(init_expr, sub.sema.as_deref_mut(), (line, col))?;
        }
        let subst_fty = substitute_type_full(fty, sub).map_err(|mut e| {
            if e.line == 0 && e.col == 0 {
                e.line = line;
                e.col = col;
            }
            e
        })?;
        result.push((fname.clone(), subst_fty, init));
    }
    Ok(result)
}

pub fn substitute_struct(
    template_class: &TemplateClass,
    instantiated_name: &str,
    ctx: &mut SubstContext<'_>,
) -> Result<Decl, CompileError> {
    let (line, col) = ctx.pos;
    let mapping = ctx.mapping;
    let target_ty = Type::Struct(instantiated_name.to_string());
    let injected = Some((template_class.name.as_str(), &target_ty));
    let mut sub = SubstContext {
        mapping,
        injected_class: injected,
        pos: ctx.pos,
        struct_aligns: ctx.struct_aligns,
        sema: ctx.sema.as_deref_mut(),
    };

    let fields = substitute_fields(&template_class.fields, injected, (line, col), &mut sub)?;

    let mut methods = Vec::new();
    for m in &template_class.methods {
        methods.push(substitute_function_with_sema(
            m,
            mapping,
            injected,
            None,
            sub.sema.as_deref_mut(),
        )?);
    }

    let mut static_fields = Vec::new();
    for (sname, sty, sinit) in &template_class.static_fields {
        let mut init = sinit.clone();
        if let Some(init_expr) = &mut init {
            substitute_expr(init_expr, mapping, injected)?;
            resolve_initializer_operands(init_expr, sub.sema.as_deref_mut(), sub.pos)?;
        }
        static_fields.push((sname.clone(), substitute_type_full(sty, &mut sub)?, init));
    }

    let base_classes = super::template_subst_align::substitute_base_classes(
        &template_class.base_classes,
        mapping,
        line,
        col,
    )?;
    let alignment = template_class
        .alignment
        .as_ref()
        .map(|ty| substitute_type_full(ty, &mut sub))
        .transpose()?;

    Ok(Decl::StructDecl {
        name: instantiated_name.to_string(),
        is_class: template_class.is_class,
        is_final: template_class.is_final,
        base_classes,
        base_access: template_class.base_access.clone(),
        fields,
        methods,
        static_fields,
        private_members: template_class.private_members.clone(),
        nested_structs: Vec::new(),
        alignment,
        line,
        col,
    })
}

pub use super::template_subst_align::collapse_reference;
