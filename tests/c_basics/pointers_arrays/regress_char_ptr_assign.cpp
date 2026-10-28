#include <cstdio>

int main() {
    const char* str = "Hello, world! A longer string above 256 bytes offset to verify 32-bit pointer width.";
    const char* p = 0;
    p = str;
    std::printf("%s\n", p);

    char arr[10];
    arr[0] = 'O';
    arr[1] = 'K';
    arr[2] = '\0';
    char* p2 = 0;
    p2 = arr;
    std::printf("%s\n", p2);
    return 0;
}
