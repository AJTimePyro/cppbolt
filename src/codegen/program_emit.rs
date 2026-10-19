use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn emit(&mut self, plan: &Plan) -> Result<(), CompileError> {
        for (idx, cand) in plan.candidates.iter().enumerate() {
            if !plan.reachable.contains(&idx) {
                continue;
            }
            match &cand.decl {
                FuncDeclRef::Function(f) => {
                    if f.body.is_none() {
                        continue;
                    }
                    let mangled = self.sema.mangle_name(&f.name, &f.params);
                    let params_wasm: Vec<u8> = f
                        .params
                        .iter()
                        .map(|p| {
                            p.ty.to_wasm_type()
                                .map(|w| match w {
                                    WasmType::I32 => TYPE_I32,
                                    WasmType::I64 => TYPE_I64,
                                    WasmType::F32 => TYPE_F32,
                                    WasmType::F64 => TYPE_F64,
                                })
                                .unwrap_or(TYPE_I32)
                        })
                        .collect();

                    let results_wasm: Vec<u8> = f
                        .return_type
                        .to_wasm_type()
                        .map(|w| match w {
                            WasmType::I32 => vec![TYPE_I32],
                            WasmType::I64 => vec![TYPE_I64],
                            WasmType::F32 => vec![TYPE_F32],
                            WasmType::F64 => vec![TYPE_F64],
                        })
                        .unwrap_or_default();

                    let type_idx = self.module.add_type(params_wasm, results_wasm);
                    let func_idx = (self.module.imports.len() + self.module.functions.len()) as u32;
                    self.module.functions.push(type_idx);
                    self.func_indices.insert(
                        QualifiedName::new(mangled.as_str())
                            .alternate()
                            .into_owned(),
                        func_idx,
                    );
                    self.func_indices.insert(
                        QualifiedName::new(f.name.as_str()).alternate().into_owned(),
                        func_idx,
                    );
                    self.func_indices.insert(mangled.clone(), func_idx);
                    self.func_indices.insert(f.name.clone(), func_idx);
                    if f.name == "main" {
                        // Export main
                        self.module.exports.push(("main".to_string(), 0, func_idx));
                    }
                }
                FuncDeclRef::Method(name, m) => {
                    let mut params = m.params.clone();
                    if !m.is_static {
                        let this_inner = if m.is_const {
                            Type::Const(Box::new(Type::Struct(name.to_string())))
                        } else {
                            Type::Struct(name.to_string())
                        };
                        params.insert(
                            0,
                            Param {
                                name: "this".to_string(),
                                ty: Type::Pointer(Box::new(this_inner)),
                                default_val: None,
                            },
                        );
                    }
                    let qual_name = format!("{}::{}", name, m.name);
                    let mangled = self.sema.mangle_name(&qual_name, &params);
                    let params_wasm: Vec<u8> = params
                        .iter()
                        .map(|p| {
                            p.ty.to_wasm_type()
                                .map(|w| match w {
                                    WasmType::I32 => TYPE_I32,
                                    WasmType::I64 => TYPE_I64,
                                    WasmType::F32 => TYPE_F32,
                                    WasmType::F64 => TYPE_F64,
                                })
                                .unwrap_or(TYPE_I32)
                        })
                        .collect();

                    let results_wasm: Vec<u8> = m
                        .return_type
                        .to_wasm_type()
                        .map(|w| match w {
                            WasmType::I32 => vec![TYPE_I32],
                            WasmType::I64 => vec![TYPE_I64],
                            WasmType::F32 => vec![TYPE_F32],
                            WasmType::F64 => vec![TYPE_F64],
                        })
                        .unwrap_or_default();

                    let type_idx = self.module.add_type(params_wasm, results_wasm);
                    let func_idx = (self.module.imports.len() + self.module.functions.len()) as u32;
                    self.module.functions.push(type_idx);

                    self.func_indices.insert(
                        QualifiedName::new(mangled.as_str())
                            .alternate()
                            .into_owned(),
                        func_idx,
                    );
                    self.func_indices.insert(
                        QualifiedName::new(qual_name.as_str())
                            .alternate()
                            .into_owned(),
                        func_idx,
                    );
                    self.func_indices.insert(mangled, func_idx);
                    self.func_indices.insert(qual_name, func_idx);
                    self.func_indices
                        .insert(format!("{}_{}", name, m.name), func_idx);
                    if m.name == *name {
                        self.func_indices
                            .insert(format!("{}_{}", name, name), func_idx);
                    }
                    if m.name == format!("~{}", name) {
                        self.func_indices
                            .insert(format!("{}_~{}", name, name), func_idx);
                    }
                }
            }
        }

        // Register deleting destructor functions
        let mut deleting_dtor_structs = Vec::new();
        for name in &self.sema.struct_order {
            if let Some(layout) = self.sema.struct_layouts.get(name)
                && layout.vtable_slots.iter().any(|s| s == "__deleting_dtor")
            {
                let type_idx = self.t_i32_v;
                let func_idx = (self.module.imports.len() + self.module.functions.len()) as u32;
                self.module.functions.push(type_idx);
                self.func_indices
                    .insert(format!("{}_deleting_dtor", name), func_idx);
                deleting_dtor_structs.push(name.clone());
            }
        }

        // Build Vtables: layout slots and store into data section
        for name in &self.sema.struct_order {
            if let Some(layout) = self.sema.struct_layouts.get(name)
                && layout.has_vtable
            {
                let vtable_addr = self.next_data_offset;
                self.vtable_offsets.insert(name.clone(), vtable_addr);

                let mut vtable_bytes = Vec::new();
                for slot_name in &layout.vtable_slots {
                    let trap_idx = plan.trap_func_idx.unwrap_or(0);
                    let func_idx = if slot_name == "__deleting_dtor" {
                        self.func_indices
                            .get(&format!("{}_deleting_dtor", name))
                            .copied()
                            .unwrap_or(trap_idx)
                    } else if slot_name.starts_with('~') {
                        let d_name = format!("{}_{}", name, slot_name);
                        self.func_indices.get(&d_name).copied().unwrap_or(trap_idx)
                    } else if let Some(idx) = self.find_class_vmethod(name, slot_name) {
                        idx
                    } else {
                        trap_idx
                    };

                    let table_elem_idx = self.vtable_func_indices.len() as u32;
                    self.vtable_func_indices.push(func_idx);

                    vtable_bytes.extend_from_slice(&table_elem_idx.to_le_bytes());
                }

                self.module.data.push((vtable_addr, vtable_bytes));
                self.next_data_offset += (layout.vtable_slots.len() * 4) as u32;
            }
        }

        // Assign table entries to address-taken functions
        for (idx, cand) in plan.candidates.iter().enumerate() {
            if !plan.reachable.contains(&idx) {
                continue;
            }
            if cand.aliases.iter().any(|a| plan.address_taken.contains(a))
                && let Some(&func_idx) = cand.aliases.iter().find_map(|a| self.func_indices.get(a))
            {
                let table_elem_idx = self.vtable_func_indices.len() as u32;
                self.vtable_func_indices.push(func_idx);
                for a in &cand.aliases {
                    self.func_table_indices.insert(a.clone(), table_elem_idx);
                }
            }
        }

        if !self.vtable_func_indices.is_empty() {
            let count = self.vtable_func_indices.len() as u32;
            self.module.table = Some((count, Some(count)));
            self.module
                .elements
                .push((0, self.vtable_func_indices.clone()));
        }

        // Emit runtime function bodies in exact registration order
        if plan.need_printf {
            self.module.codes.push(crate::runtime::emit_printf(0, 1, 2));
        }
        if plan.need_alloc {
            self.module.codes.push(crate::runtime::emit_malloc(1, 2));
            self.module.codes.push(crate::runtime::emit_free(2));
        }
        if plan.need_table {
            self.module
                .codes
                .push(crate::runtime::emit_pure_virtual_trap());
        }

        // Compile bodies ONLY for plan.reachable functions
        for (idx, cand) in plan.candidates.iter().enumerate() {
            if !plan.reachable.contains(&idx) {
                continue;
            }
            match &cand.decl {
                FuncDeclRef::Function(f) => {
                    if f.body.is_none() {
                        continue;
                    }
                    if let Some(body) = &f.body {
                        let code = self.compile_function(f, body, None)?;
                        self.module.codes.push(code);
                    }
                }
                FuncDeclRef::Method(name, m) => {
                    if let Some(body) = &m.body {
                        let mut func_decl = (*m).clone();
                        if !m.is_static {
                            let this_inner = if m.is_const {
                                Type::Const(Box::new(Type::Struct(name.to_string())))
                            } else {
                                Type::Struct(name.to_string())
                            };
                            func_decl.params.insert(
                                0,
                                Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer(Box::new(this_inner)),
                                    default_val: None,
                                },
                            );
                        }
                        let code =
                            self.compile_function(&func_decl, body, Some(name.to_string()))?;
                        self.module.codes.push(code);
                    }
                }
            }
        }

        // Compile deleting destructors
        for name in &deleting_dtor_structs {
            let mut code = Vec::new();
            let dtor_name = format!("{}_~{}", name, name);
            if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, 0); // this
                code.push(OP_CALL);
                write_u32(&mut code, dtor_idx);
            }
            code.push(OP_LOCAL_GET);
            write_u32(&mut code, 0); // this
            code.push(OP_CALL);
            write_u32(&mut code, self.func_indices["free"]);
            code.push(OP_END);
            self.module.codes.push((vec![], code));
        }

        // Store strings into data section
        let mut string_entries: Vec<(&String, &u32)> = self.string_table.iter().collect();
        string_entries.sort_by_key(|&(_, offset)| *offset);
        for (s, &offset) in string_entries {
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            self.module.data.push((offset, bytes));
        }

        // Global 0: Stack pointer (starts at 8MB + 64KB = 8454144)
        self.module
            .globals
            .push((TYPE_I32, true, memory_layout::DATA_BASE as i32));
        self.module
            .exports
            .push(("__stack_pointer".to_string(), 3, 0));

        // Global 1: Heap pointer (starts at next_data_offset aligned to 16)
        let heap_start = (self.next_data_offset + 15) & !15;
        self.module
            .globals
            .push((TYPE_I32, true, heap_start as i32));
        self.module.exports.push(("__heap_base".to_string(), 3, 1));

        // Global 2: Free list head pointer (starts at 0)
        self.module.globals.push((TYPE_I32, true, 0));

        // Memory export: calculate needed pages covering stack, data, BSS
        let needed_pages =
            memory_layout::pages_for(heap_start).max(memory_layout::DEFAULT_MEMORY_PAGES);
        // WebAssembly32 memories cannot exceed 65536 pages. Clamp user input
        // before it can overflow the page calculation.
        let requested_pages = self.max_memory_mb.saturating_mul(16).min(65_536);
        let max_pages = requested_pages.max(needed_pages).min(65_536);
        self.module.memory_pages = (needed_pages, Some(max_pages));
        self.module.exports.push(("memory".to_string(), 2, 0));

        Ok(())
    }
}
