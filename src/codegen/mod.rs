//! Direct WebAssembly lowering of the typed AST (see mdBook internals/wasm-codegen.md).

mod analysis;
mod analysis_vars;
mod call_args;
mod call_types;
mod cond;
mod construct;
mod emit_context;
mod exception_can_catch;
mod expr;
mod expr_assign;
mod expr_binary;
mod expr_binary_operator;
mod expr_call;
mod expr_call_member;
mod expr_call_member_support;
mod expr_delete;
mod expr_identifier;
mod expr_new;
mod expr_new_array;
mod expr_printf;
mod expr_unary;
mod function;
mod function_context;
mod function_global_init;
mod function_lifecycle;
mod function_lifecycle_init;
mod function_lookup;
mod initializer_list;
mod lvalue;
mod member_pointer;
mod member_pointer_call;
mod member_pointer_receiver;
mod member_pointer_types;
mod memory;
mod method_lookup;
mod overload;
mod program;
mod program_emit;
mod program_prepare;
mod raii;
mod stmt;
mod stmt_loop;
mod stmt_return;
mod stmt_throw;
mod stmt_try;
mod stmt_var;
mod stmt_var_init;
mod strings;
mod type_of;
mod unwind;

use crate::ast::types::{Type, WasmType};
use crate::ast::*;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use crate::runtime::memory_layout;
use crate::runtime::*;
use crate::sema::{FunctionSig, Sema};
use crate::wasm::*;
use analysis::{
    FuncCandidate, FuncDeclRef, check_stmt_has_new_delete, collect_address_taken_expr,
    collect_address_taken_stmt, collect_expr_calls, collect_stmt_calls, collect_type_lifecycle,
};
use analysis_vars::{check_stmt_has_try_throw, collect_static_vars};
use emit_context::{
    ArrayLocal, ArrayLoop, ArraySlots, BinaryTypes, CallSite, CtorCall, GlobalInit, MemberAccess,
    NamedCall, ScalarAssign, SwitchSlots,
};
pub(super) use member_pointer_types::{MemberPointerCall, MemberPointerCmp};
use program::Plan;
use std::collections::{HashMap, HashSet};

/// Decodes the `class@@mangled-name` marker sema writes onto the identifier of
/// a resolved `&Class::member` method address.
pub(crate) fn decode_member_pointer_name(name: &str) -> Option<(&str, &str)> {
    name.split_once("@@")
}

/// Mutable state for one compilation: the Wasm module, symbol offsets, and
/// the data/vtable/table bookkeeping shared by the lowering passes.
pub struct Codegen<'a> {
    pub max_memory_mb: u32,
    pub sema: &'a Sema,
    pub module: WasmModule,
    pub string_table: HashMap<String, u32>,
    pub next_data_offset: u32,
    pub func_indices: HashMap<String, u32>,
    pub func_printf: u32,
    pub loop_depths: Vec<(u32, u32, usize)>,
    pub current_block_depth: u32,
    /// Per class: `(vptr_offset_in_object, vtable_data_address)` for every
    /// polymorphic subobject of that class.
    pub vtable_offsets: HashMap<String, Vec<(u32, u32)>>,
    pub vtable_func_indices: Vec<u32>,
    pub t_i32_v: u32,
    pub global_offsets: HashMap<String, u32>,
    pub global_inits: Vec<(String, Type, Option<Expr>)>,
    pub global_dtors: Vec<(String, u32)>,
    pub func_table_indices: HashMap<String, u32>,
    pub dtor_table_indices: HashMap<String, u32>,
    /// The position of the statement being lowered.
    ///
    /// Lowering errors that cannot name an expression report this position,
    /// so a diagnostic never points at a fabricated line and column.
    pub current_line: usize,
    pub current_col: usize,
    pub current_scratch1: std::cell::Cell<u32>,
    pub current_scratch_i64: std::cell::Cell<u32>,
    depth: crate::depth::DepthCounters,
}

