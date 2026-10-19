#include <cstdio>

int main() {
    std::printf("int: %d\n", -42);
    std::printf("octal: %#o\n", 64);
    std::printf("hex: %#X\n", 48879);
    std::printf("text: %s\n", "mini stdio");
    std::printf("char: %c\n", 'M');
    std::printf("real: %.2f\n", 2.71828);
    return 0;
}
