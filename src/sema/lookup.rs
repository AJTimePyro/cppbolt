use super::*;
/// Result of checking a base-to-derived pointer-to-member conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemberPointerBaseConversion {
    Ok,
    Ambiguous,
    Inaccessible,
    Unrelated,
}

impl Sema {
    /// Counts distinct derivation paths from `derived` to `base`. Virtual
    /// inheritance is unsupported, so each path is a distinct subobject.
    pub fn base_subobject_path_count(&self, derived: &str, base: &str) -> usize {
        if derived == base {
            return 1;
        }
        let Some(layout) = self.struct_layouts.get(derived) else {
            return 0;
        };
        layout
            .base_classes
            .iter()
            .map(|b| self.base_subobject_path_count(b, base))
            .sum()
    }
    fn base_path_is_accessible(&self, derived: &str, base: &str) -> bool {
        if derived == base {
            return true;
        }
        let Some(layout) = self.struct_layouts.get(derived) else {
            return false;
        };
        let inside_derived = self
            .current_class
            .as_deref()
            .is_some_and(|c| self.is_subclass(c, derived));
        for (i, b) in layout.base_classes.iter().enumerate() {
            let access = layout.base_access.get(i).copied().unwrap_or_default();
            let accessible = match access {
                crate::ast::BaseAccess::Public => true,
                crate::ast::BaseAccess::Protected | crate::ast::BaseAccess::Private => {
                    inside_derived
                }
            };
            if accessible && self.base_path_is_accessible(b, base) {
                return true;
            }
        }
        false
    }
    pub(super) fn check_member_pointer_base_conversion(
        &self,
        derived: &str,
        base: &str,
    ) -> MemberPointerBaseConversion {
        if derived == base {
            return MemberPointerBaseConversion::Ok;
        }
        match self.base_subobject_path_count(derived, base) {
            0 => MemberPointerBaseConversion::Unrelated,
            n if n > 1 => MemberPointerBaseConversion::Ambiguous,
            _ if !self.base_path_is_accessible(derived, base) => {
                MemberPointerBaseConversion::Inaccessible
            }
            _ => MemberPointerBaseConversion::Ok,
        }
    }
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

    pub fn base_subobject_offset(&self, derived: &str, base: &str) -> Option<usize> {
        self.struct_layouts.get(derived).and_then(|layout| {
            layout
                .base_offsets
                .iter()
                .find(|(b, _)| b == base)
                .map(|(_, off)| *off)
        })
    }

    pub fn find_field_in_class(&self, class_name: &str, field_name: &str) -> Option<StructField> {
        if class_name.is_empty() {
            return None;
        }
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
        } else if !QualifiedName::new(class_name).in_std()
            && let Some(alias) = QualifiedName::new(class_name).implicit_alias()
        {
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
        self.find_method_in_class_for_receiver_category(
            class_name,
            method_name,
            receiver_const,
            true,
        )
    }

