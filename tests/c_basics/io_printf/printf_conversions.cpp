#include <cstdio>

int main() {
    printf("%c %s\n", 'Z', "text");
    printf("%u %lu %ld\n", 4000000000u, 1234567890ul, -1234567890l);
    printf("%ld %lld\n", -77l, 9000000000LL);
    printf("%x %X %#x %#X\n", 48879, 48879, 48879, 48879);
    printf("%08x %#010x\n", 291, 291);
    printf("%e %.2e\n", 12345.678, 0.000123456);
    printf("%g %g %g\n", 12345.678, 0.000123456, 100000.0);
    printf("%f %.3f %10.2f|\n", 2.5, 1.0 / 3.0, 3.14159);
    printf("%u %o %#o\n", 8u, 8u, 8u);
    return 0;
}
