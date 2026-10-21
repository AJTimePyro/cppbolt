use super::*;

impl Sema {
    pub(super) fn analyze_call(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Call { callee, args } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let mut arg_types = Vec::new();
        for a in args.iter_mut() {
            arg_types.push(self.analyze_expr(a)?);
        }

        if let Expr::Identifier(raw_name) = &**callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name);
            if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
                return Ok(Type::Int);
            }

            let alias = QualifiedName::new(name).alternate();
            let sname_match = if self.struct_layouts.contains_key(name) {
                Some(name.to_string())
            } else if self.struct_layouts.contains_key(alias.as_ref()) {
                Some(alias.into_owned())
            } else {
                None
            };
            if let Some(sname) = sname_match {
                return Ok(Type::Struct(sname));
            }

            if let Some(cls) = &self.current_class
                && self.find_method_in_class(cls, name).is_some()
            {
                **callee = Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: name.to_string(),
                    is_arrow: true,
                };
            }
        }

        if let Some(return_type) = self.analyze_member_callee(callee)? {
            return Ok(return_type);
        }

        if let Expr::Identifier(raw_name) = &**callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
            if let Some(var) = self.lookup_var(&name) {
                let stripped = var.ty.strip_const();
                if let Type::Pointer(inner) = stripped {
                    if let Type::Function { return_type, .. } = &**inner {
                        return Ok((**return_type).clone());
                    }
                } else if let Type::Function { return_type, .. } = stripped {
                    return Ok((**return_type).clone());
                }
            }
            return self.resolve_named_call(&name, args, &arg_types, callee);
        }

        // No named function matched: treat the callee as a function pointer.
        let callee_ty = self.analyze_expr(callee)?;
        if let Type::Pointer(inner) = callee_ty.strip_const() {
            if let Type::Function { return_type, .. } = &**inner {
                return Ok((**return_type).clone());
            }
        } else if let Type::Function { return_type, .. } = callee_ty.strip_const() {
            return Ok((**return_type).clone());
        }
        Ok(Type::Int)
    }
    fn analyze_member_callee(&mut self, callee: &mut Expr) -> Result<Option<Type>, CompileError> {
        if let Expr::MemberAccess {
            object,
            member,
            is_arrow,
        } = &mut *callee
        {
            let obj_ty = self.analyze_expr(object.as_mut())?;
            let struct_name = if *is_arrow {
                let stripped = obj_ty.strip_const();
                if let Type::Pointer(inner) | Type::Reference(inner) = stripped {
                    match inner.strip_const() {
                        Type::Struct(s) => s.clone(),
                        _ => {
                            return Err(CompileError::new(
                                "Arrow member access on non-struct pointer",
                                self.current_line,
                                self.current_col,
                            ));
                        }
                    }
                } else {
                    return Err(CompileError::new(
                        "Arrow member access on non-pointer",
                        self.current_line,
                        self.current_col,
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
                            self.current_line,
                            self.current_col,
                        ));
                    }
                }
            };

            if member.starts_with('~') {
                return Ok(Some(Type::Void));
            }
            let receiver_const = crate::sema::receiver_is_const_type(&obj_ty);
            if let Some(sig) =
                self.find_method_in_class_for_receiver(&struct_name, member, receiver_const)
            {
                return Ok(Some(sig.return_type.clone()));
            } else {
                return Err(CompileError::new(
                    format!("No member named '{}' in struct '{}'", member, struct_name),
                    self.current_line,
                    self.current_col,
                ));
            }
        }
        Ok(None)
    }

    fn resolve_named_call(
        &mut self,
        name: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        callee: &mut Expr,
    ) -> Result<Type, CompileError> {
        // Overload resolution: exact matches score best, followed by
        // reference binding, arithmetic conversions, and user conversions.
        let candidates = self.find_call_candidates(name, arg_types);
        if let Some(candidates) = candidates {
            let mut best_candidate: Option<(&FunctionSig, Vec<Expr>)> = None;
            let mut best_score = usize::MAX;

            for cand in &candidates {
                let min_args = cand.default_args.iter().take_while(|d| d.is_none()).count();
                let max_args = cand.param_types.len();

                if args.len() >= min_args && args.len() <= max_args {
                    let Some(score) = self.score_call_candidate(cand, arg_types) else {
                        continue;
                    };

                    if score < best_score {
                        best_score = score;
                        let mut full_args = args.clone();
                        for i in args.len()..max_args {
                            if let Some(def_expr) = &cand.default_args[i] {
                                full_args.push(def_expr.clone());
                            }
                        }
                        best_candidate = Some((cand, full_args));
                        if score == 0 {
                            break; // No candidate can score lower.
                        }
                    }
                }
            }

            if let Some((cand, full_args)) = best_candidate {
                let mangled = cand.mangled_name.clone();
                let ret_type = cand.return_type.clone();
                *callee = Expr::Identifier(mangled);
                *args = full_args;
                return Ok(ret_type);
            }
            let cand = &candidates[0];
            return Err(CompileError::new(
                format!(
                    "No matching function call for '{}': expected {} arguments, found {}",
                    name,
                    cand.param_types.len(),
                    args.len()
                ),
                self.current_line,
                self.current_col,
            ));
        }
        Err(CompileError::new(
            format!("Undefined function '{}'", name),
            self.current_line,
            self.current_col,
        ))
    }
    /// Scores how well a candidate matches the argument types.
    ///
    /// Exact matches add no penalty, reference bindings cost 1, and other
    /// conversions cost 2. Returns `None` when the arguments do not match.
    fn score_call_candidate(&self, cand: &FunctionSig, arg_types: &[Type]) -> Option<usize> {
        let mut score = 0;
        for (i, arg_ty) in arg_types.iter().enumerate() {
            let p = &cand.param_types[i];
            let p_decay = p.decay_array();
            let a_decay = arg_ty.decay_array();
            let p_base = p_decay.strip_const();
            let a_base = a_decay.strip_const();
            if p_base == a_base {
                // Exact matches add no conversion penalty.
            } else if p.is_reference()
                && p.deref_type().map(|d| d.strip_const()) == Some(arg_ty.strip_const())
            {
                score += 1;
            } else if self.can_assign(p, arg_ty) {
                score += 2;
            } else {
                return None;
            }
        }
        Some(score)
    }

    fn find_call_candidates(&mut self, name: &str, arg_types: &[Type]) -> Option<Vec<FunctionSig>> {
        let q1 = name.to_string();
        let q2 = QualifiedName::new(name).alternate().into_owned();
        let mut candidates = self
            .functions
            .get(&q1)
            .or_else(|| self.functions.get(&q2))
            .or_else(|| {
                self.using_decls
                    .get(name)
                    .and_then(|full| self.functions.get(full))
            })
            .or_else(|| {
                self.open_namespaces
                    .iter()
                    .find_map(|ns| self.functions.get(&format!("{}::{}", ns, name)))
            })
            .cloned();
        if candidates.is_none() {
            for aty in arg_types {
                let clean = aty.strip_const().strip_reference();
                if let Type::Struct(s) = clean
                    && let Some((ns, _)) = s.rsplit_once("::")
                    && let Some(sigs) = self.functions.get(&format!("{}::{}", ns, name))
                {
                    candidates = Some(sigs.clone());
                    break;
                }
            }
        }
        if candidates.is_none()
            && let Some((class_name, method_name)) = name.rsplit_once("::")
        {
            let static_methods = self.find_static_methods_in_class(class_name, method_name);
            if !static_methods.is_empty() {
                candidates = Some(static_methods);
            }
        }
        candidates
    }
}
