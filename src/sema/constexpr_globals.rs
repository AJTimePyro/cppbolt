//! Collection of constexpr global values for static_assert.

use super::*;

pub(super) fn collect_constexpr_globals(
    declarations: &[Decl],
    struct_sizes: &std::collections::HashMap<String, usize>,
) -> std::collections::HashMap<String, i128> {
    use crate::sema::constexpr_eval::eval_integral_constexpr;
    let mut constants = std::collections::HashMap::new();
    for decl in declarations {
        if let Decl::GlobalVar { name, ty, init, .. } = decl
            && ty.is_const()
            && let Some(init) = init
            && let Ok(value) = eval_integral_constexpr(init, Some(struct_sizes))
        {
            constants.insert(name.clone(), value);
            if let Some(short) = name.rsplit("::").next() {
                constants.insert(short.to_string(), value);
            }
        }
    }
    constants
}
