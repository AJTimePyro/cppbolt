import re

with open("src/codegen.rs", "r") as f:
    code = f.read()

# Replace FunctionContext definition
old_ctx = """pub struct FunctionContext {
    pub fp_local: u32,
    pub scratch1: u32,
    pub scratch2: u32,
    pub scratch_i64: u32,
    pub scratch_f64: u32,
    pub scratch_f32: u32,
    pub ret_temp_i32: u32,
    pub ret_temp_i64: u32,
    pub ret_temp_f64: u32,
    pub var_offsets: HashMap<String, u32>,
    pub var_types: HashMap<String, Type>,
    pub next_local_offset: u32,
    pub frame_size: i32,
    pub scope_objects: Vec<Vec<(String, Type, u32)>>,
    pub current_class: Option<String>,
}"""

# Replace compile_function prologue/epilogue
old_compile_func_start = """    fn compile_function(&mut self, f: &FunctionDecl, body: &Stmt) -> Result<(Vec<(u32, u8)>, Vec<u8>), CompileError> {
        let mut code = Vec::new();
        let frame_size = 512; // 512 bytes per function frame

        let num_params = f.params.len() as u32;
        let fp_local = num_params;
        let scratch1 = num_params + 1;
        let scratch2 = num_params + 2;
        let scratch_i64 = num_params + 3;
        let scratch_f64 = num_params + 4;
        let scratch_f32 = num_params + 5;"""

new_compile_func_start = """    fn compile_function(&mut self, f: &FunctionDecl, body: &Stmt, class_name: Option<String>) -> Result<(Vec<(u32, u8)>, Vec<u8>), CompileError> {
        let mut code = Vec::new();
        let frame_size = 1024; // 1KB per function frame

        let num_params = f.params.len() as u32;
        let fp_local = num_params;
        let scratch1 = num_params + 1;
        let scratch2 = num_params + 2;
        let ret_temp_i32 = num_params + 3;
        let scratch_i64 = num_params + 4;
        let ret_temp_i64 = num_params + 5;
        let scratch_f64 = num_params + 6;
        let ret_temp_f64 = num_params + 7;
        let scratch_f32 = num_params + 8;"""

assert old_compile_func_start in code
code = code.replace(old_compile_func_start, new_compile_func_start, 1)

old_ctx_init = """        let mut ctx = FunctionContext {
            fp_local,
            scratch1,
            scratch2,
            scratch_i64,
            scratch_f64,
            scratch_f32,
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            next_local_offset: 16,
            frame_size,
        };"""

new_ctx_init = """        let mut ctx = FunctionContext {
            fp_local,
            scratch1,
            scratch2,
            scratch_i64,
            scratch_f64,
            scratch_f32,
            ret_temp_i32,
            ret_temp_i64,
            ret_temp_f64,
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            next_local_offset: 16,
            frame_size,
            scope_objects: vec![Vec::new()],
            current_class: class_name.clone(),
        };

        // If constructor or destructor, set up vtable pointer and chain bases
        if let Some(ref cls) = class_name {
            if f.name == *cls {
                // Constructor:
                // 1. If base class exists, call base ctor
                if let Some(layout) = self.sema.struct_layouts.get(cls) {
                    if !layout.base_classes.is_empty() {
                        let base_name = &layout.base_classes[0];
                        let base_ctor = format!("{}_{}", base_name, base_name);
                        if let Some(&b_idx) = self.func_indices.get(&base_ctor) {
                            code.push(OP_LOCAL_GET);
                            write_u32(&mut code, 0); // this
                            code.push(OP_CALL);
                            write_u32(&mut code, b_idx);
                        }
                    }
                    // 2. Set this class vtable
                    if let Some(&v_off) = self.vtable_offsets.get(cls) {
                        code.push(OP_LOCAL_GET);
                        write_u32(&mut code, 0); // this
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, v_off as i32);
                        code.push(OP_I32_STORE);
                        write_u32(&mut code, 2);
                        write_u32(&mut code, 0);
                    }
                    // 3. Member initializers
                    for (mname, margs) in &f.ctor_initializers {
                        if let Some(fld) = layout.fields.iter().find(|fl| &fl.name == mname) {
                            code.push(OP_LOCAL_GET);
                            write_u32(&mut code, 0); // this
                            if !margs.is_empty() {
                                self.compile_expr(&margs[0], &mut ctx, &mut code)?;
                            } else {
                                code.push(OP_I32_CONST);
                                write_i32(&mut code, 0);
                            }
                            self.emit_store(&fld.ty, fld.offset as u32, &mut code);
                        }
                    }
                }
            } else if f.name == format!("~{}", cls) {
                // Destructor:
                // 1. Set this class vtable (for virtual calls inside destructor)
                if let Some(&v_off) = self.vtable_offsets.get(cls) {
                    code.push(OP_LOCAL_GET);
                    write_u32(&mut code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(&mut code, v_off as i32);
                    code.push(OP_I32_STORE);
                    write_u32(&mut code, 2);
                    write_u32(&mut code, 0);
                }
            }
        }"""

assert old_ctx_init in code
code = code.replace(old_ctx_init, new_ctx_init, 1)

# Update epilogue and locals
old_func_end = """        let locals = vec![(3, TYPE_I32), (1, TYPE_I64), (1, TYPE_F64), (1, TYPE_F32)];
        Ok((locals, code))"""

new_func_end = """        // If destructor of derived class, call base destructor at exit
        if let Some(ref cls) = class_name {
            if f.name == format!("~{}", cls) {
                if let Some(layout) = self.sema.struct_layouts.get(cls) {
                    if !layout.base_classes.is_empty() {
                        let base_name = &layout.base_classes[0];
                        let base_dtor = format!("{}_~{}", base_name, base_name);
                        if let Some(&b_idx) = self.func_indices.get(&base_dtor) {
                            code.push(OP_LOCAL_GET);
                            write_u32(&mut code, 0); // this
                            code.push(OP_CALL);
                            write_u32(&mut code, b_idx);
                        }
                    }
                }
            }
        }

        // Epilogue root RAII destructors
        let root_objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&root_objs, &ctx, &mut code);

        let locals = vec![(4, TYPE_I32), (2, TYPE_I64), (2, TYPE_F64), (1, TYPE_F32)];
        Ok((locals, code))"""

assert old_func_end in code
code = code.replace(old_func_end, new_func_end, 1)

with open("src/codegen.rs", "w") as f:
    f.write(code)

print("Updated compile_function successfully!")
