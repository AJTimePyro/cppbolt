//! Function template partial ordering ([temp.func.order], [temp.deduct.partial]).

use crate::ast::decl::FunctionDecl;
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use std::cmp::Ordering;

/// A candidate function template for partial ordering.
#[derive(Clone, Copy, Debug)]
pub struct FunctionTemplateCand<'a> {
    pub decl: &'a FunctionDecl,
    pub params: &'a [TemplateParam],
}

impl<'a> FunctionTemplateCand<'a> {
    pub fn new(decl: &'a FunctionDecl, params: &'a [TemplateParam]) -> Self {
        Self { decl, params }
    }
}

/// Information about a parameter at a given call position.
#[derive(Clone, Copy, Debug)]
struct ParamInfo<'a> {
    ty: &'a Type,
    is_pack: bool,
}

/// Computes the type specificity metric for template ordering.
/// Higher specificity indicates a more specialized type.
pub fn type_specificity(ty: &Type) -> usize {
    match ty {
        Type::TypeParam(_) => 0,
        Type::PackExpansion(inner) => type_specificity(inner),
        Type::Reference(inner) => 2 + type_specificity(inner),
        Type::RvalueReference(inner) => 1 + type_specificity(inner),
        Type::Const(inner) => 1 + type_specificity(inner),
        Type::Pointer(inner) => 4 + type_specificity(inner),
        Type::TemplateInstance { args, .. } => {
            6 + args.iter().map(type_specificity).sum::<usize>()
        }
        _ => 10,
    }
}

