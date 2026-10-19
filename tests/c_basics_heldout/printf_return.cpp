#include <cstdio>

int main() {
    int first = printf("start|");
    int second = printf("[%c%c] %u %X\n", 'a', 'b', 4000000000u, 48879);
    int third = printf("%-6d!%6.3f\n", -12, 9.87654);
    int fourth = printf("%s\n", "tail");
    int line = puts("");
    int ch = putchar('#');
    printf("%d,%d,%d,%d,%d,%d\n", first, second, third, fourth, line, ch);
    return 0;
}
