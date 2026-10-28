#include <cstdio>

int main() {
    int a = 25;
    int b = 4;
    long l = 100000L;
    unsigned int u = 50u;

    std::printf("add: %d\n", a + b);
    std::printf("sub: %d\n", a - b);
    std::printf("mul: %d\n", a * b);
    std::printf("div: %d\n", a / b);
    std::printf("mod: %d\n", a % b);
    std::printf("and: %d\n", a & b);
    std::printf("or: %d\n", a | b);
    std::printf("xor: %d\n", a ^ b);
    std::printf("shl: %d\n", a << 2);
    std::printf("shr: %d\n", a >> 1);
    std::printf("not: %d\n", ~b);
    std::printf("long: %ld\n", l + 5);
    std::printf("unsigned: %u\n", u * 2);

    int c = 10;
    c += 5;
    c *= 2;
    std::printf("compound: %d\n", c);
    std::printf("postfix: %d\n", c++);
    std::printf("prefix: %d\n", ++c);
    return 0;
}
