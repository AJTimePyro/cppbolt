#include <cstdio>

struct IntWrapper {
    int value;

    IntWrapper(int v) : value(v) {}

    operator int() const {
        return value;
    }

    operator double() const {
        return (double)value + 0.5;
    }
};

int main() {
    IntWrapper w(42);
    int x = w;
    double d = (double)w;
    std::printf("x=%d d=%.1f\n", x, d);
    return 0;
}
