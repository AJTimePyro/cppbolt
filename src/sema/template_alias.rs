//! Alias template registration and expansion.

use super::template_deduce::fill_default_template_args;
use super::template_subst::substitute_type;
use super::*;
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashMap;

impl Sema {
    pub fn register_alias_template(
        &mut self,
        name: String,
        ty: Type,
        params: Vec<TemplateParam>,
    ) {
        if let Some(short) = name.split("::").last() {
            self.alias_templates
                .insert(short.to_string(), (params.clone(), ty.clone()));
        }
        let qualified = QualifiedName::new(&name);
        if qualified.is_global() {
            self.alias_templates
                .insert(qualified.std_name().into_owned(), (params.clone(), ty.clone()));
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
        let effective_args = fill_default_template_args(name, &params, args, (line, col))?;
        let mut mapping = HashMap::new();
        for (param, arg) in params.iter().zip(effective_args.iter()) {
            mapping.insert(param.name.clone(), arg.clone());
        }
        let mut expanded = substitute_type(&target_ty, &mapping, None);
        self.resolve_and_instantiate_type(&mut expanded, line, col)?;
        Ok(Some(expanded))
    }
}
