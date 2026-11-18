//! Semantic analysis: symbols, overloads, record layout, and type checks.
//!
//! See the mdBook guide at `docs/book/internals/semantic-analysis.md`.

mod analyze;
mod analyze_field_init;
mod assign;
mod call_arity;
pub mod constexpr_eval;
mod constexpr_globals;
mod constructor;
mod expected_type;
mod expr;
mod expr_call;
mod expr_call_member;
mod expr_lambda;
mod expr_lambda_free_vars;
mod expr_member;
mod expr_ops;
mod expr_subscript;
pub mod fold_expansion;
#[cfg(test)]
mod fold_tests;
mod function_sig;
mod init;
mod init_aggregate;
mod init_list;
mod layout;
mod layout_dump;
mod lookup;
mod method_return_type;
mod virtual_key;
mod virtual_validate;
pub(super) use lookup::MemberPointerBaseConversion;
pub(crate) use virtual_key::{
    VirtualKey, find_sig, noexcept_compatible, return_adjustment, return_compatible,
};
mod member_pointer;
#[cfg(test)]
mod member_ptr_tests;
mod overload;
pub mod pack_expansion;
#[cfg(test)]
mod pack_expansion_tests;
mod receiver;
mod stmt;
mod stmt_decl;
mod stmt_range_for;
mod stmt_return;
mod stmt_structured_binding;
mod synthesize;
mod synthesize_ops;
mod template_alias;
pub mod template_arg_match;
mod template_deduce;
mod template_deduce_type;
mod template_func;
mod template_match;
mod template_name;
mod template_nested;
pub mod template_partial_order;
mod template_registry;
mod template_spec;
pub mod template_subst;
mod template_subst_align;
mod template_subst_array;
mod template_subst_expr_helpers;
mod template_subst_func;
mod template_subst_lambda;
mod template_subst_nontype;
mod template_subst_pack;
mod template_subst_stmt;
#[cfg(test)]
mod tests;
mod tests_pack;

pub(crate) use expected_type::is_member_pointer_address;
pub use init::InitContext;
pub(crate) use layout::RecordLayout;
#[cfg(test)]
pub use pack_expansion::TemplateArg;
pub(crate) use stmt_decl::VarDecl;

use crate::ast::types::Type;
use crate::ast::*;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
pub(crate) use function_sig::FunctionSig;
pub(crate) use layout::{VTableInfo, VTableSlotInfo};
pub(crate) use receiver::{receiver_is_const_type, sig_receiver_is_const};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub offset: usize,
    pub default_init: Option<Expr>,
}

/// Physical layout of one record.
///
/// `fields` includes inherited fields flattened into the object, while
/// `base_offsets` records where each base subobject starts. `vtable_slots` is
/// in slot order and includes the virtual destructor's special slots.
#[derive(Debug, Clone)]
pub struct StructLayout {
    pub is_class: bool,
    pub is_final: bool,
    pub has_user_declared_constructors: bool,
    pub base_classes: Vec<String>,
    pub base_access: Vec<crate::ast::BaseAccess>,
    pub base_offsets: Vec<(String, usize)>,
    /// Direct non-virtual bases in declaration order (virtual bases are
    /// unsupported), with their offsets in this class.
    pub direct_base_offsets: Vec<(String, usize)>,
    pub fields: Vec<StructField>,
    pub size: usize,
    pub align: usize,
    pub has_vtable: bool,
    pub vtable_slots: Vec<VirtualKey>,
    pub vtables: Vec<VTableInfo>,
    pub private_members: std::collections::HashSet<String>,
    pub is_abstract: bool,
    pub pure_virtuals: std::collections::HashSet<String>,
    pub final_methods: Vec<VirtualKey>,
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
    pub namespace_aliases: HashMap<String, String>,
    pub type_aliases: HashMap<String, Type>,
}

