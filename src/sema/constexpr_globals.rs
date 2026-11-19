//! Registration of namespace-scope constexpr values used in constant
//! expressions and non-type template arguments.

use super::*;

impl Sema {
    /// Registers one constexpr global in declaration order and returns the
    /// map keys it wrote. A value becomes visible only once its declaration
    /// has been reached, while earlier values remain available to later
    /// initializers (for example `constexpr bool b = !a;`).
    pub(super) fn register_constexpr_global(&mut self, decl: &Decl) -> Vec<String> {
        let mut written = Vec::new();
        if let Decl::GlobalVar { name, ty, init, .. } = decl
            && ty.is_const()
            && let Some(init) = init
        {
            // A trait-derived initializer needs the instantiated member value,
            // which `eval_integral_constexpr` cannot see on its own.
            let value = crate::sema::constexpr_eval::eval_integral_constexpr(
                init,
                Some(&self.struct_sizes),
            )
            .ok()
            .or_else(|| {
                let spelling = crate::parser::types_array::expr_to_dependent_str(init)?;
                self.eval_dependent_nontype(&spelling)
            });
            if let Some(value) = value {
                self.static_const_values.insert(name.clone(), value);
                written.push(name.clone());
                if let Some(short) = name.rsplit("::").next() {
                    self.static_const_values.insert(short.to_string(), value);
                    written.push(short.to_string());
                }
            }
        }
        written
    }

    /// Collects every namespace-scope constexpr value, including ones declared
    /// later, for `static_assert` evaluation. `static_assert` has always folded
    /// against the whole program's constants; only NTTP folding is
    /// declaration-ordered. The pre-pass removes just the constexpr-global keys
    /// afterwards, keeping class-template constants registered while evaluating
    /// initializers (their instantiations stay cached and will not re-register).
    pub(super) fn collect_constexpr_globals(
        &mut self,
        declarations: &[Decl],
    ) -> std::collections::HashMap<String, i128> {
        let mut written = Vec::new();
        for decl in declarations {
            written.extend(self.register_constexpr_global(decl));
        }
        let all = self.static_const_values.clone();
        for key in written {
            self.static_const_values.remove(&key);
        }
        all
    }
}
