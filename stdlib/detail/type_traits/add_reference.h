







#ifndef _LIBCPP___TYPE_TRAITS_ADD_REFERENCE_H
#define _LIBCPP___TYPE_TRAITS_ADD_REFERENCE_H

#include <compat/compiler/compiler.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD




template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS add_lvalue_reference {
  using type _LIBCPP_NODEBUG = _Tp&;
};
template <class _Tp>
struct add_lvalue_reference<_Tp&> {
  using type _LIBCPP_NODEBUG = _Tp&;
};
template <class _Tp>
struct add_lvalue_reference<_Tp&&> {
  using type _LIBCPP_NODEBUG = _Tp&;
};

template <class _Tp>
using __add_lvalue_reference_t _LIBCPP_NODEBUG = typename add_lvalue_reference<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using add_lvalue_reference_t = __add_lvalue_reference_t<_Tp>;
#endif

template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS add_rvalue_reference {
  using type _LIBCPP_NODEBUG = _Tp&&;
};
template <class _Tp>
struct add_rvalue_reference<_Tp&> {
  using type _LIBCPP_NODEBUG = _Tp&;
};
template <class _Tp>
struct add_rvalue_reference<_Tp&&> {
  using type _LIBCPP_NODEBUG = _Tp&&;
};

template <class _Tp>
using __add_rvalue_reference_t _LIBCPP_NODEBUG = typename add_rvalue_reference<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using add_rvalue_reference_t = __add_rvalue_reference_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_ADD_REFERENCE_H
