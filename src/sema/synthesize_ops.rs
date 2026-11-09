//! Synthesis of implicit copy and move operations for structs.

use super::*;

pub(crate) fn synthesize_copy_operations(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    synthesize_copy_constructor(name, base_classes, fields, methods);
    synthesize_copy_assignment(name, base_classes, fields, methods);
}

pub(crate) fn synthesize_move_operations(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    if is_move_suppressed(name, methods) {
        return;
    }
    synthesize_move_constructor(name, base_classes, fields, methods);
    synthesize_move_assignment(name, base_classes, fields, methods);
}

fn is_move_suppressed(name: &str, methods: &[FunctionDecl]) -> bool {
    let short = name.rsplit("::").next().unwrap_or(name);
    let has_custom_dtor = methods.iter().any(|m| m.name.starts_with('~'));
    let has_copy_or_move_op = methods.iter().any(|m| {
        let is_ctor = (m.name == name || m.name == short) && m.params.len() == 1;
        let is_assign = m.name == "operator=" && m.params.len() == 1;
        (is_ctor || is_assign)
            && matches!(
                m.params[0].ty.strip_const(),
                Type::Reference(inner) | Type::RvalueReference(inner)
                    if matches!(inner.strip_const(), Type::Struct(s) if s == name || s == short)
            )
    });
    has_custom_dtor || has_copy_or_move_op
}

fn synthesize_copy_constructor(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    let short = name.rsplit("::").next().unwrap_or(name);
    let has_copy_ctor = methods.iter().any(|m| {
        (m.name == name || m.name == short)
            && m.params.len() == 1
            && matches!(m.params[0].ty.strip_const(), Type::Reference(inner) if matches!(inner.strip_const(), Type::Struct(s) if s == name || s == short))
    });
    if !has_copy_ctor {
        let param_ty = Type::Reference(Box::new(Type::Const(Box::new(Type::Struct(
            name.to_string(),
        )))));
        let mut initializers: Vec<(String, Vec<Expr>)> = base_classes
            .iter()
            .map(|base| (base.clone(), vec![Expr::Identifier("o".to_string())]))
            .collect();
        initializers.extend(fields.iter().map(|(fname, _, _)| {
            (
                fname.clone(),
                vec![Expr::MemberAccess {
                    object: Box::new(Expr::Identifier("o".to_string())),
                    member: fname.clone(),
                    is_arrow: false,
                    pos: ExprPos::new(0, 0),
                }],
            )
        }));
        methods.push(FunctionDecl {
            name: name.to_string(),
            return_type: Type::Void,
            params: vec![Param {
                name: "o".to_string(),
                ty: param_ty,
                default_val: None,
            }],
            body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
            is_static: false,
            ctor_initializers: initializers,
            template_params: vec![],
            line: 0,
            col: 0,
        });
    }
}

fn synthesize_move_constructor(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    let param_ty = Type::RvalueReference(Box::new(Type::Struct(name.to_string())));
    let mut initializers: Vec<(String, Vec<Expr>)> = base_classes
        .iter()
        .map(|base| {
            (
                base.clone(),
                vec![Expr::Cast {
                    kind: CastKind::Static,
                    target_type: Type::RvalueReference(Box::new(Type::Struct(base.clone()))),
                    expr: Box::new(Expr::Identifier("o".to_string())),
                }],
            )
        })
        .collect();
    initializers.extend(fields.iter().map(|(fname, ftype, _)| {
        (
            fname.clone(),
            vec![Expr::Cast {
                kind: CastKind::Static,
                target_type: Type::RvalueReference(Box::new(ftype.clone())),
                expr: Box::new(Expr::MemberAccess {
                    object: Box::new(Expr::Identifier("o".to_string())),
                    member: fname.clone(),
                    is_arrow: false,
                    pos: ExprPos::new(0, 0),
                }),
            }],
        )
    }));
    methods.push(FunctionDecl {
        name: name.to_string(),
        return_type: Type::Void,
        params: vec![Param {
            name: "o".to_string(),
            ty: param_ty,
            default_val: None,
        }],
        body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
        is_virtual: false,
        is_pure_virtual: false,
        is_const: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        is_static: false,
        ctor_initializers: initializers,
        template_params: vec![],
        line: 0,
        col: 0,
    });
}

