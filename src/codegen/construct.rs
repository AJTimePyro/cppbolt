use super::*;

impl<'a> Codegen<'a> {
    /// Finds the best constructor overload for `arg_types`.
    ///
    /// Constructors are registered with the implicit `this` parameter in
    /// `param_types`, so argument matching starts at index 1. Lower conversion
    /// scores win; user-defined conversions are the last resort.
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
            let Some(sigs) = self.sema.functions.get(*qual_name) else {
                continue;
            };
            let Some(sig) = self.best_constructor_overload(sigs, arg_types) else {
                continue;
            };
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
        if arg_types.is_empty() {
            self.default_constructor_sig(class_name, short_name)
        } else {
            None
        }
    }

    /// Picks the lowest-cost constructor overload.
    fn best_constructor_overload(
        &self,
        sigs: &[FunctionSig],
        arg_types: &[Type],
    ) -> Option<FunctionSig> {
        let mut best_sig = None;
        let mut best_score = usize::MAX;
        for sig in sigs {
            let params = if !sig.param_types.is_empty() {
                &sig.param_types[1..]
            } else {
                &[]
            };
            let Some(score) = self.score_overload_params(params, arg_types) else {
                continue;
            };
            if score < best_score {
                best_score = score;
                best_sig = Some(sig.clone());
            }
        }
        best_sig
    }

    /// The implicit default constructor of a class, if it has one.
    fn default_constructor_sig(
        &self,
        class_name: &str,
        short_name: &str,
    ) -> Option<(u32, FunctionSig)> {
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
            .copied()?;
        Some((
            idx,
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
        ))
    }

    /// Returns only the function index of the selected constructor.
    pub fn find_constructor(&self, class_name: &str, arg_types: &[Type]) -> Option<u32> {
        self.find_constructor_sig(class_name, arg_types)
            .map(|(idx, _)| idx)
    }
}
