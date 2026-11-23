//! Lowering helpers for function-level and scope-level exception unwinding.

use super::*;
use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::types::Type;
use crate::wasm::write_i64;

impl<'a> Codegen<'a> {
    pub(super) fn emit_mark_cleanup_active(
        &self,
        idx: usize,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.cleanup_mask_local);
        code.push(OP_I64_CONST);
        write_i64(code, 1_i64 << idx);
        code.push(OP_I64_OR);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.cleanup_mask_local);
    }

    pub(super) fn emit_mark_cleanup_inactive(
        &self,
        idx: usize,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.cleanup_mask_local);
        code.push(OP_I64_CONST);
        write_i64(code, !(1_i64 << idx));
        code.push(OP_I64_AND);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.cleanup_mask_local);
    }

    fn emit_cleanup_entry_raw(
        &self,
        entry: &function_context::CleanupEntry,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.cleanup_mask_local);
        code.push(OP_I64_CONST);
        write_i64(code, 1_i64 << entry.index);
        code.push(OP_I64_AND);
        code.push(OP_I64_EQZ);
        code.push(OP_I32_EQZ);
        code.push(OP_IF);
        code.push(0x40);

        if entry.is_temp {
            self.emit_destroy_and_free_temp(entry.offset, entry.dtor_idx, ctx, code);
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, if entry.is_subobject { 0 } else { ctx.fp_local });
            if entry.offset != 0 {
                code.push(OP_I32_CONST);
                write_i32(code, entry.offset as i32);
                code.push(OP_I32_ADD);
            }
            code.push(OP_CALL);
            write_u32(code, entry.dtor_idx);
        }

        self.emit_mark_cleanup_inactive(entry.index, ctx, code);

        code.push(OP_END);
    }

    pub(super) fn emit_cleanup_range(
        &self,
        start: usize,
        end: usize,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        if end > ctx.cleanup_entries.len() {
            return;
        }
        let has_prior_temps = ctx.cleanup_entries[..start.min(ctx.cleanup_entries.len())]
            .iter()
            .any(|e| e.is_temp);
        if start >= end && !has_prior_temps {
            return;
        }

        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_TRY_TABLE);
        code.push(0x40);
        code.push(0x01);
        code.push(0x02); // catch_all
        write_u32(code, 0);

        if start < end {
            for entry in ctx.cleanup_entries[start..end].iter().rev() {
                self.emit_cleanup_entry_raw(entry, ctx, code);
            }
        }
        for entry in ctx.cleanup_entries[..start.min(ctx.cleanup_entries.len())]
            .iter()
            .rev()
        {
            if entry.is_temp {
                self.emit_cleanup_entry_raw(entry, ctx, code);
            }
        }

        code.push(OP_BR);
        write_u32(code, 2);

        code.push(OP_END);
        self.emit_terminate_call(code);
        code.push(OP_END);
        self.emit_terminate_call(code);
        code.push(OP_END);
    }

    pub(super) fn emit_function_unwind_handler(
        &self,
        is_noexcept: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.unwind_exnref_local);

        self.emit_cleanup_range(0, ctx.cleanup_entries.len(), ctx, code);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        ctx.emit_frame_size(code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        // [except.terminate]: an exception escaping main calls std::terminate.
        if is_noexcept || ctx.is_main {
            self.emit_terminate_call(code);
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.unwind_exnref_local);
            code.push(OP_THROW_REF);
        }
    }
}

fn expr_can_throw_or_call(expr: &Expr) -> bool {
    let mut stack = vec![expr];
    while let Some(e) = stack.pop() {
        match e {
            Expr::Call { .. } | Expr::New { .. } | Expr::Delete { .. } => return true,
            Expr::Binary { left, right, .. } => stack.extend([left.as_ref(), right.as_ref()]),
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => stack.push(expr),
            Expr::MemberAccess { object, .. } => stack.push(object),
            Expr::ArraySubscript { array, index, .. } => {
                stack.extend([array.as_ref(), index.as_ref()]);
            }
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                stack.extend(elements.iter());
            }
            Expr::PackExpansion { pattern, .. } => stack.push(pattern),
            Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
                stack.extend([target.as_ref(), value.as_ref()]);
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                stack.extend([cond.as_ref(), then_expr.as_ref(), else_expr.as_ref()]);
            }
            _ => {}
        }
    }
    false
}

pub(super) fn stmt_can_throw_or_call(stmt: &Stmt) -> bool {
    let mut stack = vec![stmt];
    while let Some(s) = stack.pop() {
        match &s.kind {
            StmtKind::Throw(..) | StmtKind::Try { .. } => return true,
            StmtKind::Expr(e) => {
                if expr_can_throw_or_call(e) {
                    return true;
                }
            }
            StmtKind::VarDecl { ty, init, .. } => {
                if matches!(ty.strip_const(), Type::Struct(_) | Type::Array(..)) {
                    return true;
                }
                if init.as_ref().map(expr_can_throw_or_call).unwrap_or(false) {
                    return true;
                }
            }
            StmtKind::VarDeclGroup(stmts) | StmtKind::Compound(stmts) => {
                stack.extend(stmts.iter());
            }
            StmtKind::If { branches, else_stmt } => {
                for (cond, then_s) in branches {
                    if expr_can_throw_or_call(cond) {
                        return true;
                    }
                    stack.push(then_s);
                }
                if let Some(e) = else_stmt {
                    stack.push(e);
                }
            }
            StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
                if expr_can_throw_or_call(cond) {
                    return true;
                }
                stack.push(body);
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(init_s) = init {
                    stack.push(init_s);
                }
                if cond.as_ref().map(expr_can_throw_or_call).unwrap_or(false)
                    || step.as_ref().map(expr_can_throw_or_call).unwrap_or(false)
                {
                    return true;
                }
                stack.push(body);
            }
            StmtKind::Switch { expr, body } => {
                if expr_can_throw_or_call(expr) {
                    return true;
                }
                stack.push(body);
            }
            StmtKind::Return(e) if e.as_ref().map(expr_can_throw_or_call).unwrap_or(false) => {
                return true;
            }
            _ => {}
        }
    }
    false
}
