//! Semantic state: the `Sema` struct and alias visibility snapshots.

use super::*;
use std::collections::{HashMap, HashSet};

/// Function, using and variable visibility captured at a namespace alias
/// declaration. `None` sets mean "unrestricted", used outside alias contexts.
#[derive(Clone, Debug, Default)]
pub struct AliasVisibility {
    pub functions: HashSet<String>,
    pub usings: HashSet<String>,
    pub namespaces: Vec<String>,
    pub variables: Option<HashSet<String>>,
    pub using_targets: Option<HashMap<String, String>>,
}

#[derive(Clone)]
pub struct Sema {
    /// Stable IDs for function and member-pointer exception types, assigned on
    /// first use because those types have no separate declaration registry.
    pub aux_exception_type_ids: std::cell::RefCell<HashMap<String, u32>>,
    pub struct_layouts: HashMap<String, StructLayout>,
    pub struct_order: Vec<String>,
    pub enum_values: HashMap<String, i64>,
    /// Constant values of instantiated static const data members, keyed by
    /// their fully qualified name. Used when such a member is a non-type
    /// template argument (e.g. `is_integral<T>::value`).
    pub static_const_values: HashMap<String, i128>,
    pub enum_types: HashMap<String, String>,
    pub scoped_enums: HashSet<String>,
    /// Source names of functions declared `= delete`.
    pub deleted_functions: HashSet<String>,
    pub functions: HashMap<String, Vec<FunctionSig>>,
    /// Friend functions defined in a class, keyed by that class's name.
    /// Found only through ADL, never through ordinary lookup.
    pub hidden_friends: HashMap<String, Vec<FunctionSig>>,
    /// Friend function templates of concrete class-template specializations,
    /// keyed by the specialization they are associated with. Found only
    /// through ADL, never by ordinary unqualified lookup.
    pub hidden_friend_templates: HashMap<String, Vec<(FunctionDecl, Vec<TemplateParam>)>>,
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
    pub try_depth: u32,
    pub var_try_depths: HashMap<String, u32>,
    pub current_class: Option<String>,
    pub current_func_name: Option<String>,
    pub open_namespaces: Vec<String>,
    pub using_decls: HashMap<String, String>,
    pub namespace_aliases: HashMap<String, String>,
    pub type_aliases: HashMap<String, Type>,
    pub alias_visibility: HashMap<String, AliasVisibility>,
    pub class_field_visibility: HashMap<String, AliasVisibility>,
    pub active_alias_visibility: Option<AliasVisibility>,
    /// Declaration-position contexts of `decltype` aliases, keyed by the
    /// alias name so re-resolution keeps the original bindings.
    pub decltype_alias_context: HashMap<String, AliasVisibility>,
    /// True while confirming a partial specialization: nested-type access and
    /// ambiguity failures are substitution failures in this immediate context.
    pub in_substitution_context: bool,
    /// Whether the analysis is inside the initializer of a variable.
    ///
    /// A constructor call in an initializer reports an initialization
    /// diagnostic, while a constructor call in an expression reports a call
    /// diagnostic.
    pub in_initializer: bool,
    /// True while an expression is being analyzed on a disposable clone to
    /// decide a template candidate's viability.
    pub speculating: bool,
    /// Set when a nested instantiation fails during speculative analysis, so
    /// the failure stays hard instead of rejecting the candidate.
    pub speculative_hard: bool,
    /// Parameters of the signature being formed, bound when resolving a
    /// `decltype` operand that names one of them.
    pub signature_params: Vec<Param>,
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
    /// The definition selected when each class-template specialization was
    /// instantiated. Nested-member validation reads this instead of selecting
    /// the specialization a second time.
    pub instantiated_class_definitions: HashMap<String, TemplateClass>,
    pub function_templates: HashMap<String, Vec<(FunctionDecl, Vec<TemplateParam>)>>,
    /// Function overloads visible when each function template was defined,
    /// keyed by `name@line:col`. Ordinary lookup inside an instantiated body
    /// only sees these, so a later declaration cannot rebind a template name.
    pub template_definition_functions: HashMap<String, HashSet<String>>,
    /// Dependent call and operator expression sites inside each template, by
    /// source position. Substitution preserves positions, so classification
    /// made on the definition AST still applies to rewritten bodies.
    pub dependent_calls: HashMap<String, HashSet<(usize, usize)>>,
    pub template_definition_usings: HashMap<String, HashSet<String>>,
    pub template_definition_namespaces: HashMap<String, Vec<String>>,
    /// Identity of the template whose body (or signature) is being analyzed.
    /// Set only while re-analyzing substituted template code.
    pub current_template_identity: Option<String>,
    pub alias_templates: HashMap<String, (Vec<TemplateParam>, Type)>,
    pub variable_templates: HashMap<String, (Vec<TemplateParam>, Type, Option<Expr>)>,
    /// Instantiated variable template specializations: rendered name ->
    /// (type, constant value). Registers each specialization once.
    pub instantiated_variable_templates: HashMap<String, (Type, Option<i128>)>,
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
    pub(super) depth: crate::depth::DepthCounters,
}

impl Sema {
    pub(super) fn with_try_depth<T>(&mut self, depth: u32, f: impl FnOnce(&mut Self) -> T) -> T {
        let old = self.try_depth;
        self.try_depth = depth;
        let result = f(self);
        self.try_depth = old;
        result
    }
}
