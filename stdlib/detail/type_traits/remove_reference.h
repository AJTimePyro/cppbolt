//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream obtains this from a compiler builtin with no
// fallback. The classic upstream partial-specialization form is used instead,
// which keeps this header independent of the builtin shim.
template <class _Tp>
struct remove_reference {
  typedef _Tp type;
};
template <class _Tp>
struct remove_reference<_Tp&> {
  typedef _Tp type;
};
template <class _Tp>
struct remove_reference<_Tp&&> {
  typedef _Tp type;
};

template <class _Tp>
using __libcpp_remove_reference_t = typename remove_reference<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_reference_t = __libcpp_remove_reference_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H
