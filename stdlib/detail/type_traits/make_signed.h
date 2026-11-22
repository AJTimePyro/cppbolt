







#ifndef _LIBCPP___TYPE_TRAITS_MAKE_SIGNED_H
#define _LIBCPP___TYPE_TRAITS_MAKE_SIGNED_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/builtins.h>
#include <detail/type_traits/is_integral.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD





template <class _Tp>
struct __make_signed {};


template <> struct __make_signed<bool>               {};
template <> struct __make_signed<char>               {typedef signed char type;};
template <> struct __make_signed<signed char>        {typedef signed char type;};
template <> struct __make_signed<unsigned char>      {typedef signed char type;};
template <> struct __make_signed<short>              {typedef short     type;};
template <> struct __make_signed<unsigned short>     {typedef short     type;};
template <> struct __make_signed<int>                {typedef int       type;};
template <> struct __make_signed<unsigned int>       {typedef int       type;};
template <> struct __make_signed<long>               {typedef long      type;};
template <> struct __make_signed<unsigned long>      {typedef long      type;};
template <> struct __make_signed<long long>          {typedef long long type;};
template <> struct __make_signed<unsigned long long> {typedef long long type;};


template <class _Tp>
using __make_signed_t = typename __make_signed<_Tp>::type;

template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS make_signed {
  using type _LIBCPP_NODEBUG = __make_signed_t<_Tp>;
};

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using make_signed_t = __make_signed_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_MAKE_SIGNED_H
