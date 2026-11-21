//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_IS_SIGNED_H
#define _LIBCPP___TYPE_TRAITS_IS_SIGNED_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream's fallback computes `_Tp(-1) < _Tp(0)` in a
// variable-template specialization. cppbolt cannot fold that functional cast
// in a constant initializer, so the upstream result is listed explicitly for
// its arithmetic types.
template <class _Tp>
struct __is_signed_impl {
  static const bool value = false;
};
template <>
struct __is_signed_impl<char> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<signed char> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<short> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<int> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<long> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<long long> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<float> {
  static const bool value = true;
};
template <>
struct __is_signed_impl<double> {
  static const bool value = true;
};
template <class _Tp>
struct __is_signed_impl<const _Tp> : __is_signed_impl<_Tp> {};

template <class _Tp>
struct is_signed : integral_constant<bool, __is_signed_impl<_Tp>::value> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
inline constexpr bool is_signed_v = is_signed<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_SIGNED_H
