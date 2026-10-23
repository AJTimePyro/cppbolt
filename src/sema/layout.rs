use super::*;

/// The declaration of one record plus the source position used for layout
/// diagnostics.
pub(crate) struct RecordLayout<'a> {
    pub(super) name: &'a str,
    pub(super) is_class: bool,
    pub(super) base_classes: &'a [String],
    pub(super) fields: &'a [(String, Type)],
    pub(super) methods: &'a [FunctionDecl],
    pub(super) private_members: &'a [String],
    pub(super) line: usize,
    pub(super) col: usize,
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

        self.layout_base_subobject(input.base_classes.first(), &mut acc, input.line, input.col)?;

        self.layout_vtable_slots(input.methods, has_vtable, &mut acc.vtable_slots);

        self.layout_own_fields(input.fields, &mut acc);

        acc.round_up();
        acc.round_up();

        let pure_virtuals = self.collect_pure_virtuals(input.base_classes, input.methods);
        let is_abstract = !pure_virtuals.is_empty();

        let layout = StructLayout {
            is_class: input.is_class,
            base_classes: input.base_classes.to_vec(),
            base_offsets: acc.base_offsets,
            fields: acc.struct_fields,
            size: acc.offset,
            align: acc.max_align,
            has_vtable,
            vtable_slots: acc.vtable_slots,
            private_members: input.private_members.iter().cloned().collect(),
            is_abstract,
            pure_virtuals,
        };

        self.record_struct_layout(input.name, layout, acc.offset, acc.max_align);

        Ok(())
    }

    /// Lays out this record's own fields after all base subobjects.
    fn layout_own_fields(&mut self, fields: &[(String, Type)], acc: &mut LayoutAccumulator) {
        for (fname, ftype) in fields {
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
            });

            acc.offset += fsize;
        }
    }

    /// Lays out the single supported base subobject. Only one base is allowed;
    /// the parser rejects a second base specifier.
    fn layout_base_subobject(
        &mut self,
        base_class: Option<&String>,
        acc: &mut LayoutAccumulator,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let Some(base_name) = base_class else {
            return Ok(());
        };
        let Some(base_layout) = self.struct_layouts.get(base_name).cloned() else {
            return Ok(());
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
                acc.offset = acc
                    .offset
                    .checked_add(base_layout.size)
                    .ok_or_else(|| CompileError::new("object layout is too large", line, col))?;
                base_offset
            };
        acc.max_align = acc.max_align.max(base_layout.align);
        acc.base_offsets.push((base_name.clone(), base_offset));
        if base_layout.has_vtable {
            acc.vtable_slots = base_layout.vtable_slots.clone();
        }
        for mut bf in base_layout.fields {
            bf.offset = base_offset
                .checked_add(bf.offset)
                .ok_or_else(|| CompileError::new("object layout is too large", line, col))?;
            acc.struct_fields.push(bf);
        }
        Ok(())
    }

    /// Adds newly introduced virtual methods to the inherited slot list.
    ///
    /// A virtual destructor occupies two slots: the destructor and the
    /// deleting destructor used by `delete` on a base pointer.
    fn layout_vtable_slots(
        &mut self,
        methods: &[FunctionDecl],
        has_vtable: bool,
        vtable_slots: &mut Vec<String>,
    ) {
        if !has_vtable {
            return;
        }
        for m in methods {
            if m.name.starts_with('~') {
                let is_virt_dtor = m.is_virtual || vtable_slots.iter().any(|s| s == "~");
                if is_virt_dtor && !vtable_slots.iter().any(|s| s == "~") {
                    vtable_slots.push("~".to_string());
                    vtable_slots.push("__deleting_dtor".to_string());
                }
            } else {
                let is_override = vtable_slots.iter().any(|s| s == &m.name);
                if (m.is_virtual || is_override) && !is_override {
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
        if let Some(base_name) = base_classes.first()
            && let Some(base_layout) = self.struct_layouts.get(base_name)
        {
            for pv in &base_layout.pure_virtuals {
                pure_virtuals.insert(pv.clone());
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

    pub(super) fn dump_base_layout(&self, base_name: &str, depth: usize, out: &mut String) {
        if let Some(bl) = self.struct_layouts.get(base_name) {
            let pad = "  ".repeat(depth);
            let tag = if bl.is_class { "class" } else { "struct" };
            if bl.fields.is_empty() && !bl.has_vtable && bl.base_classes.is_empty() {
                out.push_str(&format!(
                    "{:10} | {}{} {} (base) (empty)\n",
                    0, pad, tag, base_name
                ));
            } else {
                let kind = if bl.has_vtable {
                    "(primary base)"
                } else {
                    "(base)"
                };
                out.push_str(&format!(
                    "{:10} | {}{} {} {}\n",
                    0, pad, tag, base_name, kind
                ));
                if !bl.base_classes.is_empty() {
                    self.dump_base_layout(&bl.base_classes[0], depth + 1, out);
                } else if bl.has_vtable {
                    out.push_str(&format!(
                        "{:10} | {}  ({} vtable pointer)\n",
                        0, pad, base_name
                    ));
                }
                let base_field_count = if !bl.base_classes.is_empty() {
                    self.struct_layouts
                        .get(&bl.base_classes[0])
                        .map(|b| b.fields.len())
                        .unwrap_or(0)
                } else {
                    0
                };
                for f in bl.fields.iter().skip(base_field_count) {
                    self.dump_field(f, 0, depth + 1, out);
                }
            }
        }
    }

    pub(super) fn dump_field(
        &self,
        f: &StructField,
        base_offset: usize,
        depth: usize,
        out: &mut String,
    ) {
        let pad = "  ".repeat(depth);
        if let Type::Struct(sname) = f.ty.strip_const()
            && let Some(sl) = self.struct_layouts.get(sname)
        {
            let tag = if sl.is_class { "class" } else { "struct" };
            out.push_str(&format!(
                "{:10} | {}{} {} {}\n",
                f.offset + base_offset,
                pad,
                tag,
                sname,
                f.name
            ));
            for sf in &sl.fields {
                self.dump_field(sf, f.offset + base_offset, depth + 1, out);
            }
            return;
        }
        out.push_str(&format!(
            "{:10} | {}{} {}\n",
            f.offset + base_offset,
            pad,
            f.ty.to_c_type(),
            f.name
        ));
    }

    pub fn dump_record_layouts(&self) -> String {
        let mut out = String::new();
        for name in &self.struct_order {
            if let Some(layout) = self.struct_layouts.get(name) {
                out.push_str("\n*** Dumping AST Record Layout\n");
                let tag = if layout.is_class { "class" } else { "struct" };
                if layout.fields.is_empty() && !layout.has_vtable && layout.base_classes.is_empty()
                {
                    out.push_str(&format!("{:10} | {} {} (empty)\n", 0, tag, name));
                    out.push_str(&format!(
                        "           | [sizeof={}, dsize={}, align={},\n",
                        layout.size, layout.size, layout.align
                    ));
                    out.push_str(&format!(
                        "           |  nvsize={}, nvalign={}]\n",
                        layout.size, layout.align
                    ));
                    continue;
                }
                out.push_str(&format!("{:10} | {} {}\n", 0, tag, name));
                if !layout.base_classes.is_empty() {
                    self.dump_base_layout(&layout.base_classes[0], 1, &mut out);
                } else if layout.has_vtable {
                    out.push_str(&format!("{:10} |   ({} vtable pointer)\n", 0, name));
                }
                let base_field_count = if !layout.base_classes.is_empty() {
                    self.struct_layouts
                        .get(&layout.base_classes[0])
                        .map(|bl| bl.fields.len())
                        .unwrap_or(0)
                } else {
                    0
                };
                for f in layout.fields.iter().skip(base_field_count) {
                    self.dump_field(f, 0, 1, &mut out);
                }
                out.push_str(&format!(
                    "           | [sizeof={}, dsize={}, align={},\n",
                    layout.size, layout.size, layout.align
                ));
                out.push_str(&format!(
                    "           |  nvsize={}, nvalign={}]\n",
                    layout.size, layout.align
                ));
            }
        }
        out
    }

    /// Builds the compiler-private symbol name used to distinguish overloads.
    ///
    /// This is intentionally simpler than the Itanium C++ ABI; the resulting
    /// names only need to be stable within one compilation.
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
                Type::Pointer(inner) => s.push_str(&format!("P{:?}", inner)),
                Type::Reference(inner) => s.push_str(&format!("R{:?}", inner)),
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
