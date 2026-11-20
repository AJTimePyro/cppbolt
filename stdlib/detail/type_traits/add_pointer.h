//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef _LIBCPP___TYPE_TRAITS_ADD_POINTER_H
#define _LIBCPP___TYPE_TRAITS_ADD_POINTER_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

// cppbolt adaptation: upstream uses the `__add_pointer` builtin. Direct
// partial specializations handle the reference-to-void cases from the classic
// upstream implementation without an extra dependent `::type`.
template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS add_pointer {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
struct add_pointer<_Tp&> {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
struct add_pointer<_Tp&&> {
  using type _LIBCPP_NODEBUG = _Tp*;
};

template <class _Tp>
using __add_pointer_t _LIBCPP_NODEBUG = typename add_pointer<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using add_pointer_t = __add_pointer_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_ADD_POINTER_H
