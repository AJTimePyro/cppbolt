#include <cstdio>

int main() {
    int x = 73;
    int y = 7;
    long val = 450000L;
    unsigned int count = 99u;

    std::printf("sum: %d\n", x + y);
    std::printf("diff: %d\n", x - y);
    std::printf("prod: %d\n", x * y);
    std::printf("quot: %d\n", x / y);
    std::printf("rem: %d\n", x % y);
    std::printf("bw_and: %d\n", x & y);
    std::printf("bw_or: %d\n", x | y);
    std::printf("bw_xor: %d\n", x ^ y);
    std::printf("lshift: %d\n", y << 3);
    std::printf("rshift: %d\n", x >> 2);
    std::printf("inv: %d\n", ~y);
    std::printf("long_add: %ld\n", val + 12345);
    std::printf("uint_mul: %u\n", count * 3);

    int k = 40;
    k -= 10;
    k /= 3;
    std::printf("compound: %d\n", k);
    std::printf("postdec: %d\n", k--);
    std::printf("predec: %d\n", --k);
    return 0;
}
