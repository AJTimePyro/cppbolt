







#ifndef _LIBCPP___TYPE_TRAITS_VOID_T_H
#define _LIBCPP___TYPE_TRAITS_VOID_T_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD

#if _LIBCPP_STD_VER >= 17
template <class...>
using void_t = void;
#endif

template <class...>
using __void_t _LIBCPP_NODEBUG = void;

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_VOID_T_H
