use super::*;

impl<'a> Codegen<'a> {
    /// Returns the type of a materialized expression; references decay to the
    /// referred-to type.
    pub fn get_expr_type(&self, expr: &Expr, ctx: &FunctionContext) -> Type {
        let ty = self.get_expr_type_raw(expr, ctx);
        ty.strip_reference().clone()
    }

    /// Returns the expression's type as written, preserving references.
    pub fn get_expr_type_raw(&self, expr: &Expr, ctx: &FunctionContext) -> Type {
        match expr {
            Expr::IntLiteral(_, ty) => ty.clone(),
            Expr::FloatLiteral(_, ty) => ty.clone(),
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::CharLiteral(_) => Type::Char,
            Expr::StringLiteral(_) => Type::Pointer(Box::new(Type::Char)),
            Expr::This => {
                if let Some(cls) = &ctx.current_class {
                    Type::Pointer(Box::new(Type::Struct(cls.clone())))
                } else {
                    Type::Int
                }
            }

            Expr::Identifier(name) => self.type_of_identifier(name, ctx),
            Expr::Binary { op, left, right } => self.type_of_binary(op, left, right, ctx),
            Expr::Unary { op, expr } => self.type_of_unary(op, expr, ctx),
            Expr::Assign { target, .. } => self.get_expr_type(target, ctx),
            Expr::CompoundAssign { op, target, value } => {
                self.type_of_compound_assign(op, target, value, ctx)
            }
            Expr::Call { callee, .. } => self.type_of_call(callee, ctx),
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
            } => self.type_of_member_access(object, member, *is_arrow, ctx),
            Expr::ArraySubscript { array, index } => {
                self.type_of_array_subscript(array, index, ctx)
            }
            Expr::Ternary { then_expr, .. } => self.get_expr_type(then_expr, ctx),
            Expr::Cast { target_type, .. } => target_type.clone(),
            Expr::Delete { .. } => Type::Void,
            _ => Type::Int,
        }
    }

    fn type_of_identifier(&self, name: &str, ctx: &FunctionContext) -> Type {
        if let Some(ty) = ctx.var_types.get(name) {
            ty.clone()
        } else if self.sema.enum_values.contains_key(name) {
            Type::Int
        } else if let Some(sym) = self
            .sema
            .lookup_var_in_context(name, Some(&ctx.current_func_name))
        {
            sym.ty.clone()
        } else if let Some(cls) = &ctx.current_class {
            if let Some(fld) = self.sema.find_field_in_class(cls, name) {
                fld.ty.clone()
            } else if let Some(sym) = self.sema.lookup_var(&format!("{}::{}", cls, name)) {
                sym.ty.clone()
            } else {
                Type::Int
            }
        } else if let Some(sig) = self
            .sema
            .functions
            .get(name)
            .or_else(|| self.sema.functions.get(&format!("std::{}", name)))
            .or_else(|| self.sema.functions.get(QualifiedName::new(name).plain()))
            .and_then(|sigs| sigs.first())
        {
            Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(sig.return_type.clone()),
                param_types: sig.param_types.clone(),
            }))
        } else {
            Type::Int
        }
    }

    fn type_of_binary(
        &self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
        ctx: &FunctionContext,
    ) -> Type {
        let lty = self.get_expr_type(left, ctx);
        let rty = self.get_expr_type(right, ctx);
        if (matches!(lty.strip_const(), Type::Struct(_) | Type::Enum(_))
            || matches!(rty.strip_const(), Type::Struct(_) | Type::Enum(_)))
            && let Some(op_name) = op.op_name()
        {
            let clean_lty = lty.strip_const();
            if let Type::Struct(sname) = clean_lty
                && let Some((_, sig)) = self.find_method(sname, op_name, std::slice::from_ref(&rty))
            {
                return sig.return_type.clone();
            }
            if let Some((_, sig)) = self
                .find_function(op_name, &[lty.clone(), rty.clone()])
                .or_else(|| {
                    self.find_function(&format!("std::{}", op_name), &[lty.clone(), rty.clone()])
                })
            {
                return sig.return_type.clone();
            }
        }
        match op {
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => Type::Bool,
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Type::Bool,
            BinaryOp::Shl | BinaryOp::Shr => lty.strip_reference().integer_promotion(),
            _ => {
                let lty_d = lty.decay_array();
                let rty_d = rty.decay_array();
                if lty_d.is_pointer() && rty.is_integer() {
                    lty_d
                } else if rty_d.is_pointer() && lty.is_integer() && *op == BinaryOp::Add {
                    rty_d
                } else if lty_d.is_pointer() && rty_d.is_pointer() && *op == BinaryOp::Sub {
                    Type::Int
                } else {
                    lty.strip_reference()
                        .usual_arithmetic_conversion(rty.strip_reference())
                }
            }
        }
    }

    fn type_of_unary(&self, op: &UnaryOp, expr: &Expr, ctx: &FunctionContext) -> Type {
        let ty = self.get_expr_type(expr, ctx);
        if let Type::Struct(sname) = ty.strip_const() {
            let op_m = match op {
                UnaryOp::Neg => Some("operator-"),
                UnaryOp::PreInc => Some("operator++"),
                UnaryOp::PreDec => Some("operator--"),
                _ => None,
            };
            if let Some(m) = op_m
                && let Some((_, sig)) = self.find_method(sname, m, &[])
            {
                return sig.return_type.clone();
            }
        }
        match op {
            UnaryOp::Not => Type::Bool,
            UnaryOp::Deref => {
                let ty = self.get_expr_type(expr, ctx);
                ty.deref_type().cloned().unwrap_or(Type::Int)
            }
            UnaryOp::AddrOf => Type::Pointer(Box::new(self.get_expr_type(expr, ctx))),
            UnaryOp::BitNot | UnaryOp::Neg | UnaryOp::Plus => {
                self.get_expr_type(expr, ctx).integer_promotion()
            }
            _ => self.get_expr_type(expr, ctx),
        }
    }

    fn type_of_compound_assign(
        &self,
        op: &BinaryOp,
        target: &Expr,
        value: &Expr,
        ctx: &FunctionContext,
    ) -> Type {
        let ty = self.get_expr_type(target, ctx);
        let val_ty = self.get_expr_type(value, ctx);
        let op_name = format!(
            "operator{}=",
            match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
                _ => "",
            }
        );
        if let Type::Struct(sname) = ty.strip_const()
            && let Some((_, sig)) = self.find_method(sname, &op_name, std::slice::from_ref(&val_ty))
        {
            return sig.return_type.clone();
        }
        self.get_expr_type(target, ctx)
    }

    fn type_of_call(&self, callee: &Expr, ctx: &FunctionContext) -> Type {
        match callee {
            Expr::Identifier(name) => {
                if name == "printf"
                    || name == "std::printf"
                    || name.ends_with("::printf")
                    || name == "__builtin_wasm_memory_size"
                {
                    Type::Int
                } else if name == "print_char"
                    || name == "print_int"
                    || name == "print_double"
                    || name == "free"
                    || name == "exit"
                {
                    Type::Void
                } else if name == "read_char" {
                    Type::Int
                } else if self.sema.struct_layouts.contains_key(name) {
                    Type::Struct(name.clone())
                } else if let Some(ty) = ctx.var_types.get(name) {
                    if let Type::Pointer(inner) = ty.strip_const() {
                        if let Type::Function { return_type, .. } = &**inner {
                            (**return_type).clone()
                        } else {
                            Type::Int
                        }
                    } else if let Type::Function { return_type, .. } = ty.strip_const() {
                        (**return_type).clone()
                    } else {
                        Type::Int
                    }
                } else if let Some(sym) = self.sema.global_vars.get(name) {
                    if let Type::Pointer(inner) = sym.ty.strip_const() {
                        if let Type::Function { return_type, .. } = &**inner {
                            (**return_type).clone()
                        } else {
                            Type::Int
                        }
                    } else if let Type::Function { return_type, .. } = sym.ty.strip_const() {
                        (**return_type).clone()
                    } else {
                        Type::Int
                    }
                } else {
                    let mut found_ret = None;
                    if let Some(cls) = &ctx.current_class
                        && let Some(sig) = self.sema.find_method_in_class(cls, name)
                    {
                        found_ret = Some(sig.return_type.clone());
                    }
                    if found_ret.is_none()
                        && let Some(cands) = self
                            .sema
                            .functions
                            .get(name)
                            .or_else(|| self.sema.functions.get(&format!("std::{}", name)))
                            .or_else(|| self.sema.functions.get(QualifiedName::new(name).plain()))
                    {
                        found_ret = Some(cands[0].return_type.clone());
                    }
                    found_ret.unwrap_or(Type::Int)
                }
            }
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
            } => {
                let obj_ty = self.get_expr_type_raw(object, ctx);
                let is_ref = obj_ty.is_reference();
                let sname = if *is_arrow || is_ref {
                    if let Type::Pointer(inner) | Type::Reference(inner) = obj_ty.strip_const() {
                        if let Type::Struct(s) = inner.strip_const() {
                            s.clone()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    if let Type::Struct(s) = obj_ty.strip_const() {
                        s.clone()
                    } else {
                        String::new()
                    }
                };
                if let Some(sig) = self.sema.find_method_in_class(&sname, member) {
                    sig.return_type.clone()
                } else {
                    Type::Int
                }
            }
            _ => Type::Int,
        }
    }

    fn type_of_member_access(
        &self,
        object: &Expr,
        member: &str,
        is_arrow: bool,
        ctx: &FunctionContext,
    ) -> Type {
        let obj_ty = self.get_expr_type_raw(object, ctx);
        let is_ref = obj_ty.is_reference();
        let sname = if is_arrow || is_ref {
            if let Type::Pointer(inner) | Type::Reference(inner) = obj_ty.strip_const() {
                if let Type::Struct(s) = inner.strip_const() {
                    s.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            if let Type::Struct(s) = obj_ty.strip_const() {
                s.clone()
            } else {
                String::new()
            }
        };
        if let Some(fld) = self.sema.find_field_in_class(&sname, member) {
            fld.ty
        } else {
            Type::Int
        }
    }

    fn type_of_array_subscript(&self, array: &Expr, index: &Expr, ctx: &FunctionContext) -> Type {
        let arr_ty = self.get_expr_type(array, ctx);
        let clean_ty = arr_ty.strip_const();
        if let Type::Struct(sname) = clean_ty {
            let idx_ty = self.get_expr_type(index, ctx);
            let receiver_const = crate::sema::receiver_is_const_type(&arr_ty);
            if let Some((_, sig)) =
                self.find_method_for_receiver(sname, "operator[]", &[idx_ty], receiver_const)
            {
                return sig.return_type.clone();
            }
        }
        arr_ty.deref_type().cloned().unwrap_or(Type::Int)
    }
}
