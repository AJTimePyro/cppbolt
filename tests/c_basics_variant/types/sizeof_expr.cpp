#include <cstdio>

int main() {
    int arr[7];
    int x = 10;

    std::printf("sizeof_int: %d\n", (int)sizeof(int));
    std::printf("sizeof_char: %d\n", (int)sizeof(char));
    std::printf("sizeof_long_long: %d\n", (int)sizeof(long long));
    std::printf("sizeof_double: %d\n", (int)sizeof(double));
    std::printf("sizeof_arr: %d\n", (int)sizeof(arr));
    std::printf("sizeof_expr: %d\n", (int)sizeof(x + 5));
    return 0;
}
