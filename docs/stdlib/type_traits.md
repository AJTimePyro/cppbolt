# `<type_traits>` — LLVM libc++ 23.1.5 Integration

## Upstream source

- **Tag:** `llvmorg-23.1.5` (libc++ 23.1.5)
- **Umbrella header:** `libcxx/include/type_traits`
- **Implementation:** `libcxx/include/__type_traits/*.h`
- **License:** Apache License v2.0 with LLVM Exceptions (retained in every
  imported header; `SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception`).

Upstream `<type_traits>` is not a self-contained header. The umbrella includes
`<__config>`, `<version>` and roughly 70 internal `__type_traits/*.h` headers.
This integration imports the C++17 components from that set into
`stdlib/detail/type_traits/` and keeps the upstream umbrella structure in
`stdlib/type_traits`.

## Integration approach

The upstream implementation is the starting point. It is kept as close to the
source as cppbolt's C++17 subset permits; no mini-STL replacement was written
for a working upstream trait.

1. `stdlib/type_traits` is the adapted upstream umbrella. It includes only the
   C++17 traits cppbolt can evaluate correctly.
2. `stdlib/detail/type_traits/*.h` are the corresponding upstream internal
   headers. Their include paths were rewritten from `<__type_traits/...>` to
   `<detail/type_traits/...>`, `<__config>` to `<compat/compiler/compiler.h>`,
   and the `<__cstddef/...>` helpers to `<cstddef>`.
3. `stdlib/detail/type_traits/builtins.h` is **new reusable infrastructure**:
   it spells the compiler type-query/transform builtins libc++ expects
   (`__is_same`, `__is_const`, `__remove_pointer`, ...) as ordinary class
   template partial specializations. The upstream headers see a compiler that
   "has" those builtins, so their logic is unchanged. Future libc++ integrations
   can reuse this shim.
4. `stdlib/compat/compiler/compiler.h` gained the remaining libc++ macros
   (`_LIBCPP_STD_VER 17`, `_LIBCPP_NO_SPECIALIZATIONS`, diagnostic push/pop,
   `__has_builtin`/`__has_warning`/`__has_cpp_attribute` → `0`, feature guards
   for 128-bit integers and wide characters).

### Targeted library adaptations

These are narrow, documented patches; each preserves the upstream semantics
for the types cppbolt can represent:

| Upstream construct | Why it needed adaptation | Adaptation |
|---|---|---|
| `__config`, `<version>` | No such headers in cppbolt | Replaced by the compat shim |
| Compiler builtins with no non-builtin path | cppbolt exposes no type intrinsics | `builtins.h` partial-specialization shims |
| `enum { value }` members, `static inline constexpr` data | Not in cppbolt's C++17 subset | `static const` members |
| `[[__nodiscard__]]` attribute | Attributes are not parsed | Attribute removed from `integral_constant` |
| Conversion operator to a member typedef | cppbolt needs the template parameter directly | `operator _Tp()` |
| Explicit specialization of variable templates, functional casts in constant initializers | Not supported | Upstream class-template specialization form |
| Member alias templates (`_IfImpl`, `_OrImpl`, `_And`) | Not supported | Upstream partial-specialization form (from `conditional`/`conjunction`) |
| Nested dependent `::type` in non-type template arguments | cppbolt resolves only one level | Direct partial specializations in the transformation headers |
| Multi-dimensional array partial-specialization patterns | cppbolt binds bounds differently from Clang/GCC | `extent` omitted; `remove_all_extents` unrolled for common dimensions |

### General compiler improvements made for this integration

These live in the compiler, not the library, and benefit future integrations:

- `src/parser/template.rs`: template type arguments are parsed as full type-ids
  and accept array suffixes, so `is_array<T[3]>` and member-function-pointer
  patterns are expressible.
- `src/sema/template_match.rs`: partial-specialization matching now deduces
  from array (`T[N]`, `T[]`) and member-pointer (`T C::*`) patterns, including
  member-function parameter packs and specificity ranking between bounded and
  unbounded array patterns.

## Supported functionality

Verified against Clang 23.1.5 and GCC 16.2.1 using `-nostdinc++ -I stdlib`:

- **Helper classes:** `integral_constant`, `true_type`, `false_type`,
  `bool_constant`, `_BoolConstant`.
- **SFINAE helpers:** `enable_if` / `enable_if_t`, `void_t`.
- **Classification:** `is_same`, `is_void`, `is_integral`, `is_floating_point`,
  `is_null_pointer`, `is_array`, `is_pointer`, `is_const`, `is_reference`,
  `is_lvalue_reference`, `is_rvalue_reference`, `is_member_pointer`,
  `is_member_object_pointer`, `is_member_function_pointer`, `is_arithmetic`,
  `is_fundamental`, `is_compound`, plus the `_v` variable templates for these.
- **Transformations:** `remove_cv`, `remove_const`, `remove_volatile`,
  `add_const`, `add_volatile`, `add_cv`, `remove_reference`,
  `add_lvalue_reference`, `add_rvalue_reference`, `remove_pointer`,
  `add_pointer`, `decay`, `remove_extent`, `remove_all_extents`,
  `make_signed`, `make_unsigned` and their `_t` aliases.
- **Properties:** `rank`, `alignment_of`, `is_signed`, `is_unsigned` and the
  `_v` aliases.
- **Logical helpers:** `negation` / `negation_v`.

## Known limitations and unsupported functionality

- Traits that need compiler introspection cppbolt does not provide were **not
  imported**: `is_class`, `is_union`, `is_enum`, `is_function`, `is_object`,
  `is_scalar`, `is_final`, `is_aggregate`, `is_empty`, `is_polymorphic`,
  `is_abstract`, `is_trivial`, `is_standard_layout`, `is_trivially_copyable`,
  `is_pod`, `has_virtual_destructor`, `has_unique_object_representations`,
  `underlying_type`, `is_base_of`, the `is_constructible` / `is_assignable` /
  `is_convertible` / `is_invocable` families, `common_type`, `aligned_storage`,
  `aligned_union`, `result_of` and `invoke_result`.
- `conjunction`, `disjunction` and `is_swappable` are not imported: their
  recursive variadic metaprogramming cannot be evaluated by cppbolt.
- `extent` on multidimensional arrays is not imported: cppbolt binds
  multidimensional array partial-specialization parameters differently from
  Clang/GCC. `rank`, `remove_extent` and `remove_all_extents` are supported.
- C++20/23/26 traits (`type_identity`, `remove_cvref`,
  `is_constant_evaluated`, `is_implicit_lifetime`,
  `reference_constructs_from_temporary`, `is_within_lifetime`, ...) are outside
  cppbolt's C++17 target.
- `volatile` is not a distinct cppbolt type. CV traits therefore behave as if
  `volatile` were absent, and `long double`, `wchar_t`, `char8_t`, `char16_t`
  and `char32_t` have no distinct representation.
- The `is_integral` / `is_floating_point` / `is_signed` / `is_unsigned` value
  sets are fixed for cppbolt's integer types; they do not cover 128-bit
  integers or extended character types.
- `std::conjunction` / `std::disjunction` are deliberately absent rather than
  approximated with incorrect results.

## Verification

```sh
cargo build --release --bin cppbolt-cli --bin cppbolt-testrunner
target/release/cppbolt-testrunner target/release/cppbolt-cli all
python3 tools/verify_mini_stl_gxx.py
```

Results are recorded in the completion report and in `tests/PASSING.txt`.
