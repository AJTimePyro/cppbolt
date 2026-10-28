#include <cstdio>

#define HEADER <cstdio>
#include HEADER

#define A 1
#define B A
#define ADD(x, y) ((x) + (y))
#define STR(x) #x
#define CAT(a, b) a ## b
#define FLAG 2

int main() {
    int CAT(val, ue) = ADD(B, 2);
    printf("value=%d\n", value);
    printf("rescan=%d\n", B - 1);
#if B == A && FLAG == 2
    printf("if\n");
#elif 1
    printf("elif\n");
#else
    printf("else\n");
#endif
#if defined(A) && !defined(MISSING)
    printf("defined\n");
#endif
    printf("string=%s\n", STR(hello));
    return 0;
}
