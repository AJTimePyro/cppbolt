//! Deferred return-type and exception-specification checks for overrides.
//!
//! Function signatures are registered after layout, so these checks run in a
//! second pass once `sema.functions` is complete.

use super::*;

impl Sema {
    pub(super) fn validate_virtual_returns(&self, program: &Program) -> Result<(), CompileError> {
        for decl in &program.declarations {
            let Decl::StructDecl { name, methods, .. } = decl else {
                continue;
            };
            for m in methods.iter() {
                let key = VirtualKey::from_decl(m);
                let bases = self.overridden_bases(name, &key);
                if bases.is_empty() && !m.qualifiers.is_override {
                    continue;
                }
                for base_name in &bases {
                    let Some(base_sig) = find_sig(self, base_name, &key) else {
                        return Err(CompileError::new(
                            format!("cannot resolve virtual declaration in base '{}'", base_name),
                            m.line,
                            m.col,
                        ));
                    };
                    if !return_compatible(&base_sig.return_type, &m.return_type, self) {
                        return Err(CompileError::new(
                            format!("'{}::{}' has an incompatible return type", name, m.name),
                            m.line,
                            m.col,
                        ));
                    }
                    if !noexcept_compatible(base_sig, m) {
                        return Err(CompileError::new(
                            format!(
                                "'{}::{}' has a looser exception specification than the function it overrides",
                                name, m.name
                            ),
                            m.line,
                            m.col,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Collects every base class that declares a virtual with this key. A
    /// diamond may contain several such declarations, all of which constrain
    /// the overriding return type.
    fn overridden_bases(&self, class_name: &str, key: &VirtualKey) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_overridden_bases(class_name, key, &mut visited, &mut result);
        result
    }

    fn collect_overridden_bases(
        &self,
        class_name: &str,
        key: &VirtualKey,
        visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if !visited.insert(class_name.to_string()) {
            return;
        }
        let Some(layout) = self.struct_layouts.get(class_name) else {
            return;
        };
        for base in &layout.base_classes {
            if find_sig(self, base, key).is_some() {
                result.push(base.clone());
            }
            self.collect_overridden_bases(base, key, visited, result);
        }
    }
}
