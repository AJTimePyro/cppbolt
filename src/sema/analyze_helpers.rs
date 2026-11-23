//! Helpers for program analysis.

use super::analyze::this_param;
use super::*;

pub(super) fn method_matches_out_of_class(
    m: &FunctionDecl,
    short_method: &str,
    params: &[Param],
    is_const: bool,
) -> bool {
    (m.name == short_method || m.name == format!("~{}", short_method))
        && m.params.len() == params.len()
        && m.is_const == is_const
        && m.body.is_none()
        && m.params.iter().zip(params).all(|(p1, p2)| p1.ty == p2.ty)
}

/// Collects class names used as complete objects (never through a pointer or
/// reference). Only those odr-use a destructor; Clang leaves a specialization
/// reached solely through a pointer uninstantiated.
pub(super) fn collect_object_type(ty: &Type, out: &mut HashSet<String>) {
    match ty {
        Type::Struct(s) => {
            out.insert(s.clone());
        }
        Type::TemplateInstance { name, args } => {
            out.insert(format!(
                "{}<{}>",
                name,
                args.iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Type::Const(inner) | Type::Array(inner, _) => collect_object_type(inner, out),
        _ => {}
    }
}

pub(super) fn collect_object_expr(expr: &Expr, out: &mut HashSet<String>) {
    if let Expr::New { ty, .. } = expr {
        collect_object_type(ty, out);
    }
}

pub(super) fn collect_object_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::VarDecl { ty, init, .. } => {
            collect_object_type(ty, out);
            if let Some(e) = init {
                collect_object_expr(e, out);
            }
        }
        StmtKind::StructuredBinding { ty, init, .. } => {
            collect_object_type(ty, out);
            collect_object_expr(init, out);
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                collect_object_stmt(s, out);
            }
        }
        StmtKind::Expr(e) => collect_object_expr(e, out),
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (_, s) in branches {
                collect_object_stmt(s, out);
            }
            if let Some(s) = else_stmt {
                collect_object_stmt(s, out);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Switch { body, .. } => collect_object_stmt(body, out),
        StmtKind::RangeFor { decl, body, .. } => {
            collect_object_stmt(decl, out);
            collect_object_stmt(body, out);
        }
        _ => {}
    }
}

pub(super) fn collect_object_use_classes(program: &Program) -> HashSet<String> {
    let mut out = HashSet::new();
    for decl in &program.declarations {
        match decl {
            Decl::GlobalVar { ty, .. } => collect_object_type(ty, &mut out),
            Decl::Function(f) => {
                if let Some(body) = &f.body {
                    collect_object_stmt(body, &mut out);
                }
            }
            Decl::StructDecl {
                base_classes,
                fields,
                static_fields,
                ..
            } => {
                out.extend(base_classes.iter().cloned());
                for (_, ty, _) in fields {
                    collect_object_type(ty, &mut out);
                }
                for (_, ty, _) in static_fields {
                    collect_object_type(ty, &mut out);
                }
            }
            _ => {}
        }
    }
    out
}

impl Sema {
    /// Registers a friend function template for ADL on one class. The dedup
    /// keeps repeated declarations and instantiations from adding the same
    /// candidate twice.
    pub(super) fn register_hidden_friend_template(
        &mut self,
        class_name: &str,
        decl: FunctionDecl,
        params: Vec<TemplateParam>,
    ) {
        let plain = QualifiedName::new(class_name).plain().to_string();
        for key in [class_name.to_string(), plain] {
            let list = self.hidden_friend_templates.entry(key).or_default();
            if !list
                .iter()
                .any(|(d, _)| d.params == decl.params && d.return_type == decl.return_type)
            {
                list.push((decl.clone(), params.clone()));
            }
        }
    }

    /// Registers one friend function under its mangled name and associates it
    /// with the given class for ADL. Returns the mangled name and the resolved
    /// declaration so the caller can emit its body.
    pub(super) fn register_friend_function(
        &mut self,
        class_name: &str,
        mut func: FunctionDecl,
    ) -> Result<(String, FunctionDecl), CompileError> {
        if let Some(ref expr) = func.noexcept_expr {
            let val = crate::sema::constexpr_eval::eval_constexpr_with_sema(
                expr,
                self,
                Some(&self.static_const_values),
            )?;
            func.qualifiers.is_noexcept = val != 0;
        }
        self.resolve_and_instantiate_type(&mut func.return_type, func.line, func.col)?;
        for p in &mut func.params {
            self.resolve_and_instantiate_type(&mut p.ty, func.line, func.col)?;
        }
        let mangled = self.mangle_name(&func.name, &func.params);
        let sig = FunctionSig {
            qualifiers: func.qualifiers,
            name: func.name.clone(),
            mangled_name: mangled.clone(),
            return_type: func.return_type.clone(),
            param_types: func.params.iter().map(|p| p.ty.clone()).collect(),
            default_args: func.params.iter().map(|p| p.default_val.clone()).collect(),
            is_static: false,
            is_explicit: false,
            ref_qualifier: func.ref_qualifier,
        };
        // Only the mangled spelling is reachable by ordinary lookup; the
        // source name is reserved for ADL.
        self.functions
            .entry(mangled.clone())
            .or_default()
            .push(sig.clone());
        let plain = crate::QualifiedName::new(class_name).plain().to_string();
        let keys = if plain == class_name {
            vec![class_name.to_string()]
        } else {
            vec![class_name.to_string(), plain]
        };
        for key in keys {
            self.hidden_friends
                .entry(key)
                .or_default()
                .push(sig.clone());
        }
        Ok((mangled, func))
    }

    /// Registers the static fields and methods of a class.
    ///
    /// Non-template method signatures are completed like free function
    /// signatures, so template types in their parameter and return types are
    /// instantiated before overload lookup sees them.
    pub(super) fn register_struct_declarations(
        &mut self,
        name: &str,
        methods: &mut [FunctionDecl],
        static_fields: &[(String, Type, Option<Expr>)],
    ) -> Result<(), CompileError> {
        for sf in static_fields {
            let qual_name = format!("{}::{}", name, sf.0);
            let sym = VarSymbol {
                name: qual_name.clone(),
                ty: sf.1.clone(),
                is_global: true,
                stack_offset: 0,
            };
            self.global_vars.insert(qual_name.clone(), sym.clone());
            if let Some(alias) = QualifiedName::new(&qual_name).implicit_alias() {
                self.global_vars.insert(alias.to_string(), sym);
            }
        }
        let previous_class = self.current_class.replace(name.to_string());
        for m in methods.iter_mut() {
            if !m.template_params.is_empty() {
                self.member_templates
                    .entry(name.to_string())
                    .or_default()
                    .push(m.clone());
                let plain = QualifiedName::new(name).plain();
                if plain != name {
                    self.member_templates
                        .entry(plain.to_string())
                        .or_default()
                        .push(m.clone());
                }
                continue;
            }
            if let Some(ref expr) = m.noexcept_expr {
                let val = crate::sema::constexpr_eval::eval_constexpr_with_sema(
                    expr,
                    self,
                    Some(&self.static_const_values),
                )?;
                m.qualifiers.is_noexcept = val != 0;
            }
            self.resolve_and_instantiate_type(&mut m.return_type, m.line, m.col)?;
            for param in &mut m.params {
                self.resolve_and_instantiate_type(&mut param.ty, m.line, m.col)?;
            }
            self.register_struct_method(name, m);
        }
        self.current_class = previous_class;
        Ok(())
    }

    pub(super) fn register_struct_method(&mut self, class_name: &str, m: &FunctionDecl) {
        let mut params = m.params.clone();
        if !m.is_static {
            params.insert(0, this_param(class_name, m.is_const));
        }
        let qual_name = format!("{}::{}", class_name, m.name);
        let mut mangled = self.mangle_name(&qual_name, &params);
        match m.ref_qualifier {
            RefQualifier::LValue => mangled.push_str("_R"),
            RefQualifier::RValue => mangled.push_str("_O"),
            RefQualifier::None => {}
        }
        let mut quals = m.qualifiers;
        quals.is_const = m.is_const;
        let sig = FunctionSig {
            qualifiers: quals,
            name: m.name.clone(),
            mangled_name: mangled,
            return_type: m.return_type.clone(),
            param_types: params.iter().map(|p| p.ty.clone()).collect(),
            default_args: params.iter().map(|p| p.default_val.clone()).collect(),
            is_static: m.is_static,
            is_explicit: m.is_explicit,
            ref_qualifier: m.ref_qualifier,
        };
        self.register_function(sig.clone());
        self.functions
            .entry(qual_name)
            .or_default()
            .push(sig.clone());
        let short = QualifiedName::new(class_name).class_short_name();
        if m.name == class_name {
            let short_qual = format!("{}::{}", class_name, short);
            self.functions.entry(short_qual).or_default().push(sig);
        } else if m.name == format!("~{}", class_name) {
            let short_qual = format!("{}::~{}", class_name, short);
            self.functions.entry(short_qual).or_default().push(sig);
        }
    }
}