fn synthesize_copy_assignment(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    let short = name.rsplit("::").next().unwrap_or(name);
    let has_copy_assign = methods.iter().any(|m| {
        m.name == "operator="
            && m.params.len() == 1
            && matches!(m.params[0].ty.strip_const(), Type::Reference(inner) if matches!(inner.strip_const(), Type::Struct(s) if s == name || s == short))
    });
    if !has_copy_assign {
        let param_ty = Type::Reference(Box::new(Type::Const(Box::new(Type::Struct(
            name.to_string(),
        )))));
        let body_stmts = synthesize_assignment_body(base_classes, fields, false);
        methods.push(FunctionDecl {
            name: "operator=".to_string(),
            return_type: Type::Reference(Box::new(Type::Struct(name.to_string()))),
            params: vec![Param {
                name: "o".to_string(),
                ty: param_ty,
                default_val: None,
            }],
            body: Some(Stmt::new(StmtKind::Compound(body_stmts), 0, 0)),
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
            is_static: false,
            ctor_initializers: vec![],
            template_params: vec![],
            line: 0,
            col: 0,
        });
    }
}

fn synthesize_move_assignment(
    name: &str,
    base_classes: &[String],
    fields: &[StructFieldDecl],
    methods: &mut Vec<FunctionDecl>,
) {
    let param_ty = Type::RvalueReference(Box::new(Type::Struct(name.to_string())));
    let body_stmts = synthesize_assignment_body(base_classes, fields, true);
    methods.push(FunctionDecl {
        name: "operator=".to_string(),
        return_type: Type::Reference(Box::new(Type::Struct(name.to_string()))),
        params: vec![Param {
            name: "o".to_string(),
            ty: param_ty,
            default_val: None,
        }],
        body: Some(Stmt::new(StmtKind::Compound(body_stmts), 0, 0)),
        is_virtual: false,
        is_pure_virtual: false,
        is_const: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        is_static: false,
        ctor_initializers: vec![],
        template_params: vec![],
        line: 0,
        col: 0,
    });
}

fn synthesize_assignment_body(
    base_classes: &[String],
    fields: &[StructFieldDecl],
    is_move: bool,
) -> Vec<Stmt> {
    let mut body_stmts = Vec::new();
    for base in base_classes {
        let base_struct = Type::Struct(base.clone());
        let val_target_ty = if is_move {
            Type::RvalueReference(Box::new(base_struct))
        } else {
            Type::Reference(Box::new(Type::Const(Box::new(base_struct))))
        };
        body_stmts.push(Stmt::new(
            StmtKind::Expr(Expr::Assign {
                target: Box::new(Expr::Cast {
                    kind: CastKind::Static,
                    target_type: Type::Reference(Box::new(Type::Struct(base.clone()))),
                    expr: Box::new(Expr::Unary {
                        op: UnaryOp::Deref,
                        expr: Box::new(Expr::This),
                        pos: ExprPos::new(0, 0),
                    }),
                }),
                value: Box::new(Expr::Cast {
                    kind: CastKind::Static,
                    target_type: val_target_ty,
                    expr: Box::new(Expr::Identifier("o".to_string())),
                }),
                pos: ExprPos::new(0, 0),
            }),
            0,
            0,
        ));
    }
    for (fname, ftype, _) in fields {
        let rval_expr = Expr::MemberAccess {
            object: Box::new(Expr::Identifier("o".to_string())),
            member: fname.clone(),
            is_arrow: false,
            pos: ExprPos::new(0, 0),
        };
        let value = if is_move {
            Expr::Cast {
                kind: CastKind::Static,
                target_type: Type::RvalueReference(Box::new(ftype.clone())),
                expr: Box::new(rval_expr),
            }
        } else {
            rval_expr
        };
        body_stmts.push(Stmt::new(
            StmtKind::Expr(Expr::Assign {
                target: Box::new(Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: fname.clone(),
                    is_arrow: true,
                    pos: ExprPos::new(0, 0),
                }),
                value: Box::new(value),
                pos: ExprPos::new(0, 0),
            }),
            0,
            0,
        ));
    }
    body_stmts.push(Stmt::new(
        StmtKind::Return(Some(Expr::Unary {
            op: UnaryOp::Deref,
            expr: Box::new(Expr::This),
            pos: ExprPos::new(0, 0),
        })),
        0,
        0,
    ));
    body_stmts
}
