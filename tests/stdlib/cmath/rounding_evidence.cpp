#include <cstdio>
#include <iostream>
#include <iomanip>

int main() {
    std::printf("%.1f %.1f %.1f\n", 0.15, 0.45, 0.35);
    std::printf("%.2f %.2f %.2f\n", 2.675, 1.005, 0.35);
    std::printf("%.0f %.0f %.0f\n", 0.5, 1.5, 2.5);
    std::printf("%.2f %.2f %.2f\n", 0.125, 0.375, 0.625);
    std::printf("%f\n", 1e23);
    std::printf("%e\n", 5e-324);
    std::printf("%g %g %g %g\n", 1e5, 1e6, 1e-4, 1e-5);
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
    std::cout << std::setprecision(17) << 0.1 << "\n";
    return 0;
}
