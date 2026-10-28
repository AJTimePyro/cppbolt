#include <cstdio>

int compute(int x) {
    return x * 2;
}

int compute(int x, int y) {
    return x + y;
}

double compute(double d) {
    return d * 1.5;
}

int main() {
    std::printf("overload_1: %d\n", compute(21));
    std::printf("overload_2: %d\n", compute(15, 25));
    std::printf("overload_d: %f\n", compute(4.0));
    return 0;
}
