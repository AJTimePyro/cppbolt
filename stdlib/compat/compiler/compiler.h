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

#endif