fn get_param_info<'a>(cand: &'a FunctionTemplateCand<'a>, i: usize) -> Option<ParamInfo<'a>> {
    if i < cand.decl.params.len() {
        let p = &cand.decl.params[i];
        if p.is_pack {
            let inner_ty = match &p.ty {
                Type::PackExpansion(inner) => inner.as_ref(),
                other => other,
            };
            Some(ParamInfo {
                ty: inner_ty,
                is_pack: true,
            })
        } else {
            Some(ParamInfo {
                ty: &p.ty,
                is_pack: false,
            })
        }
    } else if let Some(last) = cand.decl.params.last() {
        if last.is_pack {
            let inner_ty = match &last.ty {
                Type::PackExpansion(inner) => inner.as_ref(),
                other => other,
            };
            Some(ParamInfo {
                ty: inner_ty,
                is_pack: true,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn compare_param(p1: ParamInfo, p2: ParamInfo) -> Ordering {
    if !p1.is_pack && p2.is_pack {
        return Ordering::Less;
    }
    if p1.is_pack && !p2.is_pack {
        return Ordering::Greater;
    }
    let s1 = type_specificity(p1.ty);
    let s2 = type_specificity(p2.ty);
    if s1 > s2 {
        Ordering::Less
    } else if s2 > s1 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// Compares two function templates for a call with `call_args_len` arguments.
///
/// Returns:
/// - `Some(Ordering::Less)` if `cand1` is strictly more specialized than `cand2`
/// - `Some(Ordering::Greater)` if `cand2` is strictly more specialized than `cand1`
/// - `Some(Ordering::Equal)` if both are equally specialized
/// - `None` if they are incomparable / conflicting
pub fn compare_function_templates(
    cand1: &FunctionTemplateCand,
    cand2: &FunctionTemplateCand,
    call_args_len: usize,
) -> Option<Ordering> {
    let mut cand1_better = false;
    let mut cand2_better = false;

    for i in 0..call_args_len {
        let p1 = get_param_info(cand1, i);
        let p2 = get_param_info(cand2, i);
        let (Some(p1), Some(p2)) = (p1, p2) else {
            break;
        };
        match compare_param(p1, p2) {
            Ordering::Less => cand1_better = true,
            Ordering::Greater => cand2_better = true,
            Ordering::Equal => {}
        }
    }

    if cand1_better && !cand2_better {
        return Some(Ordering::Less);
    }
    if cand2_better && !cand1_better {
        return Some(Ordering::Greater);
    }
    if cand1_better && cand2_better {
        return None;
    }

    let has_pack1 = cand1.decl.params.iter().any(|p| p.is_pack)
        || cand1.params.iter().any(|p| p.is_pack);
    let has_pack2 = cand2.decl.params.iter().any(|p| p.is_pack)
        || cand2.params.iter().any(|p| p.is_pack);

    if !has_pack1 && has_pack2 {
        Some(Ordering::Less)
    } else if has_pack1 && !has_pack2 {
        Some(Ordering::Greater)
    } else if cand1.params.len() < cand2.params.len() {
        Some(Ordering::Less)
    } else if cand1.params.len() > cand2.params.len() {
        Some(Ordering::Greater)
    } else {
        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::decl::{Param, RefQualifier};

    fn make_fn_decl(name: &str, params: Vec<Param>) -> FunctionDecl {
        FunctionDecl {
            name: name.to_string(),
            return_type: Type::Void,
            params,
            body: None,
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
            is_static: false,
            ctor_initializers: Vec::new(),
            template_params: Vec::new(),
            line: 1,
            col: 1,
        }
    }

    fn make_param(ty: Type, is_pack: bool) -> Param {
        Param {
            name: "x".to_string(),
            ty,
            default_val: None,
            is_pack,
        }
    }

    #[test]
    fn test_type_specificity_ranking() {
        let t_param = Type::TypeParam("T".to_string());
        let t_ref = Type::Reference(Box::new(t_param.clone()));
        let t_const_ref = Type::Reference(Box::new(Type::Const(Box::new(t_param.clone()))));
        let t_ptr = Type::Pointer(Box::new(t_param.clone()));
        let t_const_ptr = Type::Pointer(Box::new(Type::Const(Box::new(t_param.clone()))));
        let t_int = Type::Int;

        assert!(type_specificity(&t_int) > type_specificity(&t_const_ptr));
        assert!(type_specificity(&t_const_ptr) > type_specificity(&t_ptr));
        assert!(type_specificity(&t_ptr) > type_specificity(&t_const_ref));
        assert!(type_specificity(&t_const_ref) > type_specificity(&t_ref));
        assert!(type_specificity(&t_ref) > type_specificity(&t_param));
    }

    #[test]
    fn test_partial_order_non_pack_preferred_over_pack() {
        let t_param = Type::TypeParam("T".to_string());
        let rest_pack = Type::PackExpansion(Box::new(Type::TypeParam("Rest".to_string())));

        let decl_single = make_fn_decl("f", vec![make_param(t_param.clone(), false)]);
        let tparams_single = vec![TemplateParam {
            name: "T".to_string(),
            param_type: None,
            default_type: None,
            is_pack: false,
            line: 1,
            col: 1,
        }];
        let cand_single = FunctionTemplateCand::new(&decl_single, &tparams_single);

        let decl_var = make_fn_decl(
            "f",
            vec![
                make_param(t_param.clone(), false),
                make_param(rest_pack, true),
            ],
        );
        let tparams_var = vec![
            TemplateParam {
                name: "T".to_string(),
                param_type: None,
                default_type: None,
                is_pack: false,
                line: 1,
                col: 1,
            },
            TemplateParam {
                name: "Rest".to_string(),
                param_type: None,
                default_type: None,
                is_pack: true,
                line: 1,
                col: 1,
            },
        ];
        let cand_var = FunctionTemplateCand::new(&decl_var, &tparams_var);

        // For a 1-arg call, single template is preferred over variadic template
        assert_eq!(
            compare_function_templates(&cand_single, &cand_var, 1),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_function_templates(&cand_var, &cand_single, 1),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn test_partial_order_ambiguous() {
        let t_param = Type::TypeParam("T".to_string());
        let decl1 = make_fn_decl(
            "clash",
            vec![
                make_param(t_param.clone(), false),
                make_param(Type::Int, false),
            ],
        );
        let decl2 = make_fn_decl(
            "clash",
            vec![
                make_param(Type::Int, false),
                make_param(t_param.clone(), false),
            ],
        );
        let tparams = vec![TemplateParam {
            name: "T".to_string(),
            param_type: None,
            default_type: None,
            is_pack: false,
            line: 1,
            col: 1,
        }];
        let cand1 = FunctionTemplateCand::new(&decl1, &tparams);
        let cand2 = FunctionTemplateCand::new(&decl2, &tparams);

        assert_eq!(compare_function_templates(&cand1, &cand2, 2), None);
    }

    #[test]
    fn test_format_canonical_args_no_trailing_comma_when_pack_empty() {
        use crate::ast::template::TemplateParam;
        use crate::sema::pack_expansion::TemplateArg;
        use std::collections::HashMap;

        let params = vec![
            TemplateParam {
                name: "T".to_string(),
                param_type: None,
                default_type: None,
                is_pack: false,
                line: 1,
                col: 1,
            },
            TemplateParam {
                name: "Rest".to_string(),
                param_type: None,
                default_type: None,
                is_pack: true,
                line: 1,
                col: 1,
            },
        ];
        let mut mapping = HashMap::new();
        mapping.insert("T".to_string(), TemplateArg::Type(Type::Int));
        mapping.insert("Rest".to_string(), TemplateArg::TypePack(Vec::new()));

        let s = crate::sema::template_func::format_canonical_args(&params, &mapping);
        assert_eq!(s, "int");
    }
}
