with open("src/codegen.rs", "r") as f:
    text = f.read()

# 1. Update get_expr_type
old_get_expr_call = """            Expr::Call { callee, .. } => {
                if let Expr::Identifier(name) = &**callee {
                    if name == "printf" {
                        Type::Int
                    } else if let Some(cands) = self.sema.functions.get(name) {
                        cands[0].return_type.clone()
                    } else {
                        Type::Int
                    }
                } else {
                    Type::Int
                }
            }"""

new_get_expr_call = """            Expr::Call { callee, .. } => {
                match &**callee {
                    Expr::Identifier(name) => {
                        if name == "printf" {
                            Type::Int
                        } else if let Some(cands) = self.sema.functions.get(name) {
                            cands[0].return_type.clone()
                        } else {
                            Type::Int
                        }
                    }
                    Expr::MemberAccess { object, member, is_arrow } => {
                        let obj_ty = self.get_expr_type(object, ctx);
                        let sname = if *is_arrow {
                            if let Type::Pointer(inner) | Type::Reference(inner) = obj_ty.strip_const() {
                                if let Type::Struct(s) = inner.strip_const() { s.clone() } else { String::new() }
                            } else { String::new() }
                        } else {
                            if let Type::Struct(s) = obj_ty.strip_const() { s.clone() } else { String::new() }
                        };
                        if let Some(sig) = self.sema.find_method_in_class(&sname, member) {
                            sig.return_type.clone()
                        } else {
                            Type::Int
                        }
                    }
                    _ => Type::Int,
                }
            }"""

assert old_get_expr_call in text
text = text.replace(old_get_expr_call, new_get_expr_call, 1)

old_get_expr_end = """            Expr::SizeofType(_) | Expr::SizeofExpr(_) => Type::UnsignedInt,
            Expr::InitializerList(_) => Type::Void,
        }"""

new_get_expr_end = """            Expr::SizeofType(_) | Expr::SizeofExpr(_) => Type::UnsignedInt,
            Expr::InitializerList(_) => Type::Void,
            Expr::This => {
                if let Some(cls) = &ctx.current_class {
                    Type::Pointer(Box::new(Type::Struct(cls.clone())))
                } else {
                    Type::Pointer(Box::new(Type::Void))
                }
            }
            Expr::New { ty, .. } => Type::Pointer(Box::new(ty.clone())),
            Expr::Delete { .. } => Type::Void,
        }"""

assert old_get_expr_end in text
text = text.replace(old_get_expr_end, new_get_expr_end, 1)

with open("src/codegen.rs", "w") as f:
    f.write(text)

print("Stage 2 get_expr_type updated!")
