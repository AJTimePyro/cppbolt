//! Alias template registration and expansion.

use super::template_deduce::fill_default_template_args;
use super::template_subst::substitute_type;
use super::*;
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::name::QualifiedName;

impl Sema {
    pub fn register_alias_template(&mut self, name: String, ty: Type, params: Vec<TemplateParam>) {
        if let Some(short) = name.split("::").last() {
            self.alias_templates
                .insert(short.to_string(), (params.clone(), ty.clone()));
        }
        let qualified = QualifiedName::new(&name);
        if qualified.is_global() {
            self.alias_templates.insert(
                qualified.std_name().into_owned(),
                (params.clone(), ty.clone()),
            );
        } else if qualified.in_std() {
            self.alias_templates
                .insert(qualified.plain().to_string(), (params.clone(), ty.clone()));
        }
        self.alias_templates.insert(name, (params, ty));
    }

    pub fn lookup_alias_template(&self, name: &str) -> Option<(Vec<TemplateParam>, Type)> {
        self.alias_templates.get(name).cloned().or_else(|| {
            let plain = QualifiedName::new(name).plain();
            self.alias_templates.get(plain).cloned()
        })
    }

    pub fn expand_alias_template(
        &mut self,
        name: &str,
        args: &[Type],
        line: usize,
        col: usize,
    ) -> Result<Option<Type>, CompileError> {
        let Some((params, target_ty)) = self.lookup_alias_template(name) else {
            return Ok(None);
        };
        let mut resolved_args = args.to_vec();
        for (param, arg) in params.iter().zip(resolved_args.iter_mut()) {
            if param.param_type.is_some() {
                self.try_resolve_nontype_arg(arg, line, col)?;
            }
        }
        let effective_args =
            fill_default_template_args(name, &params, &resolved_args, (line, col))?;
        let mapping = super::template_deduce::build_template_arg_map(&params, &effective_args);
        let mut expanded = substitute_type(&target_ty, &mapping, None)?;
        // A qualified result such as `enable_if<false, T>::type` is a nested
        // type of the instantiated class. If it has no member alias, convert
        // it to `DependentNested` so resolution reports a substitution failure
        // instead of leaving an opaque struct that looks like a valid type.
        if let Type::Struct(nested) = &expanded
            && nested.contains("::")
            && self.lookup_type_alias(nested).is_none()
        {
            expanded = Type::DependentNested(nested.clone());
        }
        self.resolve_and_instantiate_type(&mut expanded, line, col)?;
        Ok(Some(expanded))
    }
}
