







#ifndef _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H
#define _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H

#include <compat/compiler/compiler.h>
#include <detail/type_traits/is_unsigned.h>

#if !defined(_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER)
#  pragma GCC system_header
#endif

_LIBCPP_BEGIN_NAMESPACE_STD





template <class _Tp>
struct __make_unsigned {};


template <> struct __make_unsigned<bool>               {};
template <> struct __make_unsigned<char>               {typedef unsigned char     type;};
template <> struct __make_unsigned<signed char>        {typedef unsigned char     type;};
template <> struct __make_unsigned<unsigned char>      {typedef unsigned char     type;};
template <> struct __make_unsigned<short>              {typedef unsigned short     type;};
template <> struct __make_unsigned<unsigned short>     {typedef unsigned short     type;};
template <> struct __make_unsigned<int>                {typedef unsigned int       type;};
template <> struct __make_unsigned<unsigned int>       {typedef unsigned int       type;};
template <> struct __make_unsigned<long>               {typedef unsigned long      type;};
template <> struct __make_unsigned<unsigned long>      {typedef unsigned long      type;};
template <> struct __make_unsigned<long long>          {typedef unsigned long long type;};
template <> struct __make_unsigned<unsigned long long> {typedef unsigned long long type;};


template <class _Tp>
using __make_unsigned_t = typename __make_unsigned<_Tp>::type;

template <class _Tp>
struct _LIBCPP_NO_SPECIALIZATIONS make_unsigned {
  using type _LIBCPP_NODEBUG = __make_unsigned_t<_Tp>;
};

#if _LIBCPP_STD_VER >= 14
template <class _Tp>
using make_unsigned_t = __make_unsigned_t<_Tp>;
#endif

_LIBCPP_END_NAMESPACE_STD

#endif // _LIBCPP___TYPE_TRAITS_MAKE_UNSIGNED_H
