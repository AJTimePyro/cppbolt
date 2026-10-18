#include <cstdio>

int main() {
    // Different held-out values and formatting flags
    std::printf("hex_lower: %x\n", 4095);
    std::printf("hex_upper: %X\n", 4095);
    std::printf("hex_pad: %06x\n", 1234);
    std::printf("hex_large: %08X\n", 305419896u); // 0x12345678

    std::printf("width_r: %10d\n", 9876);
    std::printf("width_l: %-10d|\n", 9876);

    std::printf("pos_sign: %+d\n", 456);
    std::printf("neg_sign: %+d\n", -456);

    std::printf("prec_int: %.6d\n", 789);
    std::printf("w_and_prec: %12.6d\n", 789);

    std::printf("flt_prec: %.3f\n", 2.71828);
    std::printf("flt_round: %.2f\n", 0.0099);
    std::printf("flt_w: %10.3f\n", 12.34);
    std::printf("flt_neg: %.2f\n", -99.999);

    std::printf("ll_val: %lld\n", 9000000000000LL);
    std::printf("ll_signed: %+lld\n", -7000000000000LL);
    return 0;
}
