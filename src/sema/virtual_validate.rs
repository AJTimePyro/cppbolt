//! Deferred return-type and exception-specification checks for overrides.
//!
//! Function signatures are registered after layout, so these checks run in a
//! second pass once `sema.functions` is complete.

use super::*;

impl Sema {
    pub(super) fn validate_virtual_returns(&self, program: &Program) -> Result<(), CompileError> {
        for decl in &program.declarations {
            let Decl::StructDecl {
                name,
                base_classes,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            let inherited: Vec<(String, VirtualKey)> = base_classes
                .iter()
                .filter_map(|base| self.struct_layouts.get(base).map(|l| (base, l)))
                .flat_map(|(base, layout)| {
                    layout
                        .vtable_slots
                        .iter()
                        .filter(|key| key.name != "__deleting_dtor")
                        .map(move |key| (base.clone(), key.clone()))
                })
                .collect();
            for m in methods.iter() {
                let key = VirtualKey::from_decl(m);
                let Some((base_name, _)) = inherited.iter().find(|(_, k)| *k == key) else {
                    continue;
                };
                let Some(base_sig) = find_sig(self, base_name, &key) else {
                    continue;
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
        Ok(())
    }
}
