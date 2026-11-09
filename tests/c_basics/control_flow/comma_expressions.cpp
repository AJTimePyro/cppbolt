#include <cstdio>

int add(int x, int y) {
    return x + y;
}

void side_effect(int* ptr) {
    *ptr += 100;
}

int main() {
    // 1. Basic parenthesized comma
    int x = (1, 2, 3);
    std::printf("x: %d\n", x);

    // 2. LHS side effects
    int a = 1;
    int b = (a++, a + 10);
    std::printf("a: %d, b: %d\n", a, b);

    // 3. Comma expression as function argument
    int r = add((10, 20), (30, 40));
    std::printf("r: %d\n", r);

    // 4. Void LHS
    int val = 5;
    int after = (side_effect(&val), val);
    std::printf("after: %d\n", after);

    // 5. For loop iteration step
    int sum = 0;
    for (int i = 0, j = 10; i < 3; i++, j--) {
        sum += (i + j);
    }
    std::printf("sum: %d\n", sum);

    return 0;
}
