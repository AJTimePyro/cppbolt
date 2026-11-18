use super::layout_types::LayoutAccumulator;
use super::*;

impl Sema {
    /// Computes field offsets, base-subobject offset, and vtable slot order.
    ///
    /// The parser permits at most one base class, so layout handles one base
    /// subobject. A single polymorphic base contributes the vtable.
    fn check_final_bases(
        &self,
        bases: &[String],
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        for base in bases {
            if self
                .struct_layouts
                .get(base.as_str())
                .is_some_and(|layout| layout.is_final)
            {
                return Err(CompileError::new(
                    format!("cannot derive from final class '{}'", base),
                    line,
                    col,
                ));
            }
        }
        // An implicitly declared destructor overrides the base destructor, so
        // deriving from a class with a final virtual destructor is ill-formed
        // even when the derived class declares no destructor.
        if bases
            .iter()
            .filter_map(|base| self.struct_layouts.get(base.as_str()))
            .any(|layout| layout.final_methods.iter().any(|key| key.is_destructor()))
        {
            return Err(CompileError::new(
                format!(
                    "cannot derive from class '{}' with a final destructor",
                    bases[0]
                ),
                line,
                col,
            ));
        }
        Ok(())
    }

    fn collect_final_methods(
        &self,
        methods: &[FunctionDecl],
        bases: &[String],
    ) -> Vec<VirtualKey> {
        methods
            .iter()
            .filter(|m| m.qualifiers.is_final)
            .map(VirtualKey::from_decl)
            .chain(
                bases
                    .iter()
                    .filter_map(|base| self.struct_layouts.get(base.as_str()))
                    .flat_map(|layout| layout.final_methods.iter().cloned()),
            )
            .collect()
    }

    fn collect_inherited_virtuals(&self, bases: &[String]) -> Vec<(String, VirtualKey)> {
        bases
            .iter()
            .filter_map(|base| {
                self.struct_layouts
                    .get(base.as_str())
                    .map(|layout| (base, layout))
            })
            .flat_map(|(base, layout)| {
                layout
                    .vtable_slots
                    .iter()
                    .filter(|key| key.name != "__deleting_dtor")
                    .map(move |key| (base.clone(), key.clone()))
            })
            .collect()
    }

    fn has_virtual_own(&self, methods: &[FunctionDecl], keys: &[VirtualKey]) -> bool {
        methods
            .iter()
            .any(|m| m.is_virtual || keys.contains(&VirtualKey::from_decl(m)))
    }

    fn has_polymorphic_base(&self, bases: &[String]) -> bool {
        bases.iter().any(|base| {
            self.struct_layouts
                .get(base.as_str())
                .is_some_and(|l| l.has_vtable)
        })
    }

    fn has_user_ctor(methods: &[FunctionDecl], name: &str, short: &str) -> bool {
        methods
            .iter()
            .any(|m| (m.name == name || m.name == short) && (m.line != 0 || m.col != 0))
    }

    pub(super) fn compute_struct_layout(
        &mut self,
        input: RecordLayout<'_>,
    ) -> Result<(), CompileError> {
        let mut acc = LayoutAccumulator::new();
        let bases: Vec<String> = input
            .base_classes
            .iter()
            .map(|b| self.canonical_class_name(b, Some(input.name)))
            .collect();
        if let Some(align) = input.alignment {
            acc.max_align = acc.max_align.max(align);
        }
        self.check_final_bases(&bases, input.line, input.col)?;
        let inherited_virtuals = self.collect_inherited_virtuals(&bases);
        self.validate_virtual_declarations(&input, &bases, &inherited_virtuals)?;
        let keys: Vec<VirtualKey> = inherited_virtuals.iter().map(|(_, k)| k.clone()).collect();
        let has_virtual_methods = self.has_virtual_own(input.methods, &keys);
        // Any polymorphic base contributes a vptr; it need not be the first
        // declared base (a non-polymorphic pad may come first).
        let primary_has_vtable = self.has_polymorphic_base(&bases);
        let has_vtable = has_virtual_methods || primary_has_vtable;
        if has_virtual_methods && !primary_has_vtable {
            // No polymorphic base: a vtable pointer precedes the fields.
            acc.offset = 4;
            acc.max_align = 4;
        }

        self.layout_base_subobjects(&bases, &mut acc, input.line, input.col)?;

        self.layout_vtable_slots(input.methods, &bases, has_vtable, &mut acc.vtable_slots);

        self.layout_own_fields(input.fields, &mut acc);

        acc.round_up();
        acc.round_up();
        let vtables = self.build_vtables(input.name, &bases, &acc, has_vtable);

        let pure_virtuals = self.collect_pure_virtuals(&bases, input.methods);
        let is_abstract = !pure_virtuals.is_empty();
        let short = QualifiedName::new(input.name).class_short_name();
        let has_user_declared_constructors =
            Self::has_user_ctor(input.methods, input.name, short);

        let layout = StructLayout {
            is_class: input.is_class,
            is_final: input.is_final,
            has_user_declared_constructors,
            base_classes: bases.clone(),
            base_access: input.base_access.to_vec(),
            base_offsets: acc.base_offsets,
            direct_base_offsets: acc.direct_base_offsets,
            fields: acc.struct_fields,
            size: acc.offset,
            align: acc.max_align,
            has_vtable,
            vtable_slots: acc.vtable_slots.clone(),
            vtables,
            private_members: input.private_members.iter().cloned().collect(),
            is_abstract,
            pure_virtuals,
            final_methods: self.collect_final_methods(input.methods, &bases),
        };

        self.record_struct_layout(input.name, layout, acc.offset, acc.max_align);

        Ok(())
    }

