//! Direct WebAssembly lowering of the typed AST.

mod analysis;
mod call_args;
mod call_types;
mod cond;
mod construct;
mod emit_context;
mod expr;
mod expr_assign;
mod expr_binary;
mod expr_binary_operator;
mod expr_call;
mod expr_call_member;
mod expr_delete;
mod expr_identifier;
mod expr_new;
mod expr_printf;
mod expr_unary;
mod function;
mod function_context;
mod function_global_init;
mod function_lifecycle;
mod function_lookup;
mod lvalue;
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
mod stmt_var;
mod stmt_var_init;
mod strings;
mod type_of;

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
    collect_address_taken_stmt, collect_static_vars, collect_stmt_calls, collect_type_lifecycle,
};
use emit_context::{
    ArrayLocal, ArraySlots, BinaryTypes, CallSite, CtorCall, GlobalInit, LocalDecl, MemberAccess,
    NamedCall, ScalarAssign, StructInit, SwitchSlots,
};
use program::Plan;
use std::collections::{HashMap, HashSet};

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
    pub vtable_offsets: HashMap<String, u32>,
    pub vtable_func_indices: Vec<u32>,
    pub t_i32_v: u32,
    pub global_offsets: HashMap<String, u32>,
    pub global_inits: Vec<(String, Type, Option<Expr>)>,
    pub global_dtors: Vec<(String, u32)>,
    pub func_table_indices: HashMap<String, u32>,
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
    /// Objects to destroy at scope exit, in construction order.
    pub scope_objects: Vec<Vec<(String, Type, u32)>>,
    /// Statement temporaries: `(class, frame slot, needs destructor)`.
    pub temp_objects: Vec<(String, u32, bool)>,
    pub current_class: Option<String>,
    pub current_func_name: String,
    pub return_type: Type,
    pub is_dtor: bool,
    pub is_reference_return: bool,
    pub is_main: bool,
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
            .ok_or_else(frame_too_large)?;
        self.next_local_offset = offset.checked_add(size).ok_or_else(frame_too_large)?;
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
        let value = u32::try_from(self.frame_size).map_err(|_| frame_too_large())?;
        for &position in &self.frame_size_patches {
            let Some(immediate) = code.get_mut(position..position.saturating_add(5)) else {
                return Err(crate::internal_error!(
                    0,
                    0,
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

fn frame_too_large() -> CompileError {
    CompileError::new("stack frame is too large", 1, 1)
        .with_kind(crate::diagnostics::ErrorKind::Internal)
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
mod tests {
    use super::*;

    fn context() -> FunctionContext {
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
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            static_var_offsets: HashMap::new(),
            next_local_offset: 16,
            frame_size: 0,
            frame_size_patches: Vec::new(),
            var_scopes: vec![Vec::new()],
            scope_objects: vec![Vec::new()],
            temp_objects: Vec::new(),
            current_class: None,
            current_func_name: "test".to_string(),
            return_type: Type::Void,
            is_dtor: false,
            is_reference_return: false,
            is_main: false,
        }
    }

    #[test]
    fn frame_allocations_align_and_reject_overflow() {
        let mut ctx = context();
        assert_eq!(ctx.alloc_frame(1, 4), Ok(16));
        assert_eq!(ctx.alloc_frame(8, 8), Ok(24));
        assert_eq!(ctx.next_local_offset, 32);

        ctx.next_local_offset = u32::MAX;
        assert!(ctx.alloc_frame(4, 4).is_err());
    }

    #[test]
    fn leaving_a_scope_restores_shadowed_names() {
        let mut ctx = context();
        ctx.bind_local("value".to_string(), 16, Type::Int);
        ctx.enter_scope();
        ctx.bind_local("value".to_string(), 20, Type::Double);
        ctx.bind_static("static_value".to_string(), 4096, Type::Int);
        assert_eq!(ctx.var_offsets["value"], 20);
        assert_eq!(ctx.var_types["value"], Type::Double);
        assert!(ctx.static_var_offsets.contains_key("static_value"));

        ctx.exit_scope();
        assert_eq!(ctx.var_offsets["value"], 16);
        assert_eq!(ctx.var_types["value"], Type::Int);
        assert!(!ctx.static_var_offsets.contains_key("static_value"));
    }

    #[test]
    fn frame_size_patches_are_fixed_width_positive_leb128() {
        let mut ctx = context();
        ctx.frame_size = 1024;
        let mut code = Vec::new();
        ctx.emit_frame_size(&mut code);
        assert!(ctx.patch_frame_sizes(&mut code).is_ok());
        assert_eq!(code, [OP_I32_CONST, 0x80, 0x88, 0x80, 0x80, 0x00]);
    }

    #[test]
    fn integer_narrowing_masks_unsigned_and_sign_extends_signed() {
        let sema = Sema::new();
        let codegen = Codegen::new(&sema);

        let mut code = Vec::new();
        codegen.emit_conversion(&Type::Int, &Type::UnsignedChar, &mut code);
        assert_eq!(code.last(), Some(&OP_I32_AND));

        let mut code = Vec::new();
        codegen.emit_conversion(&Type::LongLong, &Type::UnsignedShort, &mut code);
        assert_eq!(code.first(), Some(&OP_I32_WRAP_I64));
        assert_eq!(code.last(), Some(&OP_I32_AND));

        let mut code = Vec::new();
        codegen.emit_conversion(&Type::Int, &Type::SignedChar, &mut code);
        assert_eq!(code, [OP_I32_EXTEND8_S]);

        let mut code = Vec::new();
        codegen.emit_conversion(&Type::Int, &Type::Short, &mut code);
        assert_eq!(code, [OP_I32_EXTEND16_S]);
    }

    #[test]
    fn array_subscripts_wrap_64_bit_indices() {
        let sema = Sema::new();
        let mut codegen = Codegen::new(&sema);
        let mut ctx = context();
        ctx.bind_local(
            "values".to_string(),
            16,
            Type::Array(Box::new(Type::Int), 8),
        );
        let expr = Expr::ArraySubscript {
            array: Box::new(Expr::Identifier("values".to_string())),
            index: Box::new(Expr::IntLiteral(3, Type::LongLong)),
        };
        let mut code = Vec::new();
        assert!(
            codegen
                .compile_lvalue_addr(&expr, &mut ctx, &mut code)
                .is_ok()
        );
        assert!(code.contains(&OP_I32_WRAP_I64));
    }
}
