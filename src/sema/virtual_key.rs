//! Canonical identity of a virtual function.
//!
//! C++ virtual dispatch keys on the function's name (with destructors
//! normalized), its parameter types, and its cv/ref qualifiers. Return type,
//! `noexcept`, and `override`/`final` are not part of the identity; they are
//! validated separately when an override is declared.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VirtualKey {
    pub(crate) name: String,
    pub(crate) params: Vec<Type>,
    pub(crate) is_const: bool,
    pub(crate) is_volatile: bool,
    pub(crate) ref_qualifier: RefQualifier,
}

/// C++17 [dcl.fct]: top-level cv on a by-value parameter is ignored; array and
/// function parameters become pointers. Pointee qualification is preserved.
pub(crate) fn canonical_param(ty: &Type) -> Type {
    match ty {
        Type::Const(inner) => canonical_param(inner),
        Type::Array(elem, _) => Type::Pointer(Box::new(canonical_param(elem))),
        Type::Function { .. } => Type::Pointer(Box::new(ty.clone())),
        other => other.clone(),
    }
}

impl VirtualKey {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: normalize_name(name),
            params: Vec::new(),
            is_const: false,
            is_volatile: false,
            ref_qualifier: RefQualifier::None,
        }
    }

    pub(crate) fn from_decl(m: &FunctionDecl) -> Self {
        Self {
            name: normalize_name(&m.name),
            params: m.params.iter().map(|p| canonical_param(&p.ty)).collect(),
            is_const: m.is_const,
            is_volatile: m.qualifiers.is_volatile,
            ref_qualifier: m.ref_qualifier,
        }
    }

    pub(crate) fn from_sig(sig: &FunctionSig) -> Self {
        let params: Vec<Type> = if sig.is_static {
            sig.param_types.iter().map(canonical_param).collect()
        } else {
            sig.param_types
                .iter()
                .skip(1)
                .map(canonical_param)
                .collect()
        };
        Self {
            name: normalize_name(&sig.name),
            params,
            is_const: sig.qualifiers.is_const,
            is_volatile: sig.qualifiers.is_volatile,
            ref_qualifier: sig.ref_qualifier,
        }
    }

    pub(crate) fn is_destructor(&self) -> bool {
        self.name == "~"
    }

    pub(crate) fn matches_sig(&self, sig: &FunctionSig) -> bool {
        self == &Self::from_sig(sig)
    }
}

pub(crate) fn normalize_name(name: &str) -> String {
    if name.starts_with('~') {
        "~".to_string()
    } else {
        name.to_string()
    }
}

/// Finds a registered overload of `class_name::name` whose VirtualKey matches.
pub(crate) fn find_sig<'a>(
    sema: &'a Sema,
    class_name: &str,
    key: &VirtualKey,
) -> Option<&'a FunctionSig> {
    let qual1 = format!("{}::{}", class_name, key.name);
    let alt_class = crate::name::QualifiedName::new(class_name).alternate();
    let qual2 = format!("{}::{}", alt_class, key.name);
    for name in [
        qual1.clone(),
        qual2.clone(),
        sema.resolve_alias_in_qualified_name(&qual1),
        sema.resolve_alias_in_qualified_name(&qual2),
    ] {
        if let Some(sig) = sema
            .functions
            .get(&name)
            .and_then(|sigs| sigs.iter().find(|sig| key.matches_sig(sig)))
        {
            return Some(sig);
        }
    }
    if key.is_destructor() {
        let prefix1 = format!("{}::~", class_name);
        let prefix2 = format!("{}::~", alt_class);
        return sema
            .functions
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix1) || name.starts_with(&prefix2))
            .flat_map(|(_, sigs)| sigs)
            .find(|sig| key.matches_sig(sig));
    }
    None
}

/// C++17 exception-specification compatibility: an override may be at least
/// as restrictive as the base, but cannot be laxer.
pub(crate) fn noexcept_compatible(base: &FunctionSig, derived: &FunctionDecl) -> bool {
    !base.qualifiers.is_noexcept || derived.qualifiers.is_noexcept
}

/// Return types must match, or be a pointer/reference to a class derived from
/// the base return class through an unambiguous, accessible path (C++17
/// covariant returns). Pointee cv-qualification must be identical.
pub(crate) fn return_compatible(base: &Type, derived: &Type, sema: &Sema) -> bool {
    if base == derived {
        return true;
    }
    let (b, d) = match (base.strip_const(), derived.strip_const()) {
        (Type::Pointer(b), Type::Pointer(d)) | (Type::Reference(b), Type::Reference(d)) => (b, d),
        _ => return false,
    };
    // The derived class may be less cv-qualified than the base class, never
    // more. The pointer/reference itself must have the same cv-qualification.
    if d.is_const() && !b.is_const() {
        return false;
    }
    matches!((b.strip_const(), d.strip_const()),
        (Type::Struct(bc), Type::Struct(dc))
            if sema.check_member_pointer_base_conversion(dc, bc)
                == super::MemberPointerBaseConversion::Ok)
}

/// Byte adjustment converting an overrider's covariant return value into the
/// static return type expected by the base interface.
pub(crate) fn return_adjustment(base: &Type, derived: &Type, sema: &Sema) -> i32 {
    if base == derived {
        return 0;
    }
    let (b, d) = match (base.strip_const(), derived.strip_const()) {
        (Type::Pointer(b), Type::Pointer(d)) | (Type::Reference(b), Type::Reference(d)) => (b, d),
        _ => return 0,
    };
    match (b.strip_const(), d.strip_const()) {
        (Type::Struct(bc), Type::Struct(dc)) => {
            let off = sema.base_subobject_offset(dc, bc).unwrap_or(0);
            off as i32
        }
        _ => 0,
    }
}