    /// Validates `override`/`final` against the semantic signatures of base
    /// virtuals, including return and exception-specification compatibility.
    fn validate_virtual_declarations(
        &self,
        input: &RecordLayout<'_>,
        bases: &[String],
        inherited: &[(String, VirtualKey)],
    ) -> Result<(), CompileError> {
        let inherited_finals: Vec<VirtualKey> = bases
            .iter()
            .filter_map(|base| self.struct_layouts.get(base.as_str()))
            .flat_map(|layout| layout.final_methods.iter().cloned())
            .collect();
        for m in input.methods.iter() {
            let key = VirtualKey::from_decl(m);
            if m.is_static && (m.qualifiers.is_override || m.qualifiers.is_final) {
                return Err(CompileError::new(
                    format!(
                        "static member function '{}::{}' cannot be override or final",
                        input.name, m.name
                    ),
                    m.line,
                    m.col,
                ));
            }
            let base = inherited.iter().find(|(_, k)| *k == key);
            match base {
                Some(_) => {
                    if m.is_static {
                        return Err(CompileError::new(
                            format!(
                                "static member function '{}::{}' cannot override a virtual function",
                                input.name, m.name
                            ),
                            m.line,
                            m.col,
                        ));
                    }
                    if inherited_finals.contains(&key) {
                        return Err(CompileError::new(
                            format!("'{}::{}' overrides a final method", input.name, m.name),
                            m.line,
                            m.col,
                        ));
                    }
                }
                None if m.qualifiers.is_override => {
                    return Err(CompileError::new(
                        format!(
                            "'{}::{}' marked override but overrides no virtual method",
                            input.name, m.name
                        ),
                        m.line,
                        m.col,
                    ));
                }
                None if m.qualifiers.is_final && !m.is_virtual => {
                    return Err(CompileError::new(
                        "only virtual member functions can be marked final".to_string(),
                        m.line,
                        m.col,
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Lays out this record's own fields after all base subobjects.
    fn layout_own_fields(&mut self, fields: &[StructFieldDecl], acc: &mut LayoutAccumulator) {
        for (fname, ftype, finit) in fields {
            let fsize = ftype.size_of(&self.struct_sizes);
            let falign = ftype.align_of(&self.struct_aligns);
            acc.max_align = acc.max_align.max(falign);

            if falign > 0 && !acc.offset.is_multiple_of(falign) {
                acc.offset += falign - (acc.offset % falign);
            }

            acc.struct_fields.push(StructField {
                name: fname.clone(),
                ty: ftype.clone(),
                offset: acc.offset,
                default_init: finit.as_ref().map(|(e, _, _)| e.clone()),
            });

            acc.offset += fsize;
        }
    }

    /// Lays out all base subobjects sequentially in declaration order.
    fn layout_base_subobjects(
        &mut self,
        base_classes: &[String],
        acc: &mut LayoutAccumulator,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        for base_name in base_classes {
            let Some(base_layout) = self.struct_layouts.get(base_name).cloned() else {
                continue;
            };
            let base_offset =
                if acc.offset == 0 && base_layout.fields.is_empty() && !base_layout.has_vtable {
                    0
                } else {
                    let align = base_layout.align.max(1);
                    if !acc.offset.is_multiple_of(align) {
                        acc.offset += align - (acc.offset % align);
                    }
                    let base_offset = acc.offset;
                    acc.offset = acc.offset.checked_add(base_layout.size).ok_or_else(|| {
                        CompileError::new("object layout is too large", line, col)
                    })?;
                    base_offset
                };
            acc.max_align = acc.max_align.max(base_layout.align);
            acc.direct_base_offsets
                .push((base_name.clone(), base_offset));
            acc.base_offsets.push((base_name.clone(), base_offset));
            for (sub_b, sub_off) in &base_layout.base_offsets {
                acc.base_offsets
                    .push((sub_b.clone(), base_offset + sub_off));
            }
            if base_layout.has_vtable && acc.vtable_slots.is_empty() {
                acc.vtable_slots = base_layout.vtable_slots.clone();
            }
            for mut bf in base_layout.fields {
                bf.offset = base_offset
                    .checked_add(bf.offset)
                    .ok_or_else(|| CompileError::new("object layout is too large", line, col))?;
                acc.struct_fields.push(bf);
            }
        }
        Ok(())
    }
    /// Builds one vtable descriptor per polymorphic subobject.
    ///
    /// A derived class has no own vptr of its own; it reuses the vptr of the
    /// first polymorphic base, wherever that subobject actually sits. A class
    /// only owns a vptr at offset zero when it declares virtuals and no base
    /// is polymorphic.
    fn collect_pure_virtuals(
        &self,
        base_classes: &[String],
        methods: &[FunctionDecl],
    ) -> std::collections::HashSet<String> {
        let mut pure_virtuals = std::collections::HashSet::new();
        for base_name in base_classes {
            if let Some(base_layout) = self.struct_layouts.get(base_name) {
                for pv in &base_layout.pure_virtuals {
                    pure_virtuals.insert(pv.clone());
                }
            }
        }
        for m in methods {
            if m.is_pure_virtual {
                pure_virtuals.insert(m.name.clone());
            } else {
                pure_virtuals.remove(&m.name);
            }
        }
        pure_virtuals
    }

    fn record_struct_layout(
        &mut self,
        name: &str,
        layout: StructLayout,
        offset: usize,
        max_align: usize,
    ) {
        if !self.struct_order.contains(&name.to_string()) {
            self.struct_order.push(name.to_string());
        }
        self.struct_sizes.insert(name.to_string(), offset);
        self.struct_aligns.insert(name.to_string(), max_align);
        self.struct_layouts.insert(name.to_string(), layout.clone());
        if let Some(alias) = QualifiedName::new(name).implicit_alias() {
            self.struct_sizes.insert(alias.to_string(), offset);
            self.struct_aligns.insert(alias.to_string(), max_align);
            self.struct_layouts
                .insert(alias.to_string(), layout.clone());
        }
    }

    pub fn mangle_name(&self, name: &str, params: &[Param]) -> String {
        if name == "main"
            || name == "printf"
            || name == "std::printf"
            || name.ends_with("::printf")
            || name == "malloc"
            || name == "free"
            || name == "exit"
            || name == "__builtin_wasm_memory_size"
            || name == "print_char"
            || name == "print_int"
            || name == "print_double"
            || name == "read_char"
        {
            return name.to_string();
        }
        let mut s = format!("{}_", name);
        for p in params {
            match &p.ty {
                Type::Int => s.push('i'),
                Type::UnsignedInt => s.push('j'),
                Type::Long => s.push('l'),
                Type::UnsignedLong => s.push('m'),
                Type::LongLong => s.push('x'),
                Type::UnsignedLongLong => s.push('y'),
                Type::Short => s.push('s'),
                Type::UnsignedShort => s.push('t'),
                Type::Char => s.push('c'),
                Type::SignedChar => s.push('a'),
                Type::UnsignedChar => s.push('h'),
                Type::Bool => s.push('b'),
                Type::Float => s.push('f'),
                Type::Double => s.push('d'),
                Type::Void => s.push('v'),
                Type::Nullptr => s.push_str("Dn"),
                Type::Pointer(inner) => s.push_str(&format!("P{:?}", inner)),
                Type::Reference(inner) => s.push_str(&format!("R{:?}", inner)),
                Type::RvalueReference(inner) => s.push_str(&format!("RR{:?}", inner)),
                Type::Struct(name) => s.push_str(&format!("S{}", name)),
                Type::Enum(name) => s.push_str(&format!("E{}", name)),
                Type::Array(elem, n) => s.push_str(&format!("A{}_{:?}", n, elem)),
                Type::Const(inner) => s.push_str(&format!("K{:?}", inner)),
                _ => s.push('u'),
            }
        }
        s
    }
}
