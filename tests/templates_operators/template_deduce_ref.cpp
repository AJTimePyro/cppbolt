#include <cstdio>

template <typename T>
void increment(T& x) {
    x += 1;
}

template <typename T>
void print_val(const T& x) {
    std::printf("val: %d\n", (int)x);
}

template <typename T>
void print_ptr(T* p) {
    std::printf("ptr: %d\n", *p);
}

int main() {
    int a = 41;
    increment(a);
    print_val(a);
    print_ptr(&a);
    return 0;
}
