use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn prepare<'p>(&mut self, program: &'p Program) -> Plan<'p> {
        // Intern string literals first so their data offsets are stable.
        for decl in &program.declarations {
            self.collect_strings_decl(decl);
        }

        // Functions whose address escapes are roots for dead-code elimination
        // and need entries in the indirect-call table.
        let mut address_taken = HashSet::new();
        for decl in &program.declarations {
            match decl {
                Decl::Function(f) => {
                    if let Some(body) = &f.body {
                        collect_address_taken_stmt(body, &mut address_taken);
                    }
                }
                Decl::StructDecl { methods, .. } => {
                    for m in methods {
                        if let Some(body) = &m.body {
                            collect_address_taken_stmt(body, &mut address_taken);
                        }
                    }
                }
                Decl::GlobalVar { init: Some(e), .. } => {
                    collect_address_taken_expr(e, &mut address_taken);
                }
                _ => {}
            }
        }

        // Lay out globals, static fields, and function-local statics in the
        // data segment. Function-local statics use source locations in their
        // storage names to keep same-named statics distinct.
        for decl in &program.declarations {
            if let Decl::GlobalVar { name, ty, init, .. } = decl {
                let offset = self.next_data_offset;
                let size = ty.size_of(&self.sema.struct_sizes) as u32;
                self.next_data_offset += (size + 3) & !3;
                let align = ty.align_of(&self.sema.struct_aligns) as u32;
                self.next_data_offset = (self.next_data_offset + align - 1) & !(align - 1);
                self.global_offsets.insert(name.clone(), offset);
                if let Some(alias) = QualifiedName::new(name).implicit_alias() {
                    self.global_offsets.insert(alias.into_owned(), offset);
                }
                self.global_inits
                    .push((name.clone(), ty.clone(), init.clone()));
            } else if let Decl::StructDecl {
                name,
                static_fields,
                ..
            } = decl
            {
                for (fname, fty, finit) in static_fields {
                    let qual_name = format!("{}::{}", name, fname);
                    let offset = self.next_data_offset;
                    let size = fty.size_of(&self.sema.struct_sizes) as u32;
                    self.next_data_offset += (size + 3) & !3;
                    let align = fty.align_of(&self.sema.struct_aligns) as u32;
                    self.next_data_offset = (self.next_data_offset + align - 1) & !(align - 1);
                    self.global_offsets.insert(qual_name.clone(), offset);
                    if let Some(alias) = QualifiedName::new(&qual_name).implicit_alias() {
                        self.global_offsets.insert(alias.into_owned(), offset);
                    }
                    self.global_inits
                        .push((qual_name, fty.clone(), finit.clone()));
                }
            } else if let Decl::Function(f) = decl
                && let Some(body) = &f.body
            {
                let mut statics = Vec::new();
                collect_static_vars(body, &f.name, &mut statics);
                for (gname, gty, ginit) in statics {
                    if !self.global_offsets.contains_key(&gname) {
                        let offset = self.next_data_offset;
                        let size = gty.size_of(&self.sema.struct_sizes) as u32;
                        self.next_data_offset += (size + 3) & !3;
                        let align = gty.align_of(&self.sema.struct_aligns) as u32;
                        self.next_data_offset = (self.next_data_offset + align - 1) & !(align - 1);
                        self.global_offsets.insert(gname.clone(), offset);
                        self.global_inits.push((gname, gty, ginit));
                    }
                }
            }
        }

        // Reachability analysis: `main`, address-taken functions, and virtual
        // methods are roots; calls discovered while walking reachable bodies
        // extend the set to a fixed point.
        let mut candidates = Vec::new();
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                let mangled = self.sema.mangle_name(&f.name, &f.params);
                let aliases = vec![
                    f.name.clone(),
                    QualifiedName::new(&f.name).alternate().into_owned(),
                    mangled.clone(),
                    QualifiedName::new(&mangled).alternate().into_owned(),
                ];
                let mut calls = HashSet::new();
                if let Some(body) = &f.body {
                    collect_stmt_calls(body, &mut calls);
                }
                let is_address_taken = aliases.iter().any(|a| address_taken.contains(a));
                let is_root = f.name == "main" || is_address_taken;
                if f.name == "main" {
                    for (_, gty, _) in &self.global_inits {
                        collect_type_lifecycle(gty, &mut calls);
                    }
                }
                candidates.push(FuncCandidate {
                    decl: FuncDeclRef::Function(f),
                    is_root,
                    aliases,
                    calls,
                });
            } else if let Decl::StructDecl { name, methods, .. } = decl {
                for m in methods {
                    if !m.is_pure_virtual {
                        let mut aliases = Vec::new();
                        aliases.push(m.name.clone());
                        let qual_name = format!("{}::{}", name, m.name);
                        aliases.push(qual_name.clone());
                        aliases.push(format!("{}_{}", name, m.name));
                        let mut calls = HashSet::new();
                        if m.name == *name {
                            aliases.push(format!("{}_{}", name, name));
                            aliases.push(format!("{}::{}", name, name));
                            calls.insert(format!("{}_~{}", name, name));
                            calls.insert(format!("{}::~{}", name, name));
                        }
                        if m.name == format!("~{}", name) {
                            aliases.push(format!("{}_~{}", name, name));
                            aliases.push(format!("{}::~{}", name, name));
                        }
                        let mut params = m.params.clone();
                        if !m.is_static {
                            params.insert(
                                0,
                                Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer(Box::new(Type::Struct(name.clone()))),
                                    default_val: None,
                                },
                            );
                        }
                        let mangled = self.sema.mangle_name(&qual_name, &params);
                        aliases.push(mangled);
                        if let Some(body) = &m.body {
                            collect_stmt_calls(body, &mut calls);
                        }

                        // A constructor or destructor implicitly calls the
                        // lifecycle methods of its bases and members.
                        if let Some(layout) = self.sema.struct_layouts.get(name)
                            && (m.name == *name || m.name == format!("~{}", name))
                        {
                            for b in &layout.base_classes {
                                collect_type_lifecycle(&Type::Struct(b.clone()), &mut calls);
                            }
                            for fld in &layout.fields {
                                collect_type_lifecycle(&fld.ty, &mut calls);
                            }
                        }

                        // Virtual methods can be reached through any vtable,
                        // so they are conservative roots.
                        let is_vmethod = self
                            .sema
                            .struct_layouts
                            .get(name)
                            .map(|l| {
                                l.vtable_slots.iter().any(|s| {
                                    s == &m.name || (s.starts_with('~') && m.name.starts_with('~'))
                                })
                            })
                            .unwrap_or(false);

                        candidates.push(FuncCandidate {
                            decl: FuncDeclRef::Method(name, m),
                            is_root: is_vmethod,
                            aliases,
                            calls,
                        });
                    }
                }
            }
        }

        let mut reachable = HashSet::new();
        let mut worklist = Vec::new();
        for (idx, c) in candidates.iter().enumerate() {
            if c.is_root {
                reachable.insert(idx);
                worklist.push(idx);
            }
        }
        while let Some(idx) = worklist.pop() {
            for call_target in &candidates[idx].calls {
                for (t_idx, t_cand) in candidates.iter().enumerate() {
                    if !reachable.contains(&t_idx)
                        && t_cand.aliases.iter().any(|a| {
                            if QualifiedName::new(a).matches(QualifiedName::new(call_target)) {
                                return true;
                            }
                            if call_target.starts_with("operator") && a.contains(call_target) {
                                return true;
                            }
                            false
                        })
                    {
                        reachable.insert(t_idx);
                        worklist.push(t_idx);
                    }
                }
            }
        }

        // Union the call sets of reachable bodies. These names decide which
        // runtime helpers must exist in the module.
        let mut all_reachable_calls = HashSet::new();
        for &idx in &reachable {
            all_reachable_calls.extend(candidates[idx].calls.iter().cloned());
        }

        let has_deleting_dtors = self.sema.struct_order.iter().any(|name| {
            self.sema
                .struct_layouts
                .get(name)
                .map(|l| l.vtable_slots.iter().any(|s| s == "__deleting_dtor"))
                .unwrap_or(false)
        });
        let mut has_new_or_delete = false;
        for &idx in &reachable {
            let body_opt = match &candidates[idx].decl {
                FuncDeclRef::Function(f) => f.body.as_ref(),
                FuncDeclRef::Method(_, m) => m.body.as_ref(),
            };
            if let Some(b) = body_opt
                && check_stmt_has_new_delete(b)
            {
                has_new_or_delete = true;
                break;
            }
        }

        let need_printf = all_reachable_calls.contains("printf")
            || all_reachable_calls.contains("std::printf")
            || all_reachable_calls.iter().any(|c| c.ends_with("::printf"));
        // Keep the allocator whenever a record exists: synthesized constructors,
        // destructors, and prvalue returns may allocate even without a visible
        // `new`, `malloc`, or `free` call.
        let need_alloc = all_reachable_calls.contains("malloc")
            || all_reachable_calls.contains("free")
            || has_deleting_dtors
            || has_new_or_delete
            || !self.sema.struct_layouts.is_empty();
        let mut has_indirect_calls = false;
        for &idx in &reachable {
            let body_opt = match &candidates[idx].decl {
                FuncDeclRef::Function(f) => f.body.as_ref(),
                FuncDeclRef::Method(_, m) => m.body.as_ref(),
            };
            if let Some(b) = body_opt
                && self.check_stmt_has_indirect_call(b)
            {
                has_indirect_calls = true;
                break;
            }
        }
        let has_vtables = self.sema.struct_order.iter().any(|name| {
            self.sema
                .struct_layouts
                .get(name)
                .map(|l| l.has_vtable)
                .unwrap_or(false)
        });
        // Vtables, function pointers, and indirect calls all need a funcref
        // table. Slot 0 is reserved for the pure-virtual trap.
        let need_table = has_vtables || !address_taken.is_empty() || has_indirect_calls;

        if need_printf {
            let func_idx = (self.module.imports.len() + self.module.functions.len()) as u32;
            let t_printf = self
                .module
                .add_type(vec![TYPE_I32, TYPE_I32], vec![TYPE_I32]);
            self.module.functions.push(t_printf);
            self.func_indices.insert("printf".to_string(), func_idx);
            self.func_indices
                .insert("std::printf".to_string(), func_idx);
            self.func_printf = func_idx;
        }

        if need_alloc {
            let func_malloc = (self.module.imports.len() + self.module.functions.len()) as u32;
            let t_malloc = self.module.add_type(vec![TYPE_I32], vec![TYPE_I32]);
            self.module.functions.push(t_malloc);
            self.func_indices.insert("malloc".to_string(), func_malloc);

            let func_free = (self.module.imports.len() + self.module.functions.len()) as u32;
            let t_free = self.module.add_type(vec![TYPE_I32], vec![]);
            self.module.functions.push(t_free);
            self.func_indices.insert("free".to_string(), func_free);
        }

        let trap_func_idx = if need_table {
            let func_trap = (self.module.imports.len() + self.module.functions.len()) as u32;
            let t_trap = self.module.add_type(vec![], vec![]);
            self.module.functions.push(t_trap);
            self.func_indices
                .insert("__cxa_pure_virtual".to_string(), func_trap);
            // Table slot 0 points at the trap so an unfilled vtable slot is a
            // deterministic dynamic error.
            self.vtable_func_indices = vec![func_trap];
            Some(func_trap)
        } else {
            None
        };
        Plan {
            candidates,
            reachable,
            address_taken,
            need_printf,
            need_alloc,
            need_table,
            trap_func_idx,
        }
    }
}
