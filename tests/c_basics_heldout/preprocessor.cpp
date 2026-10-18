#include <cstdio>

#define MULTIPLY(x, y) ((x) * (y))
#define OFFSET 100
#define MODE_ACTIVE 1

int main() {
#ifdef MODE_ACTIVE
    std::printf("mode: %d\n", MODE_ACTIVE);
#endif

#ifndef UNDEFINED_FLAG
    std::printf("macro_calc: %d\n", MULTIPLY(6, 7) + OFFSET);
#endif
    return 0;
}
