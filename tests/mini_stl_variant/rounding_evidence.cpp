#include <cstdio>
#include <iostream>
#include <iomanip>

int main() {
    std::printf("%.1f %.1f %.1f\n", 0.05, 0.25, 0.15);
    std::printf("%.2f %.2f %.2f\n", 1.125, 2.375, 0.005);
    std::printf("%.0f %.0f %.0f\n", 1.5, 3.5, 4.5);
    std::printf("%.3f %.3f\n", 0.0625, 0.1875);
    std::printf("%e\n", 1e-300);
    std::printf("%g %g %g\n", 1e4, 1e-3, 123456.7);
    double zero = 0.0;
    double inf_val = 1.0 / zero;
    double ninf_val = -1.0 / zero;
    unsigned long long nan_bits = 0x7ff8000000000000ULL;
    double nan_val;
    char* src = (char*)&nan_bits;
    char* dst = (char*)&nan_val;
    for (int i = 0; i < 8; i++) dst[i] = src[i];
    double nzero_val = -0.0;
    std::printf("%f %f %f %f\n", inf_val, ninf_val, nan_val, nzero_val);
    std::cout << std::setprecision(15) << 0.1 << "\n";
    return 0;
}
