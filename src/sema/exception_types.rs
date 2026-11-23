//! Type identification and semantic validation for C++ exceptions.

use super::*;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;

pub const TYPE_ID_INT: u32 = 1;
pub const TYPE_ID_UINT: u32 = 2;
pub const TYPE_ID_BOOL: u32 = 3;
pub const TYPE_ID_CHAR: u32 = 4;
pub const TYPE_ID_UCHAR: u32 = 5;
pub const TYPE_ID_SHORT: u32 = 6;
pub const TYPE_ID_USHORT: u32 = 7;
pub const TYPE_ID_LONG_LONG: u32 = 8;
pub const TYPE_ID_ULONG_LONG: u32 = 9;
pub const TYPE_ID_FLOAT: u32 = 10;
pub const TYPE_ID_DOUBLE: u32 = 11;
pub const TYPE_ID_VOID: u32 = 14;
pub const TYPE_ID_VOID_PTR: u32 = TYPE_ID_VOID | (1 << 16);
pub const TYPE_ID_CONST_VOID_PTR: u32 = TYPE_ID_VOID | (1 << 16) | (1 << 20);
pub const TYPE_ID_LONG: u32 = 16;
pub const TYPE_ID_ULONG: u32 = 17;
pub const TYPE_ID_NULLPTR: u32 = 18;
pub const TYPE_ID_CLASS_BASE: u32 = 1000;
// Type IDs keep their base in the low 16 bits; the upper bits encode pointer
// depth and const masks. These bases must stay below 1 << 16.
pub const TYPE_ID_ENUM_BASE: u32 = 50000;
pub const TYPE_ID_FUNCTION_BASE: u32 = 55000;
pub const TYPE_ID_MEMBER_PTR_BASE: u32 = 60000;
pub const TYPE_ID_AUX_SPAN: u32 = 5000;

impl Sema {
    pub fn canonical_class_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .struct_layouts
            .keys()
            .map(|k| crate::name::QualifiedName::new(k).plain())
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    pub fn class_type_id_from_names(&self, class_name: &str, names: &[&str]) -> Option<u32> {
        let plain_target = crate::name::QualifiedName::new(class_name).plain();
        names
            .binary_search(&plain_target)
            .ok()
            .map(|i| (i as u32) + TYPE_ID_CLASS_BASE)
    }

    pub fn get_class_type_id(&self, class_name: &str) -> Option<u32> {
        let names = self.canonical_class_names();
        self.class_type_id_from_names(class_name, &names)
    }

    pub fn canonical_enum_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .scoped_enums
            .iter()
            .map(String::as_str)
            .chain(self.enum_types.values().map(String::as_str))
            .map(|name| crate::name::QualifiedName::new(name).plain())
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    pub fn aux_exception_type_id(&self, key: &str, base: u32) -> u32 {
        let mut ids = self.aux_exception_type_ids.borrow_mut();
        if let Some(&id) = ids.get(key) {
            return id;
        }
        let used = ids
            .values()
            .filter(|&&id| id >= base && id < base + TYPE_ID_AUX_SPAN)
            .count() as u32;
        let id = base + used;
        ids.insert(key.to_string(), id);
        id
    }

    pub fn get_base_type_id(&self, ty: &Type) -> Option<u32> {
        let clean = ty.strip_reference().strip_const();
        match clean {
            Type::Int => Some(TYPE_ID_INT),
            Type::UnsignedInt => Some(TYPE_ID_UINT),
            Type::Bool => Some(TYPE_ID_BOOL),
            Type::Char | Type::SignedChar => Some(TYPE_ID_CHAR),
            Type::UnsignedChar => Some(TYPE_ID_UCHAR),
            Type::Short => Some(TYPE_ID_SHORT),
            Type::UnsignedShort => Some(TYPE_ID_USHORT),
            Type::Long => Some(TYPE_ID_LONG),
            Type::UnsignedLong => Some(TYPE_ID_ULONG),
            Type::LongLong => Some(TYPE_ID_LONG_LONG),
            Type::UnsignedLongLong => Some(TYPE_ID_ULONG_LONG),
            Type::Float => Some(TYPE_ID_FLOAT),
            Type::Double => Some(TYPE_ID_DOUBLE),
            Type::Void => Some(TYPE_ID_VOID),
            Type::Nullptr => Some(TYPE_ID_NULLPTR),
            Type::Struct(sname) => self.get_class_type_id(sname),
            Type::Enum(ename) => {
                let names = self.canonical_enum_names();
                let plain = crate::name::QualifiedName::new(ename).plain();
                names
                    .binary_search(&plain)
                    .ok()
                    .map(|i| TYPE_ID_ENUM_BASE + i as u32)
            }
            Type::Function { .. } => {
                Some(self.aux_exception_type_id(&format!("F|{:?}", clean), TYPE_ID_FUNCTION_BASE))
            }
            Type::MemberPointer {
                class_name,
                member_type,
            } => {
                let plain = crate::name::QualifiedName::new(class_name).plain();
                Some(self.aux_exception_type_id(
                    &format!("M|{}|{:?}", plain, member_type),
                    TYPE_ID_MEMBER_PTR_BASE,
                ))
            }
            _ => None,
        }
    }

