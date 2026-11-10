#include <iostream>

template <typename T, int N = 4>
struct FixedArray {
    T data[N];
    int size() const { return N; }
};

int main() {
    FixedArray<int> arr1;
    FixedArray<int, 2> arr2;
    std::cout << arr1.size() << " " << arr2.size() << std::endl;
    return 0;
}
