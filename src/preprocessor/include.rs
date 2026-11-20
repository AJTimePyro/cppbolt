//! Embedded header table plus provider-backed filesystem includes.

use crate::source::{IncludeProvider, ResolvedInclude};

pub(super) fn resolve(
    includes: &dyn IncludeProvider,
    spec: &str,
    including_file: Option<&str>,
) -> Option<ResolvedInclude> {
    let clean = spec.trim_matches(|c| c == '<' || c == '>' || c == '"');
    if let Some(header) = embedded_header(spec) {
        return Some(ResolvedInclude {
            name: clean.to_string(),
            content: header.to_string(),
        });
    }
    let quoted = spec.trim_start().starts_with('"');
    includes.resolve_include(clean, including_file, quoted)
}

const EMBEDDED_HEADERS: &[(&str, &str)] = &[
    ("iostream", include_str!("../../stdlib/iostream")),
    ("string", include_str!("../../stdlib/string")),
    ("functional", include_str!("../../stdlib/functional")),
    ("iomanip", include_str!("../../stdlib/iomanip")),
    ("cstdio", include_str!("../../stdlib/cstdio")),
    ("cstddef", include_str!("../../stdlib/cstddef")),
    ("cmath", include_str!("../../stdlib/cmath")),
    ("cstdlib", include_str!("../../stdlib/cstdlib")),
    ("new", include_str!("../../stdlib/new")),
    (
        "initializer_list",
        include_str!("../../stdlib/initializer_list"),
    ),
    ("utility", include_str!("../../stdlib/utility")),
    ("memory", include_str!("../../stdlib/memory")),
    ("vector", include_str!("../../stdlib/vector")),
    ("array", include_str!("../../stdlib/array")),
    ("iterator", include_str!("../../stdlib/iterator")),
    ("type_traits", include_str!("../../stdlib/type_traits")),
    (
        "detail/type_traits/builtins.h",
        include_str!("../../stdlib/detail/type_traits/builtins.h"),
    ),
    (
        "detail/type_traits/integral_constant.h",
        include_str!("../../stdlib/detail/type_traits/integral_constant.h"),
    ),
    (
        "detail/type_traits/enable_if.h",
        include_str!("../../stdlib/detail/type_traits/enable_if.h"),
    ),
    (
        "detail/type_traits/conditional.h",
        include_str!("../../stdlib/detail/type_traits/conditional.h"),
    ),
    (
        "detail/type_traits/void_t.h",
        include_str!("../../stdlib/detail/type_traits/void_t.h"),
    ),
    (
        "detail/type_traits/is_same.h",
        include_str!("../../stdlib/detail/type_traits/is_same.h"),
    ),
    (
        "detail/type_traits/negation.h",
        include_str!("../../stdlib/detail/type_traits/negation.h"),
    ),
    (
        "detail/type_traits/remove_cv.h",
        include_str!("../../stdlib/detail/type_traits/remove_cv.h"),
    ),
    (
        "detail/type_traits/remove_const.h",
        include_str!("../../stdlib/detail/type_traits/remove_const.h"),
    ),
    (
        "detail/type_traits/remove_volatile.h",
        include_str!("../../stdlib/detail/type_traits/remove_volatile.h"),
    ),
    (
        "detail/type_traits/add_cv_quals.h",
        include_str!("../../stdlib/detail/type_traits/add_cv_quals.h"),
    ),
    (
        "detail/type_traits/remove_reference.h",
        include_str!("../../stdlib/detail/type_traits/remove_reference.h"),
    ),
    (
        "detail/type_traits/add_reference.h",
        include_str!("../../stdlib/detail/type_traits/add_reference.h"),
    ),
    (
        "detail/type_traits/remove_pointer.h",
        include_str!("../../stdlib/detail/type_traits/remove_pointer.h"),
    ),
    (
        "detail/type_traits/add_pointer.h",
        include_str!("../../stdlib/detail/type_traits/add_pointer.h"),
    ),
    (
        "detail/type_traits/decay.h",
        include_str!("../../stdlib/detail/type_traits/decay.h"),
    ),
    (
        "detail/type_traits/is_void.h",
        include_str!("../../stdlib/detail/type_traits/is_void.h"),
    ),
    (
        "detail/type_traits/is_integral.h",
        include_str!("../../stdlib/detail/type_traits/is_integral.h"),
    ),
    (
        "detail/type_traits/is_floating_point.h",
        include_str!("../../stdlib/detail/type_traits/is_floating_point.h"),
    ),
    (
        "detail/type_traits/is_null_pointer.h",
        include_str!("../../stdlib/detail/type_traits/is_null_pointer.h"),
    ),
    (
        "detail/type_traits/is_array.h",
        include_str!("../../stdlib/detail/type_traits/is_array.h"),
    ),
    (
        "detail/type_traits/is_const.h",
        include_str!("../../stdlib/detail/type_traits/is_const.h"),
    ),
    (
        "detail/type_traits/is_reference.h",
        include_str!("../../stdlib/detail/type_traits/is_reference.h"),
    ),
    (
        "detail/type_traits/is_pointer.h",
        include_str!("../../stdlib/detail/type_traits/is_pointer.h"),
    ),
    (
        "detail/type_traits/is_member_pointer.h",
        include_str!("../../stdlib/detail/type_traits/is_member_pointer.h"),
    ),
    (
        "detail/type_traits/is_arithmetic.h",
        include_str!("../../stdlib/detail/type_traits/is_arithmetic.h"),
    ),
    (
        "detail/type_traits/is_fundamental.h",
        include_str!("../../stdlib/detail/type_traits/is_fundamental.h"),
    ),
    (
        "detail/type_traits/is_compound.h",
        include_str!("../../stdlib/detail/type_traits/is_compound.h"),
    ),
    (
        "detail/type_traits/is_signed.h",
        include_str!("../../stdlib/detail/type_traits/is_signed.h"),
    ),
    (
        "detail/type_traits/is_unsigned.h",
        include_str!("../../stdlib/detail/type_traits/is_unsigned.h"),
    ),
    (
        "detail/type_traits/make_signed.h",
        include_str!("../../stdlib/detail/type_traits/make_signed.h"),
    ),
    (
        "detail/type_traits/make_unsigned.h",
        include_str!("../../stdlib/detail/type_traits/make_unsigned.h"),
    ),
    (
        "detail/type_traits/rank.h",
        include_str!("../../stdlib/detail/type_traits/rank.h"),
    ),
    (
        "detail/type_traits/remove_extent.h",
        include_str!("../../stdlib/detail/type_traits/remove_extent.h"),
    ),
    (
        "detail/type_traits/remove_all_extents.h",
        include_str!("../../stdlib/detail/type_traits/remove_all_extents.h"),
    ),
    (
        "detail/type_traits/alignment_of.h",
        include_str!("../../stdlib/detail/type_traits/alignment_of.h"),
    ),
    ("algorithm", include_str!("../../stdlib/algorithm")),
    ("map", include_str!("../../stdlib/map")),
    ("set", include_str!("../../stdlib/set")),
    ("tuple", include_str!("../../stdlib/tuple")),
    (
        "detail/containers/avl_tree.h",
        include_str!("../../stdlib/detail/containers/avl_tree.h"),
    ),
    (
        "detail_avl_tree.h",
        include_str!("../../stdlib/detail/containers/avl_tree.h"),
    ),
    (
        "detail/reverse_iterator.h",
        include_str!("../../stdlib/detail/reverse_iterator.h"),
    ),
    (
        "detail/iterator.h",
        include_str!("../../stdlib/detail/iterator.h"),
    ),
    (
        "detail/algorithms/comp.h",
        include_str!("../../stdlib/detail/algorithms/comp.h"),
    ),
    (
        "detail/algorithms/min_max.h",
        include_str!("../../stdlib/detail/algorithms/min_max.h"),
    ),
    (
        "detail/algorithms/heap.h",
        include_str!("../../stdlib/detail/algorithms/heap.h"),
    ),
    (
        "detail/algorithms/non_modifying.h",
        include_str!("../../stdlib/detail/algorithms/non_modifying.h"),
    ),
    (
        "detail/algorithms/modifying.h",
        include_str!("../../stdlib/detail/algorithms/modifying.h"),
    ),
    (
        "detail/algorithms/binary_search.h",
        include_str!("../../stdlib/detail/algorithms/binary_search.h"),
    ),
    (
        "detail/algorithms/partition.h",
        include_str!("../../stdlib/detail/algorithms/partition.h"),
    ),
    (
        "detail/algorithms/permutation.h",
        include_str!("../../stdlib/detail/algorithms/permutation.h"),
    ),
    (
        "detail/algorithms/merge.h",
        include_str!("../../stdlib/detail/algorithms/merge.h"),
    ),
    (
        "detail/algorithms/set_ops.h",
        include_str!("../../stdlib/detail/algorithms/set_ops.h"),
    ),
    (
        "detail/algorithms/selection.h",
        include_str!("../../stdlib/detail/algorithms/selection.h"),
    ),
    (
        "detail/algorithms/random.h",
        include_str!("../../stdlib/detail/algorithms/random.h"),
    ),
    (
        "compat/config/platform.h",
        include_str!("../../stdlib/compat/config/platform.h"),
    ),
    (
        "compat/compiler/compiler.h",
        include_str!("../../stdlib/compat/compiler/compiler.h"),
    ),
    (
        "compat/runtime/runtime.h",
        include_str!("../../stdlib/compat/runtime/runtime.h"),
    ),
    ("stdio.h", include_str!("../../stdlib/stdio.h")),
    ("stdlib.h", include_str!("../../stdlib/stdlib.h")),
    ("string.h", include_str!("../../stdlib/string.h")),
    ("stddef.h", STDDEF_H),
    ("stdarg.h", STDARG_H),
    ("math.h", MATH_H),
];

fn embedded_header(name: &str) -> Option<&'static str> {
    let clean = name.trim_matches(|c| c == '<' || c == '>' || c == '"');
    EMBEDDED_HEADERS
        .iter()
        .find(|(header, _)| *header == clean)
        .map(|(_, content)| *content)
}

const STDDEF_H: &str = r#"#ifndef _STDDEF_H_
#define _STDDEF_H_
typedef unsigned int size_t;
typedef int ptrdiff_t;
#define NULL 0
#endif
"#;

const STDARG_H: &str = r#"#ifndef _STDARG_H_
#define _STDARG_H_
typedef void* va_list;
#define va_start(ap, param) ((void)0)
#define va_end(ap) ((void)0)
#define va_arg(ap, type) (*(type*)0)
#define va_copy(dest, src) ((void)0)
#endif
"#;

const MATH_H: &str = r#"#ifndef _MATH_H_
#define _MATH_H_
extern "C" {
    double sqrt(double);
    double sin(double);
    double cos(double);
    double tan(double);
    double pow(double, double);
    double fabs(double);
    double floor(double);
    double ceil(double);
    double exp(double);
    double log(double);
}
#endif
"#;
