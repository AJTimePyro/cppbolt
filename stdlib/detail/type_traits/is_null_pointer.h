







#ifndef _LIBCPP___TYPE_TRAITS_IS_NULL_POINTER_H
#define _LIBCPP___TYPE_TRAITS_IS_NULL_POINTER_H

#include <compat/compiler/compiler.h>
#include <cstddef>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD



template <class _Tp>
struct __libcpp_is_null_pointer {
  static const bool value = false;
};
template <>
struct __libcpp_is_null_pointer<nullptr_t> {
  static const bool value = true;
};
template <class _Tp>
struct __libcpp_is_null_pointer<const _Tp> : __libcpp_is_null_pointer<_Tp> {};

template <class _Tp>
inline const bool __is_null_pointer_v = __libcpp_is_null_pointer<_Tp>::value;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS is_null_pointer : integral_constant<bool, __is_null_pointer_v<_Tp>> {};

#  if _LIBCPP_STD_VER >= 17
template <class _Tp>
_LIBCPP_NO_SPECIALIZATIONS inline constexpr bool is_null_pointer_v = is_null_pointer<_Tp>::value;
#  endif
#endif // _LIBCPP_STD_VER >= 14

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_NULL_POINTER_H
