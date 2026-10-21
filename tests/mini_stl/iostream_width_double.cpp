#include <iostream>
#include <iomanip>

int main() {
    std::cout << std::fixed << std::setprecision(3);
    std::cout << std::setw(9) << 1.5 << std::endl;
    std::cout << std::setw(7) << 12.25 << std::endl;
    return 0;
}
