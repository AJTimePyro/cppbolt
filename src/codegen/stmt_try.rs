//! Statement code generation for exception `try` blocks and catch dispatch.

use super::*;
use crate::ast::stmt::{CatchClause, Stmt};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::runtime::opcodes::*;
use crate::wasm::{write_i32, write_u32};

impl<'a> Codegen<'a> {
    pub(super) fn compile_try_stmt(
        &mut self,
        try_body: &Stmt,
        catches: &[CatchClause],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if catches.is_empty() {
            return self.compile_stmt(try_body, ctx, code);
        }
        let start_clean_idx = ctx.cleanup_entries.len();
        let has_catch_all = catches.last().map(|c| c.param.is_none()).unwrap_or(false);

        // Outermost block: $try_exit
        code.push(OP_BLOCK);
        code.push(0x40);

        // Optional catch-all block: $catch_all
        if has_catch_all {
            code.push(OP_BLOCK);
            code.push(TYPE_EXNREF);
        }

        // Tag 0 dispatch block: $dispatch
        code.push(OP_BLOCK);
        code.push(TYPE_I32);

        code.push(OP_TRY_TABLE);
        code.push(0x40);
        if has_catch_all {
            write_u32(code, 2);
            code.push(0x00); // catch tag 0 -> target 0 ($dispatch)
            write_u32(code, 0);
            write_u32(code, 0);
            code.push(0x03); // catch_all_ref -> target 1 ($catch_all)
            write_u32(code, 1);
        } else {
            write_u32(code, 1);
            code.push(0x00); // catch tag 0 -> target 0 ($dispatch)
            write_u32(code, 0);
            write_u32(code, 0);
        }

        self.compile_stmt(try_body, ctx, code)?;

        // Normal completion branches out of $try_exit
        code.push(OP_BR);
        write_u32(code, if has_catch_all { 3 } else { 2 });

        code.push(OP_END); // end try_table
        code.push(OP_UNREACHABLE);

        let end_clean_idx = ctx.cleanup_entries.len();

        code.push(OP_END); // end $dispatch block (i32 exn_header_ptr is on stack)
        self.emit_dispatch_typed_catches(
            catches,
            start_clean_idx..end_clean_idx,
            has_catch_all,
            ctx,
            code,
        )?;

        if has_catch_all {
            code.push(OP_END); // end $catch_all block (exnref is on stack)
            if let Some(catch_all_clause) = catches.last() {
                self.emit_catch_all_handler(
                    &catch_all_clause.body,
                    start_clean_idx..end_clean_idx,
                    ctx,
                    code,
                )?;
            }
        }

        code.push(OP_END); // end $try_exit
        Ok(())
    }

    fn emit_dispatch_typed_catches(
        &mut self,
        catches: &[CatchClause],
        clean_range: std::ops::Range<usize>,
        has_catch_all: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let save_val_slot = ctx.alloc_frame(4, 4)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        Self::emit_slot_store(save_val_slot, ctx.fp_local, ctx.last_exn_val_local, code);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.last_exn_val_local);

        self.emit_cleanup_range(clean_range.start, clean_range.end, ctx, code);

        let br_depth = if has_catch_all { 2 } else { 1 };

        for clause in catches {
            self.emit_dispatch_clause(clause, br_depth, save_val_slot, ctx, code)?;
        }

