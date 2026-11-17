//! Record-layout dump helpers for diagnostics.

use super::*;

impl Sema {
    pub(super) fn dump_base_layout(&self, base_name: &str, depth: usize, out: &mut String) {
        if let Some(bl) = self.struct_layouts.get(base_name) {
            let pad = "  ".repeat(depth);
            let tag = if bl.is_class { "class" } else { "struct" };
            if bl.fields.is_empty() && !bl.has_vtable && bl.base_classes.is_empty() {
                out.push_str(&format!(
                    "{:10} | {}{} {} (base) (empty)\n",
                    0, pad, tag, base_name
                ));
            } else {
                let kind = if bl.has_vtable {
                    "(primary base)"
                } else {
                    "(base)"
                };
                out.push_str(&format!(
                    "{:10} | {}{} {} {}\n",
                    0, pad, tag, base_name, kind
                ));
                if !bl.base_classes.is_empty() {
                    self.dump_base_layout(&bl.base_classes[0], depth + 1, out);
                } else if bl.has_vtable {
                    out.push_str(&format!(
                        "{:10} | {}  ({} vtable pointer)\n",
                        0, pad, base_name
                    ));
                }
                let base_field_count = if !bl.base_classes.is_empty() {
                    self.struct_layouts
                        .get(&bl.base_classes[0])
                        .map(|b| b.fields.len())
                        .unwrap_or(0)
                } else {
                    0
                };
                for f in bl.fields.iter().skip(base_field_count) {
                    self.dump_field(f, 0, depth + 1, out);
                }
            }
        }
    }

    pub(super) fn dump_field(
        &self,
        f: &StructField,
        base_offset: usize,
        depth: usize,
        out: &mut String,
    ) {
        let pad = "  ".repeat(depth);
        if let Type::Struct(sname) = f.ty.strip_const()
            && let Some(sl) = self.struct_layouts.get(sname)
        {
            let tag = if sl.is_class { "class" } else { "struct" };
            out.push_str(&format!(
                "{:10} | {}{} {} {}\n",
                f.offset + base_offset,
                pad,
                tag,
                sname,
                f.name
            ));
            for sf in &sl.fields {
                self.dump_field(sf, f.offset + base_offset, depth + 1, out);
            }
            return;
        }
        out.push_str(&format!(
            "{:10} | {}{} {}\n",
            f.offset + base_offset,
            pad,
            f.ty.to_c_type(),
            f.name
        ));
    }

    pub fn dump_record_layouts(&self) -> String {
        let mut out = String::new();
        for name in &self.struct_order {
            if let Some(layout) = self.struct_layouts.get(name) {
                out.push_str("\n*** Dumping AST Record Layout\n");
                let tag = if layout.is_class { "class" } else { "struct" };
                if layout.fields.is_empty() && !layout.has_vtable && layout.base_classes.is_empty()
                {
                    out.push_str(&format!("{:10} | {} {} (empty)\n", 0, tag, name));
                    out.push_str(&format!(
                        "           | [sizeof={}, dsize={}, align={},\n",
                        layout.size, layout.size, layout.align
                    ));
                    out.push_str(&format!(
                        "           |  nvsize={}, nvalign={}]\n",
                        layout.size, layout.align
                    ));
                    continue;
                }
                out.push_str(&format!("{:10} | {} {}\n", 0, tag, name));
                if !layout.base_classes.is_empty() {
                    self.dump_base_layout(&layout.base_classes[0], 1, &mut out);
                } else if layout.has_vtable {
                    out.push_str(&format!("{:10} |   ({} vtable pointer)\n", 0, name));
                }
                let base_field_count = if !layout.base_classes.is_empty() {
                    self.struct_layouts
                        .get(&layout.base_classes[0])
                        .map(|bl| bl.fields.len())
                        .unwrap_or(0)
                } else {
                    0
                };
                for f in layout.fields.iter().skip(base_field_count) {
                    self.dump_field(f, 0, 1, &mut out);
                }
                out.push_str(&format!(
                    "           | [sizeof={}, dsize={}, align={},\n",
                    layout.size, layout.size, layout.align
                ));
                out.push_str(&format!(
                    "           |  nvsize={}, nvalign={}]\n",
                    layout.size, layout.align
                ));
            }
        }
        out
    }
}
