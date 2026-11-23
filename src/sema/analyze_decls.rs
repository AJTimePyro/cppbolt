//! Declaration registration pass: builds symbol and visibility tables.

use super::*;

fn decl_position(decl: &Decl) -> Option<(usize, usize)> {
    match decl {
        Decl::Function(f) | Decl::FriendFunction { func: f, .. } => Some((f.line, f.col)),
        Decl::StructDecl { line, col, .. }
        | Decl::StaticAssert { line, col, .. }
        | Decl::GlobalVar { line, col, .. }
        | Decl::NamespaceAlias { line, col, .. }
        | Decl::UsingDecl { line, col, .. } => Some((*line, *col)),
        Decl::Template(t) => Some((t.line, t.col)),
        Decl::UsingNamespace { line, col, .. } => Some((*line, *col)),
        Decl::EnumDecl { .. } => None,
    }
}

/// Mutable state threaded through `register_declarations`. Namespace aliases
/// are snapshotted with the lookup context visible at their source position.
struct DeclSnapshot {
    aliases: Vec<(String, (usize, usize))>,
    globals: Vec<(String, (usize, usize))>,
    next_alias: usize,
    next_global: usize,
    visible_globals: HashSet<String>,
    visible_functions: HashSet<String>,
    visible_usings: HashSet<String>,
    visible_using_targets: HashMap<String, String>,
}

impl DeclSnapshot {
    fn new(sema: &Sema, program: &Program) -> Self {
        let mut aliases: Vec<(String, (usize, usize))> = program
            .namespace_type_aliases
            .iter()
            .map(|(name, pos)| (name.clone(), *pos))
            .collect();
        for decl in &program.declarations {
            if let Decl::Template(t) = decl
                && let TemplateKind::Alias(name, _) = &t.kind
            {
                aliases.push((name.clone(), (t.line, t.col)));
            }
        }
        aliases.sort_by_key(|entry| entry.1);
        let mut globals: Vec<(String, (usize, usize))> = program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                Decl::GlobalVar {
                    name, line, col, ..
                } => Some((name.clone(), (*line, *col))),
                _ => None,
            })
            .collect();
        globals.sort_by_key(|entry| entry.1);
        let visible_functions = sema
            .functions
            .values()
            .flatten()
            .map(|sig| sig.mangled_name.clone())
            .collect();
        Self {
            aliases,
            globals,
            next_alias: 0,
            next_global: 0,
            visible_globals: HashSet::new(),
            visible_functions,
            visible_usings: HashSet::new(),
            visible_using_targets: HashMap::new(),
        }
    }

    /// Snapshots the lookup context for every alias declared before `pos`.
    fn snapshot_aliases_before(&mut self, sema: &mut Sema, pos: (usize, usize)) {
        while self.next_alias < self.aliases.len() && self.aliases[self.next_alias].1 < pos {
            let alias_pos = self.aliases[self.next_alias].1;
            while self.next_global < self.globals.len()
                && self.globals[self.next_global].1 < alias_pos
            {
                self.visible_globals
                    .insert(self.globals[self.next_global].0.clone());
                self.next_global += 1;
            }
            let name = self.aliases[self.next_alias].0.clone();
            let mut namespaces = sema.open_namespaces.clone();
            let mut curr = name.as_str();
            while let Some((ns, _)) = curr.rsplit_once("::") {
                namespaces.push(ns.to_string());
                curr = ns;
            }
            sema.alias_visibility.insert(
                name,
                AliasVisibility {
                    functions: self.visible_functions.clone(),
                    usings: self.visible_usings.clone(),
                    namespaces,
                    variables: Some(self.visible_globals.clone()),
                    using_targets: Some(self.visible_using_targets.clone()),
                },
            );
            self.next_alias += 1;
        }
    }
}

impl Sema {
    pub(super) fn register_declarations(
        &mut self,
        program: &mut Program,
    ) -> Result<(), CompileError> {
        let mut state = DeclSnapshot::new(self, program);
        let mut defined_functions = HashSet::new();
        for decl in &mut program.declarations {
            if let Some(pos) = decl_position(decl) {
                state.snapshot_aliases_before(self, pos);
            }
            match decl {
                Decl::UsingNamespace { name: ns, .. } => {
                    let resolved = self.resolve_alias_in_qualified_name(ns);
                    self.open_namespaces.push(resolved);
                }
                Decl::UsingDecl { name, .. } => {
                    let resolved = self.resolve_alias_in_qualified_name(name);
                    let short_name = resolved.split("::").last().unwrap_or(&resolved).to_string();
                    state.visible_usings.insert(short_name.clone());
                    state
                        .visible_using_targets
                        .insert(short_name.clone(), resolved.clone());
                    self.using_decls.insert(short_name, resolved);
                }
                Decl::NamespaceAlias {
                    alias,
                    target,
                    line,
                    col,
                } => {
                    self.record_local_namespace_alias(alias, target, *line, *col)?;
                }
                Decl::Function(f) => {
                    self.handle_function_decl(f, &mut defined_functions, &mut state)?;
                }
                Decl::FriendFunction { class_name, func } => {
                    if let Some(replacement) =
                        self.handle_friend_decl(class_name, func, &mut state)?
                    {
                        *decl = replacement;
                    }
                }
                Decl::Template(t) => self.handle_template_decl(t, &mut state),
                Decl::StructDecl {
                    name,
                    fields,
                    methods,
                    static_fields,
                    ..
                } => self.handle_struct_decl(name, fields, methods, static_fields, &mut state)?,
                _ => {}
            }
        }
        state.snapshot_aliases_before(self, (usize::MAX, usize::MAX));
        Ok(())
    }

