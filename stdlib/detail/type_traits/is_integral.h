//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_IS_INTEGRAL_H
#define _LIBCPP___TYPE_TRAITS_IS_INTEGRAL_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream strips cv with `__remove_cv_t` before matching
// the specializations below. cppbolt cannot fold that nested dependent type in
// a non-type template argument, so the cv forms are matched directly instead.
template <class _Tp>
struct __libcpp_is_integral {
  static const bool value = false;
};
template <>
struct __libcpp_is_integral<bool> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<char> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<signed char> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<unsigned char> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<short> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<unsigned short> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<int> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<unsigned int> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<long> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<unsigned long> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<long long> {
  static const bool value = true;
};
template <>
struct __libcpp_is_integral<unsigned long long> {
  static const bool value = true;
};

template <class _Tp>
struct is_integral : integral_constant<bool, __libcpp_is_integral<_Tp>::value> {};
template <class _Tp>
struct is_integral<const _Tp> : is_integral<_Tp> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
inline constexpr bool is_integral_v = is_integral<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_INTEGRAL_H
