use super::*;
use crate::ast::types::Type;
use crate::ast::{BinaryOp, Expr, ExprPos};
use crate::diagnostics::CompileError;

/// The member-pointer type denoted by taking the address of `sig`.
fn member_pointer_type(class_name: &str, sig: &FunctionSig) -> Type {
    let fn_type = Type::Function {
        return_type: Box::new(sig.return_type.clone()),
        param_types: if sig.is_static {
            sig.param_types.clone()
        } else {
            sig.param_types.iter().skip(1).cloned().collect()
        },
    };
    if sig.is_static {
        Type::Pointer(Box::new(fn_type))
    } else {
        Type::MemberPointer {
            class_name: class_name.to_string(),
            member_type: Box::new(fn_type),
        }
    }
}

/// The resolved type of `&Class::member` plus the exact method overload chosen.
pub(super) struct MemberPointerAddress {
    pub ty: Type,
    /// Encoding `class@@mangled-name` of the selected non-static method, used
    /// by codegen to emit the exact overload. `None` for data members and
    /// static methods.
    pub encoded_name: Option<String>,
}

fn first_overload(overloads: &[FunctionSig]) -> &FunctionSig {
    overloads
        .iter()
        .find(|sig| !sig_receiver_is_const(sig))
        .unwrap_or(&overloads[0])
}

impl Sema {
    /// Picks the overload whose member-pointer type matches the type expected
    /// at this site. Falls back to the first overload when no target type is
    /// available; reports an error when a member-pointer target matches none.
    fn select_member_pointer_overload(
        &self,
        class_name: &str,
        overloads: &[FunctionSig],
        expected: Option<&Type>,
        pos: ExprPos,
    ) -> Result<FunctionSig, CompileError> {
        let Some(expected) = expected else {
            return Ok(first_overload(overloads).clone());
        };
        let (expected_class, expected_fn) = match expected.strip_const() {
            Type::MemberPointer {
                class_name,
                member_type,
            } => (Some(class_name.as_str()), member_type.as_ref()),
            Type::Pointer(inner) => (None, inner.as_ref()),
            _ => return Ok(first_overload(overloads).clone()),
        };
        for sig in overloads {
            if member_pointer_type(class_name, sig) == *expected {
                return Ok(sig.clone());
            }
        }
        if let Some(expected_class) = expected_class
            && let Type::Function { .. } = expected_fn
        {
            for sig in overloads {
                if let Type::MemberPointer {
                    class_name: sig_class,
                    member_type: sig_fn,
                } = member_pointer_type(class_name, sig)
                    && *sig_fn == *expected_fn
                    && self.is_subclass(expected_class, &sig_class)
                {
                    return Ok(sig.clone());
                }
            }
        }
        Err(CompileError::new(
            format!(
                "no overload of '{}::{}' matches member pointer type '{}'",
                class_name,
                overloads[0].name,
                expected.to_c_type()
            ),
            pos.line,
            pos.col,
        ))
    }

    pub(super) fn try_resolve_member_pointer_address(
        &mut self,
        name: &str,
        pos: ExprPos,
    ) -> Result<Option<MemberPointerAddress>, CompileError> {
        let Some((class_raw, member_name)) = name.rsplit_once("::") else {
            return Ok(None);
        };
        let class_part = class_raw.strip_prefix("::").unwrap_or(class_raw);
        let class_name = self.resolve_alias_in_qualified_name(class_part);
        self.ensure_class_instantiated(&class_name, pos.line, pos.col);

        if let Some(fld) = self.find_field_in_class(&class_name, member_name) {
            return Ok(Some(MemberPointerAddress {
                ty: Type::MemberPointer {
                    class_name: class_name.to_string(),
                    member_type: Box::new(fld.ty),
                },
                encoded_name: None,
            }));
        }

        let overloads = self.find_method_overloads_in_class(&class_name, member_name, false);
        if !overloads.is_empty() {
            let expected = self.expected_type.clone();
            let sig = self.select_member_pointer_overload(
                &class_name,
                &overloads,
                expected.as_ref(),
                pos,
            )?;
            return Ok(Some(MemberPointerAddress {
                ty: member_pointer_type(&class_name, &sig),
                encoded_name: (!sig.is_static)
                    .then(|| format!("{}@@{}", class_name, sig.mangled_name)),
            }));
        }

        if self.struct_layouts.contains_key(&class_name) {
            return Err(CompileError::new(
                format!("no member named '{}' in '{}'", member_name, class_name),
                pos.line,
                pos.col,
            ));
        }

        Ok(None)
    }

