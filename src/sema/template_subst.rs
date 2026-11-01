//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.

use crate::ast::decl::{Decl, FunctionDecl, Param};
use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::template::TemplateClass;
use crate::ast::types::Type;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) -> Type {
    match ty {
        Type::TypeParam(name) => mapping.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::Struct(name) => {
            if let Some((_, target_ty)) = injected_class.filter(|(c, _)| *c == name) {
                return target_ty.clone();
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

#[allow(dead_code)]
pub fn substitute_expr(
    expr: &mut Expr,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    match expr {
        Expr::Call { callee, args, .. } => {
            if let Expr::Identifier(name) = &mut **callee {
                substitute_callee_identifier(name, injected_class);
            }
            substitute_expr(callee, mapping, injected_class);
            for arg in args {
                substitute_expr(arg, mapping, injected_class);
            }
        }
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. } => {
            substitute_expr(expr, mapping, injected_class);
        }
        Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
            substitute_expr(target, mapping, injected_class);
            substitute_expr(value, mapping, injected_class);
        }
        Expr::ArraySubscript { array, index, .. } => {
            substitute_expr(array, mapping, injected_class);
            substitute_expr(index, mapping, injected_class);
        }
        Expr::MemberAccess { object, .. } => {
            substitute_expr(object, mapping, injected_class);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            substitute_expr(cond, mapping, injected_class);
            substitute_expr(then_expr, mapping, injected_class);
            substitute_expr(else_expr, mapping, injected_class);
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
        Expr::New {
            ty,
            args,
            array_size,
            placement,
            ..
        } => {
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
        Expr::InitializerList { elements, .. } => {
            for el in elements {
                substitute_expr(el, mapping, injected_class);
            }
        }
        _ => {}
    }
}

#[allow(dead_code)]
fn substitute_callee_identifier(name: &mut String, injected_class: Option<(&str, &Type)>) {
    if let Some((_, target_ty)) = injected_class.filter(|(c, _)| *c == name.as_str()) {
        *name = target_ty.to_c_type();
    }
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

#[allow(dead_code)]
pub fn substitute_function(
    func: &FunctionDecl,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
    new_name: Option<String>,
) -> FunctionDecl {
    let name = new_name.unwrap_or_else(|| {
        if let Some((cname, target_ty)) = injected_class {
            if func.name == cname {
                target_ty.to_c_type()
            } else if func.name == format!("~{}", cname) {
                format!("~{}", target_ty.to_c_type())
            } else {
                func.name.clone()
            }
        } else {
            func.name.clone()
        }
    });

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
        line: func.line,
        col: func.col,
    }
}

#[allow(dead_code)]
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
        line,
        col,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_scalar_type() {
        let mut mapping = HashMap::new();
        mapping.insert("T".to_string(), Type::Int);

        let t_param = Type::TypeParam("T".to_string());
        assert_eq!(substitute_type(&t_param, &mapping, None), Type::Int);

        let ptr = Type::Pointer(Box::new(Type::TypeParam("T".to_string())));
        assert_eq!(
            substitute_type(&ptr, &mapping, None),
            Type::Pointer(Box::new(Type::Int))
        );
    }

    #[test]
    fn test_substitute_injected_class_name() {
        let mapping = HashMap::new();
        let target_ty = Type::Struct("Holder<int>".to_string());
        let bare = Type::Struct("Holder".to_string());

        let res = substitute_type(&bare, &mapping, Some(("Holder", &target_ty)));
        assert_eq!(res, target_ty);
    }

    #[test]
    fn test_substitute_function() {
        let mut mapping = HashMap::new();
        mapping.insert("T".to_string(), Type::Int);

        let func = FunctionDecl {
            name: "get".to_string(),
            return_type: Type::TypeParam("T".to_string()),
            params: vec![Param {
                name: "val".to_string(),
                ty: Type::TypeParam("T".to_string()),
                default_val: None,
            }],
            body: None,
            is_virtual: false,
            is_pure_virtual: false,
            is_const: true,
            is_explicit: false,
            ref_qualifier: Default::default(),
            is_static: false,
            ctor_initializers: Vec::new(),
            line: 1,
            col: 1,
        };

        let subst = substitute_function(&func, &mapping, None, None);
        assert_eq!(subst.return_type, Type::Int);
        assert_eq!(subst.params[0].ty, Type::Int);
    }

    #[test]
    fn test_substitute_struct() {
        let mut mapping = HashMap::new();
        mapping.insert("T".to_string(), Type::Int);

        let tclass = TemplateClass {
            name: "Holder".to_string(),
            is_class: true,
            base_classes: Vec::new(),
            fields: vec![("value".to_string(), Type::TypeParam("T".to_string()))],
            methods: vec![FunctionDecl {
                name: "Holder".to_string(),
                return_type: Type::Void,
                params: vec![Param {
                    name: "val".to_string(),
                    ty: Type::TypeParam("T".to_string()),
                    default_val: None,
                }],
                body: None,
                is_virtual: false,
                is_pure_virtual: false,
                is_const: false,
                is_explicit: false,
                ref_qualifier: Default::default(),
                is_static: false,
                ctor_initializers: Vec::new(),
                line: 1,
                col: 1,
            }],
            static_fields: Vec::new(),
            private_members: Vec::new(),
        };

        let subst = substitute_struct(&tclass, &mapping, "Holder<int>", 1, 1);
        let Decl::StructDecl {
            name,
            fields,
            methods,
            ..
        } = subst
        else {
            unreachable!("expected StructDecl");
        };
        assert_eq!(name, "Holder<int>");
        assert_eq!(fields[0].1, Type::Int);
        assert_eq!(methods[0].name, "Holder<int>");
        assert_eq!(methods[0].params[0].ty, Type::Int);
    }
}
