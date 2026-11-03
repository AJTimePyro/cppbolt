//! Semantic analysis for `std::initializer_list<T>` and brace-enclosed lists.

use super::*;
use crate::sema::template_deduce::parse_types_list;

/// Whether a constructor can be called with one initializer-list argument.
fn accepts_initializer_list(cand: &FunctionSig) -> bool {
    if cand.param_types.len() < 2 {
        return false;
    }
    if cand.param_types.len() > 2 {
        let defaults = &cand.default_args;
        if defaults.len() < cand.param_types.len() || !defaults[2..].iter().all(|d| d.is_some()) {
            return false;
        }
    }
    true
}

/// Ranks a standard conversion: exact, promotion, then any other conversion.
fn conversion_rank(target: &Type, source: &Type) -> usize {
    let target = target.strip_reference().strip_const();
    let source = source.strip_reference().strip_const();
    if target == source {
        return 0;
    }
    let promoted = matches!(
        source,
        Type::Bool
            | Type::Char
            | Type::SignedChar
            | Type::UnsignedChar
            | Type::Short
            | Type::UnsignedShort
            | Type::Enum(_)
    ) && matches!(target, Type::Int | Type::UnsignedInt);
    if promoted || (source == &Type::Float && target == &Type::Double) {
        return 1;
    }
    2
}

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
        Some(self.normalize_initializer_list_element(parsed.into_iter().next()?))
    }

    /// Rewrites an instantiated template argument to the struct type the
    /// instantiation registered.
    ///
    /// The `initializer_list<E>` parameter keeps `E` as written, so a nested
    /// `std::vector<int>` element arrives as a `TemplateInstance` even though
    /// the class was instantiated. The rest of the analysis only understands
    /// the registered `Struct` name.
    fn normalize_initializer_list_element(&self, ty: Type) -> Type {
        match ty {
            Type::TemplateInstance { name, args } => {
                let args: Vec<Type> = args
                    .into_iter()
                    .map(|arg| self.normalize_initializer_list_element(arg))
                    .collect();
                let plain = QualifiedName::new(&name).plain();
                let canonical: Vec<String> = args.iter().map(|arg| arg.to_c_type()).collect();
                let inst_name = format!("{}<{}>", plain, canonical.join(", "));
                if self.struct_sizes.contains_key(&inst_name) {
                    Type::Struct(inst_name)
                } else {
                    Type::TemplateInstance { name, args }
                }
            }
            other => other,
        }
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
        self.resolve_nested_initializer_lists(elem_ty, init_elements)?;
        for elem in init_elements.iter_mut() {
            if matches!(elem, Expr::InitializerList { .. }) {
                continue;
            }
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

    /// Picks the best constructor of `sname` accepting `std::initializer_list<E>`.
    ///
    /// Every initializer-list constructor is a candidate, and the elements
    /// rank them: an exact conversion beats an integral promotion, which beats
    /// any other standard conversion. A candidate that needs a narrowing
    /// conversion is not viable, but its diagnostic is kept in case no other
    /// candidate matches.
    pub fn find_initializer_list_constructor(
        &mut self,
        sname: &str,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<Option<(FunctionSig, Type)>, CompileError> {
        let candidates: Vec<(FunctionSig, Type)> = self
            .initializer_list_constructors(sname)
            .into_iter()
            .filter(accepts_initializer_list)
            .filter_map(|cand| {
                let elem_ty = self.extract_initializer_list_element_type(&cand.param_types[1])?;
                Some((cand, elem_ty))
            })
            .collect();
        if candidates.is_empty() {
            return Ok(None);
        }

        let mut best: Option<(usize, FunctionSig, Type)> = None;
        let mut narrowing_error = None;
        for (cand, elem_ty) in candidates {
            match self.score_initializer_list_candidate(&elem_ty, init_elements, pos) {
                Ok(Some(score)) => {
                    if best
                        .as_ref()
                        .is_none_or(|(best_score, _, _)| score < *best_score)
                    {
                        best = Some((score, cand, elem_ty));
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    narrowing_error.get_or_insert(error);
                }
            }
        }

        if let Some((_, sig, elem_ty)) = best {
            self.resolve_nested_initializer_lists(&elem_ty, init_elements)?;
            return Ok(Some((sig, elem_ty)));
        }
        match narrowing_error {
            Some(error) => Err(error),
            None => Ok(None),
        }
    }

    fn initializer_list_constructors(&self, sname: &str) -> Vec<FunctionSig> {
        let qname = QualifiedName::new(sname);
        let short = qname.class_short_name();
        let alternate = qname.alternate();
        let mut candidates = Vec::new();
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
        candidates
    }

    fn score_initializer_list_candidate(
        &mut self,
        elem_ty: &Type,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<Option<usize>, CompileError> {
        let mut score = 0;
        for elem in init_elements.iter_mut() {
            if matches!(
                elem,
                Expr::InitializerList { .. } | Expr::TypedInitList { .. }
            ) {
                continue;
            }
            let actual = self.analyze_expr(elem)?;
            self.check_narrowing(elem, &actual, elem_ty, pos)?;
            if !self.can_assign_expr(elem_ty, &actual, Some(elem)) {
                return Ok(None);
            }
            score += conversion_rank(elem_ty, &actual);
        }
        Ok(Some(score))
    }

    /// Resolves braced elements that initialize the chosen element type.
    ///
    /// A nested brace list is only typed once the constructor is known, so
    /// this runs after the ranking and rewrites each nested list for the
    /// element type it initializes.
    fn resolve_nested_initializer_lists(
        &mut self,
        elem_ty: &Type,
        init_elements: &mut [Expr],
    ) -> Result<(), CompileError> {
        for elem in init_elements.iter_mut() {
            if let Expr::InitializerList {
                elements,
                pos: sub_pos,
            } = elem
            {
                self.resolve_list_init(elem_ty, elements, InitContext::DirectList, *sub_pos)?;
            }
        }
        Ok(())
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
            return self.resolve_empty_class_init(sname, init_elements, context, pos);
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

    /// Resolves `T t{}`: the default constructor, or an empty initializer list.
    fn resolve_empty_class_init(
        &mut self,
        sname: &str,
        init_elements: &mut Vec<Expr>,
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
            init_elements.push(Expr::TypedInitList {
                ty: list_ty,
                elements: Vec::new(),
                pos,
            });
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
