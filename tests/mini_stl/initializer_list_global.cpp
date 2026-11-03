#include <initializer_list>
#include <iostream>
#include <vector>

std::vector<int> numbers = {1, 2, 3};
std::initializer_list<int> values = {4, 5, 6};

int sum_globals() {
    int total = 0;
    for (size_t i = 0; i < numbers.size(); ++i) {
        total += numbers[i];
    }
    for (int value : values) {
        total += value;
    }
    return total;
}

int main() {
    std::cout << "numbers: " << numbers.size() << std::endl;
    std::cout << "values: " << values.size() << std::endl;
    std::cout << "sum: " << sum_globals() << std::endl;
    return 0;
}
