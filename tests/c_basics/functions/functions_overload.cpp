#include <cstdio>

int add(int a, int b) {
    return a + b;
}

double add(double a, double b) {
    return a + b;
}

int add(int a, int b, int c) {
    return a + b + c;
}

int main() {
    std::printf("add(int): %d\n", add(3, 4));
    std::printf("add(double): %.2f\n", add(1.5, 2.5));
    std::printf("add(3 args): %d\n", add(1, 2, 3));
    return 0;
}
