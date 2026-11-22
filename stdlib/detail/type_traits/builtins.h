//===----------------------------------------------------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// cppbolt compatibility layer for libc++'s type_traits internals.
//
// Upstream libc++ asks the compiler for type queries and type transformations
// (`__is_same`, `__remove_cv`, ...). cppbolt does not expose those intrinsics,
// so the macros below spell the same builtins in terms of ordinary class
// template partial specializations. The upstream headers are kept unchanged:
// what they see is a compiler that "has" these builtins.
//
// Upstream traits are not reimplemented here; only primitive operations.

#ifndef _LIBCPP_DETAIL_TYPE_TRAITS_BUILTINS_H
#define _LIBCPP_DETAIL_TYPE_TRAITS_BUILTINS_H

#include <compat/compiler/compiler.h>
#include <cstddef>

namespace std {

// Use `static const int value` so these helpers have no dependency on integral_constant.
template <class _A, class _B>
struct __cppbolt_is_same {
  static const int value = 0;
};
template <class _A>
struct __cppbolt_is_same<_A, _A> {
  static const int value = 1;
};

template <class _T>
struct __cppbolt_is_const {
  static const int value = 0;
};
template <class _T>
struct __cppbolt_is_const<const _T> {
  static const int value = 1;
};

template <class _T>
struct __cppbolt_is_reference {
  static const int value = 0;
};
template <class _T>
struct __cppbolt_is_reference<_T&> {
  static const int value = 1;
};
template <class _T>
struct __cppbolt_is_reference<_T&&> {
  static const int value = 1;
};

template <class _T>
struct __cppbolt_is_pointer {
  static const int value = 0;
};
template <class _T>
struct __cppbolt_is_pointer<_T*> {
  static const int value = 1;
};
template <class _T>
struct __cppbolt_is_pointer<const _T> : __cppbolt_is_pointer<_T> {};

template <class _T>
struct __cppbolt_is_array {
  static const int value = 0;
};
template <class _T, size_t _Np>
struct __cppbolt_is_array<_T[_Np]> {
  static const int value = 1;
};
template <class _T>
struct __cppbolt_is_array<_T[]> {
  static const int value = 1;
};

template <class _T>
struct __cppbolt_is_bounded_array {
  static const int value = 0;
};
template <class _T, size_t _Np>
struct __cppbolt_is_bounded_array<_T[_Np]> {
  static const int value = 1;
};

template <class _T>
struct __cppbolt_is_member_pointer {
  static const int value = 0;
};
template <class _T, class _Cp>
struct __cppbolt_is_member_pointer<_T _Cp::*> {
  static const int value = 1;
};
template <class _T>
struct __cppbolt_is_member_pointer<const _T> : __cppbolt_is_member_pointer<_T> {};

// A member function pointer is a member pointer whose member type is a
// function. cppbolt cannot express `volatile` or combined `const &` patterns,
// so the qualifier forms it can parse are listed explicitly.
template <class _Tp>
struct __cppbolt_is_member_function_pointer {
  static const int value = 0;
};
// clang-format off
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...)>                    { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) const>              { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) &>                  { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) &&>                 { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) noexcept>           { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) const noexcept>     { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) & noexcept>         { static const int value = 1; };
template <class _R, class _C, class... _A> struct __cppbolt_is_member_function_pointer<_R (_C::*)(_A...) && noexcept>        { static const int value = 1; };
// clang-format on
template <class _T>
struct __cppbolt_is_member_function_pointer<const _T> : __cppbolt_is_member_function_pointer<_T> {};

// The `T C::*` pattern alone also matches member functions, so exclude them.
template <class _T>
struct __cppbolt_is_member_object_pointer {
  static const int value = __cppbolt_is_member_pointer<_T>::value
                         - __cppbolt_is_member_function_pointer<_T>::value;
};
template <class _T>
struct __cppbolt_is_member_object_pointer<const _T> : __cppbolt_is_member_object_pointer<_T> {};

template <class _T>
struct __cppbolt_array_rank {
  static const int value = 0;
};
template <class _T, size_t _Np>
struct __cppbolt_array_rank<_T[_Np]> {
  static const int value = 1 + __cppbolt_array_rank<_T>::value;
};
template <class _T>
struct __cppbolt_array_rank<_T[]> {
  static const int value = 1 + __cppbolt_array_rank<_T>::value;
};

template <class _T>
struct __cppbolt_remove_cv {
  typedef _T type;
};
template <class _T>
struct __cppbolt_remove_cv<const _T> {
  typedef _T type;
};

