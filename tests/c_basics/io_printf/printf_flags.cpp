#include <cstdio>

int main() {
    // %x and %X
    std::printf("%x\n", 255);
    std::printf("%X\n", 255);
    std::printf("%04x\n", 42);
    std::printf("%08X\n", 3735928559u); // 0xDEADBEEF

    // width and alignment
    std::printf("%8d\n", 123);
    std::printf("%-8d|\n", 123);

    // sign flags
    std::printf("%+d\n", 123);
    std::printf("%+d\n", -123);

    // precision flags
    std::printf("%.5d\n", 42);
    std::printf("%10.5d\n", 42);

    // float precision and rounding
    std::printf("%.2f\n", 3.14159);
    std::printf("%.4f\n", -0.0125);
    std::printf("%8.2f\n", 3.14);
    std::printf("%.1f\n", 1.999);
    std::printf("%.2f\n", -42.755);

    // large values
    std::printf("%lld\n", 12345678901234LL);
    std::printf("%+lld\n", -9876543210123LL);
    return 0;
}
