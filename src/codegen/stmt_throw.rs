//! Statement code generation for exception `throw` and exception object lifecycle.

use super::*;
use crate::ast::expr::Expr;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::runtime::opcodes::*;
use crate::wasm::{write_i32, write_u32};

struct ExceptionMeta {
    type_id: u32,
    dtor_idx: i32,
    size: u32,
}

impl<'a> Codegen<'a> {
    pub(super) fn compile_throw_stmt(
        &mut self,
        expr_opt: &Option<Expr>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(expr) = expr_opt {
            let ty = self.get_expr_type(expr, ctx);
            let type_id = self.sema.get_exception_type_id(&ty).unwrap_or(0);
            let raw_size = ty.size_of(&self.sema.struct_sizes);
            let size = u32::try_from(raw_size).unwrap_or(4).max(4);
            let dtor_idx = match ty.strip_reference().strip_const() {
                Type::Struct(sname) => {
                    self.dtor_table_indices.get(sname).copied().map(|i| i as i32).unwrap_or(-1)
                }
                _ => -1,
            };

            let total_alloc = (16 + size) as i32;
            let temp_ptr_slot = ctx.alloc_frame(4, 4)?;

            code.push(OP_I32_CONST);
            write_i32(code, total_alloc);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["malloc"]);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            Self::emit_slot_store(temp_ptr_slot, ctx.fp_local, ctx.scratch1, code);

            // [except.throw] p4: terminate immediately if exception allocation fails.
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_EQZ);
            code.push(OP_IF);
            code.push(0x40);
            self.emit_terminate_call(code);
            code.push(OP_UNREACHABLE);
            code.push(OP_END);

