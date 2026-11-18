//! Default-member-initializer checking shared by struct registration.

use super::*;

impl Sema {
    pub(super) fn register_enum_decl(
        &mut self,
        name: &str,
        is_scoped: bool,
        variants: &[(String, i64)],
    ) {
        if is_scoped {
            self.scoped_enums.insert(name.to_string());
        }
        for (vname, val) in variants {
            let scoped_name = format!("{}::{}", name, vname);
            self.enum_values.insert(scoped_name.clone(), *val);
            self.enum_types.insert(scoped_name, name.to_string());
            if !is_scoped {
                self.enum_values.insert(vname.clone(), *val);
                self.enum_types.insert(vname.clone(), name.to_string());
            }
        }
    }

    pub(super) fn check_field_default_init(
        &mut self,
        fname: &str,
        fty: &Type,
        init_expr: &mut Expr,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (init_ty, is_list) = match init_expr {
            Expr::InitializerList { elements, pos } => {
                let context = if elements.is_empty() {
                    InitContext::ValueInit
                } else {
                    InitContext::CopyListInitVar
                };
                (self.resolve_list_init(fty, elements, context, *pos)?, true)
            }
            Expr::TypedInitList {
                ty: explicit_ty,
                elements,
                pos,
            } => (
                self.resolve_list_init(explicit_ty, elements, InitContext::DirectList, *pos)?,
                true,
            ),
            _ => (self.analyze_expr_keeping_reference(init_expr)?, false),
        };
        if !is_list && !self.can_assign_expr(fty, &init_ty, Some(init_expr)) {
            return Err(CompileError::new(
                format!(
                    "cannot initialize field '{}' of type '{}' with expression of type '{}'",
                    fname,
                    fty.to_c_type(),
                    init_ty.to_c_type()
                ),
                pos.0,
                pos.1,
            ));
        }
        Ok(())
    }
}