    pub fn get_exception_type_id(&self, ty: &Type) -> Option<u32> {
        let mut depth: u32 = 0;
        let mut const_mask: u32 = 0;
        let mut curr = ty.strip_reference().strip_const();

        while let Type::Pointer(inner) = curr {
            if inner.is_const() {
                const_mask |= 1 << depth;
            }
            depth += 1;
            curr = inner.strip_const();
        }

        let base_id = self.get_base_type_id(curr)?;
        if depth == 0 {
            Some(base_id)
        } else {
            Some(base_id | (depth << 16) | (const_mask << 20))
        }
    }

    fn validate_exception_construction(
        &self,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let Type::Struct(sname) = ty.strip_reference().strip_const() else {
            return Ok(());
        };
        let short = crate::name::QualifiedName::new(sname).class_short_name();
        let key = crate::name::copy_ctor_key(short);
        let explicit_key = crate::name::explicit_copy_ctor_key(short);
        let unavailable = self.deleted_functions.contains(&key)
            || self
                .struct_layouts
                .get(sname)
                .is_some_and(|layout| {
                    layout.private_members.contains(&key)
                        || layout.private_members.contains(&explicit_key)
                });
        if unavailable {
            return Err(CompileError::new(
                format!(
                    "copy constructor for exception object of type '{}' is deleted or inaccessible",
                    sname
                ),
                line,
                col,
            ));
        }
        Ok(())
    }

