use super::*;

impl Sema {
    pub fn is_subclass(&self, derived: &str, base: &str) -> bool {
        if derived == base {
            return true;
        }
        if let Some(layout) = self.struct_layouts.get(derived) {
            for b in &layout.base_classes {
                if self.is_subclass(b, base) {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_field_in_class(&self, class_name: &str, field_name: &str) -> Option<StructField> {
        if let Some(layout) = self.struct_layouts.get(class_name) {
            if let Some(fld) = layout.fields.iter().find(|f| f.name == field_name) {
                return Some(fld.clone());
            }
            for b in &layout.base_classes {
                if let Some(fld) = self.find_field_in_class(b, field_name) {
                    return Some(fld);
                }
            }
        } else if let Some(alias) = QualifiedName::new(class_name).implicit_alias() {
            return self.find_field_in_class(alias.as_ref(), field_name);
        }
        None
    }

    pub fn find_method_in_class(&self, class_name: &str, method_name: &str) -> Option<FunctionSig> {
        let pick_sig = |sigs: &[FunctionSig]| -> Option<FunctionSig> {
            if let Some(non_const) = sigs.iter().find(|s| {
                !s.param_types
                    .first()
                    .map(|p| match p {
                        Type::Pointer(inner) => matches!(**inner, Type::Const(_)),
                        _ => false,
                    })
                    .unwrap_or(false)
            }) {
                Some(non_const.clone())
            } else {
                sigs.first().cloned()
            }
        };
        let qual_name = format!("{}::{}", class_name, method_name);
        if let Some(sigs) = self.functions.get(&qual_name) {
            return pick_sig(sigs);
        }
        if let Some(alias) = QualifiedName::new(class_name).implicit_alias() {
            let alt = format!("{}::{}", alias, method_name);
            if let Some(sigs) = self.functions.get(&alt) {
                return pick_sig(sigs);
            }
        }
        if let Some(layout) = self.struct_layouts.get(class_name) {
            for b in &layout.base_classes {
                if let Some(sig) = self.find_method_in_class(b, method_name) {
                    return Some(sig);
                }
            }
        }
        None
    }
}
