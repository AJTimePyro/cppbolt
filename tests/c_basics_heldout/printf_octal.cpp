#include <cstdio>

int main() {
    printf("[%#o] [%o]\n", 64, 64);
    printf("[%#.5o] [%.5o]\n", 64, 64);
    printf("[%#6o] [%#-6o]\n", 64, 64);
    printf("[%#06o] [%#6.4o]\n", 64, 64);
    printf("[%#x] [%#10x] [%#-10x]\n", 4660, 4660, 4660);
    printf("[%#X] [%#08X]\n", 4660, 4660);
    printf("[%#o]\n", 0);
    printf("[%#.0o]\n", 0);
    printf("[%#.0x]\n", 0);
    printf("[%#o]\n", 4294967295u);
    return 0;
}
