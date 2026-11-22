







#ifndef _LIBCPP___TYPE_TRAITS_NEGATION_H
#define _LIBCPP___TYPE_TRAITS_NEGATION_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

template <class _Pred>
struct _Not : _BoolConstant<!_Pred::value> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS negation : _Not<_Tp> {};
template <class _Tp>
_LIBCPP_NO_SPECIALIZATIONS inline constexpr bool negation_v = !_Tp::value;
#endif // _LIBCPP_STD_VER >= 17

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_NEGATION_H
