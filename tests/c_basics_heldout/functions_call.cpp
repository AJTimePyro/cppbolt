#include <cstdio>

int cube(int n) {
    return n * n * n;
}

int gcd(int a, int b) {
    if (b == 0) return a;
    return gcd(b, a % b);
}

int main() {
    std::printf("cube: %d\n", cube(4));
    std::printf("gcd: %d\n", gcd(48, 18));
    std::printf("gcd2: %d\n", gcd(101, 10));
    return 0;
}
