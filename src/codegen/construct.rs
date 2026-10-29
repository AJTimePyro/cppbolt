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
        let mut saw_declared_constructor = false;
        for qual_name in &[&q1, &q2] {
            let Some(sigs) = self.sema.functions.get(*qual_name) else {
                continue;
            };
            saw_declared_constructor = true;
            let Some(sig) = self.best_constructor_overload(class_name, sigs, arg_types) else {
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
        if !saw_declared_constructor && arg_types.is_empty() {
            self.default_constructor_sig(class_name, short_name)
        } else {
            None
        }
    }

    /// Finds an explicit copy constructor for `class_name` that can bind an
    /// argument with the given constness.
    pub fn find_copy_constructor(&self, class_name: &str, is_source_const: bool) -> Option<u32> {
        let short_name = class_name.split("::").last().unwrap_or(class_name);
        let q1 = format!("{}::{}", class_name, short_name);
        let q2 = format!(
            "{}::{}",
            QualifiedName::new(class_name).alternate(),
            short_name
        );
        let mut best_idx = None;
        let mut best_is_exact = false;
        for qual_name in &[&q1, &q2] {
            let Some(sigs) = self.sema.functions.get(*qual_name) else {
                continue;
            };
            for sig in sigs {
                if let Some(param_is_const) = copy_constructor_param_const(sig, class_name) {
                    if is_source_const && !param_is_const {
                        continue;
                    }
                    let idx = self.func_indices.get(&sig.mangled_name).copied().or_else(|| {
                        self.func_indices
                            .get(&format!("{}_{}", class_name, short_name))
                            .copied()
                    });
                    if let Some(idx) = idx {
                        if !is_source_const && !param_is_const {
                            return Some(idx);
                        }
                        if best_idx.is_none() || (!best_is_exact && param_is_const == is_source_const) {
                            best_idx = Some(idx);
                            best_is_exact = param_is_const == is_source_const;
                        }
                    }
                }
            }
        }
        best_idx
    }

    /// Picks the lowest-cost constructor overload.
    ///
    /// A constructor may take fewer arguments than it has parameters when the
    /// omitted ones have default values.
    fn best_constructor_overload(
        &self,
        class_name: &str,
        sigs: &[FunctionSig],
        arg_types: &[Type],
    ) -> Option<FunctionSig> {
        let is_copy_init = arg_types.len() == 1
            && matches!(
                arg_types[0].strip_const().strip_reference(),
                Type::Struct(s) if QualifiedName::new(s).matches(QualifiedName::new(class_name))
            );
        let mut best_sig = None;
        let mut best_score = usize::MAX;
        for sig in sigs {
            if is_copy_init && copy_constructor_param_const(sig, class_name).is_none() {
                continue;
            }
            let params = if !sig.param_types.is_empty() {
                &sig.param_types[1..]
            } else {
                &[]
            };
            let defaults = if !sig.default_args.is_empty() {
                &sig.default_args[1..]
            } else {
                &[]
            };
            if arg_types.len() > params.len() {
                continue;
            }
            // Every omitted argument needs a default value.
            if defaults.len() < params.len()
                || !defaults[arg_types.len()..].iter().all(|d| d.is_some())
            {
                continue;
            }
            // Only the given arguments are scored; the omitted ones have
            // default values.
            let Some(score) = self.score_overload_params(&params[..arg_types.len()], arg_types)
            else {
                continue;
            };
            // Prefer the overload with the fewest defaulted arguments.
            let score = score + (params.len() - arg_types.len()) * 100;
            if score < best_score {
                best_score = score;
                best_sig = Some(sig.clone());
            }
        }
        best_sig
    }

    /// Finds a free operator function whose first parameter binds `operand`.
    pub fn find_free_operator(&self, name: &str, operand: &Type) -> Option<(u32, FunctionSig)> {
        let sigs = self.sema.functions.get(name)?;
        let mut best: Option<FunctionSig> = None;
        let mut best_score = usize::MAX;
        for sig in sigs {
            let Some(param) = sig.param_types.first() else {
                continue;
            };
            let score = if param.strip_all_const() == operand.strip_all_const() {
                0
            } else if param.is_reference()
                && param.deref_type().map(|d| d.strip_all_const())
                    == Some(operand.strip_all_const())
            {
                1
            } else {
                continue;
            };
            if score < best_score {
                best_score = score;
                best = Some(sig.clone());
            }
        }
        let sig = best?;
        let index = self
            .func_indices
            .get(&sig.mangled_name)
            .copied()
            .or_else(|| self.func_indices.get(name).copied())?;
        Some((index, sig))
    }

    /// The arguments of a constructor call with the default arguments filled
    /// in.
    pub fn constructor_arguments(&self, sig: &FunctionSig, args: &[Expr]) -> Vec<Expr> {
        let mut full: Vec<Expr> = args.to_vec();
        let params = if !sig.param_types.is_empty() {
            &sig.param_types[1..]
        } else {
            &[]
        };
        let defaults = if !sig.default_args.is_empty() {
            &sig.default_args[1..]
        } else {
            &[]
        };
        for index in args.len()..params.len() {
            if let Some(Some(default)) = defaults.get(index) {
                full.push(default.clone());
            }
        }
        full
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
                is_explicit: false,
                ref_qualifier: RefQualifier::None,
            },
        ))
    }

    /// Returns only the function index of the selected constructor.
    pub fn find_constructor(&self, class_name: &str, arg_types: &[Type]) -> Option<u32> {
        self.find_constructor_sig(class_name, arg_types)
            .map(|(idx, _)| idx)
    }
}

/// Returns `Some(is_const)` if `sig` is a copy constructor for `class_name`.
fn copy_constructor_param_const(sig: &FunctionSig, class_name: &str) -> Option<bool> {
    if sig.param_types.len() < 2 {
        return None;
    }
    let inner = sig.param_types[1].deref_type()?;
    if !matches!(
        inner.strip_const(),
        Type::Struct(s) if QualifiedName::new(s).matches(QualifiedName::new(class_name))
    ) {
        return None;
    }
    let defaults = sig.default_args.get(1..).unwrap_or_default();
    if defaults.len() < sig.param_types.len() - 1
        || defaults.get(1..).is_some_and(|rest| !rest.iter().all(|d| d.is_some()))
    {
        return None;
    }
    Some(inner.is_const())
}
