#include <cstdio>

int main() {
    int plain = printf("plain\n");
    int mixed = printf("%d-%s-%x\n", 7, "ok", 255);
    int padded = printf("%5d|%-5d|%05d\n", 42, 42, 42);
    int floats = printf("%.2f %.1e\n", 3.14159, 250.0);
    int line = puts("line");
    int ch = putchar('Q');
    printf("%d %d %d %d %d %d\n", plain, mixed, padded, floats, line, ch);
    return 0;
}
