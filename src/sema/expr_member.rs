use super::*;

impl Sema {
    pub(super) fn analyze_member(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::MemberAccess {
            object,
            member,
            is_arrow,
            pos,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let obj_ty = self.analyze_expr(object)?;
        let struct_name = self.member_struct_name(&obj_ty, *is_arrow, *pos)?;

        if let Some(layout) = self.struct_layouts.get(&struct_name)
            && layout.private_members.contains(member)
        {
            let cur_clean = self
                .current_class
                .as_deref()
                .map(|c| QualifiedName::new(c).plain());
            let s_clean = QualifiedName::new(&struct_name).plain();
            let in_class = cur_clean == Some(s_clean);
            if !in_class {
                return Err(CompileError::new(
                    format!("'{}' is a private member of '{}'", member, struct_name),
                    pos.line,
                    pos.col,
                ));
            }
        }

        if let Some(fld) = self.find_field_in_class(&struct_name, member) {
            Ok(fld.ty.clone())
        } else if self.find_method_in_class(&struct_name, member).is_some() {
            Ok(Type::Function {
                return_type: Box::new(Type::Int),
                param_types: Vec::new(),
            })
        } else {
            Err(CompileError::new(
                format!("No member named '{}' in struct '{}'", member, struct_name),
                pos.line,
                pos.col,
            ))
        }
    }

    /// Resolves the class of a member access target.
    fn member_struct_name(
        &self,
        obj_ty: &Type,
        is_arrow: bool,
        pos: ExprPos,
    ) -> Result<String, CompileError> {
        let struct_name = if is_arrow {
            let stripped = obj_ty.strip_const();
            if let Type::Pointer(inner) | Type::Reference(inner) = stripped {
                match inner.strip_const() {
                    Type::Struct(s) => s.clone(),
                    _ => {
                        return Err(CompileError::new(
                            "Arrow member access on non-struct pointer",
                            pos.line,
                            pos.col,
                        ));
                    }
                }
            } else {
                return Err(CompileError::new(
                    "Arrow member access on non-pointer",
                    pos.line,
                    pos.col,
                ));
            }
        } else {
            let stripped = obj_ty.strip_const();
            let target_ty = if let Type::Reference(inner) = stripped {
                inner.strip_const()
            } else {
                stripped
            };
            match target_ty {
                Type::Struct(s) => s.clone(),
                _ => {
                    return Err(CompileError::new(
                        "Dot member access on non-struct",
                        pos.line,
                        pos.col,
                    ));
                }
            }
        };
        Ok(struct_name)
    }
}
