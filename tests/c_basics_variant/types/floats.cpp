#include <cstdio>

int main() {
    float f1 = 2.5f;
    float f2 = 1.25f;
    double d1 = 12.375;
    double d2 = 4.0;

    std::printf("f_add: %f\n", (double)(f1 + f2));
    std::printf("f_sub: %f\n", (double)(f1 - f2));
    std::printf("d_mul: %f\n", d1 * d2);
    std::printf("d_div: %f\n", d1 / d2);

    int casted = (int)d1;
    std::printf("cast_to_int: %d\n", casted);
    double from_int = (double)casted;
    std::printf("cast_from_int: %f\n", from_int);
    return 0;
}
