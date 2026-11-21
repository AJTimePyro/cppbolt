//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_IS_FLOATING_POINT_H
#define _LIBCPP___TYPE_TRAITS_IS_FLOATING_POINT_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream uses explicit specializations of a variable
// template and strips cv with `__remove_cv_t`. A class template with a const
// partial specialization is the equivalent expressible in cppbolt's subset.
template <class _Tp>
struct __is_floating_point_impl {
  static const bool value = false;
};
template <>
struct __is_floating_point_impl<float> {
  static const bool value = true;
};
template <>
struct __is_floating_point_impl<double> {
  static const bool value = true;
};

template <class _Tp>
struct is_floating_point : integral_constant<bool, __is_floating_point_impl<_Tp>::value> {};
template <class _Tp>
struct is_floating_point<const _Tp> : is_floating_point<_Tp> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
inline constexpr bool is_floating_point_v = is_floating_point<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_FLOATING_POINT_H