impl<'a> Codegen<'a> {
    pub fn new(sema: &'a Sema) -> Self {
        let mut module = WasmModule::new();

        // Type 0: (i32) -> void (print_char)
        let _t_i32_v = module.add_type(vec![TYPE_I32], vec![]);
        // Type 1: (i64, i32, i32, i32, i32) -> i32 (print_int)
        let t_print_int = module.add_type(
            vec![TYPE_I64, TYPE_I32, TYPE_I32, TYPE_I32, TYPE_I32],
            vec![TYPE_I32],
        );
        // Type 2: (f64, i32, i32, i32, i32) -> i32 (print_double)
        let t_print_double = module.add_type(
            vec![TYPE_F64, TYPE_I32, TYPE_I32, TYPE_I32, TYPE_I32],
            vec![TYPE_I32],
        );
        // Type 3: () -> i32 (read_char)
        let t_v_i32 = module.add_type(vec![], vec![TYPE_I32]);
        // Type 4: (i32) -> void (exit)
        let t_exit = module.add_type(vec![TYPE_I32], vec![]);
        // Tag 0: (i32) -> void (C++ exception tag)
        let t_tag = module.add_type(vec![TYPE_I32], vec![]);
        module.tags.push(t_tag);

        // Host imports
        module
            .imports
            .push(("env".to_string(), "print_char".to_string(), _t_i32_v));
        module
            .imports
            .push(("env".to_string(), "print_int".to_string(), t_print_int));
        module.imports.push((
            "env".to_string(),
            "print_double".to_string(),
            t_print_double,
        ));
        module
            .imports
            .push(("env".to_string(), "read_char".to_string(), t_v_i32));
        module
            .imports
            .push(("env".to_string(), "exit".to_string(), t_exit));

        let mut func_indices = HashMap::new();
        func_indices.insert("print_char".to_string(), 0);
        func_indices.insert("print_int".to_string(), 1);
        func_indices.insert("print_double".to_string(), 2);
        func_indices.insert("read_char".to_string(), 3);
        func_indices.insert("exit".to_string(), 4);
        let vtable_func_indices = Vec::new();
        let func_printf = 0;

        Self {
            sema,
            module,
            string_table: HashMap::new(),
            next_data_offset: memory_layout::DATA_BASE,
            func_indices,
            func_printf,
            loop_depths: Vec::new(),
            current_block_depth: 0,
            vtable_offsets: HashMap::new(),
            vtable_func_indices,
            t_i32_v: _t_i32_v,
            global_offsets: HashMap::new(),
            global_inits: Vec::new(),
            global_dtors: Vec::new(),
            func_table_indices: HashMap::new(),
            dtor_table_indices: HashMap::new(),
            current_line: 1,
            current_col: 1,
            current_scratch1: std::cell::Cell::new(0),
            current_scratch_i64: std::cell::Cell::new(0),
            max_memory_mb: 256,
            depth: crate::depth::DepthCounters::new(),
        }
    }
}

enum VarBinding {
    Local {
        name: String,
        previous_offset: Option<u32>,
        previous_type: Option<Type>,
    },
    Static {
        name: String,
        previous_offset: Option<(u32, Type)>,
        previous_type: Option<Type>,
    },
    TypeOnly {
        name: String,
        previous_type: Option<Type>,
    },
}

pub type ScopeObjectEntry = (String, Type, u32, Option<usize>);

