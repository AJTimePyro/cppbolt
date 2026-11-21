//! Deduction: matching a pattern against a concrete type to bind parameters.

use super::pack_expansion::TemplateArg;
use super::template_deduce::split_template_call_name;
use super::template_match::{AliasMap, alias_definition, expand_alias_pattern};
use super::template_match_args::match_pattern_args;
use super::*;

/// Mirrors `confirm_resolved_pattern`: a plain parameter, a wrapper around
/// one, template arguments, and a bare non-type parameter deduce; qualified
/// names, `decltype`, and dependent expressions do not. Detectability of a
/// param is the same as its ability to be matched here.
pub(super) fn match_pattern_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    // Expand alias templates so their arguments are deducible. A dependent
    // expansion is left as a non-deduced context and confirmed after deduction
    // in `confirm_resolved_pattern`.
    if let Some(expanded) = expand_alias_pattern(pattern, aliases) {
        return match_pattern_arg(&expanded, concrete, bindings, aliases);
    }
    if let Type::TemplateInstance { name, .. } = pattern
        && alias_definition(name, aliases).is_some()
    {
        return true;
    }
    if let Some(result) = match_wrapper_arg(pattern, concrete, bindings, aliases) {
        return result;
    }
    if let Some(result) = match_array_arg(pattern, concrete, bindings, aliases) {
        return result;
    }
    match pattern {
        Type::TypeParam(p) => bind_type_param(p, concrete, bindings),
        Type::Function {
            return_type: p_ret,
            param_types: p_params,
            qualifiers: p_qual,
        } => match concrete {
            Type::Function {
                return_type: c_ret,
                param_types: c_params,
                qualifiers: c_qual,
            } => {
                // cv/ref/noexcept are part of the function type. Ignoring them
                // would let a `const`/`noexcept` pattern match a plain function.
                p_qual == c_qual
                    && match_pattern_arg(p_ret, c_ret, bindings, aliases)
                    && match_pattern_args(p_params, c_params, bindings, aliases)
            }
            _ => false,
        },
        Type::MemberPointer {
            class_name: p_class,
            member_type: p_member,
        } => match_member_pointer_arg(p_class, p_member, concrete, bindings, aliases),
        Type::TemplateInstance { name, args } => {
            match_template_instance_arg(name, args, concrete, bindings, aliases)
        }
        // A `decltype` operand, a qualified type that still mentions a
        // template parameter, or a qualified type whose template arguments
        // contain a `decltype` is a non-deduced context. It cannot bind
        // parameters by structure; `select_partial_specialization` resolves it
        // after the other arguments have been deduced.
        Type::Decltype { .. } | Type::DependentNested(_) => true,
        Type::Struct(name) if name.contains("decltype") => true,
        _ => pattern == concrete,
    }
}

fn bind_type_param(p: &str, concrete: &Type, bindings: &mut HashMap<String, TemplateArg>) -> bool {
    let arg = TemplateArg::from(concrete.clone());
    if let Some(existing) = bindings.get(p) {
        existing == &arg
    } else {
        bindings.insert(p.to_string(), arg);
        true
    }
}

/// Matches the four wrapper kinds against a concrete type of the same kind.
/// Returns `None` only when `pattern` is not a wrapper.
fn match_wrapper_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> Option<bool> {
    let (inner_p, inner_c) = match (pattern, concrete) {
        (Type::Const(p), Type::Const(c))
        | (Type::Pointer(p), Type::Pointer(c))
        | (Type::Reference(p), Type::Reference(c))
        | (Type::RvalueReference(p), Type::RvalueReference(c)) => (p.as_ref(), c.as_ref()),
        (Type::Const(_) | Type::Pointer(_) | Type::Reference(_) | Type::RvalueReference(_), _) => {
            return Some(false);
        }
        _ => return None,
    };
    Some(match_pattern_arg(inner_p, inner_c, bindings, aliases))
}

fn match_array_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> Option<bool> {
    let matched = match pattern {
        // `T[]` matches only an unbounded array; a written bound, including
        // `T[0]`, matches only that exact bound.
        Type::Array(p_inner, p_n) => match concrete {
            Type::Array(c_inner, c_n) if *p_n == *c_n => {
                match_pattern_arg(p_inner, c_inner, bindings, aliases)
            }
            _ => false,
        },
        // A deduced bound names a real array bound. It cannot bind to the
        // unbounded-array marker, which is a distinct type in C++.
        Type::DependentArray(p_inner, bound) => match concrete {
            Type::Array(c_inner, c_n) if *c_n != crate::ast::types::UNBOUNDED_ARRAY => {
                let arg = TemplateArg::NonType(*c_n as i128);
                if bindings.get(bound).is_some_and(|e| e != &arg) {
                    return Some(false);
                }
                bindings.insert(bound.clone(), arg);
                match_pattern_arg(p_inner, c_inner, bindings, aliases)
            }
            _ => false,
        },
        _ => return None,
    };
    Some(matched)
}

fn match_member_pointer_arg(
    p_class: &str,
    p_member: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    let Type::MemberPointer {
        class_name: c_class,
        member_type: c_member,
    } = concrete
    else {
        return false;
    };
    // An unqualified class name in the pattern is a deduced type
    // parameter (the `C` in `T C::*`), mirroring function-template
    // deduction for pointers to members.
    if !p_class.contains("::") {
        let arg = TemplateArg::Type(Type::Struct(c_class.clone()));
        if bindings.get(p_class).is_some_and(|e| e != &arg) {
            return false;
        }
        bindings.insert(p_class.to_string(), arg);
    } else if p_class != c_class {
        return false;
    }
    match_pattern_arg(p_member, c_member, bindings, aliases)
}

fn match_template_instance_arg(
    p_name: &str,
    p_args: &[Type],
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    match concrete {
        Type::TemplateInstance {
            name: c_name,
            args: c_args,
        } => {
            if !QualifiedName::new(p_name).matches(QualifiedName::new(c_name)) {
                return false;
            }
            match_pattern_args(p_args, c_args, bindings, aliases)
        }
        Type::Struct(sname) => {
            if let Some((s_base, s_args_str)) = split_template_call_name(sname)
                && QualifiedName::new(p_name).matches(QualifiedName::new(s_base))
                && let Ok(s_args) =
                    super::template_deduce::parse_types_list(s_args_str, &[p_name.to_string()])
            {
                match_pattern_args(p_args, &s_args, bindings, aliases)
            } else {
                false
            }
        }
        _ => false,
    }
}
