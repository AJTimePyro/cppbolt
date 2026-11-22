







#ifndef _LIBCPP___TYPE_TRAITS_DECAY_H
#define _LIBCPP___TYPE_TRAITS_DECAY_H

#include <compat/compiler/compiler.h>
#include <cstddef>
#include <detail/type_traits/remove_cv.h>
#include <detail/type_traits/remove_reference.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD





template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS decay {
  using type _LIBCPP_NODEBUG =
      typename remove_cv<typename remove_reference<_Tp>::type>::type;
};
template <class _Tp, size_t _Np>
struct decay<_Tp[_Np]> {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
struct decay<_Tp[]> {
  using type _LIBCPP_NODEBUG = _Tp*;
};
template <class _Tp>
using __decay_t _LIBCPP_NODEBUG = typename decay<_Tp>::type;

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using decay_t = __decay_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_DECAY_H
