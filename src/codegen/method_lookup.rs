use super::*;

impl<'a> Codegen<'a> {
    /// Resolves a method or destructor call on a class, including inherited
    /// methods, selecting the lowest-cost overload.
    ///
    /// Overloads that differ only in `const` are biased toward the non-const
    /// overload; use [`Self::find_method_for_receiver`] when the receiver's
    /// constness is known.
    pub fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
    ) -> Option<(u32, FunctionSig)> {
        self.find_method_filtered(class_name, method_name, arg_types, None)
    }

    /// Resolves a method for a receiver whose constness is known.
    ///
    /// A const receiver only considers const-qualified overloads; a non-const
    /// receiver prefers the non-const overload but still accepts a const one.
    pub fn find_method_for_receiver(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
        receiver_const: bool,
    ) -> Option<(u32, FunctionSig)> {
        self.find_method_filtered(class_name, method_name, arg_types, Some(receiver_const))
    }

    fn find_method_filtered(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
        receiver: Option<bool>,
    ) -> Option<(u32, FunctionSig)> {
        if let Some(result) = self.find_destructor_sig(class_name, method_name) {
            return Some(result);
        }
        let q1 = format!("{}::{}", class_name, method_name);
        let q2 = format!(
            "{}::{}",
            QualifiedName::new(class_name).alternate(),
            method_name
        );
        for qual_name in &[&q1, &q2] {
            let Some(sigs) = self.sema.functions.get(*qual_name) else {
                continue;
            };
            let Some(sig) = self.best_method_overload(sigs, arg_types, receiver) else {
                continue;
            };
            if let Some(idx) = self.resolve_method_index(class_name, method_name, &sig) {
                return Some((idx, sig));
            }
        }
        self.find_method_in_bases(class_name, method_name, arg_types, receiver)
    }

    /// Resolves a destructor call, which has no overloads.
    fn find_destructor_sig(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<(u32, FunctionSig)> {
        if !method_name.starts_with('~') {
            return None;
        }
        let short_cls = class_name.split("::").last().unwrap_or(class_name);
        let dtor_name = format!("{}_{}", class_name, method_name);
        let dtor_name_short = format!("{}_{}", short_cls, method_name);
        let idx = self
            .func_indices
            .get(&dtor_name)
            .or_else(|| self.func_indices.get(&dtor_name_short))?;
        Some((
            *idx,
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
        ))
    }

    /// Picks the lowest-cost overload of a method.
    fn best_method_overload(
        &self,
        sigs: &[FunctionSig],
        arg_types: &[Type],
        receiver: Option<bool>,
    ) -> Option<FunctionSig> {
        let mut best_sig = None;
        let mut best_score = usize::MAX;
        for sig in sigs {
            if let Some(receiver_const) = receiver
                && receiver_const
                && !crate::sema::sig_receiver_is_const(sig)
            {
                continue;
            }
            let params = if sig.is_static {
                &sig.param_types[..]
            } else if !sig.param_types.is_empty() {
                &sig.param_types[1..]
            } else {
                &[]
            };
            let Some(score) = self.score_overload_params(params, arg_types) else {
                continue;
            };
            let receiver_penalty = match receiver {
                Some(false) if crate::sema::sig_receiver_is_const(sig) => 1,
                _ => 0,
            };
            let score = score + receiver_penalty;
            if score < best_score {
                best_score = score;
                best_sig = Some(sig.clone());
            }
        }
        best_sig
    }

    /// Looks up the Wasm function index of a resolved method signature.
    fn resolve_method_index(
        &self,
        class_name: &str,
        method_name: &str,
        sig: &FunctionSig,
    ) -> Option<u32> {
        if let Some(&idx) = self.func_indices.get(&sig.mangled_name) {
            return Some(idx);
        }
        if let Some(&idx) = self
            .func_indices
            .get(&format!("{}::{}", class_name, method_name))
        {
            return Some(idx);
        }
        self.func_indices
            .get(&format!(
                "{}::{}",
                QualifiedName::new(class_name).alternate(),
                method_name
            ))
            .copied()
    }

    /// Looks for the method in the class's base classes.
    fn find_method_in_bases(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
        receiver: Option<bool>,
    ) -> Option<(u32, FunctionSig)> {
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
        let layout = layout?;
        if let Some(base) = layout.base_classes.first()
            && let Some(result) = self.find_method_filtered(base, method_name, arg_types, receiver)
        {
            return Some(result);
        }
        None
    }
}
