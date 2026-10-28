#include <cstdio>

int main() {
    float f1 = 3.5f;
    float f2 = 1.25f;
    double d1 = 10.5;
    double d2 = 2.0;

    float fsum = f1 + f2;
    double ddiv = d1 / d2;
    int casted = (int)(f1 * 2.0f);

    std::printf("fsum: %.2f\n", fsum);
    std::printf("ddiv: %.2f\n", ddiv);
    std::printf("casted: %d\n", casted);
    return 0;
}
