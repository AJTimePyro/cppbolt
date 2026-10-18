#include <cstdio>

struct ArrayWrapper {
    int buffer[3];
    ArrayWrapper() {
        buffer[0] = 111;
        buffer[1] = 222;
        buffer[2] = 333;
    }
    int& operator[](int i) {
        return buffer[i];
    }
};

void set_elem(ArrayWrapper& w, int idx, int v) {
    w[idx] = v;
}

int main() {
    ArrayWrapper arr;
    ArrayWrapper& a_ref = arr;
    std::printf("a_ref[0]: %d\n", a_ref[0]);
    set_elem(arr, 0, 777);
    std::printf("a_ref[0] modified: %d\n", a_ref[0]);
    std::printf("a_ref[2]: %d\n", a_ref[2]);
    return 0;
}
