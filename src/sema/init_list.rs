//! Semantic analysis for `std::initializer_list<T>` and brace-enclosed lists.

use super::*;
use crate::sema::template_deduce::parse_types_list;

impl Sema {
    /// Extracts the element type `E` if `ty` is `std::initializer_list<E>`.
    pub fn extract_initializer_list_element_type(&self, ty: &Type) -> Option<Type> {
        let clean = ty.strip_reference().strip_const();
        let Type::Struct(sname) = clean else {
            return None;
        };
        let stripped = sname.strip_prefix("std::").unwrap_or(sname);
        let rest = stripped.strip_prefix("initializer_list<")?;
        let inner = rest.strip_suffix('>')?;
        let known_templates: Vec<String> = self.class_templates.keys().cloned().collect();
        let parsed = parse_types_list(inner, &known_templates).ok()?;
        parsed.into_iter().next()
    }

    /// Checks whether `ty` is a `std::initializer_list` specialization.
    #[allow(dead_code)]
    pub fn is_initializer_list_type(&self, ty: &Type) -> bool {
        self.extract_initializer_list_element_type(ty).is_some()
    }

    /// Deduces the element type and instantiated type for `auto x = { ... }`.
    pub fn deduce_auto_initializer_list(
        &mut self,
        elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<(Type, Type), CompileError> {
        if elements.is_empty() {
            return Err(CompileError::new(
                "cannot deduce 'auto' for initializer_list from empty braced list",
                pos.line,
                pos.col,
            ));
        }

        let mut deduced_elem: Option<Type> = None;
        for elem in elements.iter_mut() {
            let actual = self.analyze_expr(elem)?;
            let decayed = actual.strip_reference().strip_const().decay_array();
            if let Some(ref prev) = deduced_elem {
                if prev != &decayed && prev.to_c_type() != decayed.to_c_type() {
                    return Err(CompileError::new(
                        format!(
                            "cannot deduce 'auto' for initializer_list from elements with conflicting types ('{}' and '{}')",
                            prev.to_c_type(),
                            decayed.to_c_type()
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
                self.check_narrowing(elem, &actual, prev, pos)?;
            } else {
                deduced_elem = Some(decayed);
            }
        }

        let Some(elem_ty) = deduced_elem else {
            return Err(CompileError::new(
                "cannot deduce 'auto' for initializer_list",
                pos.line,
                pos.col,
            ));
        };
        let inst_name = self.instantiate_class_template(
            "std::initializer_list",
            std::slice::from_ref(&elem_ty),
            pos.line,
            pos.col,
        )?;
        Ok((elem_ty, Type::Struct(inst_name)))
    }

    /// Validates that all elements in `init_elements` convert to `elem_ty` without narrowing.
    pub fn resolve_initializer_list_elements(
        &mut self,
        elem_ty: &Type,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        for elem in init_elements.iter_mut() {
            let actual = self.analyze_expr(elem)?;
            self.check_narrowing(elem, &actual, elem_ty, pos)?;
            if !self.can_assign_expr(elem_ty, &actual, Some(elem)) {
                return Err(CompileError::new(
                    format!(
                        "cannot initialize initializer_list element of type '{}' with expression of type '{}'",
                        elem_ty.to_c_type(),
                        actual.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ));
            }
        }
        Ok(())
    }

    /// Searches for viable constructors of `sname` accepting `std::initializer_list<E>`.
    pub fn find_initializer_list_constructor(
        &mut self,
        sname: &str,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<Option<(FunctionSig, Type)>, CompileError> {
        let qname = QualifiedName::new(sname);
        let short = qname.class_short_name();
        let alternate = qname.alternate();
        let mut candidates: Vec<FunctionSig> = Vec::new();
        for name in [
            format!("{sname}::{short}"),
            format!("{sname}::{sname}"),
            format!("{alternate}::{short}"),
            format!("{alternate}::{alternate}"),
        ] {
            if let Some(sigs) = self.functions.get(&name) {
                candidates.extend(sigs.iter().cloned());
            }
        }

        for cand in candidates {
            if cand.param_types.len() < 2 {
                continue;
            }
            if cand.param_types.len() > 2 {
                let defaults = &cand.default_args;
                if defaults.len() < cand.param_types.len()
                    || !defaults[2..].iter().all(|d| d.is_some())
                {
                    continue;
                }
            }
            let param1 = &cand.param_types[1];
            if let Some(elem_ty) = self.extract_initializer_list_element_type(param1)
                && self.check_elements_convertible_to(&elem_ty, init_elements, pos)?
            {
                return Ok(Some((cand, elem_ty)));
            }
        }
        Ok(None)
    }

    fn check_elements_convertible_to(
        &mut self,
        elem_ty: &Type,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<bool, CompileError> {
        for elem in init_elements.iter_mut() {
            let actual = self.analyze_expr(elem)?;
            self.check_narrowing(elem, &actual, elem_ty, pos)?;
            if !self.can_assign_expr(elem_ty, &actual, Some(elem)) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Resolves class constructor initialization via list-init with preference for initializer_list.
    pub fn resolve_class_list_init(
        &mut self,
        sname: &str,
        init_elements: &mut Vec<Expr>,
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        if init_elements.is_empty() {
            return self.resolve_empty_class_init(sname, context, pos);
        }

        if let Some((sig, _elem_ty)) =
            self.find_initializer_list_constructor(sname, init_elements, pos)?
        {
            self.instantiate_pending_method(&sig.mangled_name)?;
            if sig.is_explicit && context.is_copy_list() {
                return Err(CompileError::new(
                    format!(
                        "chosen constructor for '{}' is explicit in copy-list-initialization",
                        sname
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            let list_ty = sig.param_types[1].strip_reference().strip_const().clone();
            let original = std::mem::take(init_elements);
            init_elements.push(Expr::TypedInitList {
                ty: list_ty,
                elements: original,
                pos,
            });
            return Ok(Type::Struct(sname.to_string()));
        }

        self.resolve_fallback_class_init(sname, init_elements, context, pos)
    }

    fn resolve_empty_class_init(
        &self,
        sname: &str,
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let empty_args: Vec<Type> = Vec::new();
        if let Some(sig) = self.find_constructor_for(sname, &empty_args, None) {
            if sig.is_explicit && context.is_copy_list() {
                return Err(CompileError::new(
                    format!(
                        "chosen default constructor for '{}' is explicit in copy-list-initialization",
                        sname
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            return Ok(Type::Struct(sname.to_string()));
        }
        Err(CompileError::new(
            format!("no matching default constructor for '{}'", sname),
            pos.line,
            pos.col,
        ))
    }

    fn resolve_fallback_class_init(
        &mut self,
        sname: &str,
        init_elements: &mut [Expr],
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let mut arg_types = Vec::new();
        for elem in init_elements.iter_mut() {
            arg_types.push(self.analyze_expr(elem)?);
        }

        let Some(sig) = self.find_constructor_for(sname, &arg_types, Some(init_elements)) else {
            return Err(CompileError::new(
                format!("no matching constructor for initialization of '{}'", sname),
                pos.line,
                pos.col,
            ));
        };

        if sig.is_explicit && context.is_copy_list() {
            return Err(CompileError::new(
                format!(
                    "chosen constructor for '{}' is explicit in copy-list-initialization",
                    sname
                ),
                pos.line,
                pos.col,
            ));
        }

        for (i, elem) in init_elements.iter().enumerate() {
            if let Some(param_ty) = sig.param_types.get(i + 1) {
                self.check_narrowing(elem, &arg_types[i], param_ty, pos)?;
            }
        }

        Ok(Type::Struct(sname.to_string()))
    }
}
