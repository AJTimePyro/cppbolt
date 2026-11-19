//! Transactional state for class-template instantiation.
//!
//! Instantiation writes layouts, aliases, globals and synthesized
//! declarations while it builds a specialization, and recursive references
//! to the same specialization must see those in-progress writes. A failed
//! instantiation therefore cannot simply defer every write until the end;
//! instead it snapshots the state it mutates and restores it on error.
//!
//! The snapshot covers state written from `resolve_instantiation_target`
//! onward. Side effects of alias-template expansion before that point are
//! not undone.
//! shortcut: clones every mutation-bearing map on each class instantiation (~6% slower on heavy stdlib compiles); replace with a write journal if compile time ever matters.

use super::*;

pub(super) struct InstantiationSnapshot {
    struct_layouts: HashMap<String, StructLayout>,
    struct_order: Vec<String>,
    static_const_values: HashMap<String, i128>,
    struct_sizes: HashMap<String, usize>,
    struct_aligns: HashMap<String, usize>,
    global_vars: HashMap<String, VarSymbol>,
    functions: HashMap<String, Vec<FunctionSig>>,
    hidden_friends: HashMap<String, Vec<FunctionSig>>,
    hidden_friend_templates: HashMap<String, Vec<(FunctionDecl, Vec<TemplateParam>)>>,
    type_aliases: HashMap<String, Type>,
    synthesized_decls: Vec<Decl>,
    class_templates: HashMap<String, (TemplateClass, Vec<TemplateParam>)>,
    variable_templates: HashMap<
        String,
        (
            Vec<TemplateParam>,
            crate::ast::types::Type,
            Option<crate::ast::expr::Expr>,
        ),
    >,
    instantiated_variable_templates: HashMap<String, (crate::ast::types::Type, Option<i128>)>,
    instantiated_classes: HashSet<String>,
    instantiated_functions: HashSet<String>,
    pending_template_methods: HashMap<String, (String, FunctionDecl)>,
    deferred_methods_len: usize,
    member_templates: HashMap<String, Vec<FunctionDecl>>,
    structs_with_dtor: HashSet<String>,
    structs_with_move: HashSet<String>,
    range_for_counter: usize,
    lambda_counter: usize,
}

impl InstantiationSnapshot {
    pub(super) fn capture(sema: &Sema) -> Self {
        Self {
            struct_layouts: sema.struct_layouts.clone(),
            struct_order: sema.struct_order.clone(),
            static_const_values: sema.static_const_values.clone(),
            struct_sizes: sema.struct_sizes.clone(),
            struct_aligns: sema.struct_aligns.clone(),
            global_vars: sema.global_vars.clone(),
            functions: sema.functions.clone(),
            hidden_friends: sema.hidden_friends.clone(),
            hidden_friend_templates: sema.hidden_friend_templates.clone(),
            type_aliases: sema.type_aliases.clone(),
            synthesized_decls: sema.synthesized_decls.clone(),
            class_templates: sema.class_templates.clone(),
            variable_templates: sema.variable_templates.clone(),
            instantiated_variable_templates: sema.instantiated_variable_templates.clone(),
            instantiated_classes: sema.instantiated_classes.clone(),
            instantiated_functions: sema.instantiated_functions.clone(),
            pending_template_methods: sema.pending_template_methods.clone(),
            deferred_methods_len: sema.deferred_methods.len(),
            member_templates: sema.member_templates.clone(),
            structs_with_dtor: sema.structs_with_dtor.clone(),
            structs_with_move: sema.structs_with_move.clone(),
            range_for_counter: sema.range_for_counter,
            lambda_counter: sema.lambda_counter,
        }
    }

    pub(super) fn restore(self, sema: &mut Sema) {
        sema.struct_layouts = self.struct_layouts;
        sema.struct_order = self.struct_order;
        sema.static_const_values = self.static_const_values;
        sema.struct_sizes = self.struct_sizes;
        sema.struct_aligns = self.struct_aligns;
        sema.global_vars = self.global_vars;
        sema.functions = self.functions;
        sema.hidden_friends = self.hidden_friends;
        sema.hidden_friend_templates = self.hidden_friend_templates;
        sema.type_aliases = self.type_aliases;
        sema.synthesized_decls = self.synthesized_decls;
        sema.class_templates = self.class_templates;
        sema.variable_templates = self.variable_templates;
        sema.instantiated_variable_templates = self.instantiated_variable_templates;
        sema.instantiated_classes = self.instantiated_classes;
        sema.instantiated_functions = self.instantiated_functions;
        sema.pending_template_methods = self.pending_template_methods;
        sema.deferred_methods.truncate(self.deferred_methods_len);
        sema.member_templates = self.member_templates;
        sema.structs_with_dtor = self.structs_with_dtor;
        sema.structs_with_move = self.structs_with_move;
        sema.range_for_counter = self.range_for_counter;
        sema.lambda_counter = self.lambda_counter;
    }
}
