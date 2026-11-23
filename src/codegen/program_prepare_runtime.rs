//! Runtime support function registration and reachability preparation.

use super::*;
use crate::runtime::opcodes::*;
use std::collections::HashSet;

impl<'a> Codegen<'a> {
    fn collect_reachable_exception_classes(&self) -> HashSet<String> {
        let mut classes = HashSet::new();
        for sname in self.sema.thrown_classes.borrow().iter() {
            classes.insert(sname.clone());
        }
        for sname in self.sema.caught_classes.borrow().iter() {
            classes.insert(sname.clone());
        }
        let initial_classes: Vec<String> = classes.iter().cloned().collect();
        for sname in initial_classes {
            if let Some(layout) = self.sema.struct_layouts.get(&sname) {
                for (base, _) in &layout.base_offsets {
                    classes.insert(base.clone());
                }
            }
        }
        classes
    }

    pub(super) fn mark_exception_destructors_reachable(
        &self,
        candidates: &[FuncCandidate<'_>],
        reachable: &mut HashSet<usize>,
    ) -> bool {
        let has_exceptions = reachable.iter().any(|&idx| {
            let body_opt = match &candidates[idx].decl {
                FuncDeclRef::Function(f) => f.body.as_ref(),
                FuncDeclRef::Method(_, m) => m.body.as_ref(),
            };
            body_opt.map(check_stmt_has_try_throw).unwrap_or(false)
        });
        if !has_exceptions {
            return false;
        }

        let target_classes = self.collect_reachable_exception_classes();
        let mut worklist = Vec::new();
        for (idx, c) in candidates.iter().enumerate() {
            if let FuncDeclRef::Method(name, m) = c.decl {
                let short_name = QualifiedName::new(name).class_short_name();
                let matches_target =
                    target_classes.contains(name) || target_classes.contains(short_name);
                if matches_target
                    && (m.name == format!("~{name}") || m.name == format!("~{short_name}"))
                    && reachable.insert(idx)
                {
                    worklist.push(idx);
                }
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
                            let plain = QualifiedName::new(call_target).plain();
                            plain.starts_with("operator") && a.contains(plain)
                        })
                    {
                        reachable.insert(t_idx);
                        worklist.push(t_idx);
                    }
                }
            }
        }
        true
    }

    pub(super) fn register_runtime_functions(
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

        let trap_ret = if need_table {
            let func_trap = (self.module.imports.len() + self.module.functions.len()) as u32;
            let t_trap = self.module.add_type(vec![], vec![]);
            self.module.functions.push(t_trap);
            self.func_indices
                .insert("__cxa_pure_virtual".to_string(), func_trap);
            self.vtable_func_indices = vec![func_trap];
            Some(func_trap)
        } else {
            None
        };

        let func_can_catch = (self.module.imports.len() + self.module.functions.len()) as u32;
        let t_can_catch = self
            .module
            .add_type(vec![TYPE_I32, TYPE_I32], vec![TYPE_I32]);
        self.module.functions.push(t_can_catch);
        self.func_indices
            .insert("__cppbolt_can_catch".to_string(), func_can_catch);

        let func_terminate = (self.module.imports.len() + self.module.functions.len()) as u32;
        let t_terminate = self.module.add_type(vec![], vec![]);
        self.module.functions.push(t_terminate);
        self.func_indices
            .insert("__cppbolt_terminate".to_string(), func_terminate);
        trap_ret
    }

    pub(super) fn emit_uncaught_count_change(&self, code: &mut Vec<u8>, delta: i32) {
        code.push(OP_GLOBAL_GET);
        write_u32(code, UNCAUGHT_COUNT_GLOBAL);
        code.push(OP_I32_CONST);
        write_i32(code, delta);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, UNCAUGHT_COUNT_GLOBAL);
    }

    pub(super) fn emit_terminate_call(&self, code: &mut Vec<u8>) {
        code.push(OP_CALL);
        write_u32(code, self.func_indices["__cppbolt_terminate"]);
    }

    pub(super) fn emit_terminate_function(&mut self) {
        let mut code = Vec::new();
        if let Some(terminate_idx) = ["std::terminate", "terminate"]
            .iter()
            .find_map(|name| self.func_indices.get(*name).copied())
        {
            code.push(OP_CALL);
            write_u32(&mut code, terminate_idx);
        } else {
            // Without <exception>, std::terminate is not compiled; abort() is exit(134).
            code.push(OP_I32_CONST);
            write_i32(&mut code, 134);
            code.push(OP_CALL);
            write_u32(&mut code, self.func_indices["exit"]);
        }
        code.push(OP_END);
        self.module.codes.push((vec![], code));
    }
}