    fn handle_function_decl(
        &mut self,
        f: &mut FunctionDecl,
        defined_functions: &mut HashSet<String>,
        state: &mut DeclSnapshot,
    ) -> Result<(), CompileError> {
        if let Some(ref expr) = f.noexcept_expr {
            let val = crate::sema::constexpr_eval::eval_constexpr_with_sema(
                expr,
                self,
                Some(&self.static_const_values),
            )?;
            f.qualifiers.is_noexcept = val != 0;
        }
        self.resolve_and_instantiate_type(&mut f.return_type, f.line, f.col)?;
        for p in &mut f.params {
            self.resolve_and_instantiate_type(&mut p.ty, f.line, f.col)?;
        }
        let mangled = self.mangle_name(&f.name, &f.params);
        if f.body.is_some() && !defined_functions.insert(mangled.clone()) {
            return Err(CompileError::new(
                format!("Redefinition of function '{}'", f.name),
                f.line,
                f.col,
            ));
        }
        let sig = FunctionSig {
            qualifiers: f.qualifiers,
            name: f.name.clone(),
            mangled_name: mangled,
            return_type: f.return_type.clone(),
            param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
            default_args: f.params.iter().map(|p| p.default_val.clone()).collect(),
            is_static: false,
            is_explicit: f.is_explicit,
            ref_qualifier: f.ref_qualifier,
        };
        state.visible_functions.insert(sig.mangled_name.clone());
        self.register_function(sig);
        Ok(())
    }

    fn handle_friend_decl(
        &mut self,
        class_name: &str,
        func: &FunctionDecl,
        state: &mut DeclSnapshot,
    ) -> Result<Option<Decl>, CompileError> {
        if func.template_params.is_empty() {
            let (mangled, f) = self.register_friend_function(class_name, func.clone())?;
            state.visible_functions.insert(mangled);
            Ok(Some(Decl::Function(f)))
        } else {
            // A friend template has no body to emit until a call deduces its
            // parameters. Leave the declaration in place and expose it only
            // through ADL.
            self.register_hidden_friend_template(
                class_name,
                func.clone(),
                func.template_params.clone(),
            );
            Ok(None)
        }
    }

    /// Records the four definition-time binding snapshots shared by function
    /// templates, class-template methods and struct methods.
    fn snapshot_template_definition_bindings(
        &mut self,
        func: &FunctionDecl,
        template_params: &[TemplateParam],
        class_ctx: Option<&str>,
        state: &DeclSnapshot,
    ) {
        let identity =
            super::template_registry::function_template_identity(&func.name, func.line, func.col);
        let positions = super::template_call_dependence::dependent_call_positions(
            func,
            template_params,
            class_ctx,
        );
        self.dependent_calls.insert(identity.clone(), positions);
        self.template_definition_usings
            .insert(identity.clone(), state.visible_usings.clone());
        self.template_definition_namespaces
            .insert(identity.clone(), self.open_namespaces.clone());
        self.template_definition_functions
            .insert(identity, state.visible_functions.clone());
    }

    fn handle_template_decl(&mut self, t: &crate::ast::TemplateDecl, state: &mut DeclSnapshot) {
        match &t.kind {
            TemplateKind::Function(f) => {
                let identity =
                    super::template_registry::function_template_identity(&f.name, f.line, f.col);
                // A template can call itself, so it sees its own identity.
                state.visible_functions.insert(format!("T:{identity}"));
                self.snapshot_template_definition_bindings(f, &t.params, None, state);
            }
            TemplateKind::Class(c) => {
                for method in &c.methods {
                    self.snapshot_template_definition_bindings(
                        method,
                        &t.params,
                        Some(&c.name),
                        state,
                    );
                }
            }
            _ => {}
        }
    }

    fn handle_struct_decl(
        &mut self,
        name: &str,
        fields: &[StructFieldDecl],
        methods: &mut [FunctionDecl],
        static_fields: &[(String, Type, Option<Expr>)],
        state: &mut DeclSnapshot,
    ) -> Result<(), CompileError> {
        for method in methods.iter() {
            let identity = super::template_registry::function_template_identity(
                &method.name,
                method.line,
                method.col,
            );
            state.visible_functions.insert(format!("T:{identity}"));
            self.snapshot_template_definition_bindings(
                method,
                &method.template_params,
                None,
                state,
            );
        }
        self.register_struct_declarations(name, methods, static_fields)?;
        if fields.iter().any(|(_, _, init)| init.is_some()) {
            for m in methods.iter() {
                let qualified = format!("{}::{}", name, m.name);
                if let Some(sigs) = self.functions.get(&qualified) {
                    for sig in sigs {
                        state.visible_functions.insert(sig.mangled_name.clone());
                    }
                }
            }
            self.class_field_visibility.insert(
                name.to_string(),
                AliasVisibility {
                    functions: state.visible_functions.clone(),
                    usings: state.visible_usings.clone(),
                    namespaces: self.open_namespaces.clone(),
                    ..Default::default()
                },
            );
        }
        Ok(())
    }
}
