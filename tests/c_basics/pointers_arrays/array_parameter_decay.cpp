#include <cstdio>

void modify_array(int arr[], int arr_sized[5], int n) {
    for (int i = 0; i < n; i++) {
        arr[i] += 10;
        arr_sized[i] *= 2;
    }
}

int sum_elements(const int arr[], int n) {
    int s = 0;
    for (int i = 0; i < n; i++) {
        s += arr[i];
    }
    return s;
}

int main() {
    int a[4] = {1, 2, 3, 4};
    int b[4] = {10, 20, 30, 40};
    modify_array(a, b, 4);
    std::printf("%d %d %d %d\n", a[0], a[1], a[2], a[3]);
    std::printf("%d %d %d %d\n", b[0], b[1], b[2], b[3]);
    std::printf("sum=%d\n", sum_elements(a, 4));
    return 0;
}
