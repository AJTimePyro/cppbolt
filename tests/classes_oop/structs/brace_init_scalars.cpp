#include <cstdio>

int main() {
    int a{42};
    int b = {100};
    int c{};
    float f{3.5f};
    double d{};
    bool b1{true};
    bool b2{};
    const int ca{99};
    const int cb{};

    std::printf("a: %d, b: %d, c: %d\n", a, b, c);
    std::printf("f: %.1f, d: %.1f\n", f, d);
    std::printf("b1: %d, b2: %d\n", b1 ? 1 : 0, b2 ? 1 : 0);
    std::printf("ca: %d, cb: %d\n", ca, cb);

    int x = {10};
    x = {20};
    std::printf("x: %d\n", x);

    return 0;
}
