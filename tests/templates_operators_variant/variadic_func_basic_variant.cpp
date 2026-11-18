#include <cstdio>

double sum_all() {
    return 0.0;
}

template <typename T, typename... Rest>
double sum_all(T first, Rest... rest) {
    return (double)first + sum_all(rest...);
}

int main() {
    double res = sum_all(1.5, 2.5, 3.0, 4);
    printf("sum: %.1f\n", res);
    return 0;
}
