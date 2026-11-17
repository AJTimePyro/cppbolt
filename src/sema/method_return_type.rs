//! Propagation of deduced return types into registered method signatures.

use super::analyze::this_param;
use super::*;

impl Sema {
    /// Propagates a deduced `auto` return type into the registered overload.
    /// Registration runs before bodies are analyzed, so the signature still
    /// holds `Type::Auto` when the method is first called from another body.
    pub(super) fn update_registered_method_return_type(
        &mut self,
        class_name: &str,
        m: &FunctionDecl,
    ) {
        let mut params = m.params.clone();
        if !m.is_static {
            params.insert(0, this_param(class_name, m.is_const));
        }
        let mut mangled = self.mangle_name(&format!("{}::{}", class_name, m.name), &params);
        match m.ref_qualifier {
            RefQualifier::LValue => mangled.push_str("_R"),
            RefQualifier::RValue => mangled.push_str("_O"),
            RefQualifier::None => {}
        }
        for sigs in self.functions.values_mut() {
            for sig in sigs {
                if sig.mangled_name == mangled {
                    sig.return_type = m.return_type.clone();
                }
            }
        }
    }
}
