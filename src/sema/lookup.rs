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
            // Base fields are flattened before this class's own fields. Search
            // backwards so a derived field hides a same-named base field.
            if let Some(fld) = layout.fields.iter().rev().find(|f| f.name == field_name) {
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
        self.find_method_in_class_for_receiver(class_name, method_name, false)
    }

    /// Finds a method, selecting the const overload when the receiver is const.
    pub fn find_method_in_class_for_receiver(
        &self,
        class_name: &str,
        method_name: &str,
        receiver_const: bool,
    ) -> Option<FunctionSig> {
        let pick_sig = |sigs: &[FunctionSig]| -> Option<FunctionSig> {
            let receiver_matches =
                |sig: &FunctionSig| !receiver_const || crate::sema::sig_receiver_is_const(sig);
            if let Some(non_const) = sigs
                .iter()
                .find(|s| receiver_matches(s) && !crate::sema::sig_receiver_is_const(s))
            {
                Some(non_const.clone())
            } else {
                sigs.iter().find(|s| receiver_matches(s)).cloned()
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
                if let Some(sig) =
                    self.find_method_in_class_for_receiver(b, method_name, receiver_const)
                {
                    return Some(sig);
                }
            }
        }
        None
    }

    pub fn find_static_methods_in_class(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Vec<FunctionSig> {
        let mut methods = Vec::new();
        self.collect_static_methods(class_name, method_name, &mut methods, &mut Vec::new());
        methods
    }

    fn collect_static_methods(
        &self,
        class_name: &str,
        method_name: &str,
        methods: &mut Vec<FunctionSig>,
        visited: &mut Vec<String>,
    ) {
        if visited.iter().any(|name| name == class_name) {
            return;
        }
        visited.push(class_name.to_string());
        let qual_name = format!("{}::{}", class_name, method_name);
        if let Some(sigs) = self.functions.get(&qual_name) {
            methods.extend(sigs.iter().filter(|sig| sig.is_static).cloned());
        }
        if let Some(layout) = self.struct_layouts.get(class_name) {
            for base in &layout.base_classes {
                self.collect_static_methods(base, method_name, methods, visited);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn layout(base_classes: Vec<String>) -> StructLayout {
        StructLayout {
            is_class: true,
            base_classes,
            base_offsets: Vec::new(),
            fields: Vec::new(),
            size: 4,
            align: 4,
            has_vtable: false,
            vtable_slots: Vec::new(),
            private_members: HashSet::new(),
            is_abstract: false,
            pure_virtuals: HashSet::new(),
        }
    }

    fn method(name: &str, mangled_name: &str, is_static: bool) -> FunctionSig {
        FunctionSig {
            name: name.to_string(),
            mangled_name: mangled_name.to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static,
        }
    }

    #[test]
    fn finds_static_methods_in_secondary_bases_only() {
        let mut sema = Sema::new();
        sema.struct_layouts
            .insert("C".to_string(), layout(vec!["B".to_string()]));
        sema.struct_layouts
            .insert("B".to_string(), layout(Vec::new()));
        sema.register_function(method("B::add", "B_add_i_i", true));
        sema.register_function(method("B::run", "B_run_i_i", false));

        let statics = sema.find_static_methods_in_class("C", "add");
        assert_eq!(statics.len(), 1);
        assert_eq!(statics[0].mangled_name, "B_add_i_i");
        assert!(sema.find_static_methods_in_class("C", "run").is_empty());
    }
}
