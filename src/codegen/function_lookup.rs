use super::*;

impl<'a> Codegen<'a> {
    /// Resolves a free-function call, including ADL and the implicit `std`
    /// alias, selecting the lowest-cost overload.
    pub fn find_function(&self, name: &str, arg_types: &[Type]) -> Option<(u32, FunctionSig)> {
        for q in candidate_function_names(name, arg_types) {
            let Some(sigs) = self.sema.functions.get(&q) else {
                continue;
            };
            let mut best_sig = None;
            let mut best_score = usize::MAX;
            for sig in sigs {
                let Some(score) = self.score_overload_params(&sig.param_types, arg_types) else {
                    continue;
                };
                if score < best_score {
                    best_score = score;
                    best_sig = Some(sig.clone());
                }
            }
            if let Some(sig) = best_sig
                && let Some(idx) = self.resolve_signature_index(name, &sig)
            {
                return Some((idx, sig));
            }
        }
        self.func_indices
            .get(name)
            .or_else(|| {
                self.func_indices
                    .get(QualifiedName::new(name).std_name().as_ref())
            })
            .or_else(|| self.func_indices.get(QualifiedName::new(name).plain()))
            .map(|&idx| (idx, default_signature(name)))
    }

    fn resolve_signature_index(&self, name: &str, sig: &FunctionSig) -> Option<u32> {
        if let Some(&idx) = self.func_indices.get(&sig.mangled_name) {
            return Some(idx);
        }
        if let Some(&idx) = self.func_indices.get(name) {
            return Some(idx);
        }
        if let Some(&idx) = self
            .func_indices
            .get(QualifiedName::new(name).std_name().as_ref())
        {
            return Some(idx);
        }
        self.func_indices
            .get(QualifiedName::new(name).plain())
            .copied()
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

fn candidate_function_names(name: &str, arg_types: &[Type]) -> Vec<String> {
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
    candidate_names
}

fn default_signature(name: &str) -> FunctionSig {
    FunctionSig {
        name: name.to_string(),
        mangled_name: name.to_string(),
        return_type: Type::Int,
        param_types: vec![],
        default_args: vec![],
        is_static: false,
    }
}
