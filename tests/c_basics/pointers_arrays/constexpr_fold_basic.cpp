#include <iostream>

int main() {
    int arr[(sizeof(int) * 2 == 8) ? 5 : 2];
    std::cout << sizeof(arr) / sizeof(int) << std::endl;
    return 0;
}
