#include <cstdio>

class IntArray {
    int data[4];
public:
    IntArray() {
        for (int i = 0; i < 4; ++i) data[i] = 0;
    }
    int& operator[](int idx) {
        return data[idx];
    }
    int operator[](int idx) const {
        return data[idx];
    }
};

int main() {
    IntArray arr;
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    arr[3] = 40;

    std::printf("arr[1]: %d, arr[3]: %d\n", arr[1], arr[3]);
    return 0;
}
