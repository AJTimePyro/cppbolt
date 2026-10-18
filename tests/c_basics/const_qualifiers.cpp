#include <cstdio>

int compute_const(const int x, const int* p) {
    return x + *p;
}

int main() {
    const int val = 50;
    const int arr[3] = {1, 2, 3};
    const int* ptr = &val;

    std::printf("val: %d, *ptr: %d\n", val, *ptr);
    std::printf("compute: %d\n", compute_const(val, &arr[1]));
    return 0;
}
