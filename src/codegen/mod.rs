//! Direct WebAssembly lowering of the typed AST.

mod analysis;
mod call_args;
mod cond;
mod construct;
mod expr;
mod expr_assign;
mod expr_binary;
mod expr_call;
mod expr_identifier;
mod expr_new;
mod expr_unary;
mod function;
mod function_lookup;
mod lvalue;
mod memory;
mod method_lookup;
mod program;
mod program_emit;
mod program_prepare;
mod raii;
mod stmt;
mod stmt_loop;
mod stmt_return;
mod stmt_var;
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
    collect_address_taken_stmt, collect_static_vars, collect_stmt_calls,
};
use program::Plan;
use std::collections::{HashMap, HashSet};

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
}

impl<'a> Codegen<'a> {
    pub fn new(sema: &'a Sema) -> Self {
        let mut module = WasmModule::new();

        // Type 0: (i32) -> void (print_char)
        let _t_i32_v = module.add_type(vec![TYPE_I32], vec![]);
        // Type 1: (i64, i32, i32, i32, i32) -> void (print_int)
        let t_print_int = module.add_type(
            vec![TYPE_I64, TYPE_I32, TYPE_I32, TYPE_I32, TYPE_I32],
            vec![],
        );
        // Type 2: (f64, i32, i32, i32, i32) -> void (print_double)
        let t_print_double = module.add_type(
            vec![TYPE_F64, TYPE_I32, TYPE_I32, TYPE_I32, TYPE_I32],
            vec![],
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
        }
    }
}

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
    pub scope_objects: Vec<Vec<(String, Type, u32)>>,
    pub temp_objects: Vec<(String, u32, bool)>,
    pub current_class: Option<String>,
    pub current_func_name: String,
    pub is_dtor: bool,
    pub is_reference_return: bool,
    pub is_main: bool,
}