/// A virtual or destructor body whose early analysis hit a name that may be
/// declared later in the translation unit. Retried once registration is complete.
pub(crate) struct DeferredMethod {
    pub(crate) class_name: String,
    pub(crate) method: FunctionDecl,
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
    pub namespace_aliases: HashMap<String, String>,
    pub type_aliases: HashMap<String, Type>,
    /// Whether the analysis is inside the initializer of a variable.
    ///
    /// A constructor call in an initializer reports an initialization
    /// diagnostic, while a constructor call in an expression reports a call
    /// diagnostic.
    pub in_initializer: bool,
    /// Target type available at the current expression site (initializer,
    /// assignment, return, cast). Used to disambiguate overloaded
    /// `&Class::member` addresses.
    pub expected_type: Option<Type>,
    pub range_for_counter: usize,
    pub lambda_counter: usize,
    pub synthesized_decls: Vec<Decl>,
    pub current_func_is_const: bool,
    pub class_templates: HashMap<String, (TemplateClass, Vec<TemplateParam>)>,
    pub class_specializations: HashMap<String, TemplateClass>,
    pub partial_specializations: HashMap<String, Vec<template_match::PartialSpec>>,
    pub function_templates: HashMap<String, Vec<(FunctionDecl, Vec<TemplateParam>)>>,
    pub alias_templates: HashMap<String, (Vec<TemplateParam>, Type)>,
    pub instantiated_classes: HashSet<String>,
    pub instantiated_functions: HashSet<String>,
    pub pending_template_methods: HashMap<String, (String, FunctionDecl)>,
    pub(crate) deferred_methods: Vec<DeferredMethod>,
    /// Classes used as complete objects; only their bodies need checking.
    pub(crate) object_use_classes: HashSet<String>,
    pub member_templates: HashMap<String, Vec<FunctionDecl>>,
    pub instantiation_depth: usize,
    pub instantiation_stack: Vec<String>,
    pub structs_with_dtor: HashSet<String>,
    pub structs_with_move: HashSet<String>,
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
            lambda_counter: 0,
            synthesized_decls: Vec::new(),
            current_func_is_const: false,
            class_templates: HashMap::new(),
            class_specializations: HashMap::new(),
            partial_specializations: HashMap::new(),
            function_templates: HashMap::new(),
            alias_templates: HashMap::new(),
            instantiated_classes: HashSet::new(),
            instantiated_functions: HashSet::new(),
            pending_template_methods: HashMap::new(),
            deferred_methods: Vec::new(),
            object_use_classes: HashSet::new(),
            member_templates: HashMap::new(),
            instantiation_depth: 0,
            instantiation_stack: Vec::new(),
            structs_with_dtor: HashSet::new(),
            structs_with_move: HashSet::new(),
            current_class: None,
            current_func_name: None,
            open_namespaces: Vec::new(),
            using_decls: HashMap::new(),
            namespace_aliases: HashMap::new(),
            type_aliases: HashMap::new(),
            in_initializer: false,
            expected_type: None,
            depth: crate::depth::DepthCounters::new(),
        };

        sema.register_builtins();
        sema
    }

    fn register_builtins(&mut self) {
        // Built-in printf: variadic, mangled as "printf"
        self.register_function(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
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
            qualifiers: FunctionQualifiers::default(),
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
            qualifiers: FunctionQualifiers::default(),
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
            qualifiers: FunctionQualifiers::default(),
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
            qualifiers: FunctionQualifiers::default(),
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
            namespace_aliases: HashMap::new(),
            type_aliases: HashMap::new(),
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
        let resolved = self.resolve_alias_in_qualified_name(name);
        let name = resolved.as_str();
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

    pub fn record_local_namespace_alias(
        &mut self,
        alias: &str,
        target: &str,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let canonical = self.resolve_alias_in_qualified_name(target);
        let table = match self.scopes.last_mut() {
            Some(scope) => &mut scope.namespace_aliases,
            None => &mut self.namespace_aliases,
        };
        if let Some(existing) = table.get(alias)
            && existing != &canonical
        {
            return Err(CompileError::new(
                format!("redefinition of namespace alias '{}'", alias),
                line,
                col,
            ));
        }
        table.insert(alias.to_string(), canonical);
        Ok(())
    }

    pub fn lookup_namespace_alias(&self, name: &str) -> Option<&String> {
        for scope in self.scopes.iter().rev() {
            if let Some(target) = scope.namespace_aliases.get(name) {
                return Some(target);
            }
        }
        if let Some(target) = self.namespace_aliases.get(name) {
            return Some(target);
        }
        if let Some(func) = &self.current_func_name {
            let mut curr = func.as_str();
            while let Some((ns, _)) = curr.rsplit_once("::") {
                let qualified = format!("{}::{}", ns, name);
                if let Some(target) = self.namespace_aliases.get(&qualified) {
                    return Some(target);
                }
                curr = ns;
            }
        }
        for ns in &self.open_namespaces {
            let qualified = format!("{}::{}", ns, name);
            if let Some(target) = self.namespace_aliases.get(&qualified) {
                return Some(target);
            }
        }
        None
    }

    pub fn lookup_type_alias(&self, name: &str) -> Option<Type> {
        let plain = QualifiedName::new(name).plain();
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope
                .type_aliases
                .get(name)
                .or_else(|| scope.type_aliases.get(plain))
            {
                return Some(ty.clone());
            }
        }
        self.type_aliases
            .get(name)
            .or_else(|| self.type_aliases.get(plain))
            .cloned()
    }

    /// Resolves a source-spelled class name through namespace and type aliases
    /// to the canonical name used in layouts and symbol tables. Only names are
    /// resolved; offsets stay untouched, so distinct base subobjects remain distinct.
    /// When `context` names an enclosing class, aliases declared there (e.g.
    /// `Outer::Alias` used by `Outer::Inner`) are resolved as well.
    pub(crate) fn canonical_class_name(&self, name: &str, context: Option<&str>) -> String {
        let mut current = name.to_string();
        if let Some(ctx) = context {
            let mut scope = ctx.rsplit_once("::").map(|(s, _)| s);
            while let Some(s) = scope {
                let qualified = format!("{}::{}", s, name);
                if self.lookup_type_alias(&qualified).is_some() {
                    current = qualified;
                    break;
                }
                scope = s.rsplit_once("::").map(|(p, _)| p);
            }
        }
        for _ in 0..16 {
            let qualified = self.resolve_alias_in_qualified_name(&current);
            let next = match self.lookup_type_alias(&qualified) {
                Some(Type::Struct(s)) => s,
                Some(Type::TemplateInstance { name, args }) => format!(
                    "{}<{}>",
                    name,
                    args.iter()
                        .map(|a| a.to_c_type())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                _ => qualified,
            };
            if next == current {
                return current;
            }
            current = next;
        }
        current
    }

    /// Resolves leading namespace aliases in `name` to the canonical namespace.
    pub fn resolve_alias_in_qualified_name(&self, name: &str) -> String {
        let mut current = name.to_string();
        for _ in 0..16 {
            let mut matched = false;
            let parts: Vec<&str> = current.split("::").collect();
            for i in (1..parts.len()).rev() {
                let prefix = parts[..i].join("::");
                if let Some(target) = self.lookup_namespace_alias(&prefix) {
                    let suffix = parts[i..].join("::");
                    current = format!("{}::{}", target, suffix);
                    matched = true;
                    break;
                } else if let Some(target_ty) = self.lookup_type_alias(&prefix) {
                    let suffix = parts[i..].join("::");
                    current = format!("{}::{}", target_ty.to_c_type(), suffix);
                    matched = true;
                    break;
                }
            }
            if !matched && let Some(target) = self.lookup_namespace_alias(&current) {
                current = target.clone();
                matched = true;
            } else if !matched && let Some(target_ty) = self.lookup_type_alias(&current) {
                current = target_ty.to_c_type();
                matched = true;
            }
            if !matched {
                break;
            }
        }
        current
    }
}
