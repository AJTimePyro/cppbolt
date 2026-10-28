//! Semantic analysis: symbols, overloads, record layout, and type checks.
//!
//! See the mdBook guide at `docs/book/internals/semantic-analysis.md`.

mod analyze;
mod assign;
mod constructor;
mod expr;
mod expr_call;
mod expr_member;
mod expr_ops;
mod layout;
mod lookup;
mod overload;
mod stmt;
mod stmt_decl;
mod stmt_range_for;
mod stmt_structured_binding;

pub(crate) use layout::RecordLayout;
pub(crate) use stmt_decl::VarDecl;

use crate::ast::types::Type;
use crate::ast::*;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub offset: usize,
}

/// Physical layout of one record.
///
/// `fields` includes inherited fields flattened into the object, while
/// `base_offsets` records where each base subobject starts. `vtable_slots` is
/// in slot order and includes the virtual destructor's special slots.
#[derive(Debug, Clone)]
pub struct StructLayout {
    pub is_class: bool,
    pub base_classes: Vec<String>,
    pub base_offsets: Vec<(String, usize)>,
    pub fields: Vec<StructField>,
    pub size: usize,
    pub align: usize,
    pub has_vtable: bool,
    pub vtable_slots: Vec<String>,
    pub private_members: std::collections::HashSet<String>,
    pub is_abstract: bool,
    pub pure_virtuals: std::collections::HashSet<String>,
}

/// One overload after parameter adjustment.
///
/// For non-static methods, `param_types[0]` is the implicit `this` pointer.
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub mangled_name: String,
    pub return_type: Type,
    pub param_types: Vec<Type>,
    pub default_args: Vec<Option<Expr>>,
    pub is_static: bool,
    pub is_explicit: bool,
    pub ref_qualifier: RefQualifier,
}

/// The first argument of a call that does not convert to its parameter.
#[derive(Debug, Clone)]
pub(crate) struct ConversionFailure {
    /// The index of the argument, counted from zero.
    pub index: usize,
    /// The type of the argument.
    pub from: Type,
    /// The type of the parameter.
    pub to: Type,
}

/// Returns true when a type refers to a const-qualified object, looking
/// through references and pointers.
pub(crate) fn receiver_is_const_type(ty: &Type) -> bool {
    match ty {
        Type::Const(_) => true,
        Type::Reference(inner) | Type::Pointer(inner) => receiver_is_const_type(inner),
        _ => false,
    }
}

/// Returns true when a non-static method's implicit `this` is `const`.
pub(crate) fn sig_receiver_is_const(sig: &FunctionSig) -> bool {
    !sig.is_static
        && sig
            .param_types
            .first()
            .map(receiver_is_const_type)
            .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub ty: Type,
    pub is_global: bool,
    /// Offset from the local frame pointer for locals; unused for globals.
    pub stack_offset: i32,
}

pub struct Scope {
    pub vars: HashMap<String, VarSymbol>,
}

pub struct Sema {
    pub struct_layouts: HashMap<String, StructLayout>,
    pub struct_order: Vec<String>,
    pub enum_values: HashMap<String, i64>,
    pub enum_types: HashMap<String, String>,
    pub scoped_enums: HashSet<String>,
    pub functions: HashMap<String, Vec<FunctionSig>>,
    pub global_vars: HashMap<String, VarSymbol>,
    pub scopes: Vec<Scope>,
    pub current_func_return_type: Option<Type>,
    pub current_frame_size: i32,
    pub struct_sizes: HashMap<String, usize>,
    pub struct_aligns: HashMap<String, usize>,
    pub current_line: usize,
    pub current_col: usize,
    pub loop_depth: usize,
    pub switch_depth: usize,
    pub current_class: Option<String>,
    pub current_func_name: Option<String>,
    pub open_namespaces: Vec<String>,
    pub using_decls: HashMap<String, String>,
    /// Whether the analysis is inside the initializer of a variable.
    ///
    /// A constructor call in an initializer reports an initialization
    /// diagnostic, while a constructor call in an expression reports a call
    /// diagnostic.
    pub in_initializer: bool,
    pub range_for_counter: usize,
    depth: crate::depth::DepthCounters,
}

