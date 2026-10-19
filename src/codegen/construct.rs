use super::*;

impl<'a> Codegen<'a> {
    pub fn find_constructor_sig(
        &self,
        class_name: &str,
        arg_types: &[Type],
    ) -> Option<(u32, FunctionSig)> {
        let short_name = class_name.split("::").last().unwrap_or(class_name);
        let q1 = format!("{}::{}", class_name, short_name);
        let q2 = format!(
            "{}::{}",
            QualifiedName::new(class_name).alternate(),
            short_name
        );
        for qual_name in &[&q1, &q2] {
            if let Some(sigs) = self.sema.functions.get(*qual_name) {
                let mut best_sig = None;
                let mut best_score = usize::MAX;
                for sig in sigs {
                    let params = if !sig.param_types.is_empty() {
                        &sig.param_types[1..]
                    } else {
                        &[]
                    };
                    if params.len() == arg_types.len() {
                        let mut score = 0;
                        let mut ok = true;
                        for (p, a) in params.iter().zip(arg_types) {
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
                    if let Some(&idx) = self
                        .func_indices
                        .get(&format!("{}_{}", class_name, short_name))
                    {
                        return Some((idx, sig));
                    }
                }
            }
        }
        if arg_types.is_empty() {
            let idx = self
                .func_indices
                .get(&format!("{}_{}", class_name, short_name))
                .or_else(|| {
                    self.func_indices
                        .get(&format!("std::{}_{}", class_name, short_name))
                })
                .or_else(|| {
                    self.func_indices.get(&format!(
                        "{}_{}",
                        QualifiedName::new(class_name).plain(),
                        short_name
                    ))
                })
                .copied();
            idx.map(|i| {
                (
                    i,
                    FunctionSig {
                        name: format!("{}_{}", class_name, short_name),
                        mangled_name: format!("{}_{}", class_name, short_name),
                        return_type: Type::Void,
                        param_types: vec![Type::Pointer(Box::new(Type::Struct(
                            class_name.to_string(),
                        )))],
                        default_args: vec![],
                        is_static: false,
                    },
                )
            })
        } else {
            None
        }
    }

    pub fn find_constructor(&self, class_name: &str, arg_types: &[Type]) -> Option<u32> {
        self.find_constructor_sig(class_name, arg_types)
            .map(|(idx, _)| idx)
    }
}
