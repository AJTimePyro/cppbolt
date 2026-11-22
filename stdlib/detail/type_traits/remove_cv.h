







#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_CV_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_CV_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS remove_cv {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
struct remove_cv<const _Tp> {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
using __remove_cv_t _LIBCPP_NODEBUG = typename remove_cv<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_cv_t = __remove_cv_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_CV_H
