#include <iostream>
#include <iomanip>

int main() {
    double first = 2.75;
    double second = 0.5;
    std::cout << std::fixed << std::setprecision(2);
    std::cout << std::setw(8) << first << std::endl;
    std::cout << std::setw(10) << second << std::endl;
    return 0;
}
