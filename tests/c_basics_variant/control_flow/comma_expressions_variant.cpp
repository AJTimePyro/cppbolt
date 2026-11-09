#include <cstdio>

int multiply(int x, int y) {
    return x * y;
}

void bump(int* val) {
    *val += 50;
}

int main() {
    int v = (7, 8, 9);
    std::printf("v: %d\n", v);

    int m = 10;
    int n = (m += 5, m * 2);
    std::printf("m: %d, n: %d\n", m, n);

    int product = multiply((2, 3), (4, 5));
    std::printf("product: %d\n", product);

    int state = 25;
    int chained = (bump(&state), bump(&state), state);
    std::printf("chained: %d\n", chained);

    return 0;
}
