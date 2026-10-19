use super::*;

impl<'a> Codegen<'a> {
    /// Resolves a method or destructor call on a class, including inherited
    /// methods, selecting the lowest-cost overload.
    pub fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
    ) -> Option<(u32, FunctionSig)> {
        if method_name.starts_with('~') {
            let short_cls = class_name.split("::").last().unwrap_or(class_name);
            let dtor_name = format!("{}_{}", class_name, method_name);
            let dtor_name_short = format!("{}_{}", short_cls, method_name);
            if let Some(&idx) = self
                .func_indices
                .get(&dtor_name)
                .or_else(|| self.func_indices.get(&dtor_name_short))
            {
                return Some((
                    idx,
                    FunctionSig {
                        name: dtor_name.clone(),
                        mangled_name: dtor_name,
                        return_type: Type::Void,
                        param_types: vec![Type::Pointer(Box::new(Type::Struct(
                            class_name.to_string(),
                        )))],
                        default_args: vec![],
                        is_static: false,
                    },
                ));
            }
        }
        let q1 = format!("{}::{}", class_name, method_name);
        let q2 = format!(
            "{}::{}",
            QualifiedName::new(class_name).alternate(),
            method_name
        );
        for qual_name in &[&q1, &q2] {
            if let Some(sigs) = self.sema.functions.get(*qual_name) {
                let mut best_sig = None;
                let mut best_score = usize::MAX;
                for sig in sigs {
                    let params = if sig.is_static {
                        &sig.param_types[..]
                    } else if !sig.param_types.is_empty() {
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
                        .get(&format!("{}::{}", class_name, method_name))
                    {
                        return Some((idx, sig));
                    }
                    if let Some(&idx) = self.func_indices.get(&format!(
                        "{}::{}",
                        QualifiedName::new(class_name).alternate(),
                        method_name
                    )) {
                        return Some((idx, sig));
                    }
                }
            }
        }
        let layout = self
            .sema
            .struct_layouts
            .get(class_name)
            .or_else(|| {
                self.sema
                    .struct_layouts
                    .get(QualifiedName::new(class_name).std_name().as_ref())
            })
            .or_else(|| {
                self.sema
                    .struct_layouts
                    .get(QualifiedName::new(class_name).plain())
            });
        if let Some(layout) = layout {
            if let Some(base) = layout.base_classes.first()
                && let Some(result) = self.find_method(base, method_name, arg_types)
            {
                return Some(result);
            }
            for (base, _) in layout.base_offsets.iter().skip(1) {
                if let Some(result) = self.find_method(base, method_name, arg_types)
                    && result.1.is_static
                {
                    return Some(result);
                }
            }
        }
        None
    }
}
