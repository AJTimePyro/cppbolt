#include <cstdio>

int main() {
    printf("%s %c\n", "held", 'q');
    printf("%lu %ld %u\n", 4294967295ul, -2000000000l, 7u);
    printf("%lld %ld\n", -9000000000LL, 42l);
    printf("%X %x %#X %#x\n", 65535, 65535, 65535, 65535);
    printf("%#014x %-12x|\n", 4660, 4660);
    printf("%.4e %e\n", 99.5, 0.00001234);
    printf("%g %g\n", 0.0001234, 123456789.0);
    printf("%12.4f %.2f\n", -2.71828, 9.999);
    printf("%#o %o %#x\n", 511u, 511u, 511u);
    return 0;
}
