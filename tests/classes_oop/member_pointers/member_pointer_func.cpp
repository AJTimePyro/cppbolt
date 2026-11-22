#include <cstdio>

struct Calculator {
    int factor;
    int multiply(int x) {
        return x * factor;
    }
    int add(int a, int b) {
        return a + b + factor;
    }
};

int main() {
    Calculator calc;
    calc.factor = 5;

    int (Calculator::*pm_mul)(int) = &Calculator::multiply;
    int (Calculator::*pm_add)(int, int) = &Calculator::add;

    std::printf("mul: %d\n", (calc.*pm_mul)(10));
    std::printf("add: %d\n", (calc.*pm_add)(3, 4));

    Calculator* pcalc = &calc;
    std::printf("arrow mul: %d\n", (pcalc->*pm_mul)(6));
    std::printf("arrow add: %d\n", (pcalc->*pm_add)(20, 30));

    return 0;
}
