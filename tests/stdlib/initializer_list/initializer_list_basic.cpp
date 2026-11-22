#include <initializer_list>
#include <iostream>

int sum(std::initializer_list<int> values) {
    int total = 0;
    for (int value : values) {
        total += value;
    }
    return total;
}

int main() {
    std::cout << "sum: " << sum({1, 2, 3, 4}) << std::endl;
    auto values = {10, 20, 30, 40};
    std::cout << "auto: ";
    for (int value : values) {
        std::cout << value << " ";
    }
    std::cout << std::endl;
    return 0;
}
