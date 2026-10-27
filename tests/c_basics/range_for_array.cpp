#include <iostream>

int main() {
    int arr[4] = {10, 20, 30, 40};
    int sum = 0;
    for (int x : arr) {
        sum += x;
    }
    std::cout << sum << std::endl;

    for (int& x : arr) {
        x *= 2;
    }
    for (auto x : arr) {
        std::cout << x << " ";
    }
    std::cout << std::endl;

    return 0;
}
