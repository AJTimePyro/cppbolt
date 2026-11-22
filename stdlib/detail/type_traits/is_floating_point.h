







#ifndef _LIBCPP___TYPE_TRAITS_IS_FLOATING_POINT_H
#define _LIBCPP___TYPE_TRAITS_IS_FLOATING_POINT_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




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
