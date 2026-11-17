use super::*;

impl<'a> Codegen<'a> {
    /// Resolves a free-function call, including ADL and the implicit `std`
    /// alias, selecting the lowest-cost overload.
    pub fn find_function(&self, name: &str, arg_types: &[Type]) -> Option<(u32, FunctionSig)> {
        self.find_function_with_args(name, arg_types, None)
    }

    pub fn find_function_with_args(
        &self,
        name: &str,
        arg_types: &[Type],
        args: Option<&[Expr]>,
    ) -> Option<(u32, FunctionSig)> {
        for q in candidate_function_names(name, arg_types) {
            let Some(sigs) = self.sema.functions.get(&q) else {
                continue;
            };
            let mut best_sig = None;
            let mut best_score = usize::MAX;
            for sig in sigs {
                let Some(score) = self.score_overload_params(&sig.param_types, arg_types, args)
                else {
                    continue;
                };
                if score < best_score {
                    best_score = score;
                    best_sig = Some(sig.clone());
                }
            }
            if let Some(sig) = best_sig
                && let Some(idx) = self.resolve_signature_index(name, &sig)
            {
                return Some((idx, sig));
            }
        }
        self.func_indices
            .get(name)
            .or_else(|| {
                self.func_indices
                    .get(QualifiedName::new(name).std_name().as_ref())
            })
            .or_else(|| self.func_indices.get(QualifiedName::new(name).plain()))
            .map(|&idx| (idx, default_signature(name)))
    }

    fn resolve_signature_index(&self, name: &str, sig: &FunctionSig) -> Option<u32> {
        if let Some(&idx) = self.func_indices.get(&sig.mangled_name) {
            return Some(idx);
        }
        if let Some(&idx) = self.func_indices.get(name) {
            return Some(idx);
        }
        if let Some(&idx) = self
            .func_indices
            .get(QualifiedName::new(name).std_name().as_ref())
        {
            return Some(idx);
        }
        self.func_indices
            .get(QualifiedName::new(name).plain())
            .copied()
    }

    pub(super) fn lookup_struct_dtor(&self, sname: &str) -> Option<u32> {
        let short_sname = QualifiedName::new(sname).class_short_name();
        self.func_indices
            .get(&format!("{sname}_~{sname}"))
            .or_else(|| self.func_indices.get(&format!("{sname}_~{short_sname}")))
            .copied()
    }
    /// Returns the inheritance path from `cls` (at absolute `cls_offset`) to
    /// the subobject of class `owner` at absolute `target_offset`.
    ///
    /// The path is used to find a slot's final overrider: only classes on this
    /// path can override that specific subobject's virtual function. Repeated
    /// base names are distinguished by their offsets.
    /// Direct bases and their offsets within `cls`, recovered from the flat
    /// `base_offsets` list (each direct base is followed by its own bases).
    fn direct_base_offsets(&self, cls: &str) -> Vec<(String, usize)> {
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        let mut index = 0;
        for base in &layout.base_classes {
            let Some((_, offset)) = layout.base_offsets.get(index) else {
                break;
            };
            result.push((base.clone(), *offset));
            let sub_count = self
                .sema
                .struct_layouts
                .get(base)
                .map(|l| l.base_offsets.len())
                .unwrap_or(0);
            index += 1 + sub_count;
        }
        result
    }

    fn inheritance_path(
        &self,
        cls: &str,
        cls_offset: usize,
        owner: &str,
        target_offset: usize,
        depth: usize,
    ) -> Option<Vec<(String, usize)>> {
        if depth > 64 {
            return None;
        }
        if cls == owner && cls_offset == target_offset {
            return Some(vec![(cls.to_string(), cls_offset)]);
        }
        for (base, base_offset) in self.direct_base_offsets(cls) {
            let base_absolute = cls_offset + base_offset;
            if let Some(mut rest) =
                self.inheritance_path(&base, base_absolute, owner, target_offset, depth + 1)
            {
                let mut path = vec![(cls.to_string(), cls_offset)];
                path.append(&mut rest);
                return Some(path);
            }
        }
        None
    }

    /// Finds the class that originally declared a virtual method in `cls`'s
    /// own hierarchy, with its offset relative to `cls`.
    fn declaring_class(
        &self,
        cls: &str,
        method_name: &str,
        depth: usize,
    ) -> Option<(String, usize)> {
        if depth > 64 {
            return None;
        }
        if self
            .sema
            .functions
            .contains_key(&format!("{}::{}", cls, method_name))
        {
            return Some((cls.to_string(), 0));
        }
        for (base, offset) in self.direct_base_offsets(cls) {
            if let Some((decl, relative)) = self.declaring_class(&base, method_name, depth + 1) {
                return Some((decl, offset + relative));
            }
        }
        None
    }

    /// Finds the final overrider of a vtable slot owned by a specific base
    /// subobject. Returns the declaring class and its absolute offset in the
    /// most-derived object.
    pub(super) fn find_final_overrider(
        &self,
        class_name: &str,
        owner: &str,
        vptr_offset: usize,
        method_name: &str,
    ) -> Option<(String, usize)> {
        let path = self.inheritance_path(class_name, 0, owner, vptr_offset, 0)?;
        for (cls, offset) in &path {
            if self
                .sema
                .functions
                .contains_key(&format!("{}::{}", cls, method_name))
            {
                return Some((cls.clone(), *offset));
            }
        }
        // No class on the path overrides this slot. Use the class that
        // originally declared it in the owner's own hierarchy.
        let (decl, relative) = self.declaring_class(owner, method_name, 0)?;
        Some((decl, vptr_offset + relative))
    }

    /// Looks up a virtual method on a class or one of its bases.
    pub fn find_class_vmethod(&self, class_name: &str, method_name: &str) -> Option<u32> {
        let m_name = format!("{}_{}", class_name, method_name);
        if let Some(&idx) = self.func_indices.get(&m_name) {
            return Some(idx);
        }
        if let Some(layout) = self.sema.struct_layouts.get(class_name) {
            for b in &layout.base_classes {
                if let Some(idx) = self.find_class_vmethod(b, method_name) {
                    return Some(idx);
                }
            }
        }
        None
    }
}

fn candidate_function_names(name: &str, arg_types: &[Type]) -> Vec<String> {
    let mut candidate_names = vec![
        name.to_string(),
        QualifiedName::new(name).alternate().into_owned(),
    ];
    for arg in arg_types {
        if let Type::Struct(sname) = arg.strip_const().strip_reference()
            && let Some((ns, _)) = sname.rsplit_once("::")
        {
            let adl_name = format!("{}::{}", ns, name);
            if !candidate_names.contains(&adl_name) {
                candidate_names.push(adl_name);
            }
        }
    }
    candidate_names
}

fn default_signature(name: &str) -> FunctionSig {
    FunctionSig {
        qualifiers: FunctionQualifiers::default(),
        name: name.to_string(),
        mangled_name: name.to_string(),
        return_type: Type::Int,
        param_types: vec![],
        default_args: vec![],
        is_static: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
    }
}
