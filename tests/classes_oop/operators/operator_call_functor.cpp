#include <stdio.h>

struct Multiplier {
    int factor;
    Multiplier(int f) : factor(f) {}

    int operator()(int val) const {
        return val * factor;
    }

    int operator()(int a, int b) {
        return (a + b) * factor;
    }
};

int main() {
    Multiplier mult(3);
    printf("%d\n", mult(7));      // 21
    printf("%d\n", mult(4, 5));   // 27
    return 0;
}
