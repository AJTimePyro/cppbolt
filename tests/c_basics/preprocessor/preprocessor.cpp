#include <cstdio>

#define MULTIPLY(a, b) ((a) * (b))
#define CONST_VAL 42

#define FEATURE_A
#ifndef FEATURE_B
#define FEATURE_B_VALUE 100
#endif

int main() {
    int x = MULTIPLY(3 + 2, 4);
    std::printf("macro mul: %d\n", x);
    std::printf("const: %d\n", CONST_VAL);
#ifdef FEATURE_A
    std::printf("feature A active: %d\n", FEATURE_B_VALUE);
#endif
    return 0;
}
