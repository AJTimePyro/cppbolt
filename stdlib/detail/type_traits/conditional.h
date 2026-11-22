







#ifndef _LIBCPP___TYPE_TRAITS_CONDITIONAL_H
#define _LIBCPP___TYPE_TRAITS_CONDITIONAL_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <bool _Bp, class _IfRes, class _ElseRes>
struct _LIBCPP_NO_SPECIALIZATIONS conditional {
  using type _LIBCPP_NODEBUG = _IfRes;
};

template <class _IfRes, class _ElseRes>
struct conditional<false, _IfRes, _ElseRes> {
  using type _LIBCPP_NODEBUG = _ElseRes;
};

#if _LIBCPP_STD_VER >= 14
template <bool _Bp, class _IfRes, class _ElseRes>
using conditional_t _LIBCPP_NODEBUG = typename conditional<_Bp, _IfRes, _ElseRes>::type;
#endif


template <bool _Bp, class _If, class _Then>
using __conditional_t _LIBCPP_NODEBUG = typename conditional<_Bp, _If, _Then>::type;

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_CONDITIONAL_H
