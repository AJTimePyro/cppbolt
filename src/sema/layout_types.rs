//! Record layout data types.

use super::*;

/// One entry of a per-subobject virtual table.
#[derive(Debug, Clone)]
pub struct VTableSlotInfo {
    /// Semantic identity of the virtual function in this slot.
    pub key: VirtualKey,
    /// The deleting-destructor companion slot (no ordinary call targets it).
    pub is_deleting_dtor: bool,
}

/// One virtual table belonging to a base subobject of a complete object.
///
/// A polymorphic class with several polymorphic bases has one vptr (and one
/// table) per such subobject. `vptr_offset` is measured from the start of the
/// complete object; the primary table is normally at offset zero.
#[derive(Debug, Clone)]
pub struct VTableInfo {
    /// Class of the subobject that owns this vptr.
    pub owner: String,
    /// Offset of the vptr from the start of the complete object.
    pub vptr_offset: usize,
    /// Offset of the base subobject that owns this vptr (may differ from
    /// vptr_offset when the base inherited its vptr from another base).
    pub owner_offset: usize,
    /// Slots in dispatch order.
    pub slots: Vec<VTableSlotInfo>,
}

pub(super) fn element_struct_name(ty: &Type) -> Option<&str> {
    match ty.strip_const() {
        Type::Struct(name) => Some(name),
        Type::Array(elem, _) => element_struct_name(elem),
        _ => None,
    }
}

/// The declaration of one record plus the source position used for layout
/// diagnostics.
pub(crate) struct RecordLayout<'a> {
    pub(super) name: &'a str,
    pub(super) is_class: bool,
    pub(super) is_final: bool,
    pub(super) base_classes: &'a [String],
    pub(super) base_access: &'a [crate::ast::BaseAccess],
    pub(super) fields: &'a [StructFieldDecl],
    pub(super) methods: &'a [FunctionDecl],
    pub(super) private_members: &'a [String],
    pub(super) line: usize,
    pub(super) col: usize,
    pub(super) alignment: Option<usize>,
}

/// Accumulates the offsets, fields and vtable slots produced while laying out
/// one record.
pub(super) struct LayoutAccumulator {
    pub(super) offset: usize,
    pub(super) max_align: usize,
    pub(super) struct_fields: Vec<StructField>,
    pub(super) base_offsets: Vec<(String, usize)>,
    pub(super) direct_base_offsets: Vec<(String, usize)>,
    pub(super) vtable_slots: Vec<VirtualKey>,
}

impl LayoutAccumulator {
    pub(super) fn new() -> Self {
        Self {
            offset: 0,
            max_align: 1,
            struct_fields: Vec::new(),
            base_offsets: Vec::new(),
            direct_base_offsets: Vec::new(),
            vtable_slots: Vec::new(),
        }
    }

    /// Rounds the running offset up to the alignment and keeps an empty record
    /// at least one byte large.
    ///
    /// The pass runs twice while laying out a record: the first pass aligns the
    /// fields, the second re-aligns the one-byte minimum of an empty record.
    pub(super) fn round_up(&mut self) {
        if self.max_align > 0 && !self.offset.is_multiple_of(self.max_align) {
            self.offset += self.max_align - (self.offset % self.max_align);
        }
        if self.offset == 0 {
            self.offset = 1;
        }
    }
}


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
