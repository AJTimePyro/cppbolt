//! The analysis of a constructor call: the constructor that matches the
//! arguments, the default arguments, and the copy initialization of an object
//! of the class.

use super::*;

/// The outcome of a call that names a class.
pub(super) enum ConstructorCall {
    /// The temporary is the argument itself, so the call is that argument.
    Elided(Expr),
    /// A constructor was selected for the arguments.
    Selected,
}

impl Sema {
    /// The class a name refers to, if the name names one.
    pub(super) fn class_named(&self, name: &str) -> Option<String> {
        if self.struct_layouts.contains_key(name) {
            return Some(name.to_string());
        }
        let alias = QualifiedName::new(name).alternate();
        if self.struct_layouts.contains_key(alias.as_ref()) {
            Some(alias.into_owned())
        } else {
            None
        }
    }

    /// Analyzes a call that names a class, which is a constructor call.
    pub(super) fn analyze_constructor_call(
        &mut self,
        class_name: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<ConstructorCall, CompileError> {
        // C++17 guarantees that a prvalue of the class initializes the
        // temporary directly, so `T x(T(1))` and `T x = T(1)` construct the
        // object instead of copying it.
        if let [only] = args.as_slice()
            && matches!(arg_types[0].strip_const(), Type::Struct(other) if *other == class_name)
            && matches!(only, Expr::Call { .. })
        {
            let inner = std::mem::replace(&mut args[0], Expr::BoolLiteral(false));
            return Ok(ConstructorCall::Elided(inner));
        }
        // The arguments must match a constructor of the class.
        match self.find_constructor_for(class_name, arg_types) {
            Some(sig) => {
                // A constructor call may leave out arguments that have
                // default values.
                self.fill_constructor_defaults(&sig, args)?;
            }
            None => {
                if !self.is_copy_initialization(class_name, arg_types) {
                    return Err(self.no_matching_constructor(class_name, pos));
                }
            }
        }
        Ok(ConstructorCall::Selected)
    }

    /// Finds the constructor of a class that accepts the given arguments.
    ///
    /// The constructor overload with the fewest defaulted arguments wins, and
    /// an exact match beats a reference binding, which beats an arithmetic
    /// conversion.
    pub(super) fn find_constructor_for(
        &self,
        class_name: &str,
        arg_types: &[Type],
    ) -> Option<FunctionSig> {
        let short = class_name.split("::").last().unwrap_or(class_name);
        let alternate = QualifiedName::new(class_name).alternate();
        let mut candidates: Vec<FunctionSig> = Vec::new();
        for name in [
            format!("{class_name}::{short}"),
            format!("{alternate}::{short}"),
        ] {
            if let Some(sigs) = self.functions.get(&name) {
                candidates.extend(sigs.iter().cloned());
            }
        }
        if candidates.is_empty() {
            return None;
        }
        let is_copy_init = arg_types.len() == 1
            && matches!(
                arg_types[0].strip_const().strip_reference(),
                Type::Struct(s) if QualifiedName::new(s).matches(QualifiedName::new(class_name))
            );
        let mut best: Option<FunctionSig> = None;
        let mut best_score = usize::MAX;
        for cand in &candidates {
            if let Some(score) = self.score_constructor_candidate(cand, arg_types, is_copy_init)
                && score < best_score
            {
                best_score = score;
                best = Some(cand.clone());
            }
        }
        best
    }

    /// Evaluates how well a constructor candidate matches the given argument types.
    /// Returns `None` if the candidate cannot accept the arguments.
    fn score_constructor_candidate(
        &self,
        cand: &FunctionSig,
        arg_types: &[Type],
        is_copy_init: bool,
    ) -> Option<usize> {
        let params = if cand.param_types.is_empty() {
            &[][..]
        } else {
            &cand.param_types[1..]
        };
        let defaults = if cand.default_args.is_empty() {
            &[][..]
        } else {
            &cand.default_args[1..]
        };
        if arg_types.len() > params.len() || defaults.len() < params.len() {
            return None;
        }
        // Every omitted argument needs a default value.
        if !defaults[arg_types.len()..].iter().all(|d| d.is_some()) {
            return None;
        }
        let mut score = 0;
        for (index, arg_ty) in arg_types.iter().enumerate() {
            let param = &params[index];
            if is_copy_init {
                let is_copy_param = param.is_reference()
                    && param.deref_type().map(|d| d.strip_all_const())
                        == Some(arg_ty.strip_all_const());
                if !is_copy_param {
                    return None;
                }
                score += 1;
                continue;
            }
            if param.strip_all_const() == arg_ty.strip_all_const() {
                continue;
            }
            if param.is_reference()
                && param.deref_type().map(|d| d.strip_all_const())
                    == Some(arg_ty.strip_all_const())
            {
                score += 1;
                continue;
            }
            if self.can_assign(param, arg_ty) {
                score += 2;
                continue;
            }
            return None;
        }
        // Prefer the overload with the fewest defaulted arguments.
        score += (params.len() - arg_types.len()) * 100;
        Some(score)
    }

    /// Returns true when the arguments describe a copy initialization.
    ///
    /// A single argument of the class itself is initialized with the copy
    /// constructor, whether or not the class declares one.
    pub(super) fn is_copy_initialization(&self, class_name: &str, arg_types: &[Type]) -> bool {
        arg_types.len() == 1
            && matches!(arg_types[0].strip_const(), Type::Struct(other) if *other == class_name)
    }

    /// Fills in the default arguments of a constructor call.
    ///
    /// The default expressions are analyzed like ordinary arguments.
    pub(super) fn fill_constructor_defaults(
        &mut self,
        sig: &FunctionSig,
        args: &mut Vec<Expr>,
    ) -> Result<(), CompileError> {
        let given = args.len();
        let last = sig.param_types.len().saturating_sub(1);
        for index in given..last {
            if let Some(Some(default)) = sig.default_args.get(index + 1) {
                args.push(default.clone());
            }
        }
        for arg in args[given..].iter_mut() {
            self.analyze_expr(arg)?;
        }
        Ok(())
    }
}