/// Per-function lowering state.
///
/// Local frame offsets grow monotonically for the whole function, even across
/// nested scopes, so a variable keeps one address for its entire lifetime.
/// Shadowed names are restored by `exit_scope`.
pub struct FunctionContext {
    pub fp_local: u32,
    pub scratch1: u32,
    pub scratch2: u32,
    pub scratch_i64: u32,
    pub scratch_f64: u32,
    pub scratch_f32: u32,
    pub ret_temp_i32: u32,
    pub ret_temp_i64: u32,
    pub ret_temp_f64: u32,
    pub exnref_local: u32,
    pub unwind_exnref_local: u32,
    pub max_catch_depth: u32,
    pub cleanup_mask_local: u32,
    pub last_exn_val_local: u32,
    pub cleanup_entries: Vec<function_context::CleanupEntry>,
    pub in_catch_all: bool,
    pub catch_depth: usize,
    pub var_offsets: HashMap<String, u32>,
    pub var_types: HashMap<String, Type>,
    pub static_var_offsets: HashMap<String, (u32, Type)>,
    pub next_local_offset: u32,
    pub frame_size: i32,
    /// Byte positions of five-byte LEB placeholders for the frame size, used
    /// by both the prologue and epilogue.
    pub frame_size_patches: Vec<usize>,
    /// Bindings shadowed by each active block, innermost last.
    var_scopes: Vec<Vec<VarBinding>>,
    /// Objects to destroy at scope exit, in construction order: (name, ty, offset, cleanup_idx).
    pub scope_objects: Vec<Vec<ScopeObjectEntry>>,
    /// Statement temporaries: `(class, frame slot, needs destructor, cleanup_idx)`.
    pub temp_objects: Vec<(String, u32, bool, Option<usize>)>,
    /// Backing-array elements whose destructors run at full-expression end.
    pub initializer_list_temps: Vec<(String, u32)>,
    pub current_class: Option<String>,
    pub current_func_name: String,
    /// The position of the statement being lowered.
    pub line: usize,
    pub col: usize,
    pub return_type: Type,
    pub is_dtor: bool,
    pub is_reference_return: bool,
    pub is_main: bool,
}

impl Default for FunctionContext {
    fn default() -> Self {
        FunctionContext {
            fp_local: 0,
            scratch1: 1,
            scratch2: 2,
            scratch_i64: 3,
            scratch_f64: 4,
            scratch_f32: 5,
            ret_temp_i32: 6,
            ret_temp_i64: 7,
            ret_temp_f64: 8,
            exnref_local: 9,
            unwind_exnref_local: 9,
            max_catch_depth: 0,
            cleanup_mask_local: 10,
            last_exn_val_local: 11,
            cleanup_entries: Vec::new(),
            in_catch_all: false,
            catch_depth: 0,
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            static_var_offsets: HashMap::new(),
            next_local_offset: 16,
            frame_size: 0,
            frame_size_patches: Vec::new(),
            var_scopes: vec![Vec::new()],
            scope_objects: vec![Vec::new()],
            temp_objects: Vec::new(),
            initializer_list_temps: Vec::new(),
            current_class: None,
            current_func_name: String::new(),
            line: 0,
            col: 0,
            return_type: Type::Void,
            is_dtor: false,
            is_reference_return: false,
            is_main: false,
        }
    }
}

impl FunctionContext {
    pub fn enter_scope(&mut self) {
        self.var_scopes.push(Vec::new());
    }

    pub fn exit_scope(&mut self) {
        let Some(bindings) = self.var_scopes.pop() else {
            return;
        };
        for binding in bindings.into_iter().rev() {
            match binding {
                VarBinding::Local {
                    name,
                    previous_offset,
                    previous_type,
                } => {
                    restore_binding(&mut self.var_offsets, name.clone(), previous_offset);
                    restore_binding(&mut self.var_types, name, previous_type);
                }
                VarBinding::Static {
                    name,
                    previous_offset,
                    previous_type,
                } => {
                    restore_binding(&mut self.static_var_offsets, name.clone(), previous_offset);
                    restore_binding(&mut self.var_types, name, previous_type);
                }
                VarBinding::TypeOnly {
                    name,
                    previous_type,
                } => restore_binding(&mut self.var_types, name, previous_type),
            }
        }
    }