impl Sema {
    pub fn new() -> Self {
        let mut sema = Self {
            struct_layouts: HashMap::new(),
            struct_order: Vec::new(),
            enum_values: HashMap::new(),
            enum_types: HashMap::new(),
            scoped_enums: HashSet::new(),
            functions: HashMap::new(),
            global_vars: HashMap::new(),
            scopes: Vec::new(),
            current_func_return_type: None,
            current_frame_size: 0,
            struct_sizes: HashMap::new(),
            struct_aligns: HashMap::new(),
            current_line: 1,
            current_col: 1,
            loop_depth: 0,
            switch_depth: 0,
            range_for_counter: 0,
            current_class: None,
            current_func_name: None,
            open_namespaces: Vec::new(),
            using_decls: HashMap::new(),
            in_initializer: false,
            depth: crate::depth::DepthCounters::new(),
        };

        sema.register_builtins();
        sema
    }

    fn register_builtins(&mut self) {
        // Built-in printf: variadic, mangled as "printf"
        self.register_function(FunctionSig {
            name: "printf".to_string(),
            mangled_name: "printf".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Pointer(Box::new(Type::Char))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });
        self.register_function(FunctionSig {
            name: "std::printf".to_string(),
            mangled_name: "std::printf".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Pointer(Box::new(Type::Char))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        // Built-in malloc & free
        self.register_function(FunctionSig {
            name: "malloc".to_string(),
            mangled_name: "malloc".to_string(),
            return_type: Type::Pointer(Box::new(Type::Void)),
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        self.register_function(FunctionSig {
            name: "free".to_string(),
            mangled_name: "free".to_string(),
            return_type: Type::Void,
            param_types: vec![Type::Pointer(Box::new(Type::Void))],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });

        self.register_function(FunctionSig {
            name: "__builtin_wasm_memory_size".to_string(),
            mangled_name: "__builtin_wasm_memory_size".to_string(),
            return_type: Type::Int,
            param_types: vec![Type::Int],
            default_args: vec![None],
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        });
    }

    /// Registers an overload under its source name, mangled name, and any
    /// implicit `std::` alias so every spelling resolves to the same symbol.
    pub fn register_function(&mut self, sig: FunctionSig) {
        self.functions
            .entry(sig.name.clone())
            .or_default()
            .push(sig.clone());
        self.functions
            .entry(sig.mangled_name.clone())
            .or_default()
            .push(sig.clone());
        if let Some(alias) = QualifiedName::new(&sig.name).implicit_alias() {
            self.functions
                .entry(alias.into_owned())
                .or_default()
                .push(sig.clone());
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            vars: HashMap::new(),
        });
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn lookup_var(&self, name: &str) -> Option<&VarSymbol> {
        self.lookup_var_in_context(name, self.current_func_name.as_deref().unwrap_or(""))
    }

    pub fn lookup_var_in_context(&self, name: &str, func_name: &str) -> Option<&VarSymbol> {
        if let Some(stripped) = name.strip_prefix("::") {
            return self.global_vars.get(stripped);
        }
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var);
            }
        }
        let effective_func = func_name;
        if !effective_func.is_empty() {
            let mut curr = effective_func;
            while let Some((ns, _)) = curr.rsplit_once("::") {
                if let Some(var) = self.global_vars.get(&format!("{}::{}", ns, name)) {
                    return Some(var);
                }
                curr = ns;
            }
        }
        if let Some(var) = self.global_vars.get(name) {
            return Some(var);
        }
        if let Some(alias) = QualifiedName::new(name).implicit_alias()
            && let Some(var) = self.global_vars.get(alias.as_ref())
        {
            return Some(var);
        }
        if let Some(target) = self.using_decls.get(name)
            && let Some(var) = self.global_vars.get(target)
        {
            return Some(var);
        }
        for ns in &self.open_namespaces {
            if let Some(var) = self.global_vars.get(&format!("{}::{}", ns, name)) {
                return Some(var);
            }
        }
        None
    }
}
