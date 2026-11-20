//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H
#define _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/is_unsigned.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream selects the unsigned twin via `__type_list` and
// `is_enum`, then restores cv with `__copy_cv_t`. cppbolt has no enum base or
// 128-bit types and can only resolve one dependent `::type` level, so the
// upstream specializations are written out directly for its integer types.
template <class _Tp>
struct __make_unsigned {};

// clang-format off
template <> struct __make_unsigned<bool>               {};
template <> struct __make_unsigned<char>               {typedef unsigned char     type;};
template <> struct __make_unsigned<signed char>        {typedef unsigned char     type;};
template <> struct __make_unsigned<unsigned char>      {typedef unsigned char     type;};
template <> struct __make_unsigned<short>              {typedef unsigned short     type;};
template <> struct __make_unsigned<unsigned short>     {typedef unsigned short     type;};
template <> struct __make_unsigned<int>                {typedef unsigned int       type;};
template <> struct __make_unsigned<unsigned int>       {typedef unsigned int       type;};
template <> struct __make_unsigned<long>               {typedef unsigned long      type;};
template <> struct __make_unsigned<unsigned long>      {typedef unsigned long      type;};
template <> struct __make_unsigned<long long>          {typedef unsigned long long type;};
template <> struct __make_unsigned<unsigned long long> {typedef unsigned long long type;};
// clang-format on

template <class _Tp>
using __make_unsigned_t = typename __make_unsigned<_Tp>::type;

template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS make_unsigned {
  using type _LIBCPP_NODEBUG = __make_unsigned_t<_Tp>;
};

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using make_unsigned_t = __make_unsigned_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H
