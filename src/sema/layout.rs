use super::*;

impl Sema {
    /// Computes field offsets, base-subobject offsets, and vtable slot order.
    ///
    /// Only the primary base may be polymorphic; a polymorphic secondary base
    /// is rejected because this backend does not adjust virtual dispatch
    /// through secondary subobjects.
    pub(super) fn compute_struct_layout(
        &mut self,
        name: &str,
        is_class: bool,
        base_classes: &[String],
        fields: &[(String, Type)],
        methods: &[FunctionDecl],
        private_members: &[String],
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let mut offset = 0;
        let mut max_align = 1;
        let mut struct_fields = Vec::new();
        let mut base_offsets = Vec::new();
        let mut vtable_slots = Vec::new();
        let has_virtual_methods = methods.iter().any(|m| m.is_virtual);
        let primary_has_vtable = base_classes
            .first()
            .and_then(|base| self.struct_layouts.get(base))
            .map(|layout| layout.has_vtable)
            .unwrap_or(false);
        let has_vtable = has_virtual_methods || primary_has_vtable;
        if has_virtual_methods && !primary_has_vtable {
            // A vtable pointer precedes the primary base subobject.
            offset = 4;
            max_align = 4;
        }

        self.layout_base_subobjects(
            base_classes,
            &mut offset,
            &mut max_align,
            &mut struct_fields,
            &mut base_offsets,
            &mut vtable_slots,
            line,
            col,
        )?;

        self.layout_vtable_slots(methods, has_vtable, &mut vtable_slots);

        self.layout_own_fields(fields, &mut offset, &mut max_align, &mut struct_fields);

        if max_align > 0 && offset % max_align != 0 {
            offset += max_align - (offset % max_align);
        }
        if offset == 0 {
            offset = 1;
        }

        if max_align > 0 && offset % max_align != 0 {
            offset += max_align - (offset % max_align);
        }
        if offset == 0 {
            offset = 1;
        }

        let pure_virtuals = self.collect_pure_virtuals(base_classes, methods);
        let is_abstract = !pure_virtuals.is_empty();

        let layout = StructLayout {
            is_class,
            base_classes: base_classes.to_vec(),
            base_offsets,
            fields: struct_fields,
            size: offset,
            align: max_align,
            has_vtable,
            vtable_slots,
            private_members: private_members.iter().cloned().collect(),
            is_abstract,
            pure_virtuals,
        };

        self.record_struct_layout(name, layout, offset, max_align);

        Ok(())
    }

    /// Lays out this record's own fields after all base subobjects.
    fn layout_own_fields(
        &mut self,
        fields: &[(String, Type)],
        offset: &mut usize,
        max_align: &mut usize,
        struct_fields: &mut Vec<StructField>,
    ) {
        // 3. Lay out this record's own fields after all base subobjects.
        for (fname, ftype) in fields {
            let fsize = ftype.size_of(&self.struct_sizes);
            let falign = ftype.align_of(&self.struct_aligns);
            *max_align = (*max_align).max(falign);

            if falign > 0 && !(*offset).is_multiple_of(falign) {
                *offset += falign - (*offset % falign);
            }

            struct_fields.push(StructField {
                name: fname.clone(),
                ty: ftype.clone(),
                offset: *offset,
            });

            *offset += fsize;
        }
    }

    /// Lays out every base subobject in declaration order. Only the primary
    /// base contributes the vtable; non-virtual multiple bases are laid out
    /// consecutively so inherited fields keep distinct offsets.
    #[allow(clippy::too_many_arguments)]
    fn layout_base_subobjects(
        &mut self,
        base_classes: &[String],
        offset: &mut usize,
        max_align: &mut usize,
        struct_fields: &mut Vec<StructField>,
        base_offsets: &mut Vec<(String, usize)>,
        vtable_slots: &mut Vec<String>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        for (index, base_name) in base_classes.iter().enumerate() {
            let Some(base_layout) = self.struct_layouts.get(base_name).cloned() else {
                continue;
            };
            if index > 0 && base_layout.has_vtable {
                return Err(CompileError::new(
                    "multiple inheritance with a polymorphic secondary base is not supported",
                    line,
                    col,
                ));
            }
            let base_offset = if index == 0
                && *offset == 0
                && base_layout.fields.is_empty()
                && !base_layout.has_vtable
            {
                0
            } else {
                let align = base_layout.align.max(1);
                if !(*offset).is_multiple_of(align) {
                    *offset += align - (*offset % align);
                }
                let base_offset = *offset;
                *offset = offset
                    .checked_add(base_layout.size)
                    .ok_or_else(|| CompileError::new("object layout is too large", 1, 1))?;
                base_offset
            };
            *max_align = (*max_align).max(base_layout.align);
            base_offsets.push((base_name.clone(), base_offset));
            if index == 0 && base_layout.has_vtable {
                *vtable_slots = base_layout.vtable_slots.clone();
            }
            for mut bf in base_layout.fields {
                bf.offset = base_offset
                    .checked_add(bf.offset)
                    .ok_or_else(|| CompileError::new("object layout is too large", 1, 1))?;
                struct_fields.push(bf);
            }
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
