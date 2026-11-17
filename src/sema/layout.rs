use super::*;

/// One entry of a per-subobject virtual table.
#[derive(Debug, Clone)]
pub struct VTableSlotInfo {
    /// Source-level virtual function name. `~` and `__deleting_dtor` denote
    /// the destructor and its deleting counterpart.
    pub name: String,
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
    vtable_slots: Vec<String>,
}

impl LayoutAccumulator {
    fn new() -> Self {
        Self {
            offset: 0,
            max_align: 1,
            struct_fields: Vec::new(),
            base_offsets: Vec::new(),
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
        if let Some(align) = input.alignment {
            acc.max_align = acc.max_align.max(align);
        }
        let has_virtual_methods = input.methods.iter().any(|m| m.is_virtual);
        let primary_has_vtable = input
            .base_classes
            .first()
            .and_then(|base| self.struct_layouts.get(base))
            .map(|layout| layout.has_vtable)
            .unwrap_or(false);
        let has_vtable = has_virtual_methods || primary_has_vtable;
        if has_virtual_methods && !primary_has_vtable {
            // A vtable pointer precedes the primary base subobject.
            acc.offset = 4;
            acc.max_align = 4;
        }

        self.layout_base_subobjects(input.base_classes, &mut acc, input.line, input.col)?;

        self.layout_vtable_slots(
            input.methods,
            input.base_classes,
            has_vtable,
            &mut acc.vtable_slots,
        );

        self.layout_own_fields(input.fields, &mut acc);

        acc.round_up();
        acc.round_up();
        let vtables = self.build_vtables(input.name, input.base_classes, &acc, has_vtable);

        let pure_virtuals = self.collect_pure_virtuals(input.base_classes, input.methods);
        let is_abstract = !pure_virtuals.is_empty();
        let short = QualifiedName::new(input.name).class_short_name();
        let has_user_declared_constructors = input
            .methods
            .iter()
            .any(|m| (m.name == input.name || m.name == short) && (m.line != 0 || m.col != 0));

        let layout = StructLayout {
            is_class: input.is_class,
            has_user_declared_constructors,
            base_classes: input.base_classes.to_vec(),
            base_access: input.base_access.to_vec(),
            base_offsets: acc.base_offsets,
            fields: acc.struct_fields,
            size: acc.offset,
            align: acc.max_align,
            has_vtable,
            vtable_slots: acc.vtable_slots.clone(),
            vtables,
            private_members: input.private_members.iter().cloned().collect(),
            is_abstract,
            pure_virtuals,
        };

        self.record_struct_layout(input.name, layout, acc.offset, acc.max_align);

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
    /// The first polymorphic base shares the vptr at offset zero with the
    /// complete object (the "primary" table). Every other polymorphic base
    /// keeps its own vptr at its base-subobject offset. Bases are enumerated
    /// through the flattened `base_offsets` list, so indirect and repeated
    /// bases each receive their own descriptor.
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
        let primary_base = base_classes
            .first()
            .filter(|b| self.struct_layouts.get(*b).is_some_and(|l| l.has_vtable));
        let primary_owner = primary_base
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let as_slots = |names: &[String]| -> Vec<VTableSlotInfo> {
            names
                .iter()
                .map(|n| VTableSlotInfo { name: n.clone() })
                .collect()
        };
        let mut tables = vec![VTableInfo {
            owner: primary_owner.clone(),
            vptr_offset: 0,
            slots: as_slots(&acc.vtable_slots),
        }];
        for (base_name, base_offset) in &acc.base_offsets {
            if *base_offset == 0 && *base_name == primary_owner {
                continue;
            }
            let Some(bl) = self.struct_layouts.get(base_name) else {
                continue;
            };
            if !bl.has_vtable {
                continue;
            }
            let Some(primary) = bl.vtables.first() else {
                continue;
            };
            if tables.iter().any(|t| t.vptr_offset == *base_offset) {
                continue;
            }
            tables.push(VTableInfo {
                owner: base_name.clone(),
                vptr_offset: *base_offset,
                slots: primary.slots.clone(),
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
        vtable_slots: &mut Vec<String>,
    ) {
        if !has_vtable {
            return;
        }
        // An override of a virtual declared in *any* base (including a
        // secondary polymorphic base) replaces that base's slot; it does not
        // introduce a new slot in the primary table.
        let mut inherited: Vec<String> = vtable_slots.clone();
        for base in base_classes {
            if let Some(bl) = self.struct_layouts.get(base) {
                for slot in &bl.vtable_slots {
                    if !inherited.contains(slot) {
                        inherited.push(slot.clone());
                    }
                }
            }
        }
        for m in methods {
            if m.name.starts_with('~') {
                let is_virt_dtor = m.is_virtual || inherited.iter().any(|s| s == "~");
                if is_virt_dtor && !vtable_slots.iter().any(|s| s == "~") {
                    vtable_slots.push("~".to_string());
                    vtable_slots.push("__deleting_dtor".to_string());
                }
            } else {
                let is_override = inherited.iter().any(|s| s == &m.name);
                if (m.is_virtual || is_override) && !vtable_slots.contains(&m.name) {
                    vtable_slots.push(m.name.clone());
                }
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
