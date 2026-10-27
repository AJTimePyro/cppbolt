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

    int sum2 = 0;
    for (const auto x : arr) {
        sum2 += x;
    }
    std::cout << "const auto: " << sum2 << std::endl;

    int sum3 = 0;
    for (const auto& x : arr) {
        sum3 += x;
    }
    std::cout << "const auto&: " << sum3 << std::endl;

    int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}};
    int matrix_sum = 0;
    for (auto& row : matrix) {
        matrix_sum += row[0] + row[1] + row[2];
    }
    std::cout << "matrix sum: " << matrix_sum << std::endl;

    return 0;
}
