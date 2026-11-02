//! The analysis of a constructor call: the constructor that matches the
//! arguments, the default arguments, and the copy initialization of an object
//! of the class.

use super::analyze::this_param;
use super::template_deduce::{
    deduce_template_arguments, parse_types_list, split_template_call_name,
};
use super::template_subst::substitute_function;
use super::*;
use crate::ast::decl::RefQualifier;

/// The outcome of a call that names a class.
pub(super) enum ConstructorCall {
    /// The temporary is the argument itself, so the call is that argument.
    Elided(Expr),
    /// A constructor was selected for the arguments.
    Selected,
}

fn element_struct_name(ty: &Type) -> Option<&str> {
    match ty.strip_const() {
        Type::Struct(name) => Some(name),
        Type::Array(elem, _) => element_struct_name(elem),
        _ => None,
    }
}

impl Sema {
    /// The class a name refers to, if the name names one.
    pub(super) fn class_named(&mut self, name: &str, line: usize, col: usize) -> Option<String> {
        if self.struct_layouts.contains_key(name) {
            return Some(name.to_string());
        }
        if let Some(cls) = &self.current_class {
            let scoped = format!("{}::{}", cls, name);
            if self.struct_layouts.contains_key(&scoped) {
                return Some(scoped);
            }
        }
        if let Some((base, args_str)) = split_template_call_name(name) {
            let known: Vec<String> = self.class_templates.keys().cloned().collect();
            if let Ok(args) = parse_types_list(args_str, &known)
                && let Ok(inst_name) = self.instantiate_class_template(base, &args, line, col)
            {
                return Some(inst_name);
            }
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
        match self.find_constructor_for(class_name, arg_types, Some(args)) {
            Some(sig) => {
                // A constructor call may leave out arguments that have
                // default values.
                self.fill_constructor_defaults(&sig, args)?;
                self.instantiate_pending_method(&sig.mangled_name)?;
            }
            None => {
                if let Some(sig) = self
                    .try_instantiate_member_template_constructor(class_name, arg_types, args, pos)?
                {
                    self.fill_constructor_defaults(&sig, args)?;
                    self.instantiate_pending_method(&sig.mangled_name)?;
                } else if !self.is_copy_initialization(class_name, arg_types) {
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
        args: Option<&[Expr]>,
    ) -> Option<FunctionSig> {
        let qname = QualifiedName::new(class_name);
        let short = qname.class_short_name();
        let alternate = qname.alternate();
        let mut candidates: Vec<FunctionSig> = Vec::new();
        for name in [
            format!("{class_name}::{short}"),
            format!("{class_name}::{class_name}"),
            format!("{alternate}::{short}"),
            format!("{alternate}::{alternate}"),
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
            if let Some(score) =
                self.score_constructor_candidate(cand, arg_types, args, is_copy_init)
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
        args: Option<&[Expr]>,
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
            if param.strip_const() == arg_ty.strip_const() {
                continue;
            }
            if self.check_pointer_conversion(param, arg_ty) {
                score += 1;
                continue;
            }
            if param.is_reference()
                && param.deref_type().map(|d| d.strip_all_const()) == Some(arg_ty.strip_all_const())
            {
                score += 1;
                continue;
            }
            let is_null_ptr_const = args
                .and_then(|a| a.get(index))
                .is_some_and(Sema::is_null_pointer_constant);
            if param.decay_array().is_pointer() && is_null_ptr_const {
                score += 2;
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

    pub(super) fn try_instantiate_member_template(
        &mut self,
        class_name: &str,
        method_name: Option<&str>,
        arg_types: &[Type],
        args: &[Expr],
        pos: ExprPos,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let short = QualifiedName::new(class_name).class_short_name();
        let plain = QualifiedName::new(class_name).plain();
        let templates = self
            .member_templates
            .get(class_name)
            .cloned()
            .or_else(|| self.member_templates.get(plain).cloned());
        let Some(templates) = templates else {
            return Ok(None);
        };

        for cand in templates {
            let matches_name = match method_name {
                Some(m) => cand.name == m,
                None => cand.name == short || cand.name == class_name || cand.name == plain,
            };
            if !matches_name || cand.template_params.is_empty() {
                continue;
            }
            if let Ok(mapping) =
                deduce_template_arguments(&cand.template_params, &cand, &[], (arg_types, args), pos)
            {
                let new_name = if method_name.is_none() {
                    Some(short.to_string())
                } else {
                    None
                };
                let mut spec_func = substitute_function(&cand, &mapping, None, new_name);
                self.resolve_and_instantiate_type(&mut spec_func.return_type, pos.line, pos.col)?;
                for p in &mut spec_func.params {
                    self.resolve_and_instantiate_type(&mut p.ty, pos.line, pos.col)?;
                }
                self.register_struct_method(class_name, &spec_func);
                let qual_name = format!("{}::{}", class_name, spec_func.name);
                let mut params = spec_func.params.clone();
                if !spec_func.is_static {
                    params.insert(0, this_param(class_name, spec_func.is_const));
                }
                let mut mangled = self.mangle_name(&qual_name, &params);
                match spec_func.ref_qualifier {
                    RefQualifier::LValue => mangled.push_str("_R"),
                    RefQualifier::RValue => mangled.push_str("_O"),
                    RefQualifier::None => {}
                }
                self.pending_template_methods
                    .insert(mangled.clone(), (class_name.to_string(), spec_func.clone()));

                let sig = FunctionSig {
                    name: spec_func.name,
                    mangled_name: mangled,
                    return_type: spec_func.return_type,
                    param_types: params.iter().map(|p| p.ty.clone()).collect(),
                    default_args: params.into_iter().map(|p| p.default_val).collect(),
                    is_static: spec_func.is_static,
                    is_explicit: spec_func.is_explicit,
                    ref_qualifier: spec_func.ref_qualifier,
                };
                return Ok(Some(sig));
            }
        }
        Ok(None)
    }

    pub(super) fn try_instantiate_member_template_constructor(
        &mut self,
        class_name: &str,
        arg_types: &[Type],
        args: &[Expr],
        pos: ExprPos,
    ) -> Result<Option<FunctionSig>, CompileError> {
        self.try_instantiate_member_template(class_name, None, arg_types, args, pos)
    }

    pub(super) fn instantiate_constructor_dependencies(
        &mut self,
        cls: &str,
        func: &mut FunctionDecl,
    ) -> Result<(), CompileError> {
        let short_cls = cls.split("::").last().unwrap_or(cls).to_string();
        let has_delegating = func
            .ctor_initializers
            .iter()
            .any(|(name, _)| name == cls || name == &short_cls);
        if has_delegating && func.ctor_initializers.len() > 1 {
            return Err(CompileError::new(
                "mem-initializer for delegating constructor cannot be combined with other member or base initializers",
                func.line,
                func.col,
            ));
        }
        let mut arg_types_map: HashMap<String, Vec<Type>> = HashMap::new();
        for (init_name, args) in &mut func.ctor_initializers {
            let mut tys = Vec::new();
            for arg in args.iter_mut() {
                tys.push(self.analyze_expr(arg)?);
            }
            if (init_name == cls || init_name == &short_cls)
                && let Some(sig) = self.find_constructor_for(cls, &tys, Some(args))
            {
                self.instantiate_pending_method(&sig.mangled_name)?;
            }
            arg_types_map.insert(init_name.clone(), tys);
        }
        if has_delegating {
            return Ok(());
        }
        let Some(layout) = self.struct_layouts.get(cls).cloned() else {
            return Ok(());
        };
        for base in &layout.base_classes {
            let empty_tys = Vec::new();
            let tys = arg_types_map.get(base).unwrap_or(&empty_tys);
            if let Some(sig) = self.find_constructor_for(base, tys, None) {
                self.instantiate_pending_method(&sig.mangled_name)?;
            }
        }
        for fld in &layout.fields {
            if let Some(sname) = element_struct_name(&fld.ty) {
                let empty_tys = Vec::new();
                let tys = arg_types_map.get(&fld.name).unwrap_or(&empty_tys);
                if let Some(sig) = self.find_constructor_for(sname, tys, None) {
                    self.instantiate_pending_method(&sig.mangled_name)?;
                }
            }
        }
        Ok(())
    }
}
