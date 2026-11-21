//! Template candidate rejection diagnostics.

use super::*;
use crate::diagnostics::SubstitutionFailure;

pub(super) fn no_viable_template_error(
    candidates: &[(FunctionDecl, Vec<TemplateParam>)],
    rejections: &[String],
    last_err: Option<CompileError>,
    last_substitution: Option<SubstitutionFailure>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if rejections.len() > 1 {
        let name = candidates
            .first()
            .map(|(decl, _)| decl.name.as_str())
            .unwrap_or("template function");
        return Err(CompileError::new(
            format!(
                "no matching function for call to '{name}': {} candidates rejected: {}",
                rejections.len(),
                rejections.join("; ")
            ),
            pos.line,
            pos.col,
        )
        .with_kind(crate::diagnostics::ErrorKind::Substitution));
    }
    if let Some(mut e) = last_err {
        e.kind = crate::diagnostics::ErrorKind::Substitution;
        return Err(e);
    }
    if let Some(failure) = last_substitution {
        let name = candidates
            .first()
            .map(|(decl, _)| decl.name.as_str())
            .unwrap_or("template function");
        return Err(CompileError::new(
            format!(
                "no matching function for call to '{name}': substitution failure: {}",
                failure.message
            ),
            failure.line,
            failure.col,
        )
        .with_kind(crate::diagnostics::ErrorKind::Substitution));
    }
    Ok(())
}

/// Accumulates why each template candidate was rejected.
#[derive(Default)]
pub(super) struct Rejections {
    pub(super) messages: Vec<String>,
    pub(super) last_err: Option<CompileError>,
    pub(super) last_substitution: Option<SubstitutionFailure>,
}

impl Rejections {
    pub(super) fn into_error(
        self,
        candidates: &[(FunctionDecl, Vec<TemplateParam>)],
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        no_viable_template_error(
            candidates,
            &self.messages,
            self.last_err,
            self.last_substitution,
            pos,
        )
    }
}
