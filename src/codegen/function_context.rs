use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn function_context(
        f: &FunctionDecl,
        class_name: Option<&str>,
    ) -> (Vec<(u32, u8)>, FunctionContext) {
        // Non-static methods carry the implicit `this` pointer as parameter 0.
        let num_params = f.params.len() as u32;
        let fp_local = num_params;
        let scratch1 = num_params + 1;
        let scratch2 = num_params + 2;
        let ret_temp_i32 = num_params + 3;
        let scratch_i64 = num_params + 4;
        let ret_temp_i64 = num_params + 5;
        let scratch_f64 = num_params + 6;
        let ret_temp_f64 = num_params + 7;
        let scratch_f32 = num_params + 8;

        let locals = vec![
            (4, TYPE_I32), // fp_local, scratch1, scratch2, ret_temp_i32
            (2, TYPE_I64), // scratch_i64, ret_temp_i64
            (2, TYPE_F64), // scratch_f64, ret_temp_f64
            (1, TYPE_F32), // scratch_f32
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
            scratch_i64,
            scratch_f64,
            scratch_f32,
            ret_temp_i32,
            ret_temp_i64,
            ret_temp_f64,
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            static_var_offsets: HashMap::new(),
            next_local_offset: 16,
            frame_size: 0,
            frame_size_patches: Vec::new(),
            var_scopes: vec![Vec::new()],
            scope_objects: vec![Vec::new()],
            temp_objects: Vec::new(),
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
