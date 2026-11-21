//! Template registration, lookup, instantiation caching, and lazy member instantiation.

pub(crate) use super::template_registry_helpers::insert_template_aliases;
use super::*;
use crate::ast::decl::FunctionDecl;
use crate::ast::template::TemplateParam;
use crate::name::QualifiedName;

/// Stable identity for a function template definition, used to find the
/// ordinary lookup set captured when it was declared.
pub(crate) fn function_template_identity(name: &str, line: usize, col: usize) -> String {
    format!("{name}@{line}:{col}")
}

impl Sema {
    pub fn register_function_template(&mut self, decl: FunctionDecl, params: Vec<TemplateParam>) {
        let short = decl
            .name
            .rsplit("::")
            .next()
            .unwrap_or(&decl.name)
            .to_string();
        let q = QualifiedName::new(&decl.name);
        let keys = [
            decl.name.clone(),
            short,
            q.plain().to_string(),
            q.std_name().into_owned(),
        ];
        let item = (decl, params);
        for key in keys {
            let list = self.function_templates.entry(key).or_default();
            if let Some((existing_decl, _)) = list.iter_mut().find(|(d, _)| {
                d.params.len() == item.0.params.len()
                    && d.return_type == item.0.return_type
                    && d.params
                        .iter()
                        .zip(&item.0.params)
                        .all(|(p1, p2)| p1.ty == p2.ty)
            }) {
                if existing_decl.body.is_none() && item.0.body.is_some() {
                    existing_decl.body = item.0.body.clone();
                }
            } else {
                list.push(item.clone());
            }
        }
    }
}
