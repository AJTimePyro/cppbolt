#include <cstdio>

struct Small {
    char c;
    int i;
};

int main() {
    std::printf("sizeof(char): %zu\n", sizeof(char));
    std::printf("sizeof(int): %zu\n", sizeof(int));
    std::printf("sizeof(double): %zu\n", sizeof(double));
    std::printf("sizeof(Small): %zu\n", sizeof(Small));

    int arr[10];
    std::printf("sizeof(arr): %zu\n", sizeof(arr));
    std::printf("elements: %zu\n", sizeof(arr) / sizeof(arr[0]));
    return 0;
}
