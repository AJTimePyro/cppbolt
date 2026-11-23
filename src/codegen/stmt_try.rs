//! Statement code generation for exception `try` blocks and catch dispatch.

use super::*;
use crate::ast::stmt::{CatchClause, Stmt};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::runtime::opcodes::*;
use crate::wasm::{write_i32, write_u32};

struct ClauseDispatchCtx<'a> {
    clause: &'a CatchClause,
    param: Option<(&'a Type, &'a str)>,
    try_exit_target: u32,
    save_val_slot: u32,
}

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
        let try_exit_target = self.emit_try_block_prelude(has_catch_all, code);

        ctx.try_depth += 1;
        self.compile_stmt(try_body, ctx, code)?;
        ctx.try_depth -= 1;

        // Normal completion branches out of $try_exit
        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - try_exit_target);

        code.push(OP_END); // end try_table
        self.current_block_depth -= 1;
        code.push(OP_UNREACHABLE);

        let end_clean_idx = ctx.cleanup_entries.len();

        code.push(OP_END); // end $dispatch block (i32 exn_header_ptr is on stack)
        self.current_block_depth -= 1;
        self.emit_dispatch_typed_catches(
            catches,
            start_clean_idx..end_clean_idx,
            try_exit_target,
            ctx,
            code,
        )?;

        if has_catch_all {
            code.push(OP_END); // end $catch_all block (exnref is on stack)
            self.current_block_depth -= 1;
            if let Some(catch_all_clause) = catches.last() {
                self.emit_catch_all_handler(
                    &catch_all_clause.body,
                    start_clean_idx..end_clean_idx,
                    try_exit_target,
                    ctx,
                    code,
                )?;
            }
        }

        code.push(OP_END); // end $try_exit
        self.current_block_depth -= 1;
        Ok(())
    }

    fn emit_try_block_prelude(&mut self, has_catch_all: bool, code: &mut Vec<u8>) -> u32 {
        code.push(OP_BLOCK);
        code.push(0x40);
        self.current_block_depth += 1;
        let try_exit_target = self.current_block_depth - 1;

        if has_catch_all {
            code.push(OP_BLOCK);
            code.push(TYPE_EXNREF);
            self.current_block_depth += 1;
        }

        code.push(OP_BLOCK);
        code.push(TYPE_I32);
        self.current_block_depth += 1;

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
        self.current_block_depth += 1;
        try_exit_target
    }

    fn emit_dispatch_typed_catches(
        &mut self,
        catches: &[CatchClause],
        clean_range: std::ops::Range<usize>,
        try_exit_target: u32,
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

        for clause in catches {
            self.emit_dispatch_clause(clause, try_exit_target, save_val_slot, ctx, code)?;
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
        try_exit_target: u32,
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
            self.current_block_depth += 1;

            self.emit_typed_clause_body(
                &ClauseDispatchCtx {
                    clause,
                    param: Some((ty, name)),
                    try_exit_target,
                    save_val_slot,
                },
                ctx,
                code,
            )?;

            code.push(OP_END);
            self.current_block_depth -= 1;
        } else {
            // catch (...) handles tag 0 exception
            self.emit_typed_clause_body(
                &ClauseDispatchCtx {
                    clause,
                    param: None,
                    try_exit_target,
                    save_val_slot,
                },
                ctx,
                code,
            )?;
        }
        Ok(())
    }

    fn emit_typed_clause_body(
        &mut self,
        dctx: &ClauseDispatchCtx<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let handled_slot = ctx.alloc_frame(4, 4)?;
        Self::emit_slot_store(handled_slot, ctx.fp_local, ctx.last_exn_val_local, code);
        ctx.active_exception_slots.push(handled_slot);
        let thrown_slot = ctx.alloc_frame(4, 4)?;

        ctx.enter_scope();
        ctx.scope_objects.push(Vec::new());
        let old_in_catch_all = ctx.in_catch_all;
        ctx.in_catch_all = false;
        ctx.catch_depth += 1;
        self.emit_uncaught_count_change(code, -1);
        let clean_start = ctx.cleanup_entries.len();
        let clause_exit_target = self.emit_clause_try_table_prelude(code);

        // Bind catch parameter if present
        if let Some((ty, name)) = dctx.param {
            self.bind_catch_param_dispatch(name, ty, handled_slot, ctx, code)?;
        }

        // Execute catch clause body
        self.compile_stmt(&dctx.clause.body, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);

        // Run destructors for objects in catch scope
        let catch_objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&catch_objs, ctx, code);

        // Normal completion: branch to $clause_exit
        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - clause_exit_target);

        // End try_table
        code.push(OP_END);
        self.current_block_depth -= 1;
        code.push(OP_UNREACHABLE);

        self.emit_clause_other_unwind(clean_start, (handled_slot, dctx.save_val_slot), ctx, code);
        self.emit_clause_tag0_unwind(
            clean_start,
            (handled_slot, thrown_slot, dctx.save_val_slot),
            ctx,
            code,
        );
        self.emit_clause_normal_exit(
            (handled_slot, dctx.save_val_slot, dctx.try_exit_target),
            old_in_catch_all,
            ctx,
            code,
        );

        Ok(())
    }

    fn emit_clause_try_table_prelude(&mut self, code: &mut Vec<u8>) -> u32 {
        code.push(OP_BLOCK);
        code.push(0x40);
        self.current_block_depth += 1;
        let clause_exit_target = self.current_block_depth - 1;

        code.push(OP_BLOCK);
        code.push(TYPE_I32);
        self.current_block_depth += 1;

        code.push(OP_BLOCK);
        code.push(TYPE_EXNREF);
        self.current_block_depth += 1;

        code.push(OP_TRY_TABLE);
        code.push(0x40);
        write_u32(code, 2);
        code.push(0x00); // catch tag 0 -> target 1 ($clause_tag0)
        write_u32(code, 0);
        write_u32(code, 1);
        code.push(0x03); // catch_all_ref -> target 0 ($clause_other)
        write_u32(code, 0);
        self.current_block_depth += 1;
        clause_exit_target
    }

    fn emit_clause_other_unwind(
        &mut self,
        clean_start: usize,
        (handled_slot, save_val_slot): (u32, u32),
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_END);
        self.current_block_depth -= 1;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.unwind_exnref_local);

        self.emit_cleanup_range(clean_start, ctx.cleanup_entries.len(), ctx, code);

        Self::emit_slot_load(handled_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);
        self.emit_destroy_exception_if_not_rethrown(ctx.scratch2, ctx, code);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, handled_slot);

        Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.last_exn_val_local);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.unwind_exnref_local);
        code.push(OP_THROW_REF);
    }

    fn emit_clause_tag0_unwind(
        &mut self,
        clean_start: usize,
        (handled_slot, thrown_slot, save_val_slot): (u32, u32, u32),
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_END);
        self.current_block_depth -= 1;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        Self::emit_slot_store(thrown_slot, ctx.fp_local, ctx.scratch1, code);

        self.emit_cleanup_range(clean_start, ctx.cleanup_entries.len(), ctx, code);

        Self::emit_slot_load(handled_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);

        Self::emit_slot_load(thrown_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_NE);
        code.push(OP_IF);
        code.push(0x40);
        self.emit_destroy_exception_if_not_rethrown(ctx.scratch2, ctx, code);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, handled_slot);
        code.push(OP_END);

        Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.last_exn_val_local);

        Self::emit_slot_load(thrown_slot, ctx.fp_local, code);
        code.push(OP_THROW);
        write_u32(code, 0);
    }

    fn emit_clause_normal_exit(
        &mut self,
        (handled_slot, save_val_slot, try_exit_target): (u32, u32, u32),
        old_in_catch_all: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_END);
        self.current_block_depth -= 1;

        Self::emit_slot_load(handled_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        self.emit_destroy_exception_if_not_rethrown(ctx.scratch1, ctx, code);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, handled_slot);

        ctx.active_exception_slots.pop();
        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();

        Self::emit_slot_load(save_val_slot, ctx.fp_local, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.last_exn_val_local);

        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - try_exit_target);
    }

    fn emit_catch_all_handler(
        &mut self,
        body: &Stmt,
        clean_range: std::ops::Range<usize>,
        try_exit_target: u32,
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
        self.emit_uncaught_count_change(code, -1);

        self.compile_stmt(body, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);

        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();

        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - try_exit_target);
        Ok(())
    }
}
