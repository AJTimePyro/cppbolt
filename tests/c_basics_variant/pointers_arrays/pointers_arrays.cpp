#include <cstdio>

int main() {
    int arr[6];
    for (int i = 0; i < 6; i = i + 1) {
        arr[i] = (i + 1) * 11;
    }

    int* p = arr + 2;
    std::printf("ptr_val: %d\n", *p);
    std::printf("ptr_offset: %d\n", *(p + 2));
    *(p + 1) = 999;
    std::printf("arr_modified: %d\n", arr[3]);

    int val = 42;
    int* ptr = &val;
    *ptr = 84;
    std::printf("val_mod: %d\n", val);
    return 0;
}
