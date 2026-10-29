use super::*;

#[test]
fn test_narrowing_float_to_int_rejected() {
    let sema = Sema::new();
    let expr = Expr::FloatLiteral(3.25, Type::Double);
    let res = sema.check_narrowing(&expr, &Type::Double, &Type::Int, ExprPos::new(1, 1));
    assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
}

#[test]
fn test_narrowing_negative_to_unsigned_rejected() {
    let sema = Sema::new();
    let expr = Expr::IntLiteral(-1, Type::Int);
    let res = sema.check_narrowing(&expr, &Type::Int, &Type::UnsignedInt, ExprPos::new(1, 1));
    assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
}

#[test]
fn test_narrowing_overflow_constant_rejected() {
    let sema = Sema::new();
    let expr = Expr::IntLiteral(300, Type::Int);
    let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
    assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
}

#[test]
fn test_fitting_constant_accepted() {
    let sema = Sema::new();
    let expr = Expr::IntLiteral(42, Type::Int);
    let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
    assert!(res.is_ok());
}

#[test]
fn test_fitting_negative_constant_accepted() {
    let sema = Sema::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::IntLiteral(1, Type::Int)),
        pos: ExprPos::new(1, 1),
    };
    let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
    assert!(res.is_ok());
}

#[test]
fn test_user_declared_default_ctor_disqualifies_aggregate() {
    let mut sema = Sema::new();
    let layout = StructLayout {
        is_class: false,
        has_user_declared_constructors: true,
        base_classes: vec![],
        base_offsets: vec![],
        fields: vec![],
        size: 4,
        align: 4,
        has_vtable: false,
        vtable_slots: vec![],
        private_members: HashSet::new(),
        is_abstract: false,
        pure_virtuals: HashSet::new(),
    };
    sema.struct_layouts.insert("HasDefCtor".to_string(), layout);
    assert!(!sema.is_cxx17_aggregate("HasDefCtor"));
}

#[test]
fn test_scalar_excess_elements_rejected() {
    let mut sema = Sema::new();
    let mut elems = vec![
        Expr::IntLiteral(1, Type::Int),
        Expr::IntLiteral(2, Type::Int),
    ];
    let res = sema.resolve_list_init(
        &Type::Int,
        &mut elems,
        InitContext::DirectList,
        ExprPos::new(1, 1),
    );
    assert!(
        matches!(res, Err(err) if err.to_string().contains("scalar initializer has multiple expressions"))
    );
}

#[test]
fn test_explicit_constructor_in_copy_list_rejected() {
    let mut sema = Sema::new();
    let layout = StructLayout {
        is_class: true,
        has_user_declared_constructors: true,
        base_classes: vec![],
        base_offsets: vec![],
        fields: vec![],
        size: 4,
        align: 4,
        has_vtable: false,
        vtable_slots: vec![],
        private_members: HashSet::new(),
        is_abstract: false,
        pure_virtuals: HashSet::new(),
    };
    sema.struct_layouts.insert("Foo".to_string(), layout);
    let sig = FunctionSig {
        name: "Foo::Foo".to_string(),
        mangled_name: "_ZN3FooC1Ei".to_string(),
        return_type: Type::Void,
        param_types: vec![
            Type::Pointer(Box::new(Type::Struct("Foo".to_string()))),
            Type::Int,
        ],
        default_args: vec![None, None],
        is_static: false,
        is_explicit: true,
        ref_qualifier: RefQualifier::None,
    };
    sema.functions.insert("Foo::Foo".to_string(), vec![sig]);

    let mut elems = vec![Expr::IntLiteral(42, Type::Int)];
    let direct_res = sema.resolve_list_init(
        &Type::Struct("Foo".to_string()),
        &mut elems,
        InitContext::DirectList,
        ExprPos::new(1, 1),
    );
    assert!(direct_res.is_ok());

    let copy_res = sema.resolve_list_init(
        &Type::Struct("Foo".to_string()),
        &mut elems,
        InitContext::CopyListInitVar,
        ExprPos::new(1, 1),
    );
    assert!(
        matches!(copy_res, Err(err) if err.to_string().contains("is explicit in copy-list-initialization"))
    );
}
