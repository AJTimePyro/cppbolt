#ifndef _MINI_STL_COMPAT_COMPILER_COMPILER_H_
#define _MINI_STL_COMPAT_COMPILER_COMPILER_H_

#if defined(__clang__)
#define _MINI_STL_CLANG_DIAG_PUSH _Pragma("clang diagnostic push")
#define _MINI_STL_CLANG_DIAG_POP  _Pragma("clang diagnostic pop")
#define _MINI_STL_CLANG_IGNORE_INLINE_NEW_DELETE _Pragma("clang diagnostic ignored \"-Winline-new-delete\"")
#else
#define _MINI_STL_CLANG_DIAG_PUSH
#define _MINI_STL_CLANG_DIAG_POP
#define _MINI_STL_CLANG_IGNORE_INLINE_NEW_DELETE
#endif

// Upstream libc++ compatibility definitions
#ifndef _LIBCPP_HIDE_FROM_ABI
#define _LIBCPP_HIDE_FROM_ABI
#endif

#ifndef _LIBCPP_NODEBUG
#define _LIBCPP_NODEBUG
#endif

#ifndef _LIBCPP_CONSTEXPR
#define _LIBCPP_CONSTEXPR constexpr
#endif

#ifndef _LIBCPP_CONSTEXPR_SINCE_CXX14
#define _LIBCPP_CONSTEXPR_SINCE_CXX14 constexpr
#endif

#ifndef _LIBCPP_CONSTEXPR_SINCE_CXX17
#define _LIBCPP_CONSTEXPR_SINCE_CXX17 constexpr
#endif

#ifndef _LIBCPP_CONSTEXPR_SINCE_CXX20
#define _LIBCPP_CONSTEXPR_SINCE_CXX20 constexpr
#endif

#ifndef _NOEXCEPT
#define _NOEXCEPT noexcept
#endif

#ifndef _NOEXCEPT_
#define _NOEXCEPT_(x) noexcept
#endif

#ifndef _LIBCPP_ASSERT_VALID_ELEMENT_ACCESS
#define _LIBCPP_ASSERT_VALID_ELEMENT_ACCESS(cond, msg) ((void)0)
#endif

#ifndef _LIBCPP_PUSH_MACROS
#define _LIBCPP_PUSH_MACROS
#endif

#ifndef _LIBCPP_POP_MACROS
#define _LIBCPP_POP_MACROS
#endif

#ifndef _LIBCPP_BEGIN_NAMESPACE_STD
#define _LIBCPP_BEGIN_NAMESPACE_STD namespace std {
#endif

#ifndef _LIBCPP_END_NAMESPACE_STD
#define _LIBCPP_END_NAMESPACE_STD }
#endif

// Additional definitions needed by upstream libc++ type_traits internals.
#ifndef _LIBCPP_STD_VER
#define _LIBCPP_STD_VER 17
#endif

#ifndef _LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER
#define _LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER 1
#endif

#ifndef _LIBCPP_NO_SPECIALIZATIONS
#define _LIBCPP_NO_SPECIALIZATIONS
#endif

#ifndef _LIBCPP_DIAGNOSTIC_PUSH
#define _LIBCPP_DIAGNOSTIC_PUSH
#endif

#ifndef _LIBCPP_DIAGNOSTIC_POP
#define _LIBCPP_DIAGNOSTIC_POP
#endif

#ifndef _LIBCPP_CLANG_DIAGNOSTIC_IGNORED
#define _LIBCPP_CLANG_DIAGNOSTIC_IGNORED(x)
#endif

// cppbolt has no 128-bit integers, wide characters or char8_t.
#define _LIBCPP_HAS_INT128 0
#define _LIBCPP_HAS_WIDE_CHARACTERS 0
#define _LIBCPP_HAS_CHAR8_T 0

// Let upstream headers choose their pure-template fallback branches. The only
// primitive without an upstream fallback (`remove_reference`) is supplied by
// detail/type_traits/builtins.h via a cppbolt adaptation of that header.
#define __has_builtin(x) 0
#ifndef __has_warning
#define __has_warning(x) 0
#endif
#ifndef __has_cpp_attribute
#define __has_cpp_attribute(x) 0
#endif
#ifndef __has_feature
#define __has_feature(x) 0
#endif

#endif
