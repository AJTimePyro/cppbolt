#include <stdio.h>

int main() {
    // 1. Captureless lambda stored in auto
    auto add = [](int a, int b) -> int {
        return a + b;
    };
    printf("add(3, 4) = %d\n", add(3, 4));

    // 2. Immediately-invoked captureless lambda
    int sq = [](int x) -> int {
        return x * x;
    }(5);
    printf("sq = %d\n", sq);

    // 3. Explicit by-value capture
    int multiplier = 10;
    auto scale = [multiplier](int x) -> int {
        return x * multiplier;
    };
    printf("scale(7) = %d\n", scale(7));

    // 4. Explicit by-reference capture with mutation
    int counter = 0;
    auto inc = [&counter](int amount) -> int {
        counter += amount;
        return counter;
    };
    printf("inc(5) = %d\n", inc(5));
    printf("counter after inc = %d\n", counter);

    return 0;
}
