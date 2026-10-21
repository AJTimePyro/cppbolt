#include <iostream>
#include <iomanip>

int main() {
    std::cout << std::fixed << std::setprecision(2);
    std::cout << std::setw(8) << 2.75 << std::endl;
    std::cout << std::setw(10) << 0.5 << std::endl;
    return 0;
}
