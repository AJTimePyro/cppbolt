#include <initializer_list>
#include <iostream>

int product(std::initializer_list<int> values) {
    int result = 1;
    for (int value : values) {
        result *= value;
    }
    return result;
}

int main() {
    std::cout << "product: " << product({2, 3, 4}) << std::endl;
    auto values = {5, 15, 25};
    std::cout << "auto: ";
    for (int value : values) {
        std::cout << value << " ";
    }
    std::cout << std::endl;
    return 0;
}
