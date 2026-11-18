use super::*;

/// One entry of a per-subobject virtual table.
#[derive(Debug, Clone)]
pub struct VTableSlotInfo {
    /// Semantic identity of the virtual function in this slot.
    pub key: VirtualKey,
    /// The deleting-destructor companion slot (no ordinary call targets it).
    pub is_deleting_dtor: bool,
}

/// One virtual table belonging to a base subobject of a complete object.
///
/// A polymorphic class with several polymorphic bases has one vptr (and one
/// table) per such subobject. `vptr_offset` is measured from the start of the
/// complete object; the primary table is normally at offset zero.
#[derive(Debug, Clone)]
pub struct VTableInfo {
    /// Class of the subobject that owns this vptr.
    pub owner: String,
    /// Offset of the vptr from the start of the complete object.
    pub vptr_offset: usize,
    /// Offset of the base subobject that owns this vptr (may differ from
    /// vptr_offset when the base inherited its vptr from another base).
    pub owner_offset: usize,
    /// Slots in dispatch order.
    pub slots: Vec<VTableSlotInfo>,
}

pub(super) fn element_struct_name(ty: &Type) -> Option<&str> {
    match ty.strip_const() {
        Type::Struct(name) => Some(name),
        Type::Array(elem, _) => element_struct_name(elem),
        _ => None,
    }
}

/// The declaration of one record plus the source position used for layout
/// diagnostics.
pub(crate) struct RecordLayout<'a> {
    pub(super) name: &'a str,
    pub(super) is_class: bool,
    pub(super) is_final: bool,
    pub(super) base_classes: &'a [String],
    pub(super) base_access: &'a [crate::ast::BaseAccess],
    pub(super) fields: &'a [StructFieldDecl],
    pub(super) methods: &'a [FunctionDecl],
    pub(super) private_members: &'a [String],
    pub(super) line: usize,
    pub(super) col: usize,
    pub(super) alignment: Option<usize>,
}

/// Accumulates the offsets, fields and vtable slots produced while laying out
/// one record.
struct LayoutAccumulator {
    offset: usize,
    max_align: usize,
    struct_fields: Vec<StructField>,
    base_offsets: Vec<(String, usize)>,
    direct_base_offsets: Vec<(String, usize)>,
    vtable_slots: Vec<VirtualKey>,
}

impl LayoutAccumulator {
    fn new() -> Self {
        Self {
            offset: 0,
            max_align: 1,
            struct_fields: Vec::new(),
            base_offsets: Vec::new(),
            direct_base_offsets: Vec::new(),
            vtable_slots: Vec::new(),
        }
    }

    /// Rounds the running offset up to the alignment and keeps an empty record
    /// at least one byte large.
    ///
    /// The pass runs twice while laying out a record: the first pass aligns the
    /// fields, the second re-aligns the one-byte minimum of an empty record.
    fn round_up(&mut self) {
        if self.max_align > 0 && !self.offset.is_multiple_of(self.max_align) {
            self.offset += self.max_align - (self.offset % self.max_align);
        }
        if self.offset == 0 {
            self.offset = 1;
        }
    }
}

impl Sema {
    /// Computes field offsets, base-subobject offset, and vtable slot order.
    ///
    /// The parser permits at most one base class, so layout handles one base
    /// subobject. A single polymorphic base contributes the vtable.
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
        for base in &bases {
            if self
                .struct_layouts
                .get(base.as_str())
                .is_some_and(|layout| layout.is_final)
            {
                return Err(CompileError::new(
                    format!("cannot derive from final class '{}'", base),
                    input.line,
                    input.col,
                ));
            }
        }
        let inherited_virtuals: Vec<(String, VirtualKey)> = bases
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
            .collect();
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
                input.line,
                input.col,
            ));
        }
        self.validate_virtual_declarations(&input, &bases, &inherited_virtuals)?;
        let keys: Vec<VirtualKey> = inherited_virtuals.iter().map(|(_, k)| k.clone()).collect();
        let has_virtual_methods = input
            .methods
            .iter()
            .any(|m| m.is_virtual || keys.contains(&VirtualKey::from_decl(m)));
        // Any polymorphic base contributes a vptr; it need not be the first
        // declared base (a non-polymorphic pad may come first).
        let primary_has_vtable = bases.iter().any(|base| {
            self.struct_layouts
                .get(base.as_str())
                .is_some_and(|l| l.has_vtable)
        });
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
        let has_user_declared_constructors = input
            .methods
            .iter()
            .any(|m| (m.name == input.name || m.name == short) && (m.line != 0 || m.col != 0));

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
            final_methods: input
                .methods
                .iter()
                .filter(|m| m.qualifiers.is_final)
                .map(VirtualKey::from_decl)
                .chain(
                    bases
                        .iter()
                        .filter_map(|base| self.struct_layouts.get(base.as_str()))
                        .flat_map(|layout| layout.final_methods.iter().cloned()),
                )
                .collect(),
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
    fn build_vtables(
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
    fn layout_vtable_slots(
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
