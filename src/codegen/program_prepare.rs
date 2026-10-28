use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn prepare<'p>(&mut self, program: &'p Program) -> Plan<'p> {
        // Intern string literals first so their data offsets are stable.
        for decl in &program.declarations {
            self.collect_strings_decl(decl);
        }

        let address_taken = Self::collect_address_taken(program);
        self.lay_out_globals(program);
        let candidates = self.build_candidates(program, &address_taken);
        let reachable = Self::compute_reachable(&candidates);
        let all_reachable_calls = Self::collect_reachable_calls(&candidates, &reachable);

        let has_deleting_dtors = self.has_deleting_dtors();
        let has_new_or_delete = self.has_new_or_delete(&candidates, &reachable);
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
        let has_indirect_calls = self.has_indirect_calls(&candidates, &reachable);
        let has_vtables = self.has_vtables();
        // Vtables, function pointers, and indirect calls all need a funcref
        // table. Slot 0 is reserved for the pure-virtual trap.
        let need_table = has_vtables || !address_taken.is_empty() || has_indirect_calls;

        let trap_func_idx = self.register_runtime_functions(need_printf, need_alloc, need_table);
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

    fn collect_address_taken(program: &Program) -> HashSet<String> {
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
        address_taken
    }

    fn layout_global(&mut self, name: &str, ty: &Type) -> u32 {
        let offset = self.next_data_offset;
        let size = ty.size_of(&self.sema.struct_sizes) as u32;
        self.next_data_offset += (size + 3) & !3;
        let align = ty.align_of(&self.sema.struct_aligns) as u32;
        self.next_data_offset = (self.next_data_offset + align - 1) & !(align - 1);
        self.global_offsets.insert(name.to_string(), offset);
        if let Some(alias) = QualifiedName::new(name).implicit_alias() {
            self.global_offsets.insert(alias.into_owned(), offset);
        }
        offset
    }

    fn lay_out_globals(&mut self, program: &Program) {
        for decl in &program.declarations {
            if let Decl::GlobalVar { name, ty, init, .. } = decl {
                self.layout_global(name, ty);
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
                    self.layout_global(&qual_name, fty);
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
                        self.layout_global(&gname, &gty);
                        self.global_inits.push((gname, gty, ginit));
                    }
                }
            }
        }
    }

    fn build_candidates<'p>(
        &mut self,
        program: &'p Program,
        address_taken: &HashSet<String>,
    ) -> Vec<FuncCandidate<'p>> {
        let mut candidates = Vec::new();
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                candidates.push(self.build_function_candidate(f, address_taken));
            } else if let Decl::StructDecl { name, methods, .. } = decl {
                for m in methods {
                    if m.is_pure_virtual {
                        continue;
                    }
                    if let Some(candidate) = self.build_method_candidate(name, m) {
                        candidates.push(candidate);
                    }
                }
            }
        }
        candidates
    }

    /// Builds the reachability candidate of a free function.
    fn build_function_candidate<'p>(
        &mut self,
        f: &'p FunctionDecl,
        address_taken: &HashSet<String>,
    ) -> FuncCandidate<'p> {
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
            for (_, gty, ginit) in &self.global_inits {
                collect_type_lifecycle(gty, &mut calls);
                if let Some(init) = ginit {
                    collect_expr_calls(init, &mut calls);
                }
            }
        }
        FuncCandidate {
            decl: FuncDeclRef::Function(f),
            is_root,
            aliases,
            calls,
        }
    }

    /// Builds the reachability candidate of a class method.
    fn build_method_candidate<'p>(
        &mut self,
        name: &'p String,
        m: &'p FunctionDecl,
    ) -> Option<FuncCandidate<'p>> {
        let qual_name = format!("{}::{}", name, m.name);
        let mut aliases = vec![
            m.name.clone(),
            qual_name.clone(),
            format!("{}_{}", name, m.name),
        ];
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
        for (c_name, c_args) in &m.ctor_initializers {
            calls.insert(c_name.clone());
            calls.insert(format!("{}_{}", name, c_name));
            for arg in c_args {
                collect_expr_calls(arg, &mut calls);
            }
        }

        // A constructor or destructor implicitly calls the lifecycle methods
        // of its bases and members.
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

        // Virtual methods can be reached through any vtable, so they are
        // conservative roots.
        let is_vmethod = self
            .sema
            .struct_layouts
            .get(name)
            .map(|l| {
                l.vtable_slots
                    .iter()
                    .any(|s| s == &m.name || (s.starts_with('~') && m.name.starts_with('~')))
            })
            .unwrap_or(false);

        Some(FuncCandidate {
            decl: FuncDeclRef::Method(name, m),
            is_root: is_vmethod,
            aliases,
            calls,
        })
    }

    fn compute_reachable(candidates: &[FuncCandidate<'_>]) -> HashSet<usize> {
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
        reachable
    }

    fn collect_reachable_calls(
        candidates: &[FuncCandidate<'_>],
        reachable: &HashSet<usize>,
    ) -> HashSet<String> {
        let mut all_reachable_calls = HashSet::new();
        for &idx in reachable {
            all_reachable_calls.extend(candidates[idx].calls.iter().cloned());
        }
        all_reachable_calls
    }

    fn has_deleting_dtors(&self) -> bool {
        self.sema.struct_order.iter().any(|name| {
            self.sema
                .struct_layouts
                .get(name)
                .map(|l| l.vtable_slots.iter().any(|s| s == "__deleting_dtor"))
                .unwrap_or(false)
        })
    }

    fn has_new_or_delete(
        &self,
        candidates: &[FuncCandidate<'_>],
        reachable: &HashSet<usize>,
    ) -> bool {
        let mut has_new_or_delete = false;
        for &idx in reachable {
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
        has_new_or_delete
    }

    fn has_indirect_calls(
        &self,
        candidates: &[FuncCandidate<'_>],
        reachable: &HashSet<usize>,
    ) -> bool {
        let mut has_indirect_calls = false;
        for &idx in reachable {
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
        has_indirect_calls
    }

    fn has_vtables(&self) -> bool {
        self.sema.struct_order.iter().any(|name| {
            self.sema
                .struct_layouts
                .get(name)
                .map(|l| l.has_vtable)
                .unwrap_or(false)
        })
    }

    fn register_runtime_functions(
        &mut self,
        need_printf: bool,
        need_alloc: bool,
        need_table: bool,
    ) -> Option<u32> {
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

        if need_table {
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
        }
    }
}