        // No catch clause matched: restore previous and rethrow
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.last_exn_val_local);
        Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.last_exn_val_local);
        code.push(OP_THROW);
        write_u32(code, 0);
        Ok(())
    }

    fn emit_dispatch_clause(
        &mut self,
        clause: &CatchClause,
        br_depth: u32,
        save_val_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some((ty, name)) = &clause.param {
            let can_catch_idx = self.func_indices["__cppbolt_can_catch"];
            let handler_id = self.sema.get_exception_type_id(ty).unwrap_or(0);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.last_exn_val_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, 0); // thrown type_id

            code.push(OP_I32_CONST);
            write_i32(code, handler_id as i32);
            code.push(OP_CALL);
            write_u32(code, can_catch_idx);

            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1); // offset

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_CONST);
            write_i32(code, 0);
            code.push(OP_I32_GE_S);
            code.push(OP_IF);
            code.push(0x40);

            self.emit_typed_clause_body(clause, ty, name, ctx, code)?;
            self.emit_destroy_exception_if_not_rethrown(ctx.last_exn_val_local, ctx, code);

            Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.last_exn_val_local);

            code.push(OP_BR);
            write_u32(code, br_depth);
            code.push(OP_END);
        } else {
            // catch (...) handles tag 0 exception
            self.emit_catch_clause_body(&clause.body, ctx, code)?;
            self.emit_destroy_exception_if_not_rethrown(ctx.last_exn_val_local, ctx, code);

            Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.last_exn_val_local);

            code.push(OP_BR);
            write_u32(code, 1);
        }
        Ok(())
    }

    fn emit_typed_clause_body(
        &mut self,
        clause: &CatchClause,
        ty: &Type,
        name: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        ctx.enter_scope();
        ctx.scope_objects.push(Vec::new());
        let old_in_catch_all = ctx.in_catch_all;
        ctx.in_catch_all = false;
        ctx.catch_depth += 1;

        if !name.is_empty() {
            let payload_addr_slot = ctx.alloc_frame(4, 4)?;
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.last_exn_val_local);
            code.push(OP_I32_CONST);
            write_i32(code, 16);
            code.push(OP_I32_ADD);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_ADD);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch2);
            Self::emit_slot_store(payload_addr_slot, ctx.fp_local, ctx.scratch2, code);

            self.bind_catch_param(name, ty, payload_addr_slot, ctx, code)?;
        }

        self.compile_stmt(&clause.body, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);
        let catch_objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&catch_objs, ctx, code);
        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();
        Ok(())
    }

    fn bind_catch_param(
        &mut self,
        name: &str,
        ty: &Type,
        payload_addr_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if ty.is_reference() {
            let offset = ctx.alloc_frame(4, 4)?;
            ctx.bind_local(name.to_string(), offset, ty.clone());
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, offset);
        } else if let Type::Struct(sname) = ty.strip_const() {
            let raw_size = self.sema.struct_sizes.get(sname).copied().unwrap_or(4);
            let size = u32::try_from(raw_size).unwrap_or(4);
            let align = u32::try_from(self.sema.struct_aligns.get(sname).copied().unwrap_or(4))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(size, align)?;
            ctx.bind_local(name.to_string(), offset, ty.clone());

            let is_const = ty.is_const();
            if let Some(ctor_idx) = self.find_copy_constructor(sname, is_const) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                code.push(OP_I32_ADD);
                Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            } else {
                Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                raii::emit_struct_byte_copy(
                    raw_size,
                    offset,
                    raii::ByteCopyAddr::Local(ctx.fp_local),
                    raii::ByteCopyAddr::Local(ctx.scratch1),
                    code,
                );
            }
            let clean_idx = if let Some(dtor_idx) = self.lookup_struct_dtor(sname) {
                let idx = ctx.register_cleanup(sname.to_string(), dtor_idx, offset, false)?;
                self.emit_mark_cleanup_active(idx, ctx, code);
                Some(idx)
            } else {
                None
            };
            if let Some(scope) = ctx.scope_objects.last_mut() {
                scope.push((name.to_string(), ty.clone(), offset, clean_idx));
            }
        } else {
            let raw_size = ty.size_of(&self.sema.struct_sizes);
            let size = u32::try_from(raw_size).unwrap_or(4);
            let align = u32::try_from(ty.align_of(&self.sema.struct_aligns))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(size, align)?;
            ctx.bind_local(name.to_string(), offset, ty.clone());

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
            self.emit_load(ty, 0, code);
            self.emit_store(ty, offset, code);
        }
        Ok(())
    }

    fn emit_catch_clause_body(
        &mut self,
        body: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        ctx.enter_scope();
        let old_in_catch_all = ctx.in_catch_all;
        ctx.in_catch_all = false;
        ctx.catch_depth += 1;

        self.compile_stmt(body, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);

        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();
        Ok(())
    }

    fn emit_catch_all_handler(
        &mut self,
        body: &Stmt,
        clean_range: std::ops::Range<usize>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let handler_depth = ctx.catch_depth as u32;
        if handler_depth >= ctx.max_catch_depth {
            return Err(CompileError::new(
                "function exceeds maximum catch nesting depth",
                body.line,
                body.col,
            ));
        }
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.exnref_local + handler_depth);

        self.emit_cleanup_range(clean_range.start, clean_range.end, ctx, code);

        ctx.enter_scope();
        let old_in_catch_all = ctx.in_catch_all;
        ctx.in_catch_all = true;
        ctx.catch_depth += 1;

        self.compile_stmt(body, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);

        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();

        code.push(OP_BR);
        write_u32(code, 0);
        Ok(())
    }
}
