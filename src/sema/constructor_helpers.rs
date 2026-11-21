//! Constructor overload scoring helpers.

use super::layout_types::element_struct_name;
use super::*;

pub(super) fn score_copy_init_param(
    param: &Type,
    arg_ty: &Type,
    arg_expr: Option<&Expr>,
) -> Option<usize> {
    let clean_param = param.deref_type()?.strip_all_const();
    let clean_arg = arg_ty.strip_reference().strip_all_const();
    let is_copy_param = match (clean_param, clean_arg) {
        (Type::Struct(p), Type::Struct(a)) => {
            QualifiedName::new(p.as_str()).matches(QualifiedName::new(a.as_str()))
        }
        (p, a) => p == a,
    };
    if !is_copy_param {
        return None;
    }
    let is_lval = arg_expr.is_some_and(Expr::is_lvalue) || arg_ty.is_lvalue_reference();
    if param.is_rvalue_reference() {
        if is_lval {
            return None;
        }
        Some(1)
    } else if let Some(inner) = param.deref_type() {
        if !inner.is_const() {
            if !is_lval {
                return None;
            }
            Some(1)
        } else {
            Some(if is_lval { 2 } else { 3 })
        }
    } else {
        None
    }
}

impl Sema {
    pub(super) fn instantiate_constructor_dependencies(
        &mut self,
        cls: &str,
        func: &mut FunctionDecl,
    ) -> Result<(), CompileError> {
        let short_cls = cls.split("::").last().unwrap_or(cls).to_string();
        let has_delegating = func.ctor_initializers.iter().any(|(name, _)| {
            let canonical = self.canonical_class_name(name, Some(cls));
            canonical == cls || canonical == short_cls
        });
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
            let canonical = self.canonical_class_name(init_name, Some(cls));
            if (canonical == cls || canonical == short_cls)
                && let Some(sig) = self.find_constructor_for(cls, &tys, Some(args))
            {
                self.instantiate_pending_method(&sig.mangled_name)?;
            }
            arg_types_map.insert(init_name.clone(), tys);
        }
        let Some(layout) = self.struct_layouts.get(cls).cloned() else {
            return Ok(());
        };
        // A mem-initializer must name a direct base, a member, or the class
        // itself (delegating). Compare via canonical class names so an alias
        // spelling still matches its specialization, but keep the raw name in
        // the diagnostic.
        for (init_name, _) in &func.ctor_initializers {
            let canonical = self.canonical_class_name(init_name, Some(cls));
            let is_delegating = canonical == cls || canonical == short_cls;
            let is_base = layout.base_classes.contains(&canonical);
            let is_field = layout.fields.iter().any(|f| f.name == *init_name);
            if !is_delegating && !is_base && !is_field {
                return Err(CompileError::new(
                    format!(
                        "mem-initializer '{}' does not name a base, member, or delegating constructor of '{}'",
                        init_name, cls
                    ),
                    func.line,
                    func.col,
                ));
            }
        }
        if has_delegating {
            return Ok(());
        }
        for base in &layout.base_classes {
            let tys = arg_types_map
                .iter()
                .find(|(k, _)| self.canonical_class_name(k, Some(cls)) == *base)
                .map(|(_, v)| v.as_slice())
                .unwrap_or(&[]);
            if let Some(sig) = self.find_constructor_for(base, tys, None) {
                self.instantiate_pending_method(&sig.mangled_name)?;
            }
        }
        for fld in &layout.fields {
            if let Some(sname) = element_struct_name(&fld.ty) {
                let tys = arg_types_map
                    .get(&fld.name)
                    .map_or(&[][..], |v| v.as_slice());
                if let Some(sig) = self.find_constructor_for(sname, tys, None) {
                    self.instantiate_pending_method(&sig.mangled_name)?;
                }
            }
        }
        Ok(())
    }
}
