//! The macro expansion of one line.

use super::macros::{
    FunctionExpansion, FunctionMacro, MacroDef, MacroExpansion, extract_macro_args, substitute_body,
};
use super::*;

impl<'a> Preprocessor<'a> {
    /// Expands the macro `ident` at the current position.
    pub(super) fn expand_macro(
        &self,
        definition: &MacroDef,
        ident: &str,
        chars: &[char],
        index: usize,
        disabled: &mut HashSet<String>,
    ) -> MacroExpansion {
        match definition {
            MacroDef::Object(body) => {
                let expanded = self.expand_object_macro(body, ident, disabled);
                if let Some(joined) =
                    self.expand_joined(&expanded, &chars[index..], ident, disabled)
                {
                    return MacroExpansion::Joined(joined);
                }
                MacroExpansion::Expanded(expanded, index)
            }
            MacroDef::Function { params, body } => {
                let definition = FunctionMacro {
                    params,
                    body,
                    ident,
                };
                match self.expand_function_macro(&definition, chars, index, disabled) {
                    FunctionExpansion::Expanded(expanded, end_index) => {
                        if let Some(joined) =
                            self.expand_joined(&expanded, &chars[end_index..], ident, disabled)
                        {
                            return MacroExpansion::Joined(joined);
                        }
                        MacroExpansion::Expanded(expanded, end_index)
                    }
                    FunctionExpansion::NotACall => MacroExpansion::NotAMacro,
                }
            }
        }
    }

    /// Expands an object-like macro.
    fn expand_object_macro(
        &self,
        body: &str,
        ident: &str,
        disabled: &mut HashSet<String>,
    ) -> String {
        disabled.insert(ident.to_string());
        let expanded = self.expand_with_hide_set(body, disabled);
        disabled.remove(ident);
        expanded
    }

    /// Expands a function-like macro at the call site.
    fn expand_function_macro(
        &self,
        definition: &FunctionMacro<'_>,
        chars: &[char],
        index: usize,
        disabled: &mut HashSet<String>,
    ) -> FunctionExpansion {
        let FunctionMacro {
            params,
            body,
            ident,
        } = *definition;
        let mut call_start = index;
        while call_start < chars.len() && chars[call_start].is_whitespace() {
            call_start += 1;
        }
        if chars.get(call_start) != Some(&'(') {
            return FunctionExpansion::NotACall;
        }
        let Some((raw_args, end_index)) = extract_macro_args(chars, call_start) else {
            return FunctionExpansion::NotACall;
        };
        let expanded_args: Vec<String> = raw_args
            .iter()
            .map(|arg| self.expand_with_hide_set(arg, disabled))
            .collect();
        let substituted = substitute_body(body, params, &raw_args, &expanded_args);
        disabled.insert(ident.to_string());
        let expanded = self.expand_with_hide_set(&substituted, disabled);
        disabled.remove(ident);
        FunctionExpansion::Expanded(expanded, end_index)
    }
}
