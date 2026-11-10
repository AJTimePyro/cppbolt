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

fn embedded_header(name: &str) -> Option<&'static str> {
    let clean = name.trim_matches(|c| c == '<' || c == '>' || c == '"');
    match clean {
        "iostream" => Some(include_str!("../../stdlib/iostream")),
        "string" => Some(include_str!("../../stdlib/string")),
        "iomanip" => Some(include_str!("../../stdlib/iomanip")),
        "cstdio" => Some(include_str!("../../stdlib/cstdio")),
        "cstddef" => Some(include_str!("../../stdlib/cstddef")),
        "cmath" => Some(include_str!("../../stdlib/cmath")),
        "cstdlib" => Some(include_str!("../../stdlib/cstdlib")),
        "new" => Some(include_str!("../../stdlib/new")),
        "initializer_list" => Some(include_str!("../../stdlib/initializer_list")),
        "utility" => Some(include_str!("../../stdlib/utility")),
        "memory" => Some(include_str!("../../stdlib/memory")),
        "vector" => Some(include_str!("../../stdlib/vector")),
        "array" => Some(include_str!("../../stdlib/array")),
        "iterator" => Some(include_str!("../../stdlib/iterator")),
        "type_traits" => Some(include_str!("../../stdlib/type_traits")),
        "algorithm" => Some(include_str!("../../stdlib/algorithm")),
        "map" => Some(include_str!("../../stdlib/map")),
        "set" => Some(include_str!("../../stdlib/set")),
        "detail/containers/avl_tree.h" | "detail_avl_tree.h" => {
            Some(include_str!("../../stdlib/detail/containers/avl_tree.h"))
        }
        "detail/reverse_iterator.h" => {
            Some(include_str!("../../stdlib/detail/reverse_iterator.h"))
        }
        "detail/iterator.h" => {
            Some(include_str!("../../stdlib/detail/iterator.h"))
        }
        "detail/algorithms/comp.h" => {
            Some(include_str!("../../stdlib/detail/algorithms/comp.h"))
        }
        "detail/algorithms/min_max.h" => {
            Some(include_str!("../../stdlib/detail/algorithms/min_max.h"))
        }
        "detail/algorithms/heap.h" => {
            Some(include_str!("../../stdlib/detail/algorithms/heap.h"))
        }
        "detail/algorithms/non_modifying.h" => {
            Some(include_str!("../../stdlib/detail/algorithms/non_modifying.h"))
        }
        "detail/algorithms/modifying.h" => {
            Some(include_str!("../../stdlib/detail/algorithms/modifying.h"))
        }
        "compat/config/platform.h" => Some(include_str!("../../stdlib/compat/config/platform.h")),
        "compat/compiler/compiler.h" => {
            Some(include_str!("../../stdlib/compat/compiler/compiler.h"))
        }
        "compat/runtime/runtime.h" => Some(include_str!("../../stdlib/compat/runtime/runtime.h")),
        "stdio.h" => Some(include_str!("../../stdlib/stdio.h")),
        "stdlib.h" => Some(include_str!("../../stdlib/stdlib.h")),
        "string.h" => Some(include_str!("../../stdlib/string.h")),
        "stddef.h" => Some(STDDEF_H),
        "stdarg.h" => Some(STDARG_H),
        "math.h" => Some(MATH_H),
        _ => None,
    }
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
