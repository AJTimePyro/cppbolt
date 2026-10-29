use super::*;

fn context() -> FunctionContext {
    FunctionContext {
        fp_local: 0,
        scratch1: 1,
        scratch2: 2,
        scratch_i64: 3,
        scratch_f64: 4,
        scratch_f32: 5,
        ret_temp_i32: 6,
        ret_temp_i64: 7,
        ret_temp_f64: 8,
        var_offsets: HashMap::new(),
        var_types: HashMap::new(),
        static_var_offsets: HashMap::new(),
        next_local_offset: 16,
        frame_size: 0,
        frame_size_patches: Vec::new(),
        var_scopes: vec![Vec::new()],
        scope_objects: vec![Vec::new()],
        temp_objects: Vec::new(),
        current_class: None,
        current_func_name: "test".to_string(),
        line: 1,
        col: 1,
        return_type: Type::Void,
        is_dtor: false,
        is_reference_return: false,
        is_main: false,
    }
}

#[test]
fn frame_allocations_align_and_reject_overflow() {
    let mut ctx = context();
    assert_eq!(ctx.alloc_frame(1, 4), Ok(16));
    assert_eq!(ctx.alloc_frame(8, 8), Ok(24));
    assert_eq!(ctx.next_local_offset, 32);

    ctx.next_local_offset = u32::MAX;
    assert!(ctx.alloc_frame(4, 4).is_err());
}

#[test]
fn leaving_a_scope_restores_shadowed_names() {
    let mut ctx = context();
    ctx.bind_local("value".to_string(), 16, Type::Int);
    ctx.enter_scope();
    ctx.bind_local("value".to_string(), 20, Type::Double);
    ctx.bind_static("static_value".to_string(), 4096, Type::Int);
    assert_eq!(ctx.var_offsets["value"], 20);
    assert_eq!(ctx.var_types["value"], Type::Double);
    assert!(ctx.static_var_offsets.contains_key("static_value"));

    ctx.exit_scope();
    assert_eq!(ctx.var_offsets["value"], 16);
    assert_eq!(ctx.var_types["value"], Type::Int);
    assert!(!ctx.static_var_offsets.contains_key("static_value"));
}

#[test]
fn frame_size_patches_are_fixed_width_positive_leb128() {
    let mut ctx = context();
    ctx.frame_size = 1024;
    let mut code = Vec::new();
    ctx.emit_frame_size(&mut code);
    assert!(ctx.patch_frame_sizes(&mut code).is_ok());
    assert_eq!(code, [OP_I32_CONST, 0x80, 0x88, 0x80, 0x80, 0x00]);
}

#[test]
fn integer_narrowing_masks_unsigned_and_sign_extends_signed() {
    let sema = Sema::new();
    let codegen = Codegen::new(&sema);

    let mut code = Vec::new();
    codegen.emit_conversion(&Type::Int, &Type::UnsignedChar, &mut code);
    assert_eq!(code.last(), Some(&OP_I32_AND));

    let mut code = Vec::new();
    codegen.emit_conversion(&Type::LongLong, &Type::UnsignedShort, &mut code);
    assert_eq!(code.first(), Some(&OP_I32_WRAP_I64));
    assert_eq!(code.last(), Some(&OP_I32_AND));

    let mut code = Vec::new();
    codegen.emit_conversion(&Type::Int, &Type::SignedChar, &mut code);
    assert_eq!(code, [OP_I32_EXTEND8_S]);

    let mut code = Vec::new();
    codegen.emit_conversion(&Type::Int, &Type::Short, &mut code);
    assert_eq!(code, [OP_I32_EXTEND16_S]);
}

#[test]
fn array_subscripts_wrap_64_bit_indices() {
    let sema = Sema::new();
    let mut codegen = Codegen::new(&sema);
    let mut ctx = context();
    ctx.bind_local(
        "values".to_string(),
        16,
        Type::Array(Box::new(Type::Int), 8),
    );
    let expr = Expr::ArraySubscript {
        array: Box::new(Expr::Identifier("values".to_string())),
        index: Box::new(Expr::IntLiteral(3, Type::LongLong)),
        pos: ExprPos::default(),
    };
    let mut code = Vec::new();
    assert!(
        codegen
            .compile_lvalue_addr(&expr, &mut ctx, &mut code)
            .is_ok()
    );
    assert!(code.contains(&OP_I32_WRAP_I64));
}

#[test]
fn copy_constructor_lookup_ignores_conversion_operators_and_scalar_constructors() {
    let mut sema = Sema::new();
    sema.register_function(FunctionSig {
        name: "Foo::Foo".to_string(),
        mangled_name: "Foo_Foo_i".to_string(),
        return_type: Type::Void,
        param_types: vec![
            Type::Pointer(Box::new(Type::Struct("Foo".to_string()))),
            Type::Int,
        ],
        default_args: vec![None, None],
        is_static: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
    });
    sema.register_function(FunctionSig {
        name: "Foo::operator int".to_string(),
        mangled_name: "Foo_operator_int".to_string(),
        return_type: Type::Int,
        param_types: vec![Type::Pointer(Box::new(Type::Const(Box::new(
            Type::Struct("Foo".to_string()),
        ))))],
        default_args: vec![None],
        is_static: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
    });

    sema.register_function(FunctionSig {
        name: "Bar::Bar".to_string(),
        mangled_name: "Bar_Bar_R".to_string(),
        return_type: Type::Void,
        param_types: vec![
            Type::Pointer(Box::new(Type::Struct("Bar".to_string()))),
            Type::Reference(Box::new(Type::Const(Box::new(Type::Struct(
                "Bar".to_string(),
            ))))),
        ],
        default_args: vec![None, None],
        is_static: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
    });

    let mut codegen = Codegen::new(&sema);
    codegen.func_indices.insert("Foo_Foo_i".to_string(), 10);
    codegen.func_indices.insert("Bar_Bar_R".to_string(), 20);

    assert_eq!(codegen.find_copy_constructor("Foo", false), None);
    assert_eq!(codegen.find_copy_constructor("Foo", true), None);
    assert!(
        codegen
            .find_constructor_sig("Foo", &[Type::Struct("Foo".to_string())])
            .is_none()
    );

    assert_eq!(codegen.find_copy_constructor("Bar", false), Some(20));
    assert_eq!(codegen.find_copy_constructor("Bar", true), Some(20));
    assert!(
        codegen
            .find_constructor_sig("Bar", &[Type::Struct("Bar".to_string())])
            .is_some()
    );
}

#[test]
fn dereferencing_pointer_to_array_does_not_emit_scalar_load() {
    let sema = Sema::new();
    let mut codegen = Codegen::new(&sema);
    let mut ctx = context();
    let mut code = Vec::new();

    let ptr_expr = Expr::Identifier("ptr".to_string());
    let deref_expr = Expr::Unary {
        op: UnaryOp::Deref,
        expr: Box::new(ptr_expr),
        pos: ExprPos { line: 1, col: 1 },
    };
    ctx.bind_local(
        "ptr".to_string(),
        16,
        Type::Pointer(Box::new(Type::Array(Box::new(Type::Int), 3))),
    );

    assert!(
        codegen
            .compile_expr(&deref_expr, &mut ctx, &mut code)
            .is_ok()
    );
    let load_count = code.iter().filter(|&&b| b == OP_I32_LOAD).count();
    assert_eq!(load_count, 1);
}
