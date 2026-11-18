use super::*;
use crate::ast::types::Type;
use crate::ast::{BinaryOp, Expr, ExprPos};
use crate::diagnostics::CompileError;

/// The member-pointer type denoted by taking the address of `sig`.
fn member_pointer_type(class_name: &str, sig: &FunctionSig) -> Type {
    let mut quals = sig.qualifiers;
    if !sig.is_static {
        quals.is_const = crate::sema::sig_receiver_is_const(sig);
        quals.ref_qualifier = sig.ref_qualifier;
    }
    let fn_type = Type::Function {
        return_type: Box::new(sig.return_type.clone()),
        param_types: if sig.is_static {
            sig.param_types.clone()
        } else {
            sig.param_types.iter().skip(1).cloned().collect()
        },
        qualifiers: quals,
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

/// C++17 permits converting a pointer to a `noexcept` member function to the
/// corresponding potentially-throwing type, but never the reverse. Other than
/// that exception the function types must match exactly.
fn function_type_compatible(target: &Type, value: &Type) -> bool {
    let (
        Type::Function {
            return_type: tr,
            param_types: tp,
            qualifiers: tq,
        },
        Type::Function {
            return_type: vr,
            param_types: vp,
            qualifiers: vq,
        },
    ) = (target.strip_const(), value.strip_const())
    else {
        return target == value;
    };
    tr == vr
        && tp == vp
        && tq.is_const == vq.is_const
        && tq.is_volatile == vq.is_volatile
        && tq.ref_qualifier == vq.ref_qualifier
        && (!tq.is_noexcept || vq.is_noexcept)
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
            return Self::resolve_unqualified_overload(class_name, overloads, pos);
        };
        let (expected_class, expected_fn) = match expected.strip_const() {
            Type::MemberPointer {
                class_name,
                member_type,
            } => (Some(class_name.as_str()), member_type.as_ref()),
            Type::Pointer(inner) => (None, inner.as_ref()),
            _ => return Self::resolve_unqualified_overload(class_name, overloads, pos),
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
                    && function_type_compatible(expected_fn, &sig_fn)
                    && matches!(
                        self.check_member_pointer_base_conversion(expected_class, &sig_class),
                        super::MemberPointerBaseConversion::Ok
                    )
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
    /// Resolves `&Class::name` when no concrete target type is available.
    /// An overload set cannot initialize `auto` (or any untyped context).
    fn resolve_unqualified_overload(
        class_name: &str,
        overloads: &[FunctionSig],
        pos: ExprPos,
    ) -> Result<FunctionSig, CompileError> {
        if overloads.len() > 1 {
            return Err(CompileError::new(
                format!(
                    "cannot resolve address of overloaded function '{}::{}' without a target type",
                    class_name, overloads[0].name
                ),
                pos.line,
                pos.col,
            ));
        }
        Ok(first_overload(overloads).clone())
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
        self.check_member_access(&class_name, member_name, pos)?;

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
        // Static data members are ordinary globals with a qualified name, not
        // fields of the record layout. Taking their address yields a plain
        // object pointer, not a pointer-to-member.
        let qualified = format!("{}::{}", class_name, member_name);
        if let Some(var) = self.global_vars.get(&qualified) {
            return Ok(Some(MemberPointerAddress {
                ty: Type::Pointer(Box::new(var.ty.clone())),
                encoded_name: None,
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
                        format!(
                            "left operand of '.*' must be a class object, found '{}'",
                            lty.to_c_type()
                        ),
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
                    format!(
                        "left operand of '->*' must be a pointer to class, found '{}'",
                        lty.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn check_member_pointer_application_base(
        &self,
        obj_class: &str,
        target_class: &str,
        rty: &Type,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let obj_class_canonical = self.resolve_alias_in_qualified_name(obj_class);
        let target_class_canonical = self.resolve_alias_in_qualified_name(target_class);
        // Application converts the receiver to the member's class, so the same
        // unambiguous/accessible base rules as Base::* -> Derived::* apply.
        match self
            .check_member_pointer_base_conversion(&obj_class_canonical, &target_class_canonical)
        {
            super::MemberPointerBaseConversion::Ok => {}
            super::MemberPointerBaseConversion::Unrelated => {
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
            super::MemberPointerBaseConversion::Ambiguous => {
                return Err(CompileError::new(
                    format!(
                        "base '{}' is ambiguous in object of type '{}'",
                        target_class, obj_class
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            super::MemberPointerBaseConversion::Inaccessible => {
                return Err(CompileError::new(
                    format!(
                        "base '{}' is inaccessible in object of type '{}'",
                        target_class, obj_class
                    ),
                    pos.line,
                    pos.col,
                ));
            }
        }
        Ok(())
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

        self.check_member_pointer_application_base(&obj_class, target_class, rty, pos)?;

        if let Type::Function { qualifiers, .. } = member_type.strip_const() {
            if is_const_obj && !qualifiers.is_const {
                return Err(CompileError::new(
                    "cannot apply a non-const member function pointer to a const object",
                    pos.line,
                    pos.col,
                ));
            }
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
        if let Some(e) = expr
            && Self::is_null_pointer_constant(e)
        {
            return true;
        }
        let Type::MemberPointer {
            class_name: val_cls,
            member_type: val_mty,
        } = value.strip_const()
        else {
            return false;
        };
        let functions_compatible = matches!(target_mty.strip_const(), Type::Function { .. })
            && matches!(val_mty.strip_const(), Type::Function { .. })
            && function_type_compatible(target_mty, val_mty);
        if !functions_compatible
            && target_mty != val_mty
            && (target_mty.strip_const() != val_mty.strip_const()
                || (!target_mty.is_const() && val_mty.is_const()))
        {
            return false;
        }
        let target_cls = self.resolve_alias_in_qualified_name(target_cls);
        let val_cls = self.resolve_alias_in_qualified_name(val_cls);
        matches!(
            self.check_member_pointer_base_conversion(&target_cls, &val_cls),
            super::MemberPointerBaseConversion::Ok
        )
    }
}
