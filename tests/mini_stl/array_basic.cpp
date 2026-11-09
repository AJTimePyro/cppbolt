#include <iostream>
#include <array>
#include <algorithm>

int main() {
    std::array<int, 4> arr{3, 1, 4, 2};
    std::sort(arr.begin(), arr.end());
    for (int x : arr) {
        std::cout << x << " ";
    }
    std::cout << std::endl;
    std::cout << "size: " << arr.size() << std::endl;
    std::cout << "front: " << arr.front() << " back: " << arr.back() << std::endl;
    return 0;
}
