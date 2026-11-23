//! Lowering for the dynamic C++ exception catch matching helper `__cppbolt_can_catch`.

use super::*;
use crate::runtime::opcodes::*;
use crate::sema::exception_types::{
    TYPE_ID_AUX_SPAN, TYPE_ID_CONST_VOID_PTR, TYPE_ID_MEMBER_PTR_BASE, TYPE_ID_NULLPTR,
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

    fn emit_check_same_base_and_depth(code: &mut Vec<u8>) {
        // Same base ID: (thrown & 0xFFFF) == (handler & 0xFFFF)
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 0xFFFF);
        code.push(OP_I32_AND);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 0xFFFF);
        code.push(OP_I32_AND);
        code.push(OP_I32_EQ);

        // Same depth: ((thrown >> 16) & 0xF) == ((handler >> 16) & 0xF)
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_I32_EQ);
        code.push(OP_I32_AND);

        // depth >= 1
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_GT_U);
        code.push(OP_I32_AND);
    }

    fn emit_check_qualification_masks(code: &mut Vec<u8>) {
        // handler has const at level 1 (bit 20 set)
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 1 << 20);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_NE);

        // (thrown_mask & handler_mask) == thrown_mask
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20);
        code.push(OP_I32_SHR_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 20);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_AND);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 20);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_EQ);
        code.push(OP_I32_AND);
    }

    fn emit_can_catch_qualification(code: &mut Vec<u8>) {
        Self::emit_check_same_base_and_depth(code);
        Self::emit_check_qualification_masks(code);
        code.push(OP_I32_AND);

        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
    }

    fn emit_can_catch_void_pointers(code: &mut Vec<u8>) {
        // const void* matches any pointer (depth >= 1)
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
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_GT_U);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_RETURN);
        code.push(OP_END);
        code.push(OP_END);

        // void* matches only pointers where level 1 is NOT const
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
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_GT_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        code.push(OP_I32_CONST);
        write_i32(code, 1 << 20);
        code.push(OP_I32_AND);
        code.push(OP_I32_EQZ);
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
        // nullptr (18) matches pointer handlers (depth >= 1) and member-pointer
        // handlers, whose base ID lives in its own reserved range.
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
        write_i32(code, 16);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 0xF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_GT_U);

        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 0xFFFF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, TYPE_ID_MEMBER_PTR_BASE as i32);
        code.push(OP_I32_GE_U);
        code.push(OP_LOCAL_GET);
        write_u32(code, 1);
        code.push(OP_I32_CONST);
        write_i32(code, 0xFFFF);
        code.push(OP_I32_AND);
        code.push(OP_I32_CONST);
        write_i32(code, (TYPE_ID_MEMBER_PTR_BASE + TYPE_ID_AUX_SPAN) as i32);
        code.push(OP_I32_LT_U);
        code.push(OP_I32_AND);
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
        let mut seen = std::collections::HashSet::new();
        let canonical_names = self.sema.canonical_class_names();
        for (derived_name, layout) in &self.sema.struct_layouts {
            if let Some(derived_id) =
                self.sema.class_type_id_from_names(derived_name, &canonical_names)
            {
                for (base_name, base_off) in &layout.base_offsets {
                    // [except.handle]: catch(Base&) matches Derived only for an
                    // unambiguous public base subobject.
                    if self.sema.base_subobject_path_count(derived_name, base_name) != 1
                        || !self.sema.base_path_is_public(derived_name, base_name)
                    {
                        continue;
                    }
                    if let Some(base_id) =
                        self.sema.class_type_id_from_names(base_name, &canonical_names)
                    {
                        if !seen.insert((derived_id, base_id, *base_off)) {
                            continue;
                        }
                        Self::emit_can_catch_pair(derived_id, base_id, *base_off, code);
                        let d_ptr = (1 << 16) | derived_id;
                        let b_ptr = (1 << 16) | base_id;
                        let d_const = (1 << 16) | (1 << 20) | derived_id;
                        let b_const = (1 << 16) | (1 << 20) | base_id;
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
