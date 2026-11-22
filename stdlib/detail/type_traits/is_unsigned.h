







#ifndef _LIBCPP___TYPE_TRAITS_IS_UNSIGNED_H
#define _LIBCPP___TYPE_TRAITS_IS_UNSIGNED_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD





template <class _Tp>
struct __is_unsigned_impl {
  static const bool value = false;
};
template <>
struct __is_unsigned_impl<bool> {
  static const bool value = true;
};
template <>
struct __is_unsigned_impl<unsigned char> {
  static const bool value = true;
};
template <>
struct __is_unsigned_impl<unsigned short> {
  static const bool value = true;
};
template <>
struct __is_unsigned_impl<unsigned int> {
  static const bool value = true;
};
template <>
struct __is_unsigned_impl<unsigned long> {
  static const bool value = true;
};
template <>
struct __is_unsigned_impl<unsigned long long> {
  static const bool value = true;
};
template <class _Tp>
struct __is_unsigned_impl<const _Tp> : __is_unsigned_impl<_Tp> {};

template <class _Tp>
struct is_unsigned : integral_constant<bool, __is_unsigned_impl<_Tp>::value> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
inline constexpr bool is_unsigned_v = is_unsigned<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_UNSIGNED_H
