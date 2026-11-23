//! Type identification and semantic validation for C++ exceptions.

use super::*;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;

#[allow(dead_code)]
pub const TYPE_ID_NONE: u32 = 0;
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
pub const TYPE_ID_CONST_CHAR_PTR: u32 = 12;
pub const TYPE_ID_CHAR_PTR: u32 = 13;
pub const TYPE_ID_VOID_PTR: u32 = 14;
pub const TYPE_ID_CONST_VOID_PTR: u32 = 15;
pub const TYPE_ID_LONG: u32 = 16;
pub const TYPE_ID_ULONG: u32 = 17;
pub const TYPE_ID_CLASS_BASE: u32 = 1000;
pub const TYPE_ID_PTR_CLASS_BASE: u32 = 20000;

impl Sema {
    pub fn get_class_type_id(&self, class_name: &str) -> Option<u32> {
        let mut names: Vec<&String> = self.struct_layouts.keys().collect();
        names.sort();
        names
            .iter()
            .position(|k| *k == class_name)
            .map(|i| (i as u32) + TYPE_ID_CLASS_BASE)
    }

    #[allow(dead_code)]
    pub fn get_class_name_from_id(&self, class_id: u32) -> Option<String> {
        if class_id < TYPE_ID_CLASS_BASE {
            return None;
        }
        let idx = (class_id - TYPE_ID_CLASS_BASE) as usize;
        let mut names: Vec<&String> = self.struct_layouts.keys().collect();
        names.sort();
        names.get(idx).map(|s| (*s).clone())
    }

    pub fn get_exception_type_id(&self, ty: &Type) -> Option<u32> {
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
            Type::Pointer(inner) => match inner.strip_const() {
                Type::Char => {
                    if inner.is_const() {
                        Some(TYPE_ID_CONST_CHAR_PTR)
                    } else {
                        Some(TYPE_ID_CHAR_PTR)
                    }
                }
                Type::Void => {
                    if inner.is_const() {
                        Some(TYPE_ID_CONST_VOID_PTR)
                    } else {
                        Some(TYPE_ID_VOID_PTR)
                    }
                }
                Type::Struct(sname) => self
                    .get_class_type_id(sname)
                    .map(|id| id - TYPE_ID_CLASS_BASE + TYPE_ID_PTR_CLASS_BASE),
                _ => Some(TYPE_ID_VOID_PTR),
            },
            Type::Struct(sname) => self.get_class_type_id(sname),
            Type::Enum(_) => Some(TYPE_ID_INT),
            _ => None,
        }
    }

    pub fn validate_throw_type(
        &self,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let clean = ty.strip_reference().strip_const();
        if matches!(clean, Type::Void) {
            return Err(CompileError::new("cannot throw void expression", line, col));
        }
        if let Type::Struct(sname) = clean {
            if let Some(layout) = self.struct_layouts.get(sname) {
                if layout.is_abstract {
                    return Err(CompileError::new(
                        format!("cannot throw object of abstract class type '{}'", sname),
                        line,
                        col,
                    ));
                }
            } else {
                return Err(CompileError::new(
                    format!("cannot throw incomplete type '{}'", sname),
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
        Ok(())
    }
}
