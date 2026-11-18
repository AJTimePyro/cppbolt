//! Virtual table construction for record layouts.

use super::layout_types::{LayoutAccumulator, VTableInfo, VTableSlotInfo};
use super::*;

impl Sema {
    pub(super) fn build_vtables(
        &self,
        class_name: &str,
        base_classes: &[String],
        acc: &LayoutAccumulator,
        has_vtable: bool,
    ) -> Vec<VTableInfo> {
        if !has_vtable {
            return Vec::new();
        }
        let as_slots = |keys: &[VirtualKey]| -> Vec<VTableSlotInfo> {
            keys.iter()
                .map(|key| VTableSlotInfo {
                    is_deleting_dtor: key.name == "__deleting_dtor",
                    key: key.clone(),
                })
                .collect()
        };
        let primary = base_classes.iter().find_map(|base| {
            let layout = self.struct_layouts.get(base)?;
            if !layout.has_vtable {
                return None;
            }
            let base_offset = acc
                .base_offsets
                .iter()
                .find(|(name, _)| name == base)
                .map(|(_, off)| *off)?;
            // The base's vptr may live inside it at a nonzero offset when the
            // base itself inherited polymorphism from a non-primary base.
            let vptr_offset = layout.vtables.first().map(|v| v.vptr_offset).unwrap_or(0);
            Some((base.clone(), base_offset, base_offset + vptr_offset))
        });
        let mut tables = Vec::new();
        if let Some((owner, owner_offset, vptr_offset)) = primary {
            tables.push(VTableInfo {
                owner,
                vptr_offset,
                owner_offset,
                slots: as_slots(&acc.vtable_slots),
            });
        }
        for (base_name, base_offset) in &acc.base_offsets {
            let Some(bl) = self.struct_layouts.get(base_name) else {
                continue;
            };
            if !bl.has_vtable {
                continue;
            }
            let Some(first) = bl.vtables.first() else {
                continue;
            };
            // Several subobjects can share one physical vptr. Installing two
            // tables there would overwrite the first one's slots.
            let vptr_offset = *base_offset + first.vptr_offset;
            if tables.iter().any(|t| t.vptr_offset == vptr_offset) {
                continue;
            }
            tables.push(VTableInfo {
                owner: base_name.clone(),
                vptr_offset,
                owner_offset: *base_offset,
                slots: first.slots.clone(),
            });
        }
        if tables.is_empty() {
            tables.push(VTableInfo {
                owner: class_name.to_string(),
                vptr_offset: 0,
                owner_offset: 0,
                slots: as_slots(&acc.vtable_slots),
            });
        }
        tables
    }

    /// Adds newly introduced virtual methods to the inherited slot list.
    ///
    /// A virtual destructor occupies two slots: the destructor and the
    /// deleting destructor used by `delete` on a base pointer.
    pub(super) fn layout_vtable_slots(
        &mut self,
        methods: &[FunctionDecl],
        base_classes: &[String],
        has_vtable: bool,
        vtable_slots: &mut Vec<VirtualKey>,
    ) {
        if !has_vtable {
            return;
        }
        // An override of a virtual declared in *any* base (including a
        // secondary polymorphic base) replaces that base's slot; it does not
        // introduce a new slot in the primary table.
        let mut inherited: Vec<VirtualKey> = vtable_slots.clone();
        for base in base_classes {
            if let Some(bl) = self.struct_layouts.get(base) {
                for key in &bl.vtable_slots {
                    if !inherited.contains(key) {
                        inherited.push(key.clone());
                    }
                }
            }
        }
        for m in methods {
            let key = VirtualKey::from_decl(m);
            if key.is_destructor() {
                let is_virt_dtor = m.is_virtual || inherited.contains(&key);
                if is_virt_dtor && !vtable_slots.contains(&key) {
                    vtable_slots.push(key);
                    vtable_slots.push(VirtualKey::new("__deleting_dtor"));
                }
            } else if (m.is_virtual || inherited.contains(&key)) && !vtable_slots.contains(&key) {
                vtable_slots.push(key);
            }
        }
    }
}