template <class _T>
struct __cppbolt_remove_reference {
  typedef _T type;
};
template <class _T>
struct __cppbolt_remove_reference<_T&> {
  typedef _T type;
};
template <class _T>
struct __cppbolt_remove_reference<_T&&> {
  typedef _T type;
};

template <class _T>
struct __cppbolt_remove_pointer {
  typedef _T type;
};
template <class _T>
struct __cppbolt_remove_pointer<_T*> {
  typedef _T type;
};

template <class _T>
struct __cppbolt_add_pointer {
  typedef _T* type;
};
template <class _T>
struct __cppbolt_add_pointer<_T&> {
  typedef _T* type;
};
template <class _T>
struct __cppbolt_add_pointer<_T&&> {
  typedef _T* type;
};

template <class _T>
struct __cppbolt_add_lvalue_reference {
  typedef _T& type;
};
template <class _T>
struct __cppbolt_add_lvalue_reference<_T&> {
  typedef _T& type;
};
template <class _T>
struct __cppbolt_add_lvalue_reference<_T&&> {
  typedef _T& type;
};

template <class _T>
struct __cppbolt_add_rvalue_reference {
  typedef _T&& type;
};
template <class _T>
struct __cppbolt_add_rvalue_reference<_T&> {
  typedef _T& type;
};
template <class _T>
struct __cppbolt_add_rvalue_reference<_T&&> {
  typedef _T&& type;
};

template <class _T>
struct __cppbolt_remove_extent {
  typedef _T type;
};
template <class _T, size_t _Np>
struct __cppbolt_remove_extent<_T[_Np]> {
  typedef _T type;
};
template <class _T>
struct __cppbolt_remove_extent<_T[]> {
  typedef _T type;
};

template <class _T>
struct __cppbolt_remove_all_extents {
  typedef _T type;
};
template <class _T, size_t _Np>
struct __cppbolt_remove_all_extents<_T[_Np]> {
  typedef typename __cppbolt_remove_all_extents<_T>::type type;
};
template <class _T>
struct __cppbolt_remove_all_extents<_T[]> {
  typedef typename __cppbolt_remove_all_extents<_T>::type type;
};

template <class _T>
struct __cppbolt_decay_impl {
  typedef typename __cppbolt_remove_cv<_T>::type type;
};
template <class _T, size_t _Np>
struct __cppbolt_decay_impl<_T[_Np]> {
  typedef _T* type;
};
template <class _T>
struct __cppbolt_decay_impl<_T[]> {
  typedef _T* type;
};
template <class _T>
struct __cppbolt_decay {
  typedef typename __cppbolt_decay_impl<
      typename __cppbolt_remove_reference<_T>::type>::type type;
};

}  // namespace std

// Value-like builtins: upstream uses these as constant expressions.
#define __is_same(a, b) (std::__cppbolt_is_same<a, b>::value)
#define __is_const(t) (std::__cppbolt_is_const<t>::value)
#define __is_reference(t) (std::__cppbolt_is_reference<t>::value)
#define __is_pointer(t) (std::__cppbolt_is_pointer<t>::value)
#define __is_array(t) (std::__cppbolt_is_array<t>::value)
#define __is_bounded_array(t) (std::__cppbolt_is_bounded_array<t>::value)
#define __is_member_pointer(t) (std::__cppbolt_is_member_pointer<t>::value)
#define __is_member_object_pointer(t) (std::__cppbolt_is_member_object_pointer<t>::value)
#define __is_member_function_pointer(t) (std::__cppbolt_is_member_function_pointer<t>::value)
#define __array_rank(t) (std::__cppbolt_array_rank<t>::value)

// Type-like builtins: upstream uses these as type-ids.
#define __remove_cv(t) typename std::__cppbolt_remove_cv<t>::type
#define __remove_pointer(t) typename std::__cppbolt_remove_pointer<t>::type
#define __add_pointer(t) typename std::__cppbolt_add_pointer<t>::type
#define __add_lvalue_reference(t) typename std::__cppbolt_add_lvalue_reference<t>::type
#define __add_rvalue_reference(t) typename std::__cppbolt_add_rvalue_reference<t>::type
#define __remove_extent(t) typename std::__cppbolt_remove_extent<t>::type
#define __remove_all_extents(t) typename std::__cppbolt_remove_all_extents<t>::type
#define __decay(t) typename std::__cppbolt_decay<t>::type
#define _LIBCPP_ALIGNOF(t) alignof(t)

#endif  // _LIBCPP_DETAIL_TYPE_TRAITS_BUILTINS_H