    pub fn find_method_in_class_for_receiver_category(
        &self,
        class_name: &str,
        method_name: &str,
        receiver_const: bool,
        receiver_is_lvalue: bool,
    ) -> Option<FunctionSig> {
        if class_name.is_empty() {
            return None;
        }
        let pick_sig = |sigs: &[FunctionSig]| -> Option<FunctionSig> {
            let receiver_matches = |sig: &FunctionSig| {
                if receiver_const && !crate::sema::sig_receiver_is_const(sig) {
                    return false;
                }
                if receiver_is_lvalue && sig.ref_qualifier == RefQualifier::RValue {
                    return false;
                }
                if !receiver_is_lvalue
                    && sig.ref_qualifier == RefQualifier::LValue
                    && !crate::sema::sig_receiver_is_const(sig)
                {
                    return false;
                }
                true
            };
            if !receiver_is_lvalue
                && let Some(rval) = sigs
                    .iter()
                    .find(|s| receiver_matches(s) && s.ref_qualifier == RefQualifier::RValue)
            {
                return Some(rval.clone());
            }
            if receiver_is_lvalue
                && let Some(lval) = sigs
                    .iter()
                    .find(|s| receiver_matches(s) && s.ref_qualifier == RefQualifier::LValue)
            {
                return Some(lval.clone());
            }
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
                if let Some(sig) = self.find_method_in_class_for_receiver_category(
                    b,
                    method_name,
                    receiver_const,
                    receiver_is_lvalue,
                ) {
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

    /// Collects every overload of a method, including the ones of a base class.
    ///
    /// A const receiver only considers the const-qualified overloads.
    pub fn find_method_overloads_in_class(
        &self,
        class_name: &str,
        method_name: &str,
        receiver_const: bool,
    ) -> Vec<FunctionSig> {
        let mut overloads = Vec::new();
        let mut visited = Vec::new();
        self.collect_method_overloads(
            class_name,
            method_name,
            receiver_const,
            &mut overloads,
            &mut visited,
        );
        overloads
    }

    /// Finds a method with an exact parameter count (including the implicit receiver).
    pub fn find_method_with_param_count(
        &self,
        class_name: &str,
        method_name: &str,
        param_count: usize,
        receiver_const: bool,
    ) -> Option<FunctionSig> {
        let overloads =
            self.find_method_overloads_in_class(class_name, method_name, receiver_const);
        overloads
            .into_iter()
            .find(|s| s.param_types.len() == param_count)
    }

    fn collect_method_overloads(
        &self,
        class_name: &str,
        method_name: &str,
        receiver_const: bool,
        out: &mut Vec<FunctionSig>,
        visited: &mut Vec<String>,
    ) {
        if visited.iter().any(|name| name == class_name) {
            return;
        }
        visited.push(class_name.to_string());
        let qual_name = format!("{class_name}::{method_name}");
        if let Some(sigs) = self.functions.get(&qual_name) {
            for sig in sigs {
                if !receiver_const || crate::sema::sig_receiver_is_const(sig) {
                    out.push(sig.clone());
                }
            }
        }
        if let Some(alias) = QualifiedName::new(class_name).implicit_alias() {
            let alt = format!("{alias}::{method_name}");
            if let Some(sigs) = self.functions.get(&alt) {
                for sig in sigs {
                    if !receiver_const || crate::sema::sig_receiver_is_const(sig) {
                        out.push(sig.clone());
                    }
                }
            }
        }
        // A declaration in a derived class hides same-named base declarations
        // regardless of parameter types.
        if !out.is_empty() {
            return;
        }
        if let Some(layout) = self.struct_layouts.get(class_name) {
            for base in &layout.base_classes {
                self.collect_method_overloads(base, method_name, receiver_const, out, visited);
            }
        }
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

    pub(super) fn check_nested_type_access(
        &self,
        sname: &str,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let Some((parent, child)) = sname.rsplit_once("::") else {
            return Ok(());
        };
        let p_clean = QualifiedName::new(parent).plain();
        let Some(layout) = self
            .struct_layouts
            .get(parent)
            .or_else(|| self.struct_layouts.get(p_clean))
        else {
            return Ok(());
        };
        if layout.private_members.contains(child) || layout.private_members.contains(sname) {
            let cur_clean = self
                .current_class
                .as_deref()
                .map(|c| QualifiedName::new(c).plain());
            let s_clean = QualifiedName::new(sname).plain();
            let in_class = match cur_clean {
                Some(cur) => {
                    cur == p_clean
                        || cur.starts_with(&format!("{p_clean}::"))
                        || cur.starts_with(&format!("{p_clean}<"))
                        || cur.contains(sname)
                        || cur.contains(s_clean)
                }
                None => false,
            };
            if !in_class {
                let error = CompileError::new(
                    format!("'{child}' is a private member of '{p_clean}'"),
                    line,
                    col,
                );
                return Err(if self.in_substitution_context {
                    error.with_kind(crate::diagnostics::ErrorKind::Substitution)
                } else {
                    error
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn layout(base_classes: Vec<String>) -> StructLayout {
        StructLayout {
            is_class: true,
            is_final: false,
            has_user_declared_constructors: false,
            base_classes,
            base_access: Vec::new(),
            base_offsets: Vec::new(),
            direct_base_offsets: Vec::new(),
            fields: Vec::new(),
            size: 4,
            align: 4,
            has_vtable: false,
            vtable_slots: Vec::new(),
            vtables: Vec::new(),
            private_members: HashSet::new(),
            is_abstract: false,
            pure_virtuals: HashSet::new(),
            final_methods: Vec::new(),
        }
    }

    fn method(name: &str, mangled_name: &str, is_static: bool) -> FunctionSig {
        FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: name.to_string(),
            mangled_name: mangled_name.to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
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
