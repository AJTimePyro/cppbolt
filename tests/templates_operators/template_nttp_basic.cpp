#include <iostream>

template <typename T, int N>
struct FixedBuffer {
    T data[N];
    int size() const { return N; }
};

int main() {
    FixedBuffer<int, 2 + 2> buf1;
    FixedBuffer<int, 4> buf2;
    buf1.data[0] = 42;
    buf2 = buf1;
    std::cout << buf2.size() << " " << buf2.data[0] << std::endl;
    return 0;
}
