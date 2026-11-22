







#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_EXTENT_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_EXTENT_H

#include <compat/compiler/compiler.h>
#include <cstddef>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD



template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS remove_extent {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp, size_t _Np>
struct remove_extent<_Tp[_Np]> {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
struct remove_extent<_Tp[]> {
  using type _LIBCPP_NODEBUG = _Tp;
};

template <class _Tp>
using __remove_extent_t _LIBCPP_NODEBUG = typename remove_extent<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_extent_t = __remove_extent_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_EXTENT_H
