//! Semantic analysis and closure synthesis for C++17 lambda expressions.

use super::analyze::this_param;
use super::*;

struct LambdaCaptureInfo {
    fields: Vec<(String, Type)>,
    ctor_params: Vec<Param>,
    ctor_inits: Vec<(String, Vec<Expr>)>,
    ctor_call_args: Vec<Expr>,
}

impl Sema {
    pub(super) fn analyze_lambda(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Lambda {
            default_capture,
            captures,
            params,
            return_type,
            body,
            is_mutable,
            pos,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        self.lambda_counter += 1;
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        let capture_info =
            self.collect_lambda_captures(*default_capture, captures, body, params, *pos)?;

        let ctor_decl = build_lambda_ctor(&lambda_name, &capture_info, *pos);
        let mut op_decl = build_lambda_op(params, body, return_type.as_ref(), *is_mutable, *pos);

        self.register_lambda_closure(&lambda_name, &capture_info, &ctor_decl, &mut op_decl, *pos)?;

        *expr = Expr::Call {
            callee: Box::new(Expr::Identifier(lambda_name)),
            args: capture_info.ctor_call_args,
            pos: *pos,
        };
        self.analyze_call(expr)
    }

    fn register_lambda_closure(
        &mut self,
        lambda_name: &str,
        capture_info: &LambdaCaptureInfo,
        ctor_decl: &FunctionDecl,
        op_decl: &mut FunctionDecl,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        self.compute_struct_layout(RecordLayout {
            name: lambda_name,
            is_class: true,
            base_classes: &[],
            fields: &capture_info.fields,
            methods: std::slice::from_ref(ctor_decl),
            private_members: &[],
            line: pos.line,
            col: pos.col,
        })?;

        self.register_struct_method(lambda_name, ctor_decl);
        self.analyze_lambda_operator(lambda_name, op_decl)?;
        self.register_struct_method(lambda_name, op_decl);

        self.synthesized_decls.push(Decl::StructDecl {
            name: lambda_name.to_string(),
            is_class: true,
            base_classes: vec![],
            fields: capture_info.fields.clone(),
            methods: vec![ctor_decl.clone(), op_decl.clone()],
            static_fields: vec![],
            private_members: vec![],
            line: pos.line,
            col: pos.col,
        });

        Ok(())
    }

    fn collect_lambda_captures(
        &self,
        _default_capture: Option<DefaultCapture>,
        captures: &[LambdaCapture],
        _body: &Stmt,
        _params: &[Param],
        pos: ExprPos,
    ) -> Result<LambdaCaptureInfo, CompileError> {
        let mut fields = Vec::new();
        let mut ctor_params = Vec::new();
        let mut ctor_inits = Vec::new();
        let mut ctor_call_args = Vec::new();

        for cap in captures {
            let Some(var) = self.lookup_var(&cap.name) else {
                return Err(CompileError::new(
                    format!(
                        "variable '{}' cannot be implicitly captured in a lambda with no default capture",
                        cap.name
                    ),
                    pos.line,
                    pos.col,
                ));
            };

            let (field_ty, param_ty) = if cap.is_reference {
                let inner = var.ty.strip_reference().clone();
                let ref_ty = Type::Reference(Box::new(inner));
                (ref_ty.clone(), ref_ty)
            } else {
                let val_ty = var.ty.strip_reference().strip_const().clone();
                (val_ty.clone(), val_ty)
            };

            let param_name = format!("__p_{}", cap.name);
            ctor_params.push(Param {
                name: param_name.clone(),
                ty: param_ty,
                default_val: None,
            });
            ctor_inits.push((cap.name.clone(), vec![Expr::Identifier(param_name)]));
            ctor_call_args.push(Expr::Identifier(cap.name.clone()));
            fields.push((cap.name.clone(), field_ty));
        }

        Ok(LambdaCaptureInfo {
            fields,
            ctor_params,
            ctor_inits,
            ctor_call_args,
        })
    }

    fn analyze_lambda_operator(
        &mut self,
        lambda_name: &str,
        op_decl: &mut FunctionDecl,
    ) -> Result<(), CompileError> {
        let saved_scopes = std::mem::take(&mut self.scopes);
        let saved_class = self.current_class.take();
        let saved_func_ret = self.current_func_return_type.take();
        let saved_func_name = self.current_func_name.take();
        let saved_frame_size = self.current_frame_size;
        let saved_line = self.current_line;
        let saved_col = self.current_col;

        self.current_class = Some(lambda_name.to_string());

        let mut op_to_analyze = op_decl.clone();
        op_to_analyze
            .params
            .insert(0, this_param(lambda_name, op_to_analyze.is_const));

        let res = self.analyze_function(&mut op_to_analyze);

        self.scopes = saved_scopes;
        self.current_class = saved_class;
        self.current_func_return_type = saved_func_ret;
        self.current_func_name = saved_func_name;
        self.current_frame_size = saved_frame_size;
        self.current_line = saved_line;
        self.current_col = saved_col;

        res?;
        op_decl.return_type = op_to_analyze.return_type;
        op_decl.body = op_to_analyze.body;
        Ok(())
    }
}

fn build_lambda_ctor(name: &str, info: &LambdaCaptureInfo, pos: ExprPos) -> FunctionDecl {
    FunctionDecl {
        name: name.to_string(),
        return_type: Type::Void,
        params: info.ctor_params.clone(),
        body: Some(Stmt::new(StmtKind::Compound(vec![]), pos.line, pos.col)),
        is_virtual: false,
        is_pure_virtual: false,
        is_const: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        is_static: false,
        ctor_initializers: info.ctor_inits.clone(),
        line: pos.line,
        col: pos.col,
    }
}

fn build_lambda_op(
    params: &[Param],
    body: &Stmt,
    return_type: Option<&Type>,
    is_mutable: bool,
    pos: ExprPos,
) -> FunctionDecl {
    FunctionDecl {
        name: "operator()".to_string(),
        return_type: return_type.cloned().unwrap_or(Type::Auto),
        params: params.to_vec(),
        body: Some(body.clone()),
        is_virtual: false,
        is_pure_virtual: false,
        is_const: !is_mutable,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        is_static: false,
        ctor_initializers: vec![],
        line: pos.line,
        col: pos.col,
    }
}