    pub fn has_move_constructor(&self, class_name: &str) -> bool {
        let short_name = crate::name::QualifiedName::new(class_name).class_short_name();
        let move_key = crate::name::move_ctor_key(short_name);
        if self.deleted_functions.contains(&move_key)
            || self
                .struct_layouts
                .get(class_name)
                .is_some_and(|l| l.private_members.contains(&move_key))
        {
            return true;
        }
        let q1 = format!("{class_name}::{short_name}");
        let q2 = format!(
            "{}::{short_name}",
            crate::name::QualifiedName::new(class_name).alternate()
        );
        for qual in [&q1, &q2] {
            if let Some(sigs) = self.functions.get(qual) {
                for sig in sigs {
                    if sig.param_types.len() >= 2 && sig.param_types[1].is_rvalue_reference() {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn validate_throw_expr(
        &self,
        expr: &Expr,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let clean = ty.strip_reference().strip_const();
        if matches!(clean, Type::Void) {
            return Err(CompileError::new("cannot throw void expression", line, col));
        }
        let Type::Struct(sname) = clean else {
            return Ok(());
        };
        let Some(layout) = self.struct_layouts.get(sname) else {
            return Err(CompileError::new(
                format!("cannot throw incomplete type '{}'", sname),
                line,
                col,
            ));
        };
        if layout.is_abstract {
            return Err(CompileError::new(
                format!("cannot throw object of abstract class type '{}'", sname),
                line,
                col,
            ));
        }

        let is_xval = match expr {
            Expr::Cast { target_type, .. } => target_type.is_rvalue_reference(),
            _ => false,
        };

        let is_move_eligible_local = if let Expr::Identifier(name) = expr {
            let sym = self.lookup_var(name);
            sym.is_some_and(|s| {
                !s.is_global
                    && !s.ty.is_reference()
                    && !s.ty.strip_reference().is_const()
                    && self.var_try_depths.get(name).copied().unwrap_or(0) >= self.try_depth
            })
        } else {
            false
        };

        let is_prval = !expr.is_lvalue() && !is_xval;
        // Under C++17 guaranteed copy elision [except.throw] p3:
        // A prvalue directly initializes the exception object; no copy or move
        // constructor is invoked or required to be accessible/non-deleted.
        if is_prval {
            return Ok(());
        }

        let short = crate::name::QualifiedName::new(sname).class_short_name();
        let is_rvalue = is_xval || is_move_eligible_local;

        if is_rvalue {
            let move_key = crate::name::move_ctor_key(short);
            let explicit_move_key = crate::name::explicit_move_ctor_key(short);
            if self.has_move_constructor(sname) {
                if self.deleted_functions.contains(&move_key) {
                    return Err(CompileError::new(
                        format!(
                            "move constructor for exception object of type '{}' is deleted",
                            sname
                        ),
                        line,
                        col,
                    ));
                }
                if layout.private_members.contains(&move_key) {
                    return Err(CompileError::new(
                        format!(
                            "move constructor for exception object of type '{}' is inaccessible",
                            sname
                        ),
                        line,
                        col,
                    ));
                }
                if layout.private_members.contains(&explicit_move_key) {
                    return Err(CompileError::new(
                        format!(
                            "explicit move constructor for exception object of type '{}' cannot be used in copy-initialization",
                            sname
                        ),
                        line,
                        col,
                    ));
                }
                return Ok(());
            }

            // If move ctor is not declared, rvalues fall back to copy constructor
            let copy_key = crate::name::copy_ctor_key(short);
            let explicit_copy_key = crate::name::explicit_copy_ctor_key(short);
            if self.deleted_functions.contains(&copy_key) {
                return Err(CompileError::new(
                    format!(
                        "copy constructor for exception object of type '{}' is deleted or inaccessible",
                        sname
                    ),
                    line,
                    col,
                ));
            }
            if layout.private_members.contains(&copy_key) {
                return Err(CompileError::new(
                    format!(
                        "copy constructor for exception object of type '{}' is deleted or inaccessible",
                        sname
                    ),
                    line,
                    col,
                ));
            }
            if layout.private_members.contains(&explicit_copy_key) {
                return Err(CompileError::new(
                    format!(
                        "explicit copy constructor for exception object of type '{}' cannot be used in copy-initialization",
                        sname
                    ),
                    line,
                    col,
                ));
            }
        } else {
            // Lvalue: overload resolution selects copy constructor.
            let copy_key = crate::name::copy_ctor_key(short);
            let explicit_copy_key = crate::name::explicit_copy_ctor_key(short);
            if self.deleted_functions.contains(&copy_key) {
                return Err(CompileError::new(
                    format!(
                        "copy constructor for exception object of type '{}' is deleted or inaccessible",
                        sname
                    ),
                    line,
                    col,
                ));
            }
            if layout.private_members.contains(&copy_key) {
                return Err(CompileError::new(
                    format!(
                        "copy constructor for exception object of type '{}' is deleted or inaccessible",
                        sname
                    ),
                    line,
                    col,
                ));
            }
            if layout.private_members.contains(&explicit_copy_key) {
                return Err(CompileError::new(
                    format!(
                        "explicit copy constructor for exception object of type '{}' cannot be used in copy-initialization",
                        sname
                    ),
                    line,
                    col,
                ));
            }
        }
        Ok(())
    }

    pub fn validate_catch_type(
        &self,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        if ty.is_rvalue_reference() {
            return Err(CompileError::new(
                "cannot declare catch handler with rvalue reference type",
                line,
                col,
            ));
        }
        if !ty.is_reference()
            && let Type::Struct(sname) = ty.strip_const()
            && let Some(layout) = self.struct_layouts.get(sname)
            && layout.is_abstract
        {
            return Err(CompileError::new(
                format!("cannot catch abstract class '{}' by value", sname),
                line,
                col,
            ));
        }
        if !ty.is_reference() {
            self.validate_exception_construction(ty, line, col)?;
        }
        Ok(())
    }
}
