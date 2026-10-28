#include <cstdio>

int compute(int base, int mul = 2, int offset = 10) {
    return base * mul + offset;
}

int main() {
    std::printf("all defaults: %d\n", compute(5));
    std::printf("one default: %d\n", compute(5, 3));
    std::printf("no defaults: %d\n", compute(5, 3, 1));
    return 0;
}
