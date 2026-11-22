







#ifndef _LIBCPP___TYPE_TRAITS_ADD_POINTER_H
#define _LIBCPP___TYPE_TRAITS_ADD_POINTER_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS add_pointer {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
struct add_pointer<_Tp&> {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
struct add_pointer<_Tp&&> {
  using type _LIBCPP_NODEBUG = _Tp*;
};

template <class _Tp>
using __add_pointer_t _LIBCPP_NODEBUG = typename add_pointer<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using add_pointer_t = __add_pointer_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_ADD_POINTER_H
