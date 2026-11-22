







#ifndef _LIBCPP___TYPE_TRAITS_IS_SAME_H
#define _LIBCPP___TYPE_TRAITS_IS_SAME_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/integral_constant.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

template <class _Tp, class _Up>
struct _LIBCPP_NO_SPECIALIZATIONS is_same : _BoolConstant<__is_same(_Tp, _Up)> {};

#if _LIBCPP_STD_VER >= 17
template <class _Tp, class _Up>
_LIBCPP_NO_SPECIALIZATIONS inline constexpr bool is_same_v = __is_same(_Tp, _Up);
#endif








template <class _Tp, class _Up>
using _IsSame _LIBCPP_NODEBUG = _BoolConstant<__is_same(_Tp, _Up)>;

template <class _Tp, class _Up>
using _IsNotSame _LIBCPP_NODEBUG = _BoolConstant<!__is_same(_Tp, _Up)>;

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_IS_SAME_H
