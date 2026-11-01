#include <cstdio>

int main() {
    // 1. Octal integer literals (leading 0)
    int oct1 = 0755;
    int oct2 = 077;
    int oct_zero = 0;

    // 2. Binary integer literals (0b / 0B)
    int bin1 = 0b1010;
    int bin2 = 0B1111;

    // 3. Hex integer literals with digit separators
    int hex_sep = 0xDE'AD'BE'EF;

    // 4. Decimal integer literals with digit separators
    int dec_sep = 1'000'000;

    // 5. Octal and binary with digit separators
    int oct_sep = 0'755;
    int bin_sep = 0b1010'0101;

    // 6. Floating-point literals with digit separators
    double pi_sep = 3.141'592;
    double exp_sep = 1.0e1'0;

    std::printf("oct: %d %d %d\n", oct1, oct2, oct_zero);
    std::printf("bin: %d %d\n", bin1, bin2);
    std::printf("hex_sep: %u\n", (unsigned int)hex_sep);
    std::printf("dec_sep: %d\n", dec_sep);
    std::printf("oct_sep: %d\n", oct_sep);
    std::printf("bin_sep: %d\n", bin_sep);
    std::printf("float_sep: %.6f\n", pi_sep);
    std::printf("exp_sep: %.1e\n", exp_sep);

    return 0;
}
