#include <stdio.h>

int main() {
    // 1. Return type deduction without trailing return type
    auto mult = [](int v) {
        return v * 3;
    };
    printf("mult(4) = %d\n", mult(4));

    // 2. Default capture by value [=] with return type deduction
    int x = 10;
    int y = 20;
    auto sum_val = [=]() {
        return x + y;
    };
    printf("sum_val() = %d\n", sum_val());

    // 3. Default capture by reference [&]
    int a = 5;
    int b = 15;
    auto inc_ref = [&]() {
        a += 1;
        b += 2;
    };
    inc_ref();
    printf("a = %d, b = %d\n", a, b);

    // 4. Default capture by value with explicit reference override [=, &q]
    int p = 100;
    int q = 200;
    auto mixed = [=, &q]() {
        q += p;
        return q;
    };
    printf("mixed() = %d\n", mixed());
    printf("q after mixed = %d\n", q);

    // 5. Mutable lambda modifying by-value capture
    int state = 42;
    auto mut = [state]() mutable {
        state += 10;
        return state;
    };
    int s1 = mut();
    int s2 = mut();
    printf("s1 = %d, s2 = %d, state = %d\n", s1, s2, state);

    return 0;
}
