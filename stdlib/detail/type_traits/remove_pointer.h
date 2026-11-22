







#ifndef _LIBCPP___TYPE_TRAITS_REMOVE_POINTER_H
#define _LIBCPP___TYPE_TRAITS_REMOVE_POINTER_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS remove_pointer {
  using type _LIBCPP_NODEBUG = _Tp;
};
template <class _Tp>
struct remove_pointer<_Tp*> {
  using type _LIBCPP_NODEBUG = _Tp;
};

template <class _Tp>
using __remove_pointer_t _LIBCPP_NODEBUG = typename remove_pointer<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using remove_pointer_t = __remove_pointer_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_REMOVE_POINTER_H
