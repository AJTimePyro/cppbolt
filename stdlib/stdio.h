#pragma once
#include <stddef.h>
extern "C" {
    int printf(const char*, ...);
    int sprintf(char*, const char*, ...);
    int snprintf(char*, size_t, const char*, ...);
    int scanf(const char*, ...);
    int getchar(void);
    int putchar(int);
    int puts(const char*);
}
