#ifndef _MINI_STL_STDLIB_H_
#define _MINI_STL_STDLIB_H_

#include <stddef.h>

#ifdef __CPP_WASM__
extern "C" {
    void* malloc(size_t);
    void free(void*);
    int printf(const char*, ...);
}

inline void* memcpy(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    for (size_t i = 0; i < n; ++i) {
        d[i] = s[i];
    }
    return dest;
}

inline void* memmove(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    if (d < s) {
        for (size_t i = 0; i < n; ++i) {
            d[i] = s[i];
        }
    } else if (d > s) {
        for (size_t i = n; i > 0; --i) {
            d[i - 1] = s[i - 1];
        }
    }
    return dest;
}

inline void* memset(void* dest, int c, size_t n) {
    char* d = (char*)dest;
    for (size_t i = 0; i < n; ++i) {
        d[i] = (char)c;
    }
    return dest;
}

inline size_t strlen(const char* s) {
    if (!s) return 0;
    size_t len = 0;
    while (s[len]) len++;
    return len;
}

inline int strcmp(const char* s1, const char* s2) {
    if (!s1 && !s2) return 0;
    if (!s1) return -1;
    if (!s2) return 1;
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return (int)(unsigned char)*s1 - (int)(unsigned char)*s2;
}
#else
extern "C" {
    void* malloc(size_t);
    void free(void*);
    void* memcpy(void*, const void*, size_t);
    void* memmove(void*, const void*, size_t);
    void* memset(void*, int, size_t);
    int strcmp(const char*, const char*);
    size_t strlen(const char*);
    int printf(const char*, ...);
    int snprintf(char*, size_t, const char*, ...);
}
#endif

#endif
