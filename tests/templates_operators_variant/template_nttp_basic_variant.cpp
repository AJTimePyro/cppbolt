#include <iostream>

template <typename T, unsigned int Size>
struct StaticArray {
    T buffer[Size];
    unsigned int length() const { return Size; }
    void fill(T val) {
        for (unsigned int i = 0; i < Size; ++i) {
            buffer[i] = val;
        }
    }
    T get(unsigned int idx) const { return buffer[idx]; }
};

int main() {
    StaticArray<int, 3 * 2> arr;
    arr.fill(7);
    std::cout << arr.length() << " " << arr.get(3) << std::endl;
    return 0;
}