    pub fn bind_local(&mut self, name: String, offset: u32, ty: Type) {
        let previous_offset = self.var_offsets.insert(name.clone(), offset);
        let previous_type = self.var_types.insert(name.clone(), ty);
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.push(VarBinding::Local {
                name,
                previous_offset,
                previous_type,
            });
        }
    }

    pub fn bind_static(&mut self, name: String, offset: u32, ty: Type) {
        let previous_offset = self
            .static_var_offsets
            .insert(name.clone(), (offset, ty.clone()));
        let previous_type = self.var_types.insert(name.clone(), ty);
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.push(VarBinding::Static {
                name,
                previous_offset,
                previous_type,
            });
        }
    }

    pub fn bind_type(&mut self, name: String, ty: Type) {
        let previous_type = self.var_types.insert(name.clone(), ty);
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.push(VarBinding::TypeOnly {
                name,
                previous_type,
            });
        }
    }

    /// Allocates a checked, aligned slot in the current stack frame.
    pub fn alloc_frame(&mut self, size: u32, align: u32) -> Result<u32, CompileError> {
        let align = align.max(1);
        let offset = self
            .next_local_offset
            .checked_add(align - 1)
            .map(|value| value & !(align - 1))
            .ok_or_else(|| frame_too_large(self.line, self.col))?;
        self.next_local_offset = offset
            .checked_add(size)
            .ok_or_else(|| frame_too_large(self.line, self.col))?;
        Ok(offset)
    }

    /// Emits a fixed-width frame-size immediate that can be patched later.
    ///
    /// The frame size is only known after the body has allocated all locals,
    /// and each function may refer to it in both the prologue and epilogue.
    pub fn emit_frame_size(&mut self, code: &mut Vec<u8>) {
        code.push(OP_I32_CONST);
        self.frame_size_patches.push(code.len());
        code.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x00]);
    }

    /// Patches every emitted frame-size placeholder once the final size is known.
    pub fn patch_frame_sizes(&self, code: &mut [u8]) -> Result<(), CompileError> {
        let value =
            u32::try_from(self.frame_size).map_err(|_| frame_too_large(self.line, self.col))?;
        for &position in &self.frame_size_patches {
            let Some(immediate) = code.get_mut(position..position.saturating_add(5)) else {
                return Err(crate::internal_error!(
                    self.line,
                    self.col,
                    "invalid frame-size patch position"
                ));
            };
            write_fixed_u32(immediate, value);
        }
        Ok(())
    }
}

fn restore_binding<T>(bindings: &mut HashMap<String, T>, name: String, previous: Option<T>) {
    if let Some(previous) = previous {
        bindings.insert(name, previous);
    } else {
        bindings.remove(&name);
    }
}

/// Reports a stack frame that does not fit in the address space.
fn frame_too_large(line: usize, col: usize) -> CompileError {
    CompileError::new("stack frame is too large", line, col)
        .with_kind(crate::diagnostics::ErrorKind::Internal)
}

impl<'a> Codegen<'a> {
    /// The diagnostic for a broken internal invariant.
    ///
    /// The position is the statement being lowered, so the diagnostic never
    /// points at a fabricated line and column.
    pub(super) fn internal_error(&self, message: &str) -> CompileError {
        crate::internal_error!(self.current_line, self.current_col, "{message}")
    }
}

/// Writes a positive LEB128 number into exactly five bytes, padding with
/// continuation bits so the encoded length never changes.
fn write_fixed_u32(bytes: &mut [u8], value: u32) {
    let mut value = value;
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (value & 0x7f) as u8;
        value >>= 7;
        if index < 4 {
            *byte |= 0x80;
        }
    }
}

pub(super) fn promote_integer(ty: &Type) -> Type {
    ty.integer_promotion()
}

pub(super) fn arithmetic_common_type(left: &Type, right: &Type) -> Type {
    left.usual_arithmetic_conversion(right)
}

#[cfg(test)]
mod tests;
