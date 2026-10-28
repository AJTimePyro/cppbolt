#include <iostream>

int main() {
    int arr[3] = {10, 20, 12};
    auto sum_arr = [=]() {
        int sum = 0;
        for (int x : arr) {
            sum += x;
        }
        return sum;
    };
    std::cout << sum_arr() << std::endl;
    return 0;
}
