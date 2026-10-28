#include <cstdio>

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int* p = arr;

    std::printf("arr[0]: %d\n", *p);
    std::printf("arr[2]: %d\n", *(p + 2));
    p += 3;
    std::printf("arr[3]: %d\n", *p);

    *p = 99;
    std::printf("updated arr[3]: %d\n", arr[3]);

    int sum = 0;
    for (int i = 0; i < 5; ++i) {
        sum += arr[i];
    }
    std::printf("sum: %d\n", sum);
    return 0;
}
