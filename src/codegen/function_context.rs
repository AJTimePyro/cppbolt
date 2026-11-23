use super::*;

#[derive(Clone, Debug)]
pub struct CleanupEntry {
    pub index: usize,
    pub _sname: String,
    pub dtor_idx: u32,
    pub offset: u32,
    pub is_subobject: bool,
    pub is_temp: bool,
}

impl FunctionContext {
    pub fn register_cleanup(
        &mut self,
        _sname: String,
        dtor_idx: u32,
        offset: u32,
        is_subobject: bool,
    ) -> Result<usize, CompileError> {
        let index = self.cleanup_entries.len();
        if index >= 64 {
            return Err(CompileError::new(
                "function exceeds maximum of 64 cleanup objects for exception unwinding",
                self.line,
                self.col,
            ));
        }
        self.cleanup_entries.push(CleanupEntry {
            index,
            _sname,
            dtor_idx,
            offset,
            is_subobject,
            is_temp: false,
        });
        Ok(index)
    }

    pub fn register_temp_cleanup(
        &mut self,
        _sname: String,
        dtor_idx: u32,
        temp_slot: u32,
    ) -> Option<usize> {
        let index = self.cleanup_entries.len();
        if index >= 64 {
            return None;
        }
        self.cleanup_entries.push(CleanupEntry {
            index,
            _sname,
            dtor_idx,
            offset: temp_slot,
            is_subobject: false,
            is_temp: true,
        });
        Some(index)
    }

    pub fn current_exnref_local(&self) -> u32 {
        self.exnref_local + self.catch_depth.saturating_sub(1) as u32
    }
}

pub(super) fn max_catch_depth(stmt: &Stmt, current: u32) -> u32 {
    let mut max_d = current;
    match &stmt.kind {
        StmtKind::Try {
            try_body,
            catches,
        } => {
            let in_try = max_catch_depth(try_body, current);
            max_d = max_d.max(in_try);
            for clause in catches {
                let in_catch = max_catch_depth(&clause.body, current + 1);
                max_d = max_d.max(in_catch);
            }
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                max_d = max_d.max(max_catch_depth(s, current));
            }
        }
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (_, body) in branches {
                max_d = max_d.max(max_catch_depth(body, current));
            }
            if let Some(else_s) = else_stmt {
                max_d = max_d.max(max_catch_depth(else_s, current));
            }
        }
        StmtKind::While { body, .. } | StmtKind::DoWhile { body, .. } => {
            max_d = max_d.max(max_catch_depth(body, current));
        }
        StmtKind::For { init, body, .. } => {
            if let Some(init_s) = init {
                max_d = max_d.max(max_catch_depth(init_s, current));
            }
            max_d = max_d.max(max_catch_depth(body, current));
        }
        StmtKind::Switch { body, .. } => {
            max_d = max_d.max(max_catch_depth(body, current));
        }
        _ => {}
    }
    max_d
}

impl<'a> Codegen<'a> {
    pub(super) fn function_context(
        f: &FunctionDecl,
        class_name: Option<&str>,
        max_depth: u32,
    ) -> (Vec<(u32, u8)>, FunctionContext) {
        // Non-static methods carry the implicit `this` pointer as parameter 0.
        let num_params = f.params.len() as u32;
        let fp_local = num_params;
        let scratch1 = num_params + 1;
        let scratch2 = num_params + 2;
        let ret_temp_i32 = num_params + 3;
        let last_exn_val_local = num_params + 4;
        let scratch_i64 = num_params + 5;
        let ret_temp_i64 = num_params + 6;
        let cleanup_mask_local = num_params + 7;
        let scratch_f64 = num_params + 8;
        let ret_temp_f64 = num_params + 9;
        let scratch_f32 = num_params + 10;
        let exnref_local = num_params + 11;
        let num_exnrefs = (max_depth + 1).max(1);
        let unwind_exnref_local = exnref_local + max_depth;

        let locals = vec![
            (5, TYPE_I32), // fp_local, scratch1, scratch2, ret_temp_i32, last_exn_val_local
            (3, TYPE_I64), // scratch_i64, ret_temp_i64, cleanup_mask_local
            (2, TYPE_F64), // scratch_f64, ret_temp_f64
            (1, TYPE_F32), // scratch_f32
            (num_exnrefs, TYPE_EXNREF), // exnref_local (all nesting levels + unwind)
        ];

        let is_dtor = if let Some(cls) = class_name {
            f.name == format!("~{}", cls)
        } else {
            false
        };

        let ctx = FunctionContext {
            fp_local,
            scratch1,
            scratch2,
            ret_temp_i32,
            last_exn_val_local,
            scratch_i64,
            ret_temp_i64,
            cleanup_mask_local,
            scratch_f64,
            ret_temp_f64,
            scratch_f32,
            exnref_local,
            unwind_exnref_local,
            max_catch_depth: max_depth,
            cleanup_entries: Vec::new(),
            active_exception_slots: Vec::new(),
            in_catch_all: false,
            catch_depth: 0,
            try_depth: 0,
            var_offsets: HashMap::new(),
            var_try_depths: HashMap::new(),
            var_types: HashMap::new(),
            static_var_offsets: HashMap::new(),
            next_local_offset: 16,
            frame_size: 0,
            frame_size_patches: Vec::new(),
            var_scopes: vec![Vec::new()],
            scope_objects: vec![Vec::new()],
            temp_objects: Vec::new(),
            initializer_list_temps: Vec::new(),
            current_class: class_name.map(str::to_string),
            current_func_name: f.name.clone(),
            line: f.line,
            col: f.col,
            return_type: f.return_type.clone(),
            is_dtor,
            is_reference_return: f.return_type.is_reference(),
            is_main: f.name == "main",
        };
        (locals, ctx)
    }
}
