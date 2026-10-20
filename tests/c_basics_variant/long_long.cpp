#include <cstdio>

int main() {
    long long a = 1234567890123LL;
    long long b = 9876543210LL;

    std::printf("ll_add: %lld\n", a + b);
    std::printf("ll_sub: %lld\n", a - b);
    std::printf("ll_mul: %lld\n", b * 3LL);
    std::printf("ll_shift: %lld\n", 1LL << 35);
    return 0;
}
