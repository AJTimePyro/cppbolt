#include <cstdio>

int main() {
    double vals[9] = {0.5, 2.5, 0.125, 0.375, 0.15, 0.45, 2.675, 1.005, 0.35};
    for (int i = 0; i < 9; ++i) {
        double v = vals[i];
        printf("v=%g -> f0: %.0f, f1: %.1f, f2: %.2f | e0: %.0e, e1: %.1e, e2: %.2e | g1: %.1g, g2: %.2g\n",
               v, v, v, v, v, v, v, v, v);
    }
    return 0;
}