            let meta = ExceptionMeta { type_id, dtor_idx, size };
            self.emit_exception_header(temp_ptr_slot, &meta, ctx, code);
            self.compile_throw_payload_with_guard(expr, &ty, temp_ptr_slot, ctx, code)?;
            Self::emit_slot_load(temp_ptr_slot, ctx.fp_local, code);
            self.emit_uncaught_count_change(code, 1);
            code.push(OP_THROW);
            write_u32(code, 0);
        } else if ctx.catch_depth == 0 {
            self.emit_terminate_call(code);
        } else if ctx.in_catch_all {
            self.emit_uncaught_count_change(code, 1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.current_exnref_local());
            code.push(OP_THROW_REF);
        } else {
            self.emit_uncaught_count_change(code, 1);
            self.emit_rethrow_tag0(ctx, code);
        }
        Ok(())
    }

    fn compile_throw_payload_with_guard(
        &mut self,
        expr: &Expr,
        ty: &Type,
        temp_ptr_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_BLOCK);
        code.push(0x40);
        self.current_block_depth += 1;

        code.push(OP_BLOCK);
        code.push(TYPE_I32);
        self.current_block_depth += 1;

        code.push(OP_BLOCK);
        code.push(TYPE_EXNREF);
        self.current_block_depth += 1;

        code.push(OP_TRY_TABLE);
        code.push(0x40);
        write_u32(code, 2);
        code.push(0x00); // catch tag 0 -> target 1
        write_u32(code, 0);
        write_u32(code, 1);
        code.push(0x03); // catch_all_ref -> target 0
        write_u32(code, 0);
        self.current_block_depth += 1;

        if let Type::Struct(sname) = ty.strip_reference().strip_const() {
            self.compile_struct_payload_init(expr, sname, temp_ptr_slot, ctx, code)?;
        } else {
            Self::emit_slot_load(temp_ptr_slot, ctx.fp_local, code);
            self.compile_expr(expr, ctx, code)?;
            self.emit_store(ty, 16, code);
        }

        code.push(OP_BR);
        write_u32(code, 3);

        code.push(OP_END); // end try_table
        self.current_block_depth -= 1;
        code.push(OP_UNREACHABLE);

        code.push(OP_END); // end catch_all_ref (target 0)
        self.current_block_depth -= 1;
        Self::emit_slot_load(temp_ptr_slot, ctx.fp_local, code);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);
        code.push(OP_THROW_REF);

        code.push(OP_END); // end catch tag 0 (target 1)
        self.current_block_depth -= 1;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        Self::emit_slot_load(temp_ptr_slot, ctx.fp_local, code);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_THROW);
        write_u32(code, 0);

        code.push(OP_END); // end outer block
        self.current_block_depth -= 1;
        Ok(())
    }

    fn emit_exception_header(
        &self,
        ptr_slot: u32,
        meta: &ExceptionMeta,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        // [ptr + 0] = type_id
        Self::emit_slot_load(ptr_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, meta.type_id as i32);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, 0);

        // [ptr + 4] = dtor_idx
        Self::emit_slot_load(ptr_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, meta.dtor_idx);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, 4);

        // [ptr + 8] = flags (0)
        Self::emit_slot_load(ptr_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, 8);

        // [ptr + 12] = size
        Self::emit_slot_load(ptr_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, meta.size as i32);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, 12);
    }

    fn emit_struct_payload_lvalue(
        &mut self,
        expr: &Expr,
        sname: &str,
        (payload_ptr_slot, raw_size): (u32, usize),
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let is_xvalue = self.is_expr_xvalue(expr, ctx);
        let expr_ty = self.get_expr_type(expr, ctx);
        let raw_ty = self.get_expr_type_raw(expr, ctx);
        let is_const = expr_ty.strip_reference().is_const();
        let is_local_var = if let Expr::Identifier(name) = expr {
            ctx.var_offsets.contains_key(name)
                && !ctx.var_is_param.get(name).copied().unwrap_or(false)
                && !raw_ty.is_reference()
                && !is_const
                && ctx.var_try_depths.get(name).copied().unwrap_or(0) >= ctx.try_depth
        } else {
            false
        };
        let ctor = if is_xvalue || is_local_var {
            self.find_move_constructor(sname, false)
                .or_else(|| self.find_copy_constructor(sname, is_const))
        } else {
            self.find_copy_constructor(sname, is_const)
        };
        if let Some(ctor_idx) = ctor {
            Self::emit_slot_load(payload_ptr_slot, ctx.fp_local, code);
            if is_xvalue {
                self.compile_expr(expr, ctx, code)?;
            } else {
                self.compile_lvalue_addr(expr, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
        } else {
            if is_xvalue {
                self.compile_expr(expr, ctx, code)?;
            } else {
                self.compile_lvalue_addr(expr, ctx, code)?;
            }
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            raii::emit_struct_byte_copy(
                raw_size,
                0,
                raii::ByteCopyAddr::Slot {
                    fp_local: ctx.fp_local,
                    slot: payload_ptr_slot,
                },
                raii::ByteCopyAddr::Local(ctx.scratch1),
                code,
            );
        }
        Ok(())
    }

    fn compile_struct_payload_init(
        &mut self,
        expr: &Expr,
        sname: &str,
        temp_ptr_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let raw_size = self.sema.struct_sizes.get(sname).copied().unwrap_or(4);
        let payload_ptr_slot = ctx.alloc_frame(4, 4)?;
        Self::emit_slot_load(temp_ptr_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        Self::emit_slot_store(payload_ptr_slot, ctx.fp_local, ctx.scratch1, code);

        if self.is_struct_lvalue(expr, ctx) || self.is_expr_xvalue(expr, ctx) {
            self.emit_struct_payload_lvalue(
                expr,
                sname,
                (payload_ptr_slot, raw_size),
                ctx,
                code,
            )?;
        } else {
            let temp_count_before = ctx.temp_objects.len();
            self.compile_expr(expr, ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            raii::emit_struct_byte_copy(
                raw_size,
                0,
                raii::ByteCopyAddr::Slot {
                    fp_local: ctx.fp_local,
                    slot: payload_ptr_slot,
                },
                raii::ByteCopyAddr::Local(ctx.scratch1),
                code,
            );
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["free"]);
            let clean_indices: Vec<_> = ctx
                .temp_objects
                .drain(temp_count_before..)
                .filter_map(|t| t.3)
                .collect();
            for idx in clean_indices {
                self.emit_mark_cleanup_inactive(idx, ctx, code);
            }
        }
        Ok(())
    }

    fn emit_rethrow_tag0(&self, ctx: &FunctionContext, code: &mut Vec<u8>) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.last_exn_val_local);
        code.push(OP_THROW);
        write_u32(code, 0);
    }

    pub(super) fn emit_destroy_exception_if_not_rethrown(
        &self,
        exn_ptr_local: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let dtor_local = if exn_ptr_local == ctx.scratch1 {
            ctx.scratch2
        } else {
            ctx.scratch1
        };

        code.push(OP_LOCAL_GET);
        write_u32(code, exn_ptr_local);
        code.push(OP_IF);
        code.push(0x40);

        // Check if dtor_idx != -1
        code.push(OP_LOCAL_GET);
        write_u32(code, exn_ptr_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 4);
        code.push(OP_LOCAL_SET);
        write_u32(code, dtor_local);

        code.push(OP_LOCAL_GET);
        write_u32(code, dtor_local);
        code.push(OP_I32_CONST);
        write_i32(code, -1);
        code.push(OP_I32_NE);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, exn_ptr_local);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_ADD); // this = payload
        code.push(OP_LOCAL_GET);
        write_u32(code, dtor_local);
        code.push(OP_CALL_INDIRECT);
        write_u32(code, self.t_i32_v);
        write_u32(code, 0);
        code.push(OP_END);

        code.push(OP_LOCAL_GET);
        write_u32(code, exn_ptr_local);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);

        code.push(OP_END);
    }
}
