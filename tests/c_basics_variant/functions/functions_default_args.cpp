#include <cstdio>

int calc(int a, int b = 5, int c = 10) {
    return a * 100 + b * 10 + c;
}

int main() {
    std::printf("c1: %d\n", calc(2));
    std::printf("c2: %d\n", calc(2, 7));
    std::printf("c3: %d\n", calc(2, 7, 9));
    return 0;
}
