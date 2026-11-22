#include <iostream>
#include <iomanip>

int main() {
    double first = 6.28318;
    double second = 1.25;
    std::cout << std::fixed << std::setprecision(3) << first << std::endl;
    std::cout << std::setprecision(5) << second << std::endl;
    return 0;
}
