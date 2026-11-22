







#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <class _Tp>
struct remove_reference {
  typedef _Tp type;
};
template <class _Tp>
struct remove_reference<_Tp&> {
  typedef _Tp type;
};
template <class _Tp>
struct remove_reference<_Tp&&> {
  typedef _Tp type;
};

template <class _Tp>
using __libcpp_remove_reference_t = typename remove_reference<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_reference_t = __libcpp_remove_reference_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_REFERENCE_H
