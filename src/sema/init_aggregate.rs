//! C++17 aggregate brace elision for nested arrays and aggregate fields.

use super::*;

impl Sema {
    pub fn is_subaggregate_element_init(&self, elem_ty: &Type, expr: &Expr) -> bool {
        match expr {
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                self.is_subaggregate_element_elements(elem_ty, elements)
            }
            _ => false,
        }
    }

    fn is_subaggregate_element_elements(&self, elem_ty: &Type, elements: &[Expr]) -> bool {
        let is_elem_aggregate = match elem_ty.strip_const() {
            Type::Array(..) => true,
            Type::Struct(sname) => self.is_cxx17_aggregate(sname),
            _ => false,
        };
        if !is_elem_aggregate {
            return false;
        }
        !elements
            .iter()
            .any(|e| matches!(e, Expr::InitializerList { .. } | Expr::TypedInitList { .. }))
    }

    pub(super) fn resolve_aggregate_field_elided(
        &mut self,
        target_ty: &Type,
        init_elements: &mut [Expr],
        elem_idx: &mut usize,
        context: InitContext,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        if *elem_idx >= init_elements.len() {
            return Ok(());
        }
        if let Expr::InitializerList {
            elements: sub_elements,
            pos: sub_pos,
        } = &mut init_elements[*elem_idx]
        {
            if let Type::Array(elem_ty, len) = target_ty.strip_const()
                && self.is_subaggregate_element_elements(elem_ty, sub_elements)
            {
                for _ in 0..*len {
                    if *elem_idx >= init_elements.len() {
                        break;
                    }
                    self.resolve_aggregate_field_elided(
                        elem_ty,
                        init_elements,
                        elem_idx,
                        context,
                        pos,
                    )?;
                }
                return Ok(());
            }
            self.resolve_list_init(target_ty, sub_elements, context, *sub_pos)?;
            *elem_idx += 1;
            return Ok(());
        }
        if let Type::Array(elem_ty, len) = target_ty.strip_const() {
            for _ in 0..*len {
                if *elem_idx >= init_elements.len() {
                    break;
                }
                self.resolve_aggregate_field_elided(
                    elem_ty,
                    init_elements,
                    elem_idx,
                    context,
                    pos,
                )?;
            }
            return Ok(());
        }
        if let Type::Struct(sname) = target_ty.strip_const()
            && self.is_cxx17_aggregate(sname)
            && let Some(layout) = self.struct_layouts.get(sname).cloned()
        {
            for fld in &layout.fields {
                if *elem_idx >= init_elements.len() {
                    break;
                }
                self.resolve_aggregate_field_elided(
                    &fld.ty,
                    init_elements,
                    elem_idx,
                    context,
                    pos,
                )?;
            }
            return Ok(());
        }
        self.resolve_aggregate_scalar(target_ty, init_elements, elem_idx, pos)
    }

    fn resolve_aggregate_scalar(
        &mut self,
        target_ty: &Type,
        init_elements: &mut [Expr],
        elem_idx: &mut usize,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let elem = &mut init_elements[*elem_idx];
        *elem_idx += 1;
        let elem_ty = self.analyze_expr(elem)?;
        self.check_narrowing(elem, &elem_ty, target_ty, pos)?;
        if !self.can_assign_expr(target_ty, &elem_ty, Some(elem)) {
            return Err(CompileError::new(
                format!(
                    "cannot initialize field of type '{}' with expression of type '{}'",
                    target_ty.to_c_type(),
                    elem_ty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }
        Ok(())
    }
}
