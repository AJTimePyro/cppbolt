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

fn get_param_type<'a>(cand: &'a FunctionTemplateCand<'a>, i: usize) -> Option<&'a Type> {
    if i < cand.decl.params.len() {
        let p = &cand.decl.params[i];
        if p.is_pack {
            let inner_ty = match &p.ty {
                Type::PackExpansion(inner) => inner.as_ref(),
                other => other,
            };
            Some(inner_ty)
        } else {
            Some(&p.ty)
        }
    } else if let Some(last) = cand.decl.params.last() {
        if last.is_pack {
            let inner_ty = match &last.ty {
                Type::PackExpansion(inner) => inner.as_ref(),
                other => other,
            };
            Some(inner_ty)
        } else {
            None
        }
    } else {
        None
    }
}

fn synth_subst_type(ty: &Type, synth_map: &std::collections::HashMap<String, Type>) -> Type {
    match ty {
        Type::TypeParam(name) => synth_map.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::Pointer(inner) => Type::Pointer(Box::new(synth_subst_type(inner, synth_map))),
        Type::Reference(inner) => Type::Reference(Box::new(synth_subst_type(inner, synth_map))),
        Type::RvalueReference(inner) => {
            Type::RvalueReference(Box::new(synth_subst_type(inner, synth_map)))
        }
        Type::Const(inner) => Type::Const(Box::new(synth_subst_type(inner, synth_map))),
        Type::PackExpansion(inner) => {
            Type::PackExpansion(Box::new(synth_subst_type(inner, synth_map)))
        }
        Type::TemplateInstance { name, args } => Type::TemplateInstance {
            name: name.clone(),
            args: args.iter().map(|a| synth_subst_type(a, synth_map)).collect(),
        },
        _ => ty.clone(),
    }
}

fn deduces_from_synthetic(
    source_cand: &FunctionTemplateCand,
    target_cand: &FunctionTemplateCand,
    call_args_len: usize,
) -> bool {
    let mut synth_map = std::collections::HashMap::new();
    for (i, p) in source_cand.params.iter().enumerate() {
        let synth_name = format!("__synth_order_{}_{}", source_cand.decl.name, i);
        synth_map.insert(p.name.clone(), Type::Struct(synth_name));
    }
    let mut deduced = std::collections::HashMap::new();
    for i in 0..call_args_len {
        let p_src = get_param_type(source_cand, i);
        let p_tgt = get_param_type(target_cand, i);
        let (Some(p_src), Some(p_tgt)) = (p_src, p_tgt) else {
            return false;
        };
        let synth_arg = synth_subst_type(p_src, &synth_map);
        let is_lvalue = matches!(p_src, Type::Reference(_));
        if super::template_deduce_type::deduce_type(
            p_tgt,
            &synth_arg,
            &mut deduced,
            is_lvalue,
            crate::ast::ExprPos::default(),
        )
        .is_err()
        {
            return false;
        }
    }
    for p in target_cand.params {
        if !p.is_pack && !deduced.contains_key(&p.name) {
            return false;
        }
    }
    true
}

/// Compares two function templates for a call with `call_args_len` arguments
/// using standard deduction from synthetic types per ISO C++11 [temp.func.order].
pub fn compare_function_templates(
    cand1: &FunctionTemplateCand,
    cand2: &FunctionTemplateCand,
    call_args_len: usize,
) -> Option<Ordering> {
    let d1_to_2 = deduces_from_synthetic(cand1, cand2, call_args_len);
    let d2_to_1 = deduces_from_synthetic(cand2, cand1, call_args_len);

    if d1_to_2 && !d2_to_1 {
        return Some(Ordering::Less);
    }
    if d2_to_1 && !d1_to_2 {
        return Some(Ordering::Greater);
    }

    let has_pack1 = cand1.decl.params.iter().any(|p| p.is_pack)
        || cand1.params.iter().any(|p| p.is_pack);
    let has_pack2 = cand2.decl.params.iter().any(|p| p.is_pack)
        || cand2.params.iter().any(|p| p.is_pack);

    if !has_pack1 && has_pack2 {
        Some(Ordering::Less)
    } else if has_pack1 && !has_pack2 {
        Some(Ordering::Greater)
    } else {
        None
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
    fn test_deduction_based_pointer_more_specialized_than_type_param() {
        let t_param = Type::TypeParam("T".to_string());
        let t_ptr = Type::Pointer(Box::new(t_param.clone()));
        let decl_ptr = make_fn_decl("f", vec![make_param(t_ptr, false)]);
        let decl_param = make_fn_decl("f", vec![make_param(t_param, false)]);
        let tparams = vec![TemplateParam {
            name: "T".to_string(),
            param_type: None,
            default_type: None,
            is_pack: false,
            line: 1,
            col: 1,
        }];
        let cand_ptr = FunctionTemplateCand::new(&decl_ptr, &tparams);
        let cand_param = FunctionTemplateCand::new(&decl_param, &tparams);
        assert_eq!(
            compare_function_templates(&cand_ptr, &cand_param, 1),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_function_templates(&cand_param, &cand_ptr, 1),
            Some(Ordering::Greater)
        );
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
