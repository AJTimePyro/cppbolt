







#ifndef _LIBCPP___TYPE_TRAITS_IS_ARITHMETIC_H
#define _LIBCPP___TYPE_TRAITS_IS_ARITHMETIC_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>
#include <detail/type_traits/is_floating_point.h>
#include <detail/type_traits/is_integral.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS is_arithmetic
    : integral_constant<bool, (is_integral<_Tp>::value || is_floating_point<_Tp>::value)> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
_LIBCPP_NO_SPECIALIZATIONS inline constexpr bool is_arithmetic_v = is_arithmetic<_Tp>::value;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_ARITHMETIC_H
