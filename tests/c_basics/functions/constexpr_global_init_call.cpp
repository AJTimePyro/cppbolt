#include <cstdio>

constexpr int compute_factor(int x) {
    return x * x + 1;
}

int helper(int a, int b) {
    return a * b;
}

constexpr int g_val = compute_factor(9);
int g_helper = helper(3, 4);

int main() {
    std::printf("g_val=%d g_helper=%d\n", g_val, g_helper);
    return 0;
}
