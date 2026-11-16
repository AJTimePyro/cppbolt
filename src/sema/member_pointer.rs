use super::*;
use crate::ast::types::Type;
use crate::ast::{BinaryOp, Expr, ExprPos};
use crate::diagnostics::CompileError;

impl Sema {
    pub(super) fn try_resolve_member_pointer_address(
        &mut self,
        name: &str,
        pos: ExprPos,
    ) -> Result<Option<Type>, CompileError> {
        let Some((class_raw, member_name)) = name.rsplit_once("::") else {
            return Ok(None);
        };
        let class_part = class_raw.strip_prefix("::").unwrap_or(class_raw);
        let class_name = self.resolve_alias_in_qualified_name(class_part);
        self.ensure_class_instantiated(&class_name, pos.line, pos.col);

        if let Some(fld) = self.find_field_in_class(&class_name, member_name) {
            return Ok(Some(Type::MemberPointer {
                class_name: class_name.to_string(),
                member_type: Box::new(fld.ty),
            }));
        }

        if let Some(sig) = self.find_method_in_class(&class_name, member_name) {
            let fn_type = Type::Function {
                return_type: Box::new(sig.return_type.clone()),
                param_types: if sig.is_static {
                    sig.param_types.clone()
                } else {
                    sig.param_types.iter().skip(1).cloned().collect()
                },
            };
            if sig.is_static {
                return Ok(Some(Type::Pointer(Box::new(fn_type))));
            }
            return Ok(Some(Type::MemberPointer {
                class_name: class_name.to_string(),
                member_type: Box::new(fn_type),
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
            return Ok((**member_type).clone());
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
