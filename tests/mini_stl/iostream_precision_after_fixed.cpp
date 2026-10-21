#include <iostream>
#include <iomanip>

int main() {
    std::cout << std::fixed << std::setprecision(2) << 3.14159 << std::endl;
    std::cout << std::setprecision(4) << 2.5 << std::endl;
    return 0;
}
