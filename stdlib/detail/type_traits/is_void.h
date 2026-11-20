//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_IS_VOID_H
#define _LIBCPP___TYPE_TRAITS_IS_VOID_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream compares `__remove_cv(_Tp)` with `void` through
// a builtin. Direct partial specializations give the same result without the
// nested dependent type cppbolt cannot fold in a template argument.
template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS is_void : false_type {};
template <>
struct is_void<void> : true_type {};
template <>
struct is_void<const void> : true_type {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
_LIBCPP_NO_SPECIALIZATIONS inline constexpr bool is_void_v = is_void<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_VOID_H