    fn resolve_member_pointer_receiver(
        &self,
        op: BinaryOp,
        lty: &Type,
        pos: ExprPos,
    ) -> Result<(String, bool), CompileError> {
        match op {
            BinaryOp::DotStar => {
                let clean = lty.strip_reference();
                let is_c = clean.is_const();
                match clean.strip_const() {
                    Type::Struct(name) => Ok((name.clone(), is_c)),
                    _ => Err(CompileError::new(
                        format!("left operand of '.*' must be a class object, found '{}'", lty.to_c_type()),
                        pos.line,
                        pos.col,
                    )),
                }
            }
            BinaryOp::ArrowStar => {
                let clean = lty.strip_reference().strip_const();
                if let Type::Pointer(inner) = clean {
                    let is_c = inner.is_const();
                    if let Type::Struct(name) = inner.strip_const() {
                        return Ok((name.clone(), is_c));
                    }
                }
                Err(CompileError::new(
                    format!("left operand of '->*' must be a pointer to class, found '{}'", lty.to_c_type()),
                    pos.line,
                    pos.col,
                ))
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn check_member_pointer_binary(
        &self,
        op: BinaryOp,
        lty: &Type,
        rty: &Type,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let (obj_class, is_const_obj) = self.resolve_member_pointer_receiver(op, lty, pos)?;

        let Type::MemberPointer {
            class_name: target_class,
            member_type,
        } = rty.strip_reference().strip_const()
        else {
            return Err(CompileError::new(
                format!(
                    "right operand of '{}' must be a pointer-to-member, found '{}'",
                    if op == BinaryOp::DotStar { ".*" } else { "->*" },
                    rty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        };

        if !self.is_subclass(&obj_class, target_class) {
            return Err(CompileError::new(
                format!(
                    "pointer-to-member of type '{}' is incompatible with object of type '{}'",
                    rty.to_c_type(),
                    obj_class
                ),
                pos.line,
                pos.col,
            ));
        }

        if matches!(member_type.strip_const(), Type::Function { .. }) {
            return Ok(Type::BoundMemberFunction {
                class_name: target_class.clone(),
                member_type: member_type.clone(),
            });
        }

        let mut res_type = (**member_type).clone();
        if is_const_obj && !res_type.is_const() && !res_type.is_reference() {
            res_type = Type::Const(Box::new(res_type));
        }
        Ok(res_type)
    }

    pub(super) fn member_pointer_assignable(
        &self,
        target: &Type,
        value: &Type,
        expr: Option<&Expr>,
    ) -> bool {
        let Type::MemberPointer {
            class_name: target_cls,
            member_type: target_mty,
        } = target.strip_const()
        else {
            return false;
        };
        if matches!(value.strip_const(), Type::Nullptr) {
            return true;
        }
        if let Some(e) = expr && Self::is_null_pointer_constant(e) {
            return true;
        }
        let Type::MemberPointer {
            class_name: val_cls,
            member_type: val_mty,
        } = value.strip_const()
        else {
            return false;
        };
        if target_mty != val_mty
            && (target_mty.strip_const() != val_mty.strip_const()
                || (!target_mty.is_const() && val_mty.is_const()))
        {
            return false;
        }
        self.is_subclass(target_cls, val_cls)
    }
}
