#include <iostream>

int main() {
    float f = 12.345f;
    double d = 67.89;
    std::cout << "f: " << f << std::endl;
    std::cout << "d: " << d << std::endl;

    // Special cases: 0.1, 100.0, 1e-5
    std::cout << "val_0_1: " << 0.1 << std::endl;
    std::cout << "val_100: " << 100.0 << std::endl;
    std::cout << "val_1e_5: " << 1e-5 << std::endl;

    // Negatives, large values, and rounding
    std::cout << "neg: " << -42.75 << std::endl;
    std::cout << "large: " << 1234567.89 << std::endl;
    std::cout << "sci_large: " << 1e6 << std::endl;
    std::cout << "round_pi: " << 3.1415926535 << std::endl;
    std::cout << "round_up: " << 1.999999 << std::endl;
    return 0;
}
