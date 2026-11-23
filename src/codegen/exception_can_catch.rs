//! Lowering for the dynamic C++ exception catch matching helper `__cppbolt_can_catch`.

use super::*;
use crate::runtime::opcodes::*;
use crate::sema::exception_types::{TYPE_ID_CLASS_BASE, TYPE_ID_PTR_CLASS_BASE};
use crate::wasm::{write_i32, write_u32};

impl<'a> Codegen<'a> {
    fn emit_can_catch_exact(code: &mut Vec<u8>) {
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_EQ);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
    }

    fn emit_can_catch_pointers(code: &mut Vec<u8>) {
        // char* (13) matches const char* (12)
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 13);
        code.push(OP_I32_EQ);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 12);
        code.push(OP_I32_EQ);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);

        // void* (14) or const void* (15) matches any pointer (12..=15 or >= 20000)
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 14);
        code.push(OP_I32_EQ);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 15);
        code.push(OP_I32_EQ);
        code.push(OP_I32_OR);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 12);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 15);
        code.push(OP_I32_LE_U);
        code.push(OP_I32_AND);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20000);
        code.push(OP_I32_GE_U);
        code.push(OP_I32_OR);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
        code.push(OP_END);
    }

    fn emit_can_catch_pair(
        thrown_id: u32,
        handler_id: u32,
        base_off: usize,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, thrown_id as i32);
        code.push(OP_I32_EQ);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, handler_id as i32);
        code.push(OP_I32_EQ);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, base_off as i32);
        code.push(OP_RETURN);
        code.push(OP_END);
    }

    fn emit_can_catch_inheritance(&self, code: &mut Vec<u8>) {
        for (derived_name, layout) in &self.sema.struct_layouts {
            if let Some(derived_id) = self.sema.get_class_type_id(derived_name) {
                for (base_name, base_off) in &layout.base_offsets {
                    if let Some(base_id) = self.sema.get_class_type_id(base_name) {
                        Self::emit_can_catch_pair(derived_id, base_id, *base_off, code);
                        let d_ptr = derived_id - TYPE_ID_CLASS_BASE + TYPE_ID_PTR_CLASS_BASE;
                        let b_ptr = base_id - TYPE_ID_CLASS_BASE + TYPE_ID_PTR_CLASS_BASE;
                        Self::emit_can_catch_pair(d_ptr, b_ptr, *base_off, code);
                    }
                }
            }
        }
    }

    pub(super) fn emit_can_catch_function(&mut self) {
        let mut code = Vec::new();
        Self::emit_can_catch_exact(&mut code);
        Self::emit_can_catch_pointers(&mut code);
        self.emit_can_catch_inheritance(&mut code);

        code.push(OP_I32_CONST);
        write_i32(&mut code, -1);
        code.push(OP_RETURN);
        code.push(OP_END);

        self.module.codes.push((vec![], code));
    }
}
