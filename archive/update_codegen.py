import re

with open("src/codegen.rs", "r") as f:
    orig = f.read()

# 1. Update Codegen struct
old_codegen_struct = """pub struct Codegen<'a> {
    pub sema: &'a Sema,
    pub module: WasmModule,
    pub string_table: HashMap<String, u32>,
    pub next_data_offset: u32,
    pub func_indices: HashMap<String, u32>,
    pub func_printf: u32,
    pub loop_depths: Vec<(u32, u32)>,
    pub current_block_depth: u32,
}"""

new_codegen_struct = """pub struct Codegen<'a> {
    pub sema: &'a Sema,
    pub module: WasmModule,
    pub string_table: HashMap<String, u32>,
    pub next_data_offset: u32,
    pub func_indices: HashMap<String, u32>,
    pub func_printf: u32,
    pub loop_depths: Vec<(u32, u32)>,
    pub current_block_depth: u32,
    pub vtable_offsets: HashMap<String, u32>,
    pub vtable_func_indices: Vec<u32>,
}"""

assert old_codegen_struct in orig
orig = orig.replace(old_codegen_struct, new_codegen_struct, 1)

# 2. Update Codegen::new to add malloc, free, pure_virtual
old_new_end = """        let mut func_indices = HashMap::new();
        func_indices.insert("printf".to_string(), func_printf);

        Self {
            sema,
            module,
            string_table: HashMap::new(),
            next_data_offset: 1024,
            func_indices,
            func_printf,
            loop_depths: Vec::new(),
            current_block_depth: 0,
        }"""

new_new_end = """        let func_malloc = 6;
        let t_i32_i32 = module.add_type(vec![TYPE_I32], vec![TYPE_I32]);
        module.functions.push(t_i32_i32);
        module.codes.push(emit_malloc(1));

        let func_free = 7;
        module.functions.push(t_i32_v);
        module.codes.push(emit_free());

        let func_pure_virtual = 8;
        module.functions.push(_t_v_v);
        module.codes.push(emit_pure_virtual_trap());

        let mut func_indices = HashMap::new();
        func_indices.insert("printf".to_string(), func_printf);
        func_indices.insert("malloc".to_string(), func_malloc);
        func_indices.insert("free".to_string(), func_free);
        func_indices.insert("__cxa_pure_virtual".to_string(), func_pure_virtual);

        Self {
            sema,
            module,
            string_table: HashMap::new(),
            next_data_offset: 1024,
            func_indices,
            func_printf,
            loop_depths: Vec::new(),
            current_block_depth: 0,
            vtable_offsets: HashMap::new(),
            vtable_func_indices: Vec::new(),
        }"""

assert old_new_end in orig
orig = orig.replace(old_new_end, new_new_end, 1)

# 3. Update FunctionContext
old_ctx = """pub struct FunctionContext {
    pub fp_local: u32,
    pub scratch1: u32,
    pub scratch2: u32,
    pub scratch_i64: u32,
    pub scratch_f64: u32,
    pub scratch_f32: u32,
    pub var_offsets: HashMap<String, u32>,
    pub var_types: HashMap<String, Type>,
    pub next_local_offset: u32,
    pub frame_size: i32,
}"""

new_ctx = """pub struct FunctionContext {
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

assert old_ctx in orig
orig = orig.replace(old_ctx, new_ctx, 1)

with open("src/codegen.rs", "w") as f:
    f.write(orig)

print("Applied first stage codegen changes!")
