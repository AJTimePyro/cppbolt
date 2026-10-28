#include <cstdio>

#if (1 ? 0 : 1)
#error wrong branch
#elif (0 ? 1 : 0) || (1 ? 1 : 0)
#define VALUE 7
#else
#error wrong fallback
#endif

int main() {
    std::printf("%d\n", VALUE);
    return 0;
}
