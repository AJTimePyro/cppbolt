use super::*;

impl<'a> Codegen<'a> {
    /// Resolves a free-function call, including ADL and the implicit `std`
    /// alias, selecting the lowest-cost overload.
    pub fn find_function(&self, name: &str, arg_types: &[Type]) -> Option<(u32, FunctionSig)> {
        let mut candidate_names = vec![
            name.to_string(),
            QualifiedName::new(name).alternate().into_owned(),
        ];
        for arg in arg_types {
            if let Type::Struct(sname) = arg.strip_const().strip_reference()
                && let Some((ns, _)) = sname.rsplit_once("::")
            {
                let adl_name = format!("{}::{}", ns, name);
                if !candidate_names.contains(&adl_name) {
                    candidate_names.push(adl_name);
                }
            }
        }
        for q in &candidate_names {
            if let Some(sigs) = self.sema.functions.get(q) {
                let mut best_sig = None;
                let mut best_score = usize::MAX;
                for sig in sigs {
                    if sig.param_types.len() == arg_types.len() {
                        let mut score = 0;
                        let mut ok = true;
                        for (p, a) in sig.param_types.iter().zip(arg_types) {
                            let p_decay = p.decay_array();
                            let a_decay = a.decay_array();
                            if p_decay.strip_all_const() == a_decay.strip_all_const() {
                                if p_decay == a_decay {
                                    score += 0;
                                } else {
                                    score += 1;
                                }
                            } else if p.is_reference()
                                && p.deref_type().map(|d| d.strip_all_const())
                                    == Some(a_decay.strip_all_const())
                            {
                                score += 2;
                            } else if (p.is_integer() && a.is_integer())
                                || (p.is_floating() && a.is_floating())
                            {
                                score += 10;
                            } else if self.sema.can_assign(p, a) {
                                let is_user_conv =
                                    matches!(p.strip_const().strip_reference(), Type::Struct(_))
                                        && !matches!(
                                            a.strip_const().strip_reference(),
                                            Type::Struct(_)
                                        );
                                if is_user_conv {
                                    score += 1000;
                                } else {
                                    score += 20;
                                }
                            } else {
                                ok = false;
                                break;
                            }
                        }
                        if ok && score < best_score {
                            best_score = score;
                            best_sig = Some(sig.clone());
                        }
                    }
                }
                if let Some(sig) = best_sig {
                    if let Some(&idx) = self.func_indices.get(&sig.mangled_name) {
                        return Some((idx, sig));
                    }
                    if let Some(&idx) = self.func_indices.get(name) {
                        return Some((idx, sig));
                    }
                    if let Some(&idx) = self
                        .func_indices
                        .get(QualifiedName::new(name).std_name().as_ref())
                    {
                        return Some((idx, sig));
                    }
                    if let Some(&idx) = self.func_indices.get(QualifiedName::new(name).plain()) {
                        return Some((idx, sig));
                    }
                }
            }
        }
        self.func_indices
            .get(name)
            .or_else(|| {
                self.func_indices
                    .get(QualifiedName::new(name).std_name().as_ref())
            })
            .or_else(|| self.func_indices.get(QualifiedName::new(name).plain()))
            .map(|&idx| {
                (
                    idx,
                    FunctionSig {
                        name: name.to_string(),
                        mangled_name: name.to_string(),
                        return_type: Type::Int,
                        param_types: vec![],
                        default_args: vec![],
                        is_static: false,
                    },
                )
            })
    }

    /// Looks up a virtual method on a class or one of its bases.
    pub fn find_class_vmethod(&self, class_name: &str, method_name: &str) -> Option<u32> {
        let m_name = format!("{}_{}", class_name, method_name);
        if let Some(&idx) = self.func_indices.get(&m_name) {
            return Some(idx);
        }
        if let Some(layout) = self.sema.struct_layouts.get(class_name) {
            for b in &layout.base_classes {
                if let Some(idx) = self.find_class_vmethod(b, method_name) {
                    return Some(idx);
                }
            }
        }
        None
    }
}
