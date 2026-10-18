//! Embedded header table plus provider-backed filesystem includes.

use crate::source::IncludeProvider;

pub(super) fn resolve(includes: &dyn IncludeProvider, spec: &str) -> Option<String> {
    if let Some(header) = embedded_header(spec) {
        return Some(header.to_string());
    }
    let clean = spec.trim_matches(|c| c == '<' || c == '>' || c == '"');
    includes.read_include(clean)
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
        "new" => Some(include_str!("../../stdlib/new")),
        "initializer_list" => Some(include_str!("../../stdlib/initializer_list")),
        "utility" => Some(include_str!("../../stdlib/utility")),
        "vector" => Some(include_str!("../../stdlib/vector")),
        "algorithm" => Some(include_str!("../../stdlib/algorithm")),
        "map" => Some(include_str!("../../stdlib/map")),
        "set" => Some(include_str!("../../stdlib/set")),
        "detail_avl_tree.h" => Some(include_str!("../../stdlib/detail_avl_tree.h")),
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
