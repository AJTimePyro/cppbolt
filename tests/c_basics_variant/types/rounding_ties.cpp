#include <cstdio>

int main() {
    double inputs[9] = {1.5, 3.5, 0.625, 0.875, 0.25, 0.55, 1.675, 2.005, 0.45};
    int i = 0;
    while (i < 9) {
        double x = inputs[i];
        printf("x=%g -> f0: %.0f, f1: %.1f, f2: %.2f | e0: %.0e, e1: %.1e, e2: %.2e | g1: %.1g, g2: %.2g\n",
               x, x, x, x, x, x, x, x, x);
        i += 1;
    }
    return 0;
}
