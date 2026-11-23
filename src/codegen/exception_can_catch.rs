//! Lowering for the dynamic C++ exception catch matching helper `__cppbolt_can_catch`.

use super::*;
use crate::runtime::opcodes::*;
use crate::sema::exception_types::{
    TYPE_ID_CONST_PTR_BASE, TYPE_ID_CONST_VOID_PTR, TYPE_ID_NULLPTR, TYPE_ID_PTR_BASE,
    TYPE_ID_VOID_PTR,
};
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

    fn emit_can_catch_qualification(code: &mut Vec<u8>) {
        // T* (20000..30000) matches const T* (30000..40000) when handler == thrown + 10000
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20000);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 30000);
        code.push(OP_I32_LT_U);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 10000);
        code.push(OP_I32_ADD);
        code.push(OP_I32_EQ);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
        code.push(OP_END);
    }

    fn emit_can_catch_void_pointers(code: &mut Vec<u8>) {
        // const void* matches any pointer (20000..40000)
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, TYPE_ID_CONST_VOID_PTR as i32);
        code.push(OP_I32_EQ);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20000);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 40000);
        code.push(OP_I32_LT_U);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
        code.push(OP_END);

        // void* matches only mutable pointers (20000..30000)
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, TYPE_ID_VOID_PTR as i32);
        code.push(OP_I32_EQ);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20000);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 30000);
        code.push(OP_I32_LT_U);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
        code.push(OP_END);
    }

    fn emit_can_catch_nullptr(code: &mut Vec<u8>) {
        // nullptr (18) matches any pointer handler (20000..40000)
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, TYPE_ID_NULLPTR as i32);
        code.push(OP_I32_EQ);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 20000);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 40000);
        code.push(OP_I32_LT_U);
        code.push(OP_I32_AND);
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
                        let d_ptr = TYPE_ID_PTR_BASE + derived_id;
                        let b_ptr = TYPE_ID_PTR_BASE + base_id;
                        let d_const = TYPE_ID_CONST_PTR_BASE + derived_id;
                        let b_const = TYPE_ID_CONST_PTR_BASE + base_id;
                        Self::emit_can_catch_pair(d_ptr, b_ptr, *base_off, code);
                        Self::emit_can_catch_pair(d_ptr, b_const, *base_off, code);
                        Self::emit_can_catch_pair(d_const, b_const, *base_off, code);
                    }
                }
            }
        }
    }

    pub(super) fn emit_can_catch_function(&mut self) {
        let mut code = Vec::new();
        Self::emit_can_catch_exact(&mut code);
        Self::emit_can_catch_qualification(&mut code);
        Self::emit_can_catch_void_pointers(&mut code);
        Self::emit_can_catch_nullptr(&mut code);
        self.emit_can_catch_inheritance(&mut code);

        code.push(OP_I32_CONST);
        write_i32(&mut code, -1);
        code.push(OP_RETURN);
        code.push(OP_END);

        self.module.codes.push((vec![], code));
    }
}
