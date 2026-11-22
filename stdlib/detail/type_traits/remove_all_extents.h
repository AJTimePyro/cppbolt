







#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_ALL_EXTENTS_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_ALL_EXTENTS_H

#include <compat/compiler/compiler.h>
#include <cstddef>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD



template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS remove_all_extents {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp, size_t _Np>
struct remove_all_extents<_Tp[_Np]> {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
struct remove_all_extents<_Tp[]> {
  using type _LIBCPP_NODEBUG = _Tp;
};


template <class _Tp, size_t _Np, size_t _Mp>
struct remove_all_extents<_Tp[_Np][_Mp]> {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp, size_t _Np, size_t _Mp, size_t _Kp>
struct remove_all_extents<_Tp[_Np][_Mp][_Kp]> {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
using __remove_all_extents_t _LIBCPP_NODEBUG = typename remove_all_extents<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_all_extents_t = __remove_all_extents_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_ALL_EXTENTS_H
