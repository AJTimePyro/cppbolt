#include <cstdio>

template <typename T>
T my_max(T a, T b) {
    return (a > b) ? a : b;
}

int main() {
    int i = my_max<int>(10, 20);
    double d = my_max<double>(15.5, 9.2);
    std::printf("max int: %d\n", i);
    std::printf("max double: %.1f\n", d);
    return 0;
}
